//! The records an outline prints before its catalog: the target's leads,
//! its public surface, which modules use which others, the outside items
//! its declarations name, and the outside items they write in more than
//! one way.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::md::leads::{self, Lead};
use crate::md::slop::facts::{flag, module_path, some, sorted_paths, subject, texts_by_decl};
use crate::md::slop::items::under;
use crate::md::slop::schema::{
    Dep, DepsRecord, ExternalRecord, LeadRecord, Mode, Record, Spelling, SpellingRecord,
    SurfaceKind, SurfaceRecord,
};
use crate::md::slop::{AnalysisTarget, Outline};
use crate::model::{self, DeclKind, ModId, Text, path_order};
use crate::prelude::is_standard;
use crate::spellings;
use crate::surface::Kind;
use crate::typerefs::strip_raw;

/// Returns a record for each lead of the run that names this target, in
/// lead order: by id in glossary order, then by subjects.
pub(crate) fn leads(outline: &Outline<'_>, leads: &[Lead]) -> Vec<Record> {
    let AnalysisTarget { run, target: t } = outline.target;
    leads
        .iter()
        .filter(|lead| lead.target_files.contains(&t))
        .map(|lead| {
            let locs = lead.locs.iter().map(|at| {
                let loc = model::Loc {
                    file: at.file,
                    line: at.line,
                };
                outline.loc(at.target, loc)
            });
            Record::Lead(LeadRecord {
                id: lead.id.to_owned(),
                mode: match lead.mode {
                    leads::Mode::Resolved => Mode::Resolved,
                    leads::Mode::DirectSyntax => Mode::DirectSyntax,
                    leads::Mode::Lexical => Mode::Lexical,
                },
                subjects: (lead.subjects.iter())
                    .map(|s| subject(run, t, s.target, &s.path))
                    .collect(),
                evidence: lead.evidence.clone(),
                locs: some(locs.collect()),
                limits: some(lead.limits.iter().map(|l| (*l).to_owned()).collect()),
                module: (lead.summary)
                    .map(|summary| module_path(&run.krates[summary.target], summary.module)),
                count: lead.summary.map(|summary| summary.count),
            })
        })
        .collect()
}

/// Returns a record for each path a user of the library can name, in path
/// order, with one record for the `cfg` twins a path names alike; a target
/// that is no library has none.
pub(crate) fn surface(outline: &Outline<'_>) -> Vec<Record> {
    let krate = outline.krate();
    let AnalysisTarget { run, target: t } = outline.target;
    let mut records: Vec<SurfaceRecord> = run.surfaces[t]
        .iter()
        .map(|entry| {
            let path = entry.path.join("::");
            let name = entry.path.last().map_or("", String::as_str);
            let (kind, item) = match &entry.kind {
                Kind::Module(m) => (SurfaceKind::Mod, module_path(krate, *m)),
                Kind::Decl(d) => {
                    let kind = match krate.decls[*d].kind {
                        DeclKind::Struct => SurfaceKind::Struct,
                        DeclKind::Enum => SurfaceKind::Enum,
                        DeclKind::Union => SurfaceKind::Union,
                        DeclKind::Trait => SurfaceKind::Trait,
                        DeclKind::TraitAlias | DeclKind::Alias => SurfaceKind::Alias,
                    };
                    (kind, krate.decl_path(*d).join("::"))
                }
                Kind::Function(at) => (SurfaceKind::Fn, value_path(krate, *at)),
                Kind::Value(at) => (SurfaceKind::Value, value_path(krate, *at)),
                Kind::Macro(m) => (SurfaceKind::Macro, under(&krate.modules[*m].path, name)),
                Kind::Cross(text, _) | Kind::Unlinked(text) => {
                    (SurfaceKind::External, text.clone())
                }
            };
            SurfaceRecord {
                item: (item != path).then_some(item),
                path,
                kind,
                test_support: flag(entry.test_support),
            }
        })
        .collect();
    records.dedup();
    records.into_iter().map(Record::Surface).collect()
}

/// Returns the path of a module value below the crate root, by the name its
/// own module declares it under.
fn value_path(krate: &model::Crate, at: model::ValueRef) -> String {
    let module = &krate.modules[at.module];
    under(&module.path, &module.items[at.item].name)
}

/// Returns a record for each module of the target, in `order`, that uses
/// items of another module, as the module graph holds them.
pub(crate) fn deps(outline: &Outline<'_>, order: &[ModId]) -> Vec<Record> {
    let AnalysisTarget { run, target: t } = outline.target;
    let krate = outline.krate();
    let modules = &krate.modules;
    let mut out = Vec::new();
    for &m in order {
        let mut edges: Vec<(ModId, usize)> = (run.graph[t][m].iter())
            .map(|(&to, edge)| (to, edge.items))
            .collect();
        if edges.is_empty() {
            continue;
        }
        edges.sort_by(|a, b| path_order(&modules[a.0].path, &modules[b.0].path));
        let uses = (edges.into_iter())
            .map(|(to, n)| Dep {
                path: module_path(krate, to),
                n,
            })
            .collect();
        out.push(Record::Deps(DepsRecord {
            path: module_path(krate, m),
            uses,
        }));
    }
    out
}

/// What the declarations do with one outside item.
#[derive(Default)]
struct Named {
    uses: usize,
    users: BTreeSet<String>,
    summary: Option<String>,
}

/// Returns a record for each item of another crate that the declarations
/// and module items name, in path order: how often, by what, and the
/// summary of its doc when it is an item of the run.
pub(crate) fn external(outline: &Outline<'_>) -> Vec<Record> {
    let AnalysisTarget { run, target: t } = outline.target;
    let krate = outline.krate();
    let mut items: BTreeMap<Vec<String>, Named> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut note = |r: usize, user: &str| {
        let reference = &krate.refs[r];
        let Some(path) = reference.outside().filter(|_| !reference.comment) else {
            return;
        };
        let first = path.first().map_or("", String::as_str);
        if path.len() < 2 || is_standard(path) || ["crate", "self", "super"].contains(&first) {
            return;
        }
        let named = items.entry(path.to_vec()).or_default();
        named.uses += 1;
        named.users.insert(user.to_owned());
        if let Some(item) = run.item_of_ref(t, r).filter(|item| item.target != t) {
            named.summary = run.summary(item).map(str::to_owned);
        }
    };
    for (user, texts) in texts_by_decl(run, t) {
        for r in texts.into_iter().flat_map(Text::refs) {
            if seen.insert(r) {
                note(r, &user);
            }
        }
    }
    for (m, module) in krate.modules.iter().enumerate() {
        let user = module_path(krate, m);
        for text in module
            .items
            .iter()
            .flat_map(|item| crate::model::item_texts(&item.kind))
        {
            for r in text.refs() {
                if seen.insert(r) {
                    note(r, &user);
                }
            }
        }
    }
    let mut sorted: Vec<(Vec<String>, Named)> = items.into_iter().collect();
    sorted.sort_by(|a, b| path_order(&a.0, &b.0));
    sorted
        .into_iter()
        .map(|(path, named)| {
            Record::External(ExternalRecord {
                path: path.join("::"),
                uses: named.uses,
                named_by: sorted_paths(named.users),
                summary: named.summary,
            })
        })
        .collect()
}

/// Returns the outside items that target `t`'s declarations write in more
/// than one way, each with the spellings that count. A bare spelling of a
/// name that a declaration of the crate also has does not count: where that
/// declaration is in scope, the item must be written with its path, so the
/// two spellings are no choice.
pub(crate) fn spelled<'r>(
    run: &'r AnalysisRun<'_>,
    t: usize,
) -> Vec<(&'r spellings::Entry, Vec<&'r spellings::Spelling>)> {
    let declared: BTreeSet<String> = (run.krates[t].decls.iter())
        .map(|decl| strip_raw(&decl.name))
        .collect();
    run.spellings[t]
        .iter()
        .filter_map(|entry| {
            let kept: Vec<&spellings::Spelling> = (entry.spellings.iter())
                .filter(|s| s.text.contains("::") || !declared.contains(&s.text))
                .collect();
            (kept.len() > 1).then_some((entry, kept))
        })
        .collect()
}

/// Returns a record for each outside item the declarations write in more
/// than one way, as [`spelled`] finds them.
pub(crate) fn spellings(outline: &Outline<'_>) -> Vec<Record> {
    spelled(outline.target.run, outline.target.target)
        .into_iter()
        .map(|(entry, kept)| {
            Record::Spelling(SpellingRecord {
                path: entry.item.clone(),
                preferred: entry.preferred.clone(),
                spellings: (kept.into_iter())
                    .map(|s| Spelling {
                        text: s.text.clone(),
                        count: s.count,
                        locs: s.locs.iter().map(|&loc| outline.own(loc)).collect(),
                    })
                    .collect(),
            })
        })
        .collect()
}
