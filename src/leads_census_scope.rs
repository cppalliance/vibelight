//! What the census detectors share: the candidates they read, the scope
//! that keeps them to what the compilers leave quiet, and the guards that
//! keep a lead from resting on code the run does not see.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::analysis_uses::{Use, UseKind};
use crate::census::{Context, Form, NameCount};
use crate::leads_items::impl_module;
use crate::leads_types::leaf;
use crate::model::{
    DeclId, DeclKind, Holder, ItemKind, MacroKind, MemberKind, Meta, ModId, Receiver, Shape,
    ValueRef, Vis,
};
use crate::test_support::is_test_support;
use crate::typerefs::strip_raw;

/// The lints whose `allow` or `expect` keeps the compiler quiet about an
/// unused item.
const DEAD_LINTS: [&str; 2] = ["dead_code", "unused"];

/// The attributes that hand an item to code outside the run's sources: a
/// linker symbol, or a procedural macro's entry point.
const EXPORTS: [&str; 5] = [
    "export_name",
    "no_mangle",
    "proc_macro",
    "proc_macro_attribute",
    "proc_macro_derive",
];

/// The module problems that leave some of a target's code unread.
const UNREAD: [&str; 3] = ["missing file", "module file loop", "unparsable file"];

/// The macro that pulls a file into a source, which the census does not
/// read.
const INCLUDE: &str = "include";

/// The derives whose code the compilers leave out of dead code analysis;
/// any other derive writes code the run does not see.
const STD_DERIVES: [&str; 9] = [
    "Clone",
    "Copy",
    "Debug",
    "Default",
    "Eq",
    "Hash",
    "Ord",
    "PartialEq",
    "PartialOrd",
];

/// What kind of item a candidate is, which decides how the run can observe
/// its uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A struct, enum, union, trait, trait alias, or type alias.
    Decl(DeclKind),
    /// A module's function.
    Function,
    /// A module's constant, static, or extern item.
    Value,
    /// A named field of a struct or union.
    Field,
    Variant,
    /// A trait's own function, constant, or type.
    TraitItem,
    /// An inherent function that takes `self`.
    Method,
    /// An inherent function without `self`, or an inherent constant.
    Associated,
}

/// One item that a census detector may report.
pub(crate) struct Candidate<'a> {
    pub(crate) item: ItemId,
    /// Its name without `r#`.
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) vis: Vis,
    pub(crate) meta: &'a Meta,
    /// The module whose lint attributes reach it: its own, its
    /// declaration's, or its impl's.
    pub(crate) module: ModId,
    /// Its declaration: its own, or that of its field, variant, or trait,
    /// or the type of its impl.
    pub(crate) decl: Option<DeclId>,
    /// The inherent impl that holds it.
    pub(crate) holder: Option<usize>,
    pub(crate) support: bool,
}

/// Returns every module value, declaration, named field, variant, trait
/// item, and inherent member of the run.
pub(crate) fn candidates<'a>(run: &AnalysisRun<'a>) -> Vec<Candidate<'a>> {
    let mut out = Vec::new();
    for (t, krate) in run.krates.iter().enumerate() {
        let at = |item| ItemId { target: t, item };
        for (m, module) in krate.modules.iter().enumerate() {
            for (k, item) in module.items.iter().enumerate() {
                let kind = match item.kind {
                    ItemKind::Fn(_) => Kind::Function,
                    ItemKind::Const { .. } | ItemKind::Static { .. } | ItemKind::Foreign { .. } => {
                        Kind::Value
                    }
                    _ => continue,
                };
                out.push(Candidate {
                    item: at(ItemRef::Value(ValueRef { module: m, item: k })),
                    name: strip_raw(&item.name),
                    kind,
                    vis: item.vis,
                    meta: &item.meta,
                    module: m,
                    decl: None,
                    holder: None,
                    support: module.test_support || is_test_support(&item.meta.cfgs),
                });
            }
        }
        for (d, decl) in krate.decls.iter().enumerate() {
            let own = |item, name: &str, kind, vis, meta| Candidate {
                item: at(item),
                name: strip_raw(name),
                kind,
                vis,
                meta,
                module: decl.module,
                decl: Some(d),
                holder: None,
                support: decl.test_support,
            };
            out.push(own(
                ItemRef::Decl(d),
                &decl.name,
                Kind::Decl(decl.kind),
                decl.vis,
                &decl.meta,
            ));
            let traited = decl.kind == DeclKind::Trait;
            for (k, member) in decl.members.iter().enumerate() {
                let kind = match (&member.kind, traited) {
                    (MemberKind::Field(_), false) => Kind::Field,
                    (MemberKind::Fn(_) | MemberKind::Const { .. } | MemberKind::Type(_), true) => {
                        Kind::TraitItem
                    }
                    _ => continue,
                };
                let item = ItemRef::Own(d, k);
                out.push(own(item, &member.name, kind, member.vis, &member.meta));
            }
            if let Shape::Enum(variants) = &decl.shape {
                for (k, variant) in variants.iter().enumerate() {
                    let item = ItemRef::Variant(d, k);
                    let kind = Kind::Variant;
                    out.push(own(item, &variant.name, kind, Vis::Public, &variant.meta));
                }
            }
        }
        for (i, imp) in krate.impls.iter().enumerate() {
            let decl = run.attached[t][i];
            let support = decl.is_some_and(|d| krate.decls[d].test_support);
            for (k, member) in imp.members.iter().enumerate() {
                let kind = match &member.kind {
                    MemberKind::Fn(sig) if sig.receiver != Receiver::None => Kind::Method,
                    MemberKind::Fn(_) | MemberKind::Const { .. } => Kind::Associated,
                    _ => continue,
                };
                out.push(Candidate {
                    item: at(ItemRef::Inherent(i, k)),
                    name: strip_raw(&member.name),
                    kind,
                    vis: member.vis,
                    meta: &member.meta,
                    module: impl_module(krate, i),
                    decl,
                    holder: Some(i),
                    support: support || is_test_support(&member.meta.cfgs),
                });
            }
        }
    }
    out
}

/// Tells whether lint attributes allow or expect an unused item.
pub(crate) fn keeps_dead(lints: &[String]) -> bool {
    lints.iter().any(|lint| {
        let args = (lint.strip_prefix("allow("))
            .or_else(|| lint.strip_prefix("expect("))
            .and_then(|args| args.strip_suffix(')'));
        args.is_some_and(|args| args.split(',').any(|arg| DEAD_LINTS.contains(&arg.trim())))
    })
}

/// What the census detectors know of every target before they look at an
/// item.
pub(crate) struct Scope {
    /// For each target and module, whether a lint attribute on it or on a
    /// module around it keeps unused items quiet.
    quiet: Vec<Vec<bool>>,
    /// For each target, whether some code of its own goes unread: a source
    /// that cannot be read or lexed, a file that `include!` pulls in, or a
    /// module file that is missing or does not parse.
    own: Vec<bool>,
    /// For each target, whether code that can name its surface goes
    /// unread: its own, another target's that can reach it, or a package
    /// test, bench, example, or build script that can.
    surface: Vec<bool>,
    /// For each package, whether some of its code goes unread.
    unread: Vec<bool>,
    /// The last segment of every path that a `serde` attribute names in a
    /// string, which no token shows.
    attr_names: BTreeSet<String>,
}

impl Scope {
    pub(crate) fn new(run: &AnalysisRun<'_>) -> Scope {
        let quiet = (run.krates.iter())
            .map(|krate| {
                let by_path: BTreeMap<&[String], ModId> = (krate.modules.iter().enumerate())
                    .map(|(m, module)| (module.path.as_slice(), m))
                    .collect();
                (krate.modules.iter())
                    .map(|module| {
                        (0..=module.path.len()).any(|n| {
                            let around = by_path.get(&module.path[..n]);
                            around.is_some_and(|&a| keeps_dead(&krate.modules[a].lints))
                        })
                    })
                    .collect()
            })
            .collect();
        let n = run.krates.len();
        let mut own = vec![false; n];
        let mut unread = vec![false; run.packages.len()];
        let included = (run.census.token_hits(INCLUDE).iter())
            .filter(|hit| hit.form == Form::Macro)
            .map(|hit| hit.source);
        for source in run.limits.iter().copied().chain(included) {
            let source = &run.census.sources[source];
            unread[source.package] = true;
            if let Some(t) = source.target {
                own[t] = true;
            }
        }
        let lost = |kind: &ItemKind| matches!(kind, ItemKind::Unsupported(why) if UNREAD.contains(&why.as_str()));
        for (t, krate) in run.krates.iter().enumerate() {
            let mut items = krate.modules.iter().flat_map(|m| &m.items);
            if items.any(|item| lost(&item.kind)) {
                own[t] = true;
                unread[run.targets[t].package] = true;
            }
        }
        let surface = (0..n)
            .map(|t| {
                let reaches = |u: &usize| *u == t || run.deps[*u].values().any(|&d| d == t);
                (0..n)
                    .filter(reaches)
                    .any(|u| own[u] || unread[run.targets[u].package])
            })
            .collect();
        let attr_names = (run.krates.iter())
            .flat_map(|krate| &krate.attr_paths)
            .filter_map(|attr| attr.path.last().cloned())
            .collect();
        Scope {
            quiet,
            own,
            surface,
            unread,
            attr_names,
        }
    }

    /// Tells whether code that can name an item goes unread: its target's
    /// own, or for an item on a library's surface, any code that can reach
    /// the library.
    pub(crate) fn blind(&self, run: &AnalysisRun<'_>, item: ItemId) -> bool {
        let t = item.target;
        self.own[t] || (self.surface[t] && run.on_surface(item))
    }

    /// Tells whether code that can name the surface of target `t` goes
    /// unread.
    pub(crate) fn blind_surface(&self, t: usize) -> bool {
        self.surface[t]
    }

    /// Tells whether some code of package `p` goes unread.
    pub(crate) fn unread(&self, p: usize) -> bool {
        self.unread[p]
    }

    /// Tells whether code the run does not read may use a candidate: code
    /// that can name it goes unread, a `serde` attribute names it in a
    /// string, or it is handed off to code outside the run's sources.
    pub(crate) fn unseen(&self, run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
        self.blind(run, c.item) || self.attr_names.contains(&c.name) || handed_off(run, c)
    }

    /// Tells whether an `allow` or `expect` of `dead_code` or `unused`
    /// reaches a candidate: on itself, its declaration, or a module around
    /// it. An impl's own lint attributes go unread.
    pub(crate) fn kept(&self, run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
        let krate = &run.krates[c.item.target];
        keeps_dead(&c.meta.lints)
            || c.decl
                .is_some_and(|d| keeps_dead(&krate.decls[d].meta.lints))
            || self.quiet[c.item.target][c.module]
    }

    /// Tells whether the compilers stay quiet about a candidate that
    /// nothing uses: it is on its library's surface, or an `allow` or
    /// `expect` of `dead_code` reaches it.
    pub(crate) fn skipped(&self, run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
        run.on_surface(c.item) || self.kept(run, c)
    }
}

/// Tells whether code outside the run's sources may use a candidate: an
/// attribute macro marks it, its declaration, or its impl, or its file
/// writes an attribute that exports a symbol or a procedural macro.
fn handed_off(run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
    let t = c.item.target;
    let Some(loc) = run.loc(c.item) else {
        return true;
    };
    let marked = run.krates[t].macros.iter().any(|mark| {
        let holder = match mark.holder {
            Holder::Decl(d) => c.decl == Some(d),
            Holder::Impl(i) => c.holder == Some(i),
            Holder::Module(_) => false,
        };
        mark.kind == MacroKind::Attribute && (holder || mark.loc == loc)
    });
    let source = run.census.tree_source(t, loc.file);
    let exported = EXPORTS
        .iter()
        .any(|name| (run.census.token_hits(name).iter()).any(|hit| hit.source == source));
    marked || exported
}

/// Returns the path of edge `e` as its body writes it.
pub(crate) fn edge_path<'r>(run: &'r AnalysisRun<'_>, e: usize) -> &'r [String] {
    let edge = &run.edges[e];
    let facts = &run.krates[edge.target].bodies[edge.body].facts;
    match edge.kind {
        PathKind::Call(_) => &facts.calls[edge.fact].path,
        PathKind::Construct => &facts.constructs[edge.fact].path,
        PathKind::Pattern => &facts.patterns[edge.fact].path,
    }
}

/// Tells whether declaration `d` of target `t` is plain: only the
/// compilers' own derives, no `serde` attribute, and no attribute macro,
/// so no code the run does not see reads its fields or builds its
/// variants.
pub(crate) fn plain(run: &AnalysisRun<'_>, t: usize, d: DeclId) -> bool {
    let decl = &run.krates[t].decls[d];
    let derived =
        (decl.written_derives.iter()).all(|name| STD_DERIVES.contains(&leaf(name).as_str()));
    let marked = (run.krates[t].macros.iter())
        .any(|mark| mark.kind == MacroKind::Attribute && mark.holder == Holder::Decl(d));
    let serde = decl.meta.attrs.iter().any(|attr| attr.starts_with("serde"));
    derived && !marked && !serde
}

/// Tells whether a candidate is a field or a variant of a declaration that
/// is not plain, which code the run does not see may read or build.
pub(crate) fn generated_part(run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
    let part = matches!(c.kind, Kind::Field | Kind::Variant);
    part && c.decl.is_some_and(|d| !plain(run, c.item.target, d))
}

/// Returns the observed uses of an item that consume it: every use but a
/// doc link and a re-export in its own target.
pub(crate) fn consumers<'r>(run: &'r AnalysisRun<'_>, item: ItemId) -> Vec<&'r Use> {
    (run.uses_of(item).iter())
        .filter(|u| match u.kind {
            UseKind::Doc => false,
            UseKind::Reexport => run.census.sources[u.source].target != Some(item.target),
            _ => true,
        })
        .collect()
}

/// Tells whether a hit of a name is inside macro arguments or a
/// `macro_rules!` body.
pub(crate) fn in_macro(run: &AnalysisRun<'_>, name: &str) -> bool {
    run.census.token_hits(name).iter().any(|hit| hit.in_macro)
}

/// Returns how many hits of a name are in one context.
pub(crate) fn hits_in(count: &NameCount, context: Context) -> usize {
    (count.by_place.iter())
        .filter(|((_, _, c), _)| *c == context)
        .map(|(_, n)| n)
        .sum()
}

/// Tells whether a name has a hit outside production code.
pub(crate) fn elsewhere(count: &NameCount) -> bool {
    (count.by_place.keys()).any(|(_, _, context)| *context != Context::Production)
}

/// Returns the limits of a lead on an item: `base`, and
/// `external-consumers` when the item is on its library's surface.
pub(crate) fn limits(
    run: &AnalysisRun<'_>,
    item: ItemId,
    base: &[&'static str],
) -> Vec<&'static str> {
    let mut out = base.to_vec();
    if run.on_surface(item) {
        out.push("external-consumers");
    }
    out
}

/// Names a module of target `t` as a reader sees it: its path, or its
/// crate's name for the root.
pub(crate) fn module_name(run: &AnalysisRun<'_>, t: usize, m: ModId) -> String {
    let path = &run.krates[t].modules[m].path;
    if path.is_empty() {
        run.targets[t].crate_name.clone()
    } else {
        path.join("::")
    }
}
