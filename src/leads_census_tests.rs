//! Tests each detector of the source-observed census on a small package
//! where it finds its lead, and on a close one where it finds none, and
//! the scope that keeps the census to what the compilers leave quiet.

use crate::leads::Mode;
use crate::leads_census_scope::keeps_dead;
use crate::leads_tests::{MANIFEST, found, lib, with_leads};

/// The manifest of a package named `t` that does not allow publishing.
const UNPUBLISHED: (&str, &str) = ("Cargo.toml", "[package]\nname = \"t\"\npublish = false\n");

/// Asserts what `id` finds in a library and that it finds nothing in a
/// close one.
fn finds(id: &str, positive: &str, expected: &[&str], negative: &str) {
    assert_eq!(lib(positive, id), expected, "{id} on its case");
    assert_eq!(
        lib(negative, id),
        Vec::<String>::new(),
        "{id} on the close case"
    );
}

/// Asserts what `id` finds in a package made of `files` and that it finds
/// nothing in a close one.
fn finds_in(id: &str, positive: &[(&str, &str)], expected: &[&str], negative: &[(&str, &str)]) {
    assert_eq!(found(positive, id), expected, "{id} on its case");
    assert_eq!(
        found(negative, id),
        Vec::<String>::new(),
        "{id} on the close case"
    );
}

#[test]
fn only_an_allow_or_expect_of_dead_code_or_unused_keeps_an_item() {
    let lints =
        |list: &[&str]| keeps_dead(&list.iter().map(|l| (*l).to_owned()).collect::<Vec<_>>());
    assert!(lints(&["allow(dead_code)"]));
    assert!(lints(&["expect(dead_code, reason = \"kept\")"]));
    assert!(lints(&["allow(unused, clippy::all)"]));
    assert!(!lints(&[
        "allow(clippy::dead_code_like)",
        "deny(dead_code)",
        "warn(unused)"
    ]));
}

#[test]
fn a_quiet_item_whose_name_no_source_writes_has_no_observed_name_use() {
    finds(
        "no-observed-name-use",
        "pub fn orphan_entry() {}\n#[allow(dead_code)]\nfn kept_helper() {}\n",
        &["kept_helper", "orphan_entry"],
        "pub fn used_entry() -> Wire { Wire { x: 0 } }\nfn private_dead() {}\n\
         #[cfg(test)]\nmod tests { #[test] fn t() { super::used_entry(); } }\n\
         #[derive(serde::Serialize)]\npub struct Wire { #[serde(skip_serializing_if = \"is_blank\")] pub x: u8 }\n\
         pub fn is_blank(x: &u8) -> bool { *x == 0 }\n",
    );
    with_leads(
        &[MANIFEST, ("src/lib.rs", "pub fn orphan_entry() {}\n")],
        |_, leads| {
            let lead = leads
                .iter()
                .find(|l| l.id == "no-observed-name-use")
                .unwrap();
            assert_eq!(lead.mode, Mode::Lexical);
            assert_eq!(
                lead.limits,
                ["external-consumers", "generated-code", "macros"]
            );
        },
    );
}

#[test]
fn code_that_include_pulls_in_blinds_the_census() {
    let files = [
        MANIFEST,
        (
            "src/lib.rs",
            "pub fn orphan_entry() {}\ninclude!(\"extra.rs\");\n",
        ),
        ("src/extra.rs", "pub fn extra() { orphan_entry() }\n"),
    ];
    assert_eq!(found(&files, "no-observed-name-use"), Vec::<String>::new());
}

#[test]
fn hits_that_resolve_to_other_items_are_no_observed_path_use() {
    let base = "pub mod a { pub fn run() {} }\npub mod b { pub fn run() {} }\n";
    finds(
        "no-observed-path-use",
        &format!("{base}pub fn go() {{ a::run() }}\n"),
        &["b::run"],
        &format!("{base}pub fn go() {{ a::run(); b::run() }}\n"),
    );
}

#[test]
fn qualified_paths_in_test_code_alone_are_test_only_observed_use() {
    finds(
        "test-only-observed-use",
        "pub fn only_tested() -> u8 { 1 }\n\
         #[cfg(test)]\nmod tests { #[test] fn t() { assert_eq!(super::only_tested(), 1); } }\n",
        &["only_tested"],
        "pub fn shipped() -> u8 { 1 }\npub fn caller() -> u8 { shipped() }\npub fn bare() -> u8 { 2 }\n\
         #[cfg(test)]\nmod tests { use super::*; #[test] fn t() { assert_eq!(super::shipped(), bare()); } }\n",
    );
}

/// An enum that one function matches in full.
const MODES: &str = "pub enum Mode { Fast, Slow }\n\
    pub fn speed(m: Mode) -> u8 { match m { Mode::Fast => 1, Mode::Slow => 2 } }\n\
    pub fn fast() -> Mode { Mode::Fast }\n";

#[test]
fn a_variant_that_only_patterns_name_has_no_observed_construction() {
    finds(
        "no-observed-construction",
        MODES,
        &["Mode::Slow"],
        &format!(
            "{MODES}pub fn slow() -> Mode {{ Mode::Slow }}\n\
             #[derive(serde::Deserialize)]\npub enum Wire {{ On, Off }}\n\
             pub fn on(w: Wire) -> bool {{ match w {{ Wire::On => true, Wire::Off => false }} }}\n"
        ),
    );
}

#[test]
fn a_variant_that_is_built_but_never_matched_has_no_observed_pattern() {
    let kinds = "#[derive(PartialEq)]\npub enum Kind { Leaf, Branch }\n\
        pub fn leaf() -> Kind { Kind::Leaf }\npub fn branch() -> Kind { Kind::Branch }\n";
    finds(
        "no-observed-pattern",
        &format!(
            "{kinds}pub fn is_leaf(k: &Kind) -> bool {{ match k {{ Kind::Leaf => true, _ => false }} }}\n"
        ),
        &["Kind::Branch"],
        &format!(
            "{kinds}pub fn is_leaf(k: &Kind) -> bool {{ match k {{ Kind::Leaf => true, _ => false }} }}\n\
             pub fn is_branch(k: &Kind) -> bool {{ *k == Kind::Branch }}\n"
        ),
    );
}

#[test]
fn a_field_that_only_struct_literals_set_has_no_observed_field_read() {
    let config = "pub struct Config { pub name: String, pub retries: u8 }\n\
        pub fn make() -> Config { Config { name: String::new(), retries: 3 } }\n\
        pub fn show(c: &Config) -> &str { &c.name }\n";
    finds(
        "no-observed-field-read",
        config,
        &["Config::retries"],
        &format!(
            "{config}pub fn tries(c: Config) -> u8 {{ let Config {{ retries, .. }} = c; retries }}\n\
             #[repr(C)]\npub struct Raw {{ pub bits: u8 }}\npub fn raw() -> Raw {{ Raw {{ bits: 0 }} }}\n"
        ),
    );
}

#[test]
fn an_item_that_only_another_module_uses_and_nothing_ties_is_defined_away() {
    let lib = |parse: &str| {
        format!(
            "mod util {{\n\
             pub(crate) const LIMIT: u32 = 8;\n\
             pub(crate) fn clamp(v: u32) -> u32 {{ v.min(100) }}\n\
             }}\n\
             mod parse {{\n{parse}\n}}\n\
             mod render {{ pub(crate) fn shown(v: u32) -> u32 {{ crate::util::clamp(v) }} }}\n\
             pub fn run(v: u32) -> (bool, u32) {{ (parse::limited(v), render::shown(v)) }}\n"
        )
    };
    finds(
        "defined-away-from-observed-users",
        &lib("use crate::util::LIMIT;\npub(crate) fn limited(v: u32) -> bool { v < LIMIT }"),
        &["util::clamp", "util::LIMIT"],
        &lib(
            "use crate::util::{clamp, LIMIT};\npub(crate) fn limited(v: u32) -> bool { clamp(v) < LIMIT }",
        ),
    );
}

#[test]
fn a_pub_item_that_only_its_own_module_writes_has_no_observed_outside_module_use() {
    let inner = (
        "src/inner.rs",
        "pub(crate) fn visible() -> u8 { narrow() }\npub(crate) fn narrow() -> u8 { 1 }\n",
    );
    let lib = |body: &str| format!("mod inner;\npub fn api() -> u8 {{ {body} }}\n");
    let positive = lib("inner::visible()");
    let negative = lib("inner::visible() + inner::narrow()");
    finds_in(
        "no-observed-outside-module-use",
        &[MANIFEST, ("src/lib.rs", &positive), inner],
        &["inner::narrow"],
        &[MANIFEST, ("src/lib.rs", &negative), inner],
    );
}

#[test]
fn a_pub_use_that_no_path_imports_through_has_no_observed_reexport_use() {
    let lib = |run: &str| {
        format!(
            "mod inner {{ pub fn helper() -> u8 {{ 1 }} pub fn other() -> u8 {{ 2 }} }}\n\
             pub use inner::helper;\npub use inner::other;\npub fn run() -> u8 {{ {run} }}\n"
        )
    };
    let positive = lib("crate::other()");
    let negative = lib("crate::other() + crate::helper()");
    finds_in(
        "no-observed-reexport-use",
        &[UNPUBLISHED, ("src/lib.rs", &positive)],
        &["helper"],
        &[UNPUBLISHED, ("src/lib.rs", &negative)],
    );
    assert_eq!(
        found(
            &[MANIFEST, ("src/lib.rs", &positive)],
            "no-observed-reexport-use"
        ),
        Vec::<String>::new(),
        "a package that may be published has outside users"
    );
}

#[test]
fn a_tiny_private_function_with_one_caller_has_one_observed_caller() {
    finds(
        "one-observed-caller",
        "fn tiny(x: u8) -> u8 { x + 1 }\npub fn api(x: u8) -> u8 { tiny(x) }\n",
        &["tiny"],
        "fn twice(x: u8) -> u8 { x + 1 }\npub fn a(x: u8) -> u8 { twice(x) }\npub fn b(x: u8) -> u8 { twice(x) }\n\
         fn shown(x: u8) -> u8 { x }\npub fn c(x: u8) -> u8 { println!(\"{}\", shown(x)); shown(x) }\n\
         fn step(v: &[u8]) -> u8 { let mut total = 0u8; for x in v { total = total.wrapping_add(*x); } total }\n\
         pub fn d(v: &[u8]) -> u8 { step(v) }\n",
    );
}

/// A private function of 20 or more canonical tokens named `name` that
/// calls `next`, or sums its input when `next` is empty.
fn step(name: &str, next: &str) -> String {
    let inner = if next.is_empty() {
        "*x".to_owned()
    } else {
        format!("{next}(&[*x])")
    };
    format!(
        "fn {name}(v: &[u8]) -> u8 {{ let mut total = 0u8; for x in v {{ total = total.wrapping_add({inner}); }} total }}\n"
    )
}

#[test]
fn a_chain_of_three_single_caller_functions_is_one_lead() {
    let chain = format!("{}{}{}", step("a", "b"), step("b", "c"), step("c", ""));
    finds(
        "one-observed-caller",
        &format!("{chain}pub fn api(v: &[u8]) -> u8 {{ a(v) }}\n"),
        &["a, b, c"],
        &format!(
            "{}{}pub fn api(v: &[u8]) -> u8 {{ a(v) }}\n",
            step("a", "b"),
            step("b", "")
        ),
    );
}

#[test]
fn a_type_alias_written_once_has_one_observed_alias_use() {
    let alias = "pub type Pair = (u8, u8);\npub fn swap(p: Pair) -> (u8, u8) { (p.1, p.0) }\n";
    finds(
        "one-observed-alias-use",
        alias,
        &["Pair"],
        &format!("{alias}pub fn first(p: Pair) -> u8 {{ p.0 }}\n"),
    );
}
