//! Tests each detector of names, attributes, placement, and visibility on
//! a small library where it finds its lead, and on a close one where it
//! finds none.

use crate::leads::Mode;
use crate::leads_tests::{MANIFEST, found, lib, with_leads};

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

#[test]
fn a_deprecated_item_or_a_whole_shim_word_is_a_compat_shim() {
    finds(
        "compat-shim",
        "#[deprecated]\npub fn swap() {}\npub struct LegacyConfig;\npub fn parse_v2() {}\n",
        &["LegacyConfig", "parse_v2", "swap"],
        "pub fn bold() {}\npub struct Holder;\npub fn parse_v20() {}\n",
    );
}

#[test]
fn an_allow_or_an_expect_attribute_is_a_suppression() {
    finds(
        "lint-suppression",
        "#[allow(dead_code)]\nfn unused() {}\n\
         pub struct S { #[expect(dead_code, reason = \"kept\")] x: u8 }\n",
        &["S::x", "unused"],
        "#[deny(missing_docs)]\npub fn kept() {}\n#[warn(unused)]\npub fn used() {}\n",
    );
}

#[test]
fn a_test_word_in_a_production_name_is_a_knob_unless_its_module_is_about_it() {
    finds(
        "test-knob",
        "pub struct Client { pub mock_server: bool }\npub fn reset_for_test() {}\n",
        &["Client::mock_server", "reset_for_test"],
        "#[cfg(any(test, feature = \"support\"))]\npub fn reset_for_test() {}\n\
         pub mod overrides { pub struct ModelOverride; }\npub fn attest() {}\n",
    );
}

#[test]
fn an_item_named_like_a_std_type_shadows_it_but_a_result_alias_does_not() {
    finds(
        "prelude-name",
        "pub struct Option;\npub enum Duration { Short }\npub fn drop() {}\n",
        &["drop", "Duration", "Option"],
        "pub type Result<T> = std::result::Result<T, String>;\npub struct Options;\n",
    );
}

#[test]
fn one_path_for_a_module_and_a_function_or_one_name_for_two_kinds_collides() {
    finds(
        "name-collision",
        "pub mod parse { pub fn inner() {} }\npub fn parse() {}\n\
         pub mod load {}\nmod imp { pub fn load() {} }\npub use imp::load;\n\
         pub struct Kind;\npub mod other { pub enum Kind { A } }\n",
        &["imp::load, load", "Kind, other::Kind", "parse"],
        "pub mod parse { pub fn parse() {} }\npub struct Kind;\npub mod other { pub struct Kind; }\n",
    );
}

#[test]
fn one_name_for_two_core_types_is_a_lead_but_owned_and_borrowed_forms_are_one() {
    finds(
        "same-name-many-types",
        "pub struct Span { pub line: u32 }\npub fn goto(line: usize) {}\n",
        &["goto, Span::line"],
        "pub struct Span { pub line: u32, pub name: String }\n\
         pub fn goto(line: &u32, name: &str) {}\n\
         pub fn first(value: u8) {}\npub fn second(value: String) {}\n",
    );
}

#[test]
fn one_type_under_three_names_that_share_a_word_is_a_lead() {
    finds(
        "same-type-many-role-names",
        "pub struct Span { pub start_line: u32, pub end_line: u32 }\npub fn goto(line: u32) {}\n",
        &["goto, Span::end_line, Span::start_line"],
        "pub struct Span { pub start: u32, pub end: u32 }\npub fn goto(line: u32) {}\n",
    );
}

#[test]
fn a_role_named_in_one_synonym_and_typed_in_another_mixes_them() {
    finds(
        "synonym-mix",
        "pub enum ArgType { Text }\npub struct Arg { pub kind: ArgType }\n",
        &["Arg::kind"],
        "pub enum ArgType { Text }\npub struct Arg { pub ty: ArgType }\n\
         pub struct BodyText;\npub struct Section { pub text: BodyText }\n",
    );
}

#[test]
fn a_function_beside_its_inner_twin_is_a_wrapper_family() {
    finds(
        "wrapper-name-family",
        "pub fn parse() {}\nfn parse_inner() {}\n",
        &["parse, parse_inner"],
        "pub fn parse() {}\nmod other { pub fn parse_inner() {} }\n",
    );
}

#[test]
fn get_prefixed_getters_beside_bare_ones_mix_styles() {
    let bare = "pub struct A { name: String }\nimpl A { pub fn name(&self) -> &str { &self.name } }\n\
         pub struct B { size: u8 }\n";
    finds(
        "getter-style-mix",
        &format!("{bare}impl B {{ pub fn get_size(&self) -> u8 {{ self.size }} }}\n"),
        &["B::get_size"],
        &format!("{bare}impl B {{ pub fn size(&self) -> u8 {{ self.size }} }}\n"),
    );
}

#[test]
fn inherent_impls_in_three_files_are_spread() {
    let files = |modules: &[&str]| {
        let declared: Vec<String> = modules.iter().map(|m| format!("mod {m};")).collect();
        let lib = format!(
            "pub struct Wide;\nimpl Wide {{ pub fn a(&self) {{}} }}\n{}\n",
            declared.join("\n")
        );
        let mut files = Vec::new();
        for m in modules {
            files.push((
                format!("src/{m}.rs"),
                format!("impl crate::Wide {{ pub fn {m}(&self) {{}} }}\n"),
            ));
        }
        let mut all: Vec<(&str, &str)> = vec![MANIFEST, ("src/lib.rs", &lib)];
        all.extend(
            files
                .iter()
                .map(|(path, text)| (path.as_str(), text.as_str())),
        );
        found(&all, "impl-spread")
    };
    assert_eq!(files(&["b", "c"]), ["Wide"]);
    assert_eq!(files(&["b"]), Vec::<String>::new());
}

#[test]
fn a_module_of_one_item_not_named_after_it_is_a_lead() {
    finds(
        "single-item-module",
        "pub mod helpers { pub fn trim() {} }\n",
        &["helpers"],
        "pub mod helpers { pub fn trim() {} pub fn pad() {} }\n\
         pub mod idle_reader { pub struct IdleReader; }\n",
    );
}

#[test]
fn a_pub_item_no_public_path_reaches_is_off_the_surface() {
    let hidden = "mod inner { pub struct Hidden; }\npub fn make() {}\n";
    with_leads(&[MANIFEST, ("src/lib.rs", hidden)], |_, leads| {
        let lead = leads.iter().find(|l| l.id == "pub-off-surface").unwrap();
        assert_eq!(
            (lead.mode, lead.limits.as_slice()),
            (Mode::Resolved, &["macros"][..])
        );
    });
    finds(
        "pub-off-surface",
        hidden,
        &["inner::Hidden"],
        "mod inner { pub struct Shown; }\npub use inner::Shown;\nmod sealed { pub trait Sealed {} }\n",
    );
}

#[test]
fn non_exhaustive_off_every_surface_is_hidden() {
    finds(
        "hidden-non-exhaustive",
        "mod inner { #[non_exhaustive] pub struct Hidden { pub x: u8 } }\n",
        &["inner::Hidden"],
        "#[non_exhaustive]\npub struct Shown { pub x: u8 }\n",
    );
}

#[test]
fn a_string_that_names_a_kind_or_a_state_is_stringly_typed_but_not_on_the_wire() {
    finds(
        "stringly-typed",
        "pub struct Job { pub status: String }\npub fn run(mode: &str) {}\n",
        &["Job::status", "run"],
        "pub enum Status { Done }\npub struct Job { pub status: Status, pub name: String }\n\
         #[derive(serde::Deserialize)]\npub struct Wire { pub status: String }\n",
    );
}

#[test]
fn sibling_modules_that_import_each_other_form_a_cycle() {
    finds(
        "module-cycle",
        "mod a { use crate::b::Bee; pub struct Ay; }\nmod b { use crate::a::Ay; pub struct Bee; }\n",
        &["a, b"],
        "mod a { use crate::b::Bee; pub struct Ay; }\nmod b { pub struct Bee; }\n\
         pub mod c { pub struct Sea; pub mod d { use super::Sea; } }\npub use c::d;\n",
    );
}
