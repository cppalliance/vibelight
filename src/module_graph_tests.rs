//! Tests the module graph turned around: each module's users, each with
//! the edge from it, so its item count and its resolved flag stay; and the
//! crate graph: a binary that uses its library, and uses of items that
//! come through re-exports.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::analysis::tests::{fixture, packages, run};
use crate::model::ModId;
use crate::module_graph::{ModEdge, crates, used_by};

/// Returns the maps of `n` modules, where each `(m, other, items,
/// resolved)` puts an edge under `other` in module `m`'s map.
fn maps(n: usize, edges: &[(ModId, ModId, usize, bool)]) -> Vec<BTreeMap<ModId, ModEdge>> {
    let mut out = vec![BTreeMap::new(); n];
    for &(m, other, items, resolved) in edges {
        out[m].insert(other, ModEdge { items, resolved });
    }
    out
}

#[test]
fn each_edge_shows_under_the_module_it_uses_with_its_item_count() {
    let graph = maps(4, &[(1, 2, 3, true), (1, 3, 1, false), (2, 3, 2, true)]);
    let users = used_by(&graph);
    let turned = maps(4, &[(2, 1, 3, true), (3, 1, 1, false), (3, 2, 2, true)]);
    assert_eq!(users, turned);
    let fan_in: Vec<usize> = users.iter().map(BTreeMap::len).collect();
    let fan_out: Vec<usize> = graph.iter().map(BTreeMap::len).collect();
    assert_eq!((fan_in, fan_out), (vec![0, 0, 1, 2], vec![0, 2, 1, 0]));
}

#[test]
fn modules_that_use_each_other_are_each_others_users() {
    let graph = maps(3, &[(1, 2, 4, true), (2, 1, 1, false)]);
    let turned = maps(3, &[(1, 2, 1, false), (2, 1, 4, true)]);
    assert_eq!(used_by(&graph), turned);
}

#[test]
fn a_graph_with_no_edges_gives_no_module_a_user() {
    assert_eq!(used_by(&maps(3, &[])), maps(3, &[]));
    assert_eq!(used_by(&[]), maps(0, &[]));
}

#[test]
fn a_binary_uses_each_library_item_its_production_code_names_once() {
    let fx = fixture(&[
        ("Cargo.toml", "[package]\nname = \"gauge\"\n"),
        (
            "src/lib.rs",
            "pub struct Dial;\nimpl Dial { pub fn new() -> Dial { Dial } }\n\
             pub fn measure() -> u8 { 1 }\npub fn spare() {}\npub fn probe() {}\n",
        ),
        (
            "src/main.rs",
            "/// Reads with [`gauge::spare`].\n\
             fn main() { let _dial = gauge::Dial::new(); let _level = gauge::measure(); }\n\
             fn again() -> u8 { gauge::measure() }\n\
             #[cfg(test)]\nmod tests { fn probes() { gauge::probe(); } }\n",
        ),
    ]);
    assert_eq!(
        crates(&run(&fx)),
        [BTreeMap::new(), BTreeMap::from([(0, 3)])],
        "Dial, Dial::new, and measure, but not the doc link or the test"
    );
}

/// A library package: its name, the packages it depends on, and its
/// `src/lib.rs`.
type Package<'a> = (&'a str, &'a [&'a str], &'a str);

/// Declares a type with a method, and a function.
const BASE: Package<'static> = (
    "base",
    &[],
    "pub struct Dial;\nimpl Dial { pub fn new() -> Dial { Dial } }\npub fn measure() -> u8 { 1 }\n",
);

/// Reads library packages, each with path dependencies on the packages it
/// names, as one run, and returns each edge of its crate graph as `user
/// uses other (items)`, in target order.
fn edges(members: &[Package<'_>]) -> Vec<String> {
    let mut files = Vec::new();
    for &(name, deps, lib) in members {
        let mut manifest = format!("[package]\nname = \"{name}\"\n[dependencies]\n");
        for dep in deps {
            let _ = writeln!(manifest, "{dep} = {{ path = \"../{dep}\" }}");
        }
        files.push((format!("{name}/Cargo.toml"), manifest));
        files.push((format!("{name}/src/lib.rs"), lib.to_owned()));
    }
    let files: Vec<(&str, &str)> = (files.iter())
        .map(|(rel, text)| (rel.as_str(), text.as_str()))
        .collect();
    let dirs: Vec<&str> = members.iter().map(|&(name, _, _)| name).collect();
    let fx = packages(&files, &dirs);
    let run = run(&fx);
    let stem = |t: usize| run.targets[t].stem.clone();
    let mut out = Vec::new();
    for (t, used) in crates(&run).iter().enumerate() {
        for (&other, &items) in used {
            out.push(format!("{} uses {} ({items})", stem(t), stem(other)));
        }
    }
    out
}

#[test]
fn a_use_through_a_facade_counts_for_the_facade() {
    let facade = "pub use base::Dial;\npub use base::measure;\n";
    let app = "pub fn run() -> u8 { let _dial = facade::Dial::new(); facade::measure() }\n";
    assert_eq!(
        edges(&[
            BASE,
            ("facade", &["base"], facade),
            ("app", &["facade"], app)
        ]),
        ["app uses facade (3)", "facade uses base (2)"],
        "Dial, Dial::new, and measure come through facade, the only crate app depends on"
    );
}

#[test]
fn a_dependency_that_declares_the_item_wins_over_one_that_re_exports_it() {
    let facade = "pub use base::measure;\npub struct Panel;\n";
    let app = "pub fn run(_panel: facade::Panel) -> u8 { facade::measure() }\n";
    let found = edges(&[
        BASE,
        ("facade", &["base"], facade),
        ("app", &["base", "facade"], app),
    ]);
    assert_eq!(
        found,
        [
            "app uses base (1)",
            "app uses facade (1)",
            "facade uses base (1)"
        ],
        "measure counts for base, which app depends on, and Panel for facade"
    );
}

#[test]
fn a_re_export_of_a_re_export_reaches_the_item_it_re_exports() {
    let found = edges(&[
        BASE,
        ("mid", &["base"], "pub use base::Dial;\n"),
        ("top", &["mid"], "pub use mid::Dial;\n"),
        (
            "app",
            &["top"],
            "pub fn dial() -> top::Dial { top::Dial }\n",
        ),
    ]);
    assert_eq!(
        found,
        ["app uses top (1)", "mid uses base (1)", "top uses mid (1)"],
        "top's re-export resolves through mid's to base's Dial"
    );
}

#[test]
fn a_use_that_no_dependency_re_exports_counts_for_the_crate_that_declares_it() {
    let app = "pub fn run() -> u8 { facade::measure() }\n";
    assert_eq!(
        edges(&[
            BASE,
            ("facade", &["base"], "pub use base::*;\n"),
            ("app", &["facade"], app)
        ]),
        ["app uses base (1)"],
        "a glob re-export names no item, so measure counts for base"
    );
}
