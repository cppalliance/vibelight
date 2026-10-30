//! Tests each detector of fields, variants, and twins on a small library
//! where it finds its lead, and on a close one where it finds none.

use crate::leads_tests::{MANIFEST, found, lib};

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
fn an_optional_bool_or_unit_is_a_flag_but_not_on_the_wire() {
    finds(
        "option-as-flag",
        "pub struct Config { pub verbose: Option<bool> }\npub fn run(dry: Option<()>) {}\n",
        &["Config::verbose", "run"],
        "pub struct Config { pub verbose: bool, pub level: Option<u8> }\n\
         pub fn run(dry: bool) -> Option<u8> { None }\n\
         #[derive(serde::Deserialize)]\npub struct Wire { pub verbose: Option<bool> }\n",
    );
}

#[test]
fn a_shape_whose_every_field_another_holds_is_contained() {
    finds(
        "contained-shape",
        "pub struct Point { pub x: u32, pub y: u32, pub z: u32 }\n\
         pub enum Shape { Dot { x: u32, y: u32, z: u32, size: u8 } }\n",
        &["Point, Shape::Dot"],
        "pub struct Point { pub x: u32, pub y: u32, pub z: u64 }\n\
         pub enum Shape { Dot { x: u32, y: u32, z: u32, size: u8 } }\n",
    );
}

#[test]
fn a_span_beside_a_line_and_its_text_and_a_list_beside_its_length_are_derived() {
    finds(
        "derived-field",
        "pub struct Token { pub span: std::ops::Range<usize>, pub line: u32, src: std::sync::Arc<str> }\n\
         pub struct Mark<'a> { pub range: std::ops::Range<usize>, pub column: u32, pub content: &'a str }\n\
         pub struct List { pub items: Vec<u8>, pub items_len: usize }\n",
        &[
            "List::items, List::items_len",
            "Mark::column, Mark::range",
            "Token::line, Token::span",
        ],
        "pub struct Token { pub span: std::ops::Range<usize>, pub text: String }\n\
         pub struct List { pub items: Vec<u8>, pub total: usize }\n",
    );
}

#[test]
fn a_span_beside_a_line_without_the_text_it_measures_is_not_derived() {
    let errors = "use std::num::NonZeroU32;\nuse std::ops::Range;\n\
         pub enum Error {\n\
         ParseStructured { kind: u8, span: Option<Range<usize>>, message: String, name: Option<String>, \
         line: Option<NonZeroU32>, column: Option<NonZeroU32> },\n\
         }\n\
         pub struct ParseError { kind: u8, span: Option<Range<usize>>, name: Option<String>, \
         line: Option<NonZeroU32>, column: Option<NonZeroU32>, inner: Box<Error> }\n\
         struct Classification { kind: u8, span: Option<Range<usize>>, name: Option<String>, \
         line: Option<NonZeroU32>, column: Option<NonZeroU32> }\n";
    assert_eq!(lib(errors, "derived-field"), Vec::<String>::new());
}

#[test]
fn every_type_that_holds_one_shape_is_in_its_one_lead() {
    let classified = "pub struct Classification { kind: u8, span: u32, name: String }\n\
         pub enum Error { ParseStructured { kind: u8, span: u32, message: String, name: String } }\n\
         pub struct ParseError { kind: u8, span: u32, name: String, inner: Box<Error> }\n";
    assert_eq!(
        lib(classified, "contained-shape"),
        ["Classification, Error::ParseStructured, ParseError"]
    );
}

#[test]
fn an_option_field_of_a_type_with_a_drop_impl_is_a_lead() {
    let guard = "pub struct Guard { pub handle: Option<u32>, pub name: String }\n";
    finds(
        "drop-option",
        &format!("{guard}impl Drop for Guard {{ fn drop(&mut self) {{ self.handle.take(); }} }}\n"),
        &["Guard::handle"],
        guard,
    );
}

#[test]
fn an_accessor_that_fails_only_through_an_option_field_is_a_lead() {
    finds(
        "fallible-accessor",
        "pub struct Conn { socket: Option<u32> }\n\
         impl Conn { pub fn open_socket(&self) -> Result<u32, String> { self.socket.ok_or(String::new()) } }\n",
        &["Conn::open_socket, Conn::socket"],
        "pub struct Conn { socket: Option<u32> }\n\
         impl Conn { pub fn socket(&self) -> Option<u32> { self.socket } }\n",
    );
}

#[test]
fn methods_that_reach_separate_fields_fall_into_clusters() {
    let methods = "pub fn one(&self) -> u8 { self.a }\n\
         pub fn two(&self) -> u8 { self.a + self.b }\n\
         pub fn three(&self) -> u8 { self.c }\n\
         pub fn four(&self) -> u8 { self.c + self.d }\n";
    let engine = "pub struct Engine { a: u8, b: u8, c: u8, d: u8 }\n";
    finds(
        "direct-field-clusters",
        &format!("{engine}impl Engine {{ {methods} }}\n"),
        &["Engine"],
        &format!(
            "{engine}impl Engine {{ {methods} pub fn five(&self) -> u8 {{ self.b + self.c }} }}\n"
        ),
    );
}

#[test]
fn an_enum_of_nothing_or_one_value_is_option_shaped() {
    finds(
        "option-shaped-enum",
        "pub enum Limit { Unlimited, Count(u32) }\n",
        &["Limit"],
        "pub enum Limit { Unlimited, Range(u32, u32) }\n",
    );
}

#[test]
fn structs_with_the_same_fields_in_any_order_have_one_shape() {
    finds(
        "same-shape",
        "pub struct A { pub x: u8, pub y: String }\npub struct B { pub y: String, pub x: u8 }\n",
        &["A, B"],
        "pub struct A { pub x: u8, pub y: String }\npub struct B { pub x: u8, pub z: String }\n",
    );
}

#[test]
fn two_maps_of_different_values_with_the_same_methods_are_generic_twins() {
    let args = "use std::collections::BTreeMap;\npub struct Arg;\npub struct Role;\n\
         pub struct Args { fields: BTreeMap<String, Arg>, implicit: bool }\n\
         impl Args {\n\
         pub fn get(&self, name: &str) -> Option<&Arg> { self.fields.get(name) }\n\
         pub fn len(&self) -> usize { self.fields.len() }\n\
         pub fn is_empty(&self) -> bool { self.fields.is_empty() }\n\
         }\n\
         pub struct Roles { roles: BTreeMap<String, Role> }\n";
    let roles = |key: &str| {
        format!(
            "{args}impl Roles {{\n\
             pub fn get(&self, {key}) -> Option<&Role> {{ None }}\n\
             pub fn len(&self) -> usize {{ self.roles.len() }}\n\
             pub fn is_empty(&self) -> bool {{ self.roles.is_empty() }}\n\
             }}\n"
        )
    };
    finds(
        "generic-twin",
        &roles("label: &str"),
        &["Args, Roles"],
        &roles("index: usize"),
    );
}

#[test]
fn enums_with_the_same_variant_names_mirror_each_other_unless_one_shape_names_them() {
    finds(
        "mirror-enum",
        "pub enum Mode { Fast, Slow(u8) }\npub enum Speed { Slow, Fast }\n",
        &["Mode, Speed"],
        "pub enum Mode { Fast, Slow }\npub enum Speed { Fast, Slow }\npub enum Gear { Fast, Slower }\n",
    );
    let across = [
        MANIFEST,
        ("src/lib.rs", "pub enum Mode { Fast, Slow }\n"),
        ("src/main.rs", "enum Speed { Fast, Slow }\nfn main() {}\n"),
    ];
    assert_eq!(found(&across, "mirror-enum"), ["0:Mode, 1:Speed"]);
}

#[test]
fn a_field_as_visible_as_its_getter_has_a_redundant_accessor() {
    let getter = "impl Item { pub fn name(&self) -> &str { &self.name } }\n";
    finds(
        "field-and-accessor",
        &format!("pub struct Item {{ pub name: String }}\n{getter}"),
        &["Item::name"],
        &format!("pub struct Item {{ pub(crate) name: String }}\n{getter}"),
    );
}

#[test]
fn an_uncommon_item_two_targets_declare_alike_is_a_duplicate() {
    let lib = (
        "src/lib.rs",
        "pub fn checksum_bytes(data: &[u8], seed: u32) -> u32 { seed }\n",
    );
    let main = |args: &str| format!("fn checksum_bytes({args}) -> u32 {{ 0 }}\nfn main() {{}}\n");
    let same = main("data: &[u8], seed: u32");
    let found_in = |main: &str| {
        found(
            &[MANIFEST, lib, ("src/main.rs", main)],
            "cross-crate-duplicate",
        )
    };
    assert_eq!(found_in(&same), ["0:checksum_bytes, 1:checksum_bytes"]);
    assert_eq!(found_in(&main("data: &[u8]")), Vec::<String>::new());
}

#[test]
fn one_signature_under_two_names_in_two_modules_is_a_lead() {
    let modules = |second: &str| {
        format!(
            "pub struct Doc;\n\
             pub mod html {{ pub fn render(doc: &crate::Doc, width: usize) -> String {{ String::new() }} }}\n\
             pub mod text {{ pub fn {second}(doc: &crate::Doc, width: usize) -> String {{ String::new() }} }}\n"
        )
    };
    finds(
        "same-signature",
        &modules("print"),
        &["html::render, text::print"],
        &modules("render"),
    );
}
