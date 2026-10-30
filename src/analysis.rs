//! The immutable analysis view of a run, built after every crate of the run
//! is resolved and before any impl is attached. It associates each impl
//! with its type without moving it, resolves the paths that bodies call,
//! construct, and match, indexes the uses observed for each item, and holds
//! the lexical census, the doc summaries, the public surfaces, and the
//! spellings, for the detectors to read. Nothing in it depends on where the
//! run's files are: it holds no directory, link base, or stamp.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::analysis_items::{ItemId, ItemRef, Scopes};
use crate::analysis_paths::Edge;
use crate::analysis_uses::Use;
use crate::census::{Census, Context, Scan, SourceId};
use crate::inputs::Chosen;
use crate::manifest_facts::ManifestFacts;
use crate::model::{Crate, CrossRef, DeclId, RefId};
use crate::model_body::BodyId;
use crate::root::{Target, TargetKind};
use crate::source::SourceFile;
use crate::{analysis_items, analysis_paths, census_sources, modtree, spellings, surface};

/// What analysis knows of one target of the run.
#[derive(Debug)]
pub(crate) struct TargetInfo {
    /// The index of its package in the run.
    pub(crate) package: usize,
    /// The output stem, such as `runvil_parser` or `foo.bin`.
    pub(crate) stem: String,
    pub(crate) kind: TargetKind,
    /// The crate name that its own code and its users write.
    pub(crate) crate_name: String,
    /// Each dependency name it may write, mapped to that dependency's
    /// library target, as name resolution used them.
    pub(crate) deps: BTreeMap<String, usize>,
    /// The files that its test-only `mod` items load.
    pub(crate) files: Vec<Extra>,
}

/// What analysis knows of one package of the run.
#[derive(Debug)]
pub(crate) struct PackageInfo {
    pub(crate) facts: ManifestFacts,
    /// Its targets, as indexes into the run's targets, ascending.
    pub(crate) targets: Vec<usize>,
    /// The files of its tests, benches, examples, and build script: each
    /// root and the module files it loads, each once per context.
    pub(crate) files: Vec<Extra>,
}

/// One file outside a target's module tree that the census reads.
#[derive(Debug)]
pub(crate) struct Extra {
    /// The crate-relative path, with forward slashes.
    pub(crate) rel: String,
    pub(crate) context: Context,
    /// The text with LF line endings; `None` when it cannot be read.
    pub(crate) text: Option<String>,
}

/// Gathers what analysis reads beside the crates: each target's identity,
/// its dependency map from `deps`, and the files its test-only `mod` items
/// load, and each package's manifest facts and the files of its tests,
/// benches, examples, and build script. `targets` lists the run's targets
/// in order, each with the index of its package in `chosen`.
pub(crate) fn inputs(
    chosen: &Chosen,
    targets: &[(usize, &Target)],
    krates: &[Crate],
    deps: Vec<BTreeMap<String, usize>>,
) -> (Vec<TargetInfo>, Vec<PackageInfo>) {
    let mut packages: Vec<PackageInfo> = chosen
        .packages
        .iter()
        .map(|package| PackageInfo {
            facts: package.facts.clone(),
            targets: Vec::new(),
            files: package_files(&package.input.dir, &package.facts),
        })
        .collect();
    let mut infos = Vec::new();
    for (t, ((&(p, target), krate), deps)) in targets.iter().zip(krates).zip(deps).enumerate() {
        packages[p].targets.push(t);
        let dir = &chosen.packages[p].input.dir;
        infos.push(TargetInfo {
            package: p,
            stem: target.stem.clone(),
            kind: target.kind,
            crate_name: target.crate_name.clone(),
            deps,
            files: krate
                .test_files
                .iter()
                .map(|rel| read(dir, rel, Context::Test))
                .collect(),
        });
    }
    (infos, packages)
}

/// Reads the module tree of each test, bench, example, and build script
/// root of a package, each file once per context. A root that cannot be
/// read is kept without text.
fn package_files(dir: &Path, facts: &ManifestFacts) -> Vec<Extra> {
    let roots = (facts.tests.iter().map(|r| (r, Context::Test)))
        .chain(facts.benches.iter().map(|r| (r, Context::Bench)))
        .chain(facts.examples.iter().map(|r| (r, Context::Example)))
        .chain(facts.build.iter().map(|r| (r, Context::Build)));
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (root, context) in roots {
        let alone = read(dir, root, context);
        if alone
            .text
            .as_deref()
            .is_some_and(|text| !may_load_modules(text))
        {
            if seen.insert((root.clone(), context)) {
                out.push(alone);
            }
            continue;
        }
        let Ok(tree) = modtree::walk(dir, root) else {
            if seen.insert((root.clone(), context)) {
                out.push(Extra {
                    rel: root.clone(),
                    context,
                    text: None,
                });
            }
            continue;
        };
        for file in tree.files {
            if seen.insert((file.rel.clone(), context)) {
                out.push(Extra {
                    rel: file.rel,
                    context,
                    text: Some(file.text),
                });
            }
        }
        for rel in &tree.test_files {
            if seen.insert((rel.clone(), context)) {
                out.push(read(dir, rel, context));
            }
        }
    }
    out
}

/// Tells whether a text may load other files: it holds the word `mod`
/// with whitespace after it. A root that does not is a tree of one file,
/// which needs no parse.
fn may_load_modules(text: &str) -> bool {
    text.match_indices("mod").any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + 3..].chars().next();
        before.is_none_or(|c| !c.is_alphanumeric() && c != '_')
            && after.is_some_and(char::is_whitespace)
    })
}

/// Reads one crate-relative file, normalized as source files are.
fn read(dir: &Path, rel: &str, context: Context) -> Extra {
    let text = fs::read_to_string(dir.join(rel))
        .ok()
        .map(|raw| SourceFile::new(rel.to_owned(), &raw).text);
    Extra {
        rel: rel.to_owned(),
        context,
        text,
    }
}

/// The immutable analysis view of a run. Every id refers to the crates as
/// resolve and `link` left them, before attach.
#[derive(Debug)]
pub(crate) struct AnalysisRun<'a> {
    pub(crate) krates: &'a [Crate],
    pub(crate) targets: &'a [TargetInfo],
    pub(crate) packages: &'a [PackageInfo],
    /// Each target's dependency map for analysis: its own, and for a
    /// binary also its package's library.
    pub(crate) deps: Vec<BTreeMap<String, usize>>,
    /// For each target, what that map reaches for each reference that
    /// nothing reached before.
    pub(crate) cross: Vec<BTreeMap<RefId, CrossRef>>,
    /// For each target and impl, the declaration the impl belongs to.
    pub(crate) attached: Vec<Vec<Option<DeclId>>>,
    /// For each target and declaration, the impls that belong to it.
    pub(crate) impls_of: Vec<Vec<Vec<usize>>>,
    /// For each target and body, the item the body belongs to.
    pub(crate) body_items: Vec<Vec<Option<ItemRef>>>,
    /// For each target, the body of each item that has one.
    pub(crate) item_bodies: Vec<BTreeMap<ItemRef, BodyId>>,
    /// For each target and file, the modules there with their bytes.
    pub(crate) scopes: Vec<Scopes>,
    /// Every path of every body, in target, body, kind, and source order.
    pub(crate) edges: Vec<Edge>,
    /// The indexes in `edges` of the edges that reach each item.
    pub(crate) edges_in: BTreeMap<ItemId, Vec<usize>>,
    /// Every observed use, sorted by item.
    pub(crate) uses: Vec<Use>,
    pub(crate) census: Census,
    /// For each target, the summary of each declaration and module value
    /// whose raw doc text has one.
    pub(crate) summaries: Vec<BTreeMap<ItemRef, String>>,
    /// For each target, its public surface; empty for a target that is not
    /// a library.
    pub(crate) surfaces: Vec<Vec<surface::Entry>>,
    /// For each target, the items its surface names.
    pub(crate) surface_items: Vec<BTreeSet<ItemRef>>,
    /// For each target, the outside items its declarations write in more
    /// than one way, with the self types of the impls that belong to its
    /// declarations left out, as after attach.
    pub(crate) spellings: Vec<Vec<spellings::Entry>>,
    /// The census sources that could not be read or lexed, ascending.
    pub(crate) limits: Vec<SourceId>,
}

impl<'a> AnalysisRun<'a> {
    /// Builds the view of a run from its resolved, unattached crates and
    /// what `inputs` gathered for them, and changes none of them.
    pub(crate) fn build(
        krates: &'a [Crate],
        targets: &'a [TargetInfo],
        packages: &'a [PackageInfo],
    ) -> AnalysisRun<'a> {
        let deps = analysis_paths::augmented(targets, packages);
        let cross = analysis_paths::cross(krates, &deps);
        let attached: Vec<Vec<Option<DeclId>>> =
            krates.iter().map(analysis_items::attach).collect();
        let impls_of = krates
            .iter()
            .zip(&attached)
            .map(|(krate, attached)| analysis_items::impls_by_decl(krate, attached))
            .collect();
        let body_items: Vec<Vec<Option<ItemRef>>> =
            krates.iter().map(analysis_items::body_items).collect();
        let item_bodies = body_items
            .iter()
            .map(|items| analysis_items::bodies_by_item(items))
            .collect();
        let scopes: Vec<Scopes> = krates.iter().map(analysis_items::scopes).collect();
        let lib = |t: usize| targets[t].kind == TargetKind::Lib;
        let surfaces: Vec<Vec<surface::Entry>> = (0..krates.len())
            .map(|t| {
                if lib(t) {
                    surface::walk(&krates[t])
                } else {
                    Vec::new()
                }
            })
            .collect();
        let surface_items = surfaces
            .iter()
            .enumerate()
            .map(|(t, entries)| {
                if lib(t) {
                    analysis_items::surface_items(&krates[t], entries)
                } else {
                    BTreeSet::new()
                }
            })
            .collect();
        let spellings = krates
            .iter()
            .zip(&attached)
            .map(|(krate, attached)| spellings::find_hiding(krate, |i| attached[i].is_some()))
            .collect();
        let census = census_sources::build(krates, targets, packages, &scopes);
        let limits = (census.sources.iter().enumerate())
            .filter(|(_, source)| source.scan != Scan::Lexed)
            .map(|(id, _)| id)
            .collect();
        let mut run = AnalysisRun {
            krates,
            targets,
            packages,
            deps,
            cross,
            attached,
            impls_of,
            body_items,
            item_bodies,
            scopes,
            edges: Vec::new(),
            edges_in: BTreeMap::new(),
            uses: Vec::new(),
            census,
            summaries: krates.iter().map(analysis_items::summaries).collect(),
            surfaces,
            surface_items,
            spellings,
            limits,
        };
        run.edges = run.link_bodies();
        for (e, edge) in run.edges.iter().enumerate() {
            if let Some(item) = edge.item {
                run.edges_in.entry(item).or_default().push(e);
            }
        }
        run.uses = run.observe();
        run
    }
}
