//! Tests effective visibility: the access each written visibility gives,
//! the narrowing of a path through private modules, re-exports and globs,
//! the module a `pub(in path)` names, and the module an impl is written in.

use crate::access::{Access, covers, join, written};
use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_tests::{MANIFEST, fixture, run};
use crate::model::{Crate, DeclId, ModId, ValueRef, Vis};

fn module(krate: &Crate, path: &str) -> ModId {
    (krate.modules.iter())
        .position(|m| m.path.join("::") == path)
        .unwrap_or_else(|| panic!("no module {path}"))
}

fn decl(krate: &Crate, path: &str) -> DeclId {
    (0..krate.decls.len())
        .find(|&d| krate.decl_path(d).join("::") == path)
        .unwrap_or_else(|| panic!("no declaration {path}"))
}

fn value(krate: &Crate, path: &str) -> ValueRef {
    let (owner, name) = path.rsplit_once("::").unwrap_or(("", path));
    let m = module(krate, owner);
    let item = (krate.modules[m].items.iter())
        .position(|item| item.name == name)
        .unwrap_or_else(|| panic!("no value {path}"));
    ValueRef { module: m, item }
}

/// Runs the checks a test needs on the analysis view of a one-file library.
fn with_lib(source: &str, check: impl FnOnce(&AnalysisRun<'_>, &Crate)) {
    let fx = fixture(&[MANIFEST, ("src/lib.rs", source)]);
    let run = run(&fx);
    check(&run, &run.krates[0]);
}

#[test]
fn pub_super_reaches_the_crate_from_a_root_child_and_the_parent_below_it() {
    let source = "mod a {\n    pub(super) fn f() {}\n    pub mod b {\n        pub(super) fn g() {}\n    }\n}\n";
    with_lib(source, |run, krate| {
        let (a, b) = (module(krate, "a"), module(krate, "a::b"));
        let (f, g) = (value(krate, "a::f"), value(krate, "a::b::g"));
        let vis = |at: ValueRef| krate.modules[at.module].items[at.item].vis;
        assert_eq!(vis(f), Vis::Super);
        assert_eq!(written(krate, vis(f), a), Access::Within(0));
        assert_eq!(written(krate, vis(g), b), Access::Within(a));
        assert_eq!(run.accesses[0].value(g), Access::Within(a));
    });
}

#[test]
fn a_pub_item_in_a_private_module_reaches_only_the_crate() {
    with_lib("mod hidden {\n    pub struct X;\n}\n", |run, krate| {
        assert_eq!(
            run.accesses[0].decls[decl(krate, "hidden::X")],
            Access::Within(0)
        );
        assert!(!run.on_surface(ItemId::decl(0, decl(krate, "hidden::X"))));
    });
}

#[test]
fn a_root_reexport_makes_a_hidden_item_reach_anywhere() {
    with_lib(
        "mod hidden {\n    pub struct Y;\n}\npub use hidden::Y;\n",
        |run, krate| {
            let y = decl(krate, "hidden::Y");
            assert_eq!(run.accesses[0].decls[y], Access::World);
            assert!(run.on_surface(ItemId::decl(0, y)));
        },
    );
}

#[test]
fn a_glob_never_widens_an_item_past_its_own_visibility() {
    let source = "mod g {\n    pub(crate) fn r() {}\n    pub fn s() {}\n}\npub use g::*;\n";
    with_lib(source, |run, krate| {
        assert_eq!(
            run.accesses[0].value(value(krate, "g::r")),
            Access::Within(0)
        );
        assert_eq!(run.accesses[0].value(value(krate, "g::s")), Access::World);
    });
}

#[test]
fn a_restricted_reexport_reaches_only_as_far_as_it_is_written() {
    let source = "pub mod m {\n    pub fn run() {}\n    pub(crate) fn other() {}\n}\n\
                  pub(crate) use m::other as run;\n";
    with_lib(source, |run, krate| {
        let other = value(krate, "m::other");
        let public = value(krate, "m::run");
        assert_eq!(run.accesses[0].value(other), Access::Within(0));
        let on = |at: ValueRef| {
            run.on_surface(ItemId {
                target: 0,
                item: ItemRef::Value(at),
            })
        };
        assert!(!on(other) && on(public));
    });
}

#[test]
fn pub_in_path_names_the_ancestor_its_path_names() {
    let source = "pub mod a {\n    pub mod b {\n        pub(in crate::a) fn k() {}\n    }\n}\n";
    with_lib(source, |run, krate| {
        let a = module(krate, "a");
        let k = value(krate, "a::b::k");
        assert_eq!(krate.modules[k.module].items[k.item].vis, Vis::In(a));
        assert_eq!(run.accesses[0].value(k), Access::Within(a));
    });
}

#[test]
fn an_impl_records_the_module_it_is_written_in() {
    let source = "pub struct T;\nmod imp {\n    impl super::T {\n        pub(super) fn m(&self) {}\n    }\n}\n";
    with_lib(source, |_, krate| {
        let imp = module(krate, "imp");
        assert_eq!(krate.impls[0].module, imp);
        let member = &krate.impls[0].members[0];
        assert_eq!(written(krate, member.vis, imp), Access::Within(0));
    });
}

#[test]
fn covers_and_join_follow_the_module_tree() {
    with_lib("mod a {\n    mod b {}\n}\nmod c {}\n", |_, krate| {
        let (a, b, c) = (
            module(krate, "a"),
            module(krate, "a::b"),
            module(krate, "c"),
        );
        let within = Access::Within;
        assert!(covers(krate, Access::World, within(b)));
        assert!(covers(krate, within(a), within(b)));
        assert!(!covers(krate, within(b), within(a)));
        assert!(!covers(krate, within(a), Access::World));
        assert_eq!(join(krate, within(b), within(c)), within(0));
        assert_eq!(join(krate, within(b), within(a)), within(a));
        assert_eq!(join(krate, within(b), Access::World), Access::World);
    });
}
