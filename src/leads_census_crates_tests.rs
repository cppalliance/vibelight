//! Tests the census detectors that read across packages or a package's
//! manifest: a library item that no other target of the run names, and a
//! dependency entry that no source of its package writes where the entry
//! can be used.

use std::fs;

use crate::analysis::{AnalysisRun, inputs};
use crate::exclude::Exclude;
use crate::inputs::{Selection, choose};
use crate::leads;
use crate::leads_tests::found;

/// Writes `files` under one directory, reads the packages in `dirs` below
/// it as one run the way a run reads them, and returns the subjects of
/// each checked lead with `id`, each after its target's stem.
fn run_leads(files: &[(&str, &str)], dirs: &[&str], id: &str) -> Vec<String> {
    let tmp = tempfile::tempdir().unwrap();
    for (rel, text) in files {
        let path = tmp.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let named = dirs.iter().map(|dir| tmp.path().join(dir)).collect();
    let chosen = choose(&Selection::Named(named), &[], &Exclude::default()).unwrap();
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
    let run = AnalysisRun::build(&krates, &targets, &packages);
    run.validate().unwrap();
    let leads = leads::find(&run);
    leads::validate(&leads, &run).unwrap();
    (leads.iter())
        .filter(|lead| lead.id == id)
        .map(|lead| {
            let subjects: Vec<String> = (lead.subjects.iter())
                .map(|s| format!("{}:{}", run.targets[s.target].stem, s.path.join("::")))
                .collect();
            subjects.join(", ")
        })
        .collect()
}

/// A library package and a binary package that calls one of its functions;
/// `publish` is the library's `publish` line.
fn library_and_user(publish: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "core/Cargo.toml",
            format!("[package]\nname = \"core_lib\"\n{publish}"),
        ),
        (
            "core/src/lib.rs",
            "pub fn helper() -> u8 { 1 }\npub fn shared() -> u8 { helper() }\n".to_owned(),
        ),
        (
            "app/Cargo.toml",
            "[package]\nname = \"app\"\n[dependencies]\ncore_lib = { path = \"../core\" }\n"
                .to_owned(),
        ),
        (
            "app/src/main.rs",
            "fn main() { let _ = core_lib::shared(); }\n".to_owned(),
        ),
    ]
}

#[test]
fn a_library_item_that_no_other_target_names_has_no_observed_external_use() {
    let id = "no-observed-external-use";
    let leads_of = |publish: &str| {
        let files = library_and_user(publish);
        let files: Vec<(&str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
        run_leads(&files, &["core", "app"], id)
    };
    assert_eq!(leads_of("publish = false\n"), ["core_lib:helper"]);
    assert_eq!(
        leads_of(""),
        Vec::<String>::new(),
        "a package that may be published has outside users"
    );
}

#[test]
fn an_entry_that_no_source_able_to_use_it_writes_has_no_observed_dependency_use() {
    let id = "no-observed-dependency-use";
    let positive = [
        (
            "Cargo.toml",
            "[package]\nname = \"t\"\n[dependencies]\nserde = \"1\"\nlog = \"0.4\"\n\
             [dev-dependencies]\ntempfile = \"3\"\n",
        ),
        ("src/lib.rs", "pub fn f() -> u8 { 1 }\n"),
        (
            "tests/it.rs",
            "#[test]\nfn smoke() { log::info!(\"a dev hit\"); }\n",
        ),
    ];
    assert_eq!(found(&positive, id), ["log", "serde", "tempfile"]);
    let negative = [
        (
            "Cargo.toml",
            "[package]\nname = \"t\"\n[dependencies]\nserde = \"1\"\nlibz-sys = \"1\"\n\
             [dev-dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\n\
             tempfile = \"3\"\nt = { path = \".\" }\n[build-dependencies]\ncc = \"1\"\n",
        ),
        (
            "src/lib.rs",
            "pub fn f() -> Option<serde::de::IgnoredAny> { None }\n",
        ),
        (
            "tests/it.rs",
            "#[test]\nfn smoke() { let _ = tempfile::tempdir(); }\n",
        ),
        ("build.rs", "fn main() { cc::Build::new(); }\n"),
    ];
    assert_eq!(found(&negative, id), Vec::<String>::new());
}
