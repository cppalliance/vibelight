//! Compares every fixture's outlines with its hand-written goldens. A
//! golden holds no link or source line, since those depend on where the
//! output is written and how the source is laid out, so the outlines are
//! compared once normalized; the `links` fixture checks the links
//! themselves. Each run also lists its files on stdout, sorted, and says
//! nothing on stderr, but for the fixtures built to warn.

use std::fs;

use crate::support::{fixtures, has_ext, input_of, outputs_with_streams};

/// The fixtures whose runs warn on stderr on purpose and whose own tests in
/// `cli.rs` check every line of it.
const WARNS: [&str; 2] = ["spellings", "unparsable"];

/// The other fixtures whose runs warn on purpose, with every line of it.
const WARNINGS: [(&str, &[&str]); 2] = [
    (
        "base-lists",
        &[
            "warning: 1 type names unresolved, 0 ambiguous; they stay plain",
            "warning: 2 items written more than one way; see spelling records",
            "2 warnings",
        ],
    ),
    (
        "names",
        &[
            "warning: 0 type names unresolved, 1 ambiguous; they stay plain",
            "1 warning",
        ],
    ),
];

/// Runs one fixture and compares each markdown and HTML file with its
/// golden, byte for byte; an output without a golden fails.
fn check(name: &str) {
    let dir = fixtures().join(name);
    let (written, stdout, stderr) = outputs_with_streams(&input_of(&dir));
    let markdown: Vec<&(String, String)> =
        written.iter().filter(|(n, _)| has_ext(n, "md")).collect();
    assert!(!markdown.is_empty(), "{name} wrote no markdown file");
    let mut sorted = stdout.clone();
    sorted.sort();
    assert_eq!(stdout, sorted, "{name}: stdout is not sorted");
    assert_eq!(stdout.len(), written.len() + 2, "{name}: {stdout:?}");
    if !WARNS.contains(&name) {
        let expected = WARNINGS
            .iter()
            .find(|(fixture, _)| *fixture == name)
            .map_or(&[][..], |(_, lines)| *lines);
        assert_eq!(stderr, expected, "{name}: stderr");
    }
    let single = markdown.len() == 1;
    for (file, text) in &written {
        let (stem, ext) = file
            .rsplit_once('.')
            .expect("an output file has an extension");
        let golden = if single {
            dir.join(format!("expected.{ext}"))
        } else {
            dir.join(format!("expected.{stem}.{ext}"))
        };
        let expected = fs::read_to_string(&golden)
            .unwrap_or_else(|_| panic!("{name}: missing golden {}", golden.display()));
        assert!(
            *text == expected,
            "{name}: {file} differs from {}\n--- expected\n{expected}\n--- actual\n{text}",
            golden.display()
        );
    }
}

macro_rules! goldens {
    ($($test:ident => $fixture:literal,)*) => {
        $(
            #[test]
            fn $test() {
                check($fixture);
            }
        )*

        #[test]
        fn every_fixture_has_a_golden_test() {
            let listed = [$($fixture),*];
            for name in crate::support::fixture_names() {
                assert!(listed.contains(&name.as_str()), "fixture {name} has no golden test");
            }
        }
    };
}

goldens! {
    aliases => "aliases",
    attributes => "attributes",
    base_lists => "base-lists",
    body_uses => "body-uses",
    cfg_attr => "cfg-attr",
    cfg_test => "cfg-test",
    comments_attach => "comments-attach",
    comments_groups => "comments-groups",
    comments_impls => "comments-impls",
    comments_kinds => "comments-kinds",
    consts => "consts",
    doc_forms => "doc-forms",
    dual => "dual",
    enums => "enums",
    extern_block => "extern-block",
    facts => "facts",
    field_index => "field-index",
    functions => "functions",
    impls => "impls",
    kill_list => "kill-list",
    long_lines => "long-lines",
    macros => "macros",
    member_order => "member-order",
    member_order_impl => "member-order-impl",
    modules => "modules",
    multi => "multi",
    names => "names",
    order_cycles => "order-cycles",
    order_groups => "order-groups",
    order_modules => "order-modules",
    order_nesting => "order-nesting",
    prelude_order => "prelude-order",
    prompt => "prompt",
    raw_idents => "raw-idents",
    receivers => "receivers",
    reexports => "reexports",
    references => "references",
    single => "single",
    special_members => "special-members",
    spellings => "spellings",
    spellings_none => "spellings-none",
    struct_class => "struct-class",
    structs => "structs",
    surface => "surface",
    test_support => "test-support",
    traits => "traits",
    unions => "unions",
    unparsable => "unparsable",
    value_links => "value-links",
    visibility => "visibility",
}
