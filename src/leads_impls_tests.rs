//! Tests each detector of impls, conversions, parameters, traits, and
//! enum payloads on a small library where it finds its lead, and on a
//! close one where it finds none.

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
fn constructors_of_one_parameter_list_and_a_new_beside_default_are_redundant() {
    finds(
        "redundant-constructor",
        "pub struct Port(u16);\n\
         impl Port { pub fn new(n: u16) -> Self { Port(n) } pub fn from_number(n: u16) -> Self { Port(n) } }\n\
         #[derive(Default)]\npub struct Pool { size: u8 }\n\
         impl Pool { pub fn new() -> Self { Pool { size: 1 } } }\n",
        &["Pool, Pool::new", "Port::from_number, Port::new"],
        "pub struct Port(u16);\n\
         impl Port { pub fn new(n: u16) -> Self { Port(n) } pub fn pair(a: u16, b: u16) -> Self { Port(a) } }\n\
         #[derive(Default)]\npub struct Pool { size: u8 }\n\
         impl Pool { pub fn new() -> Self { Self::default() } }\n\
         pub enum Failure { Read(String), Write(String) }\n\
         impl Failure { pub fn read(m: String) -> Self { Failure::Read(m) } pub fn write(m: String) -> Self { Failure::Write(m) } }\n",
    );
}

#[test]
fn conversions_both_ways_are_an_inverse_pair_but_a_newtype_is_not() {
    let temps = "pub struct Celsius(f64);\npub struct Fahrenheit(f64);\n\
         impl From<Celsius> for Fahrenheit { fn from(c: Celsius) -> Self { Fahrenheit(c.0) } }\n";
    finds(
        "inverse-pair",
        &format!(
            "{temps}impl From<Fahrenheit> for Celsius {{ fn from(f: Fahrenheit) -> Self {{ Celsius(f.0) }} }}\n\
             pub struct Code(u8);\npub enum Kind {{ A }}\n\
             pub fn to_code(kind: Kind) -> Code {{ Code(0) }}\n\
             pub fn from_code(code: Code) -> Kind {{ Kind::A }}\n"
        ),
        &["Celsius, Fahrenheit", "from_code, to_code"],
        &format!(
            "{temps}pub struct Code(u8);\n\
             impl Code {{ pub fn new(n: u8) -> Self {{ Code(n) }} pub fn get(&self) -> u8 {{ self.0 }} }}\n"
        ),
    );
}

#[test]
fn a_unit_enum_spelled_in_two_places_has_a_spelling_table() {
    let color = "pub enum Color { Red, Green }\n\
         impl Color { pub fn as_str(self) -> &'static str { \"red\" } }\n";
    finds(
        "spelling-table",
        &format!(
            "{color}pub fn color_name(color: Color) -> &'static str {{ \"red\" }}\n\
             #[derive(serde::Deserialize)]\n#[serde(rename_all = \"lowercase\")]\n\
             pub enum Shade {{ Light, Dark }}\n\
             impl std::fmt::Display for Shade {{ fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{ Ok(()) }} }}\n"
        ),
        &["Color, Color::as_str, color_name", "Shade"],
        color,
    );
}

#[test]
fn an_impl_file_that_reaches_only_fields_another_file_reaches_is_split() {
    let files = |lib: &str, extra: &str| {
        let lib = format!(
            "pub struct Store {{ a: u8, b: u8, c: u8 }}\nimpl Store {{ {lib} }}\nmod extra;\n"
        );
        let extra = format!("impl crate::Store {{ {extra} }}\n");
        found(
            &[MANIFEST, ("src/lib.rs", &lib), ("src/extra.rs", &extra)],
            "direct-impl-field-split",
        )
    };
    let wide =
        "pub fn all(&self) -> u8 { self.a + self.b + self.c } pub fn first(&self) -> u8 { self.a }";
    let second = "pub fn second(&self) -> u8 { self.b }";
    assert_eq!(files(wide, second), ["Store, Store::second"]);
    let first = "pub fn first(&self) -> u8 { self.a }";
    assert_eq!(files(first, second), Vec::<String>::new());
}

#[test]
fn parameters_that_travel_together_through_three_functions_clump() {
    finds(
        "param-clump",
        "pub struct Ctx;\npub fn a(cx: &Ctx, depth: usize) {}\n\
         pub fn b(cx: &Ctx, depth: usize, x: u8) {}\npub fn c(cx: &Ctx, depth: usize) {}\n",
        &["a, b, c"],
        "pub struct Ctx;\npub fn a(cx: &Ctx, depth: usize) {}\n\
         pub fn b(cx: &Ctx, level: usize) {}\npub fn c(cx: &Ctx, depth: usize) {}\n",
    );
}

#[test]
fn a_tuple_of_three_or_of_two_alike_is_a_tuple_return() {
    finds(
        "tuple-return",
        "pub fn bounds() -> (u32, u32) { (0, 0) }\n\
         pub fn parts() -> Result<(String, u8, bool), ()> { Err(()) }\n",
        &["bounds", "parts"],
        "pub fn pair() -> (u32, String) { (0, String::new()) }\n",
    );
}

#[test]
fn a_mutable_primitive_parameter_is_a_cursor() {
    finds(
        "out-param-cursor",
        "pub fn advance(pos: &mut usize) {}\n",
        &["advance"],
        "pub fn advance(pos: usize) -> usize { pos }\npub fn fill(out: &mut String) {}\n",
    );
}

#[test]
fn three_parameters_of_one_primitive_type_are_a_lead() {
    finds(
        "same-type-params",
        "pub fn paint(red: bool, green: bool, blue: bool) {}\n",
        &["paint"],
        "pub fn paint(red: bool, green: bool, blue: u8) {}\n",
    );
}

#[test]
fn an_unused_parameter_counts_unless_a_trait_or_a_callback_sets_it() {
    finds(
        "underscore-param",
        "pub fn handle(_request: u8) {}\n",
        &["handle"],
        "pub trait Hook { fn on(&self, value: u8); }\npub struct Quiet;\n\
         impl Hook for Quiet { fn on(&self, _value: u8) {} }\n\
         pub fn callback(_event: u8) {}\n\
         pub fn register() { let _f: fn(u8) = callback; }\n",
    );
}

#[test]
fn error_types_that_share_a_variant_are_a_family() {
    finds(
        "error-family",
        "pub enum ReadError { Io(String), Missing }\npub enum WriteError { Io(String), Full }\n",
        &["ReadError, WriteError"],
        "pub enum ReadError { Io(String), Missing }\npub enum WriteError { Disk(String), Full }\n",
    );
}

#[test]
fn variants_that_all_carry_one_payload_are_uniform() {
    finds(
        "uniform-payload",
        "pub enum Event { Start(u64), Stop(u64) }\n",
        &["Event"],
        "pub enum Event { Start(u64), Stop(u32) }\n",
    );
}

#[test]
fn an_enum_of_one_variant_is_a_lead() {
    finds(
        "single-variant-enum",
        "pub enum Only { One }\n",
        &["Only"],
        "pub enum Two { One, Other }\n",
    );
}

#[test]
fn a_trait_with_one_observed_impl_names_its_blind_spots() {
    let store = "pub trait Store { fn get(&self) -> u8; }\npub struct Disk;\n\
         impl Store for Disk { fn get(&self) -> u8 { 1 } }\n";
    let files = [MANIFEST, ("src/lib.rs", store)];
    with_leads(&files, |_, leads| {
        let lead = leads
            .iter()
            .find(|l| l.id == "one-observed-trait-impl")
            .unwrap();
        assert_eq!(
            (lead.mode, lead.limits.as_slice()),
            (Mode::Resolved, &["external-impls", "macros"][..])
        );
    });
    assert_eq!(lib(store, "one-observed-trait-impl"), ["Disk, Store"]);
    let two = format!(
        "{store}pub struct Memory;\nimpl Store for Memory {{ fn get(&self) -> u8 {{ 2 }} }}\n"
    );
    assert_eq!(lib(&two, "one-observed-trait-impl"), Vec::<String>::new());
    let tested = format!(
        "{store}#[cfg(test)]\nmod tests {{ struct Fake; impl super::Store for Fake {{ fn get(&self) -> u8 {{ 0 }} }} }}\n"
    );
    assert_eq!(
        lib(&tested, "one-observed-trait-impl"),
        Vec::<String>::new()
    );
}

#[test]
fn a_one_field_struct_without_methods_that_one_module_uses_is_a_thin_wrapper() {
    finds(
        "thin-wrapper-one-observed-user",
        "struct Meters(u32);\npub fn total() -> u32 { let m = Meters(3); m.0 }\n",
        &["Meters"],
        "struct Meters(u32);\nimpl Meters { fn get(&self) -> u32 { self.0 } }\n\
         pub fn total() -> u32 { Meters(3).get() }\n",
    );
}

#[test]
fn a_hand_written_deserialize_beside_a_wire_twin_or_for_a_tiny_enum_is_manual() {
    finds(
        "manual-serde",
        "use serde::Deserialize;\npub struct Decl { kind: u8 }\n\
         #[derive(Deserialize)]\nstruct DeclWire { kind: u8 }\n\
         impl<'de> Deserialize<'de> for Decl {\n\
         fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {\n\
         let wire = DeclWire::deserialize(deserializer)?;\n\
         Ok(Decl { kind: wire.kind })\n\
         }\n\
         }\n\
         pub enum Mode { On, Off }\n\
         impl<'de> serde::Deserialize<'de> for Mode {\n\
         fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> { todo!() }\n\
         }\n",
        &["Decl, DeclWire", "Mode"],
        "#[derive(serde::Deserialize)]\npub enum Mode { On, Off }\n",
    );
}
