//! Tests the analysis view on small packages that are read, resolved, and
//! linked the way a run reads them: resolution from a binary into its own
//! library, call edges, the test contexts of the census, name-use counts,
//! validation, and determinism.

use std::fs;
use std::path::Path;

use crate::analysis::{AnalysisRun, PackageInfo, TargetInfo, inputs};
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::{PathKind, Via};
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::exclude::Exclude;
use crate::inputs::{Selection, choose};
use crate::model::{Crate, ValueRef};
use crate::model_body::Role;
use crate::root::TargetKind;

/// The crates of a one-package run and what analysis gathered beside them.
pub(crate) struct Fixture {
    _tmp: tempfile::TempDir,
    krates: Vec<Crate>,
    targets: Vec<TargetInfo>,
    packages: Vec<PackageInfo>,
}

/// Writes a package and reads, resolves, and links it as a run does.
pub(crate) fn fixture(files: &[(&str, &str)]) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    for (rel, text) in files {
        let path = tmp.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let (krates, targets, packages) = read(tmp.path());
    Fixture {
        _tmp: tmp,
        krates,
        targets,
        packages,
    }
}

fn read(dir: &Path) -> (Vec<Crate>, Vec<TargetInfo>, Vec<PackageInfo>) {
    let selection = Selection::Named(vec![dir.to_path_buf()]);
    let chosen = choose(&selection, &[], &Exclude::default()).unwrap();
    let listed = crate::pipeline::targets(&chosen).unwrap();
    let mut krates = Vec::new();
    for &(p, target) in &listed {
        let dir = &chosen.packages[p].input.dir;
        let tree = crate::modtree::walk(dir, &target.root).unwrap();
        let mut krate = crate::collect::collect(tree, dir, &target.crate_name, &target.title);
        crate::test_support::mark(&mut krate);
        crate::resolve::resolve(&mut krate);
        krates.push(krate);
    }
    let deps = crate::pipeline::crate_map(&chosen, &listed);
    crate::resolve_run::link(&mut krates, &deps);
    let (targets, packages) = inputs(&chosen, &listed, &krates, deps);
    (krates, targets, packages)
}

/// Builds and checks the analysis view of a fixture.
pub(crate) fn run(fixture: &Fixture) -> AnalysisRun<'_> {
    let run = AnalysisRun::build(&fixture.krates, &fixture.targets, &fixture.packages);
    run.validate().unwrap();
    run
}

/// The item of target `t` at `path`: a module value, a declaration, or a
/// member of one.
fn item(run: &AnalysisRun<'_>, t: usize, path: &str) -> ItemId {
    let krate = &run.krates[t];
    let mut all: Vec<ItemRef> = (0..krate.decls.len()).map(ItemRef::Decl).collect();
    for (m, module) in krate.modules.iter().enumerate() {
        let values =
            (0..module.items.len()).map(|k| ItemRef::Value(ValueRef { module: m, item: k }));
        all.extend(values);
    }
    for d in 0..krate.decls.len() {
        all.extend(run.members(t, d).map(|(member, _)| member));
    }
    all.into_iter()
        .map(|item| ItemId { target: t, item })
        .find(|&id| run.name(id).is_some() && run.path(id).join("::") == path)
        .unwrap_or_else(|| panic!("{path} exists"))
}

/// Shows an item as its target and path, such as `0:Table::new`.
fn shown(run: &AnalysisRun<'_>, item: ItemId) -> String {
    format!("{}:{}", item.target, run.path(item).join("::"))
}

/// The edges out of the body of `owner`: each path, what it reaches, and how.
fn edges(run: &AnalysisRun<'_>, owner: ItemId) -> Vec<(String, Option<String>, Via)> {
    let body = run.body_of(owner).expect("it has a body");
    let facts = &run.krates[owner.target].bodies[body].facts;
    let found = run.edges_from(owner.target, body).iter().map(|edge| {
        let path = match edge.kind {
            PathKind::Call(_) => &facts.calls[edge.fact].path,
            PathKind::Construct => &facts.constructs[edge.fact].path,
            PathKind::Pattern => &facts.patterns[edge.fact].path,
        };
        (
            path.join("::"),
            edge.item.map(|item| shown(run, item)),
            edge.via,
        )
    });
    found.collect()
}

const TABLE: &str = "pub struct Table;\n\
    impl Table {\n\
    pub fn new() -> Table { Table }\n\
    pub fn make() -> Self { Self::new() }\n\
    }\n\
    impl Default for Table { fn default() -> Self { Self::new() } }\n\
    pub trait Shape { fn area(&self) -> u8; fn twice(&self) -> u8 { Self::area(self) * 2 } }\n\
    pub fn build() -> Table { Table::new() }\n\
    pub fn fallback() -> Table { Table::default() }\n\
    pub fn spread(s: &dyn Shape) -> u8 { Shape::area(s) }\n\
    pub fn pick() -> fn() -> Table { Table::new }\n\
    fn hidden() {}\n";

#[test]
fn a_binary_reaches_its_own_library_without_changing_the_crates() {
    let fx = fixture(&[
        ("Cargo.toml", "[package]\nname = \"gauge\"\n"),
        (
            "src/lib.rs",
            "pub struct Dial;\nimpl Dial { pub fn new() -> Dial { Dial } }\npub fn measure() -> u8 { 1 }\n",
        ),
        (
            "src/main.rs",
            "fn main() {\n    let _dial = gauge::Dial::new();\n    let _level = gauge::measure();\n}\n",
        ),
    ]);
    let run = run(&fx);
    let kinds: Vec<TargetKind> = run.targets.iter().map(|t| t.kind).collect();
    assert_eq!(kinds, [TargetKind::Lib, TargetKind::Bin]);
    assert!(run.targets[1].deps.is_empty(), "the crate map has no edge");
    assert_eq!(
        run.deps[1].get("gauge"),
        Some(&0),
        "analysis adds the library"
    );
    let main = item(&run, 1, "main");
    let s = str::to_owned;
    assert_eq!(
        edges(&run, main),
        [
            (s("gauge::Dial::new"), Some(s("0:Dial::new")), Via::Member),
            (s("gauge::measure"), Some(s("0:measure")), Via::Path),
        ]
    );
    let body = run.body_of(main).expect("main has a body");
    for call in &run.krates[1].bodies[body].facts.calls {
        assert_eq!(
            run.krates[1].refs[call.full_ref].foreign, None,
            "link stays as it was"
        );
    }
    let measure = item(&run, 0, "measure");
    assert_eq!(
        run.observed(measure).targets,
        [1].into(),
        "the binary uses it"
    );
    let dial = item(&run, 0, "Dial");
    let prefix = run.krates[1].bodies[body].facts.calls[0]
        .prefix_ref
        .unwrap();
    assert_eq!(run.decl_of_ref(1, prefix), Some((0, 0)));
    assert!(
        run.cross[1].contains_key(&prefix),
        "only analysis reaches it"
    );
    assert!(
        run.uses_of(dial)
            .iter()
            .any(|u| matches!(u.kind, UseKind::Qualifier(_)))
    );
}

#[test]
fn type_paths_reach_a_unique_inherent_member_and_self_paths_only_in_inherent_impls() {
    let fx = fixture(&[
        ("Cargo.toml", "[package]\nname = \"t\"\n"),
        ("src/lib.rs", TABLE),
    ]);
    let run = run(&fx);
    let s = str::to_owned;
    let new = Some(s("0:Table::new"));
    let one = |path: &str, reached: Option<String>, via| vec![(s(path), reached, via)];
    assert_eq!(
        edges(&run, item(&run, 0, "build")),
        one("Table::new", new.clone(), Via::Member)
    );
    let make = item(&run, 0, "Table::make");
    assert_eq!(
        edges(&run, make),
        one("Self::new", new.clone(), Via::SelfType)
    );
    let pick = item(&run, 0, "pick");
    assert_eq!(
        edges(&run, pick),
        one("Table::new", new.clone(), Via::Member)
    );
    let escapes = run.edges_from(0, run.body_of(pick).unwrap());
    assert_eq!(escapes[0].kind, PathKind::Call(Role::ValueEscape));
    assert_eq!(run.edges_to(item(&run, 0, "Table::new")).count(), 3);
    let table = item(&run, 0, "Table");
    let members: Vec<String> = (run.members(0, 0))
        .map(|(member, _)| {
            shown(
                &run,
                ItemId {
                    target: 0,
                    item: member,
                },
            )
        })
        .collect();
    assert_eq!(
        members,
        ["0:Table::new", "0:Table::make", "0:Table::default"]
    );
    assert!(run.on_surface(table) && run.on_surface(item(&run, 0, "Table::new")));
    assert!(!run.on_surface(item(&run, 0, "hidden")));
}

#[test]
fn trait_dispatch_and_trait_impl_self_paths_stay_unresolved() {
    let fx = fixture(&[
        ("Cargo.toml", "[package]\nname = \"t\"\n"),
        ("src/lib.rs", TABLE),
    ]);
    let run = run(&fx);
    let s = str::to_owned;
    let unresolved = |path: &str| vec![(s(path), None, Via::Unresolved)];
    let default = item(&run, 0, "Table::default");
    assert!(matches!(default.item, ItemRef::TraitImpl(..)));
    assert_eq!(edges(&run, default), unresolved("Self::new"));
    let prefix = run.edges_from(0, run.body_of(default).unwrap())[0].prefix;
    assert_eq!(prefix.map(|p| shown(&run, p)).as_deref(), Some("0:Table"));
    assert_eq!(
        edges(&run, item(&run, 0, "Shape::twice")),
        unresolved("Self::area")
    );
    assert_eq!(
        edges(&run, item(&run, 0, "fallback")),
        unresolved("Table::default")
    );
    assert_eq!(
        edges(&run, item(&run, 0, "spread")),
        unresolved("Shape::area")
    );
}

const PROBE: &str = "/// Reads the level.\n\
    ///\n\
    /// ```\n\
    /// let level = probe::reading();\n\
    /// ```\n\
    ///\n\
    /// ```ignore\n\
    /// probe::ignored();\n\
    /// ```\n\
    pub fn reading() -> u8 { 1 }\n\
    pub fn spare() -> u8 { 2 }\n\
    #[cfg(test)]\n\
    mod tests {\n\
    #[test]\n\
    fn inline() { assert_eq!(super::reading(), 1); }\n\
    }\n\
    #[cfg(test)]\n\
    mod outside;\n";

fn probe() -> Fixture {
    fixture(&[
        ("Cargo.toml", "[package]\nname = \"probe\"\n"),
        ("src/lib.rs", PROBE),
        (
            "src/outside.rs",
            "#[test]\nfn loaded() { assert_eq!(crate::reading(), 1); }\n",
        ),
        (
            "tests/it.rs",
            "#[test]\nfn external() { assert_eq!(probe::reading(), 1); }\n",
        ),
    ])
}

#[test]
fn test_modules_test_files_package_tests_and_doc_tests_are_their_own_contexts() {
    let fx = probe();
    let run = run(&fx);
    let census = &run.census;
    let places: Vec<(&str, Context)> = census
        .paths_ending("reading")
        .map(|p| (census.sources[p.source].rel.as_str(), p.context))
        .collect();
    assert_eq!(
        places,
        [
            ("src/lib.rs", Context::Test),
            ("src/outside.rs", Context::Test),
            ("tests/it.rs", Context::Test),
            ("src/lib.rs", Context::Doctest),
        ]
    );
    let reading = item(&run, 0, "reading");
    let doctest = census
        .sources
        .iter()
        .find(|s| s.context == Context::Doctest)
        .unwrap();
    assert_eq!(Some(doctest.line), run.loc(reading).map(|loc| loc.line));
    assert!(
        census.token_hits("ignored").is_empty(),
        "an ignored block is no doc test"
    );
    let observed = run.observed(reading);
    assert_eq!(observed.contexts, [Context::Test, Context::Doctest].into());
    assert!(
        run.uses_of(reading)
            .iter()
            .all(|u| matches!(u.kind, UseKind::Qualified(_)))
    );
    assert_eq!(run.uses_of(item(&run, 0, "spare")), []);
    let count = run.name_count(reading).unwrap();
    let by = |target, context| count.by_place.get(&(0, target, context)).copied();
    assert_eq!(by(Some(0), Context::Production), Some(1), "the declaration");
    assert_eq!(
        by(Some(0), Context::Test),
        Some(2),
        "the test module and file"
    );
    assert_eq!(by(None, Context::Test), Some(1), "the package test");
    assert_eq!(by(Some(0), Context::Doctest), Some(1));
}

#[test]
fn a_name_use_count_leaves_out_one_hit_per_declaration() {
    let fx = fixture(&[
        ("Cargo.toml", "[package]\nname = \"t\"\n"),
        (
            "src/lib.rs",
            "pub fn alpha() -> u8 { 1 }\npub fn beta() -> u8 { alpha() }\n\
             pub mod inner { pub fn alpha() -> u8 { 2 } }\n",
        ),
    ]);
    let run = run(&fx);
    let count = |name: &str| {
        let count = run.census.count(name);
        (count.total, count.declared, count.uses())
    };
    assert_eq!(count("alpha"), (3, 2, 1));
    assert_eq!(count("beta"), (1, 1, 0));
    assert_eq!(count("inner"), (1, 1, 0));
    assert_eq!(count("u8"), (3, 0, 3));
    let alpha = item(&run, 0, "inner::alpha");
    assert_eq!(run.name_count(alpha).map(|c| c.uses()), Some(1));
}

#[test]
fn a_broken_view_fails_validation() {
    let fx = fixture(&[
        ("Cargo.toml", "[package]\nname = \"t\"\n"),
        ("src/lib.rs", TABLE),
    ]);
    let mut run = AnalysisRun::build(&fx.krates, &fx.targets, &fx.packages);
    run.validate().unwrap();
    run.edges.reverse();
    assert!(run.validate().unwrap_err().contains("edges"));
    run.edges.reverse();
    let path = run
        .uses
        .iter()
        .position(|u| matches!(u.kind, UseKind::Path(_)));
    run.uses.remove(path.unwrap());
    assert!(
        run.validate().unwrap_err().contains("edge"),
        "an edge's use is gone"
    );
    let mut run = AnalysisRun::build(&fx.krates, &fx.targets, &fx.packages);
    run.census.declarations.insert("phantom".to_owned(), 1);
    assert!(run.validate().unwrap_err().contains("phantom"));
}

#[test]
fn the_view_is_the_same_wherever_and_however_often_it_is_built() {
    let (first, second) = (probe(), probe());
    let shown = |fx: &Fixture| {
        format!(
            "{:?}",
            AnalysisRun::build(&fx.krates, &fx.targets, &fx.packages)
        )
    };
    assert_eq!(shown(&first), shown(&second));
    assert_eq!(shown(&first), shown(&first));
}
