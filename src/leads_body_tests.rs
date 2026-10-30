//! Tests each detector of bodies on a small library where it finds its
//! lead, and on a close one where it finds none.

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

/// Asserts the mode, limits, and evidence of the one lead with `id` in a
/// library.
fn shows(source: &str, id: &str, mode: Mode, limits: &[&str], evidence: &str) {
    with_leads(&[MANIFEST, ("src/lib.rs", source)], |_, leads| {
        let lead = leads.iter().find(|l| l.id == id).unwrap();
        assert_eq!((lead.mode, lead.limits.as_slice()), (mode, limits));
        assert_eq!(lead.evidence, evidence);
    });
}

const STORE: &str =
    "pub mod store { pub fn save(path: &str, data: &[u8]) -> bool { path.len() > data.len() } }\n";

#[test]
fn a_body_that_only_passes_its_parameters_on_or_reads_a_private_field_forwards() {
    finds(
        "forwarder",
        &format!(
            "{STORE}pub fn persist(path: &str, data: &[u8]) -> bool {{ store::save(path, data) }}\n\
             pub struct Cache {{ size: usize }}\n\
             impl Cache {{\n\
             fn size(&self) -> usize {{ self.size }}\n\
             pub fn keep(&self, path: &str, data: &[u8]) -> bool {{ store::save(path, data) }}\n\
             }}\n"
        ),
        &["Cache::keep", "Cache::size", "persist"],
        &format!(
            "{STORE}pub fn persist(path: &str, data: &[u8]) -> bool {{ store::save(path, &data[1..]) }}\n\
             pub fn twice(path: &str) -> bool {{ store::save(path, path.as_bytes()) }}\n\
             pub fn wrap(path: &str) -> Option<&str> {{ Some(path) }}\n\
             #[cfg(unix)]\npub fn native(path: &str, data: &[u8]) -> bool {{ store::save(path, data) }}\n\
             pub struct Cache {{ size: usize, pub(crate) limit: usize }}\n\
             impl Cache {{ pub fn size(&self) -> usize {{ self.size }} fn limit(&self) -> usize {{ self.limit }} }}\n"
        ),
    );
}

#[test]
fn bodies_of_one_token_shape_are_clones() {
    let even = "pub fn total_even(items: &[u32]) -> u32 {\n\
         let mut sum = 0;\n\
         for item in items { if item % 2 == 0 { sum += item * 3; } else { sum += 1; } }\n\
         let half = sum / 2;\n\
         half + sum\n\
         }\n";
    finds(
        "body-clone",
        &format!(
            "{even}pub fn total_odd(values: &[u64]) -> u64 {{\n\
             let mut acc = 7;\n\
             for value in values {{ if value % 3 == 1 {{ acc += value * 5; }} else {{ acc += 2; }} }}\n\
             let part = acc / 4;\n\
             part + acc\n\
             }}\n"
        ),
        &["total_even, total_odd"],
        &format!(
            "{even}pub fn describe(items: &[u32]) -> String {{\n\
             let mut out = String::new();\n\
             for item in items {{ out.push_str(&item.to_string()); out.push(','); }}\n\
             out.trim_end_matches(',').to_owned()\n\
             }}\n"
        ),
    );
}

#[test]
fn a_pub_result_function_with_no_error_path_of_its_own_is_a_lead() {
    let config = "pub struct Config { pub port: usize }\n";
    let load =
        "pub fn load(text: &str) -> Result<Config, String> { Ok(Config { port: text.len() }) }\n";
    finds(
        "no-direct-error-path",
        &format!("{config}{load}"),
        &["load"],
        &format!(
            "{config}pub fn load(text: &str) -> Result<Config, String> {{\n\
             if text.is_empty() {{ return Err(String::new()); }}\n\
             Ok(Config {{ port: 1 }})\n\
             }}\n\
             pub fn parse(text: &str) -> Result<usize, String> {{ Ok(text.parse::<usize>().map_err(|e| e.to_string())? + 1) }}\n\
             pub fn first(text: &str) -> Result<char, String> {{ Ok(text.chars().next().ok_or(String::new())?) }}\n\
             pub fn reload(text: &str) -> Result<Config, String> {{ if text.is_empty() {{ Ok(Config {{ port: 0 }}) }} else {{ load(text) }} }}\n\
             fn hidden() -> Result<Config, String> {{ Ok(Config {{ port: 2 }}) }}\n"
        ),
    );
    shows(
        &format!("{config}{load}"),
        "no-direct-error-path",
        Mode::Resolved,
        &["dynamic-dispatch", "macros"],
        "returns `Result<Config,String>` and calls `Ok`, with no `?`, `Err`, or error constructor; a returned callee may still fail",
    );
}

#[test]
fn a_parameter_only_passed_on_through_two_functions_is_tramp_data() {
    let chain = |section_line: &str| {
        format!(
            "pub struct Ctx {{ pub depth: usize }}\n\
             pub fn render(ctx: &Ctx, name: &str) -> String {{ let title = name.to_uppercase(); section(ctx, &title) }}\n\
             fn section(ctx: &Ctx, title: &str) -> String {{ let line = {section_line}; leaf(ctx, &line) }}\n\
             fn leaf(ctx: &Ctx, line: &str) -> String {{ format!(\"{{}}{{}}\", ctx.depth, line) }}\n"
        )
    };
    finds(
        "tramp-param",
        &chain("title.repeat(2)"),
        &["render"],
        &chain("title.repeat(ctx.depth)"),
    );
    let variant = chain("title.repeat(2)").replace("section", "render_with");
    assert_eq!(lib(&variant, "tramp-param"), Vec::<String>::new());
}

#[test]
fn every_call_of_a_private_function_passing_one_literal_is_a_constant_argument() {
    let pad = "fn pad(text: &str, width: usize) -> String { format!(\"{text:>width$}\") }\n\
         pub fn left(a: &str) -> String { pad(a, 8) }\n";
    let positive = format!("{pad}pub fn right(b: &str) -> String {{ pad(b, 8) }}\n");
    finds(
        "constant-arg",
        &positive,
        &["pad"],
        &format!("{pad}pub fn right(b: &str) -> String {{ pad(b, 4) }}\n"),
    );
    shows(
        &positive,
        "constant-arg",
        Mode::Resolved,
        &["macros"],
        "all 2 observed calls pass `8` as `width`",
    );
    let escaped = format!("{positive}pub fn pick() -> fn(&str, usize) -> String {{ pad }}\n");
    assert_eq!(lib(&escaped, "constant-arg"), Vec::<String>::new());
    let tested = format!(
        "{positive}#[cfg(test)]\nmod tests {{ #[test] fn wide() {{ super::pad(\"x\", 3); }} }}\n"
    );
    assert_eq!(lib(&tested, "constant-arg"), Vec::<String>::new());
}

#[test]
fn a_long_string_in_three_bodies_or_two_targets_is_repeated() {
    let two = "pub fn a() -> &'static str { \"config.toml\" }\n\
         pub fn b() -> String { \"config.toml\".to_owned() }\n";
    finds(
        "repeated-literal",
        &format!("{two}pub fn c() -> bool {{ std::path::Path::new(\"config.toml\").exists() }}\n"),
        &["a, b, c"],
        &format!(
            "{two}pub fn c() -> &'static str {{ \"a.toml\" }}\n\
             pub fn d() -> &'static str {{ \"a.toml\" }}\npub fn e() -> &'static str {{ \"a.toml\" }}\n"
        ),
    );
    let across = [
        MANIFEST,
        (
            "src/lib.rs",
            "pub fn a() -> &'static str { \"config.toml\" }\n",
        ),
        ("src/main.rs", "fn main() { println!(\"config.toml\"); }\n"),
    ];
    assert_eq!(found(&across, "repeated-literal"), ["0:a, 1:main"]);
    let table = |name: &str| {
        format!(
            "pub fn {name}(n: u8) -> &'static str {{ match n {{ 0 => \"first one\", 1 => \"second one\", _ => \"{{n}}: {{n:?}}\" }} }}\n"
        )
    };
    let tables = format!("{}{}{}", table("a"), table("b"), table("c"));
    assert_eq!(lib(&tables, "repeated-literal"), ["a, b, c"]);
    shows(
        &tables,
        "repeated-literal",
        Mode::DirectSyntax,
        &[],
        "2 literals in 3 bodies: \"first one\", \"second one\"",
    );
}

#[test]
fn a_number_other_than_zero_one_or_two_in_three_bodies_is_repeated() {
    let three = "pub fn a() -> usize { 1024 }\npub fn b(x: usize) -> usize { x * 1024 + 7 }\n\
         pub fn c() -> Vec<u8> { vec![7; 1024] }\n";
    finds(
        "repeated-number",
        three,
        &["a, b, c"],
        "pub fn a() -> f64 { 2.0 - 1.0 }\npub fn b(x: i64) -> i64 { x * -1 + 2 }\n\
         pub fn c() -> u8 { 2 }\npub const LIMIT: usize = 1024;\n\
         pub fn d() -> usize { 1024 }\npub fn e() -> usize { 1024 }\n\
         pub enum A { X = 7 }\npub enum B { Y = 7 }\npub enum C { Z = 7 }\n",
    );
    shows(
        &format!("{three}pub fn d() -> usize {{ 7 }}\n"),
        "repeated-number",
        Mode::DirectSyntax,
        &[],
        "`1024` in 3 bodies",
    );
}

#[test]
fn a_doc_three_times_longer_than_its_function_body_outweighs_it() {
    let prose = |n: usize| vec!["word"; n].join(" ");
    let doc = |text: &str| format!("/// {text}\npub fn tiny() -> u8 {{ 1 }}\n");
    finds(
        "doc-outweighs-body",
        &doc(&prose(100)),
        &["tiny"],
        &format!(
            "/// {}\n/// ```\n/// {}\n/// ```\npub fn tiny() -> u8 {{ 1 }}\n\
             /// {}\npub const BIG: u8 = 1;\n",
            prose(60),
            prose(60),
            prose(120)
        ),
    );
}
