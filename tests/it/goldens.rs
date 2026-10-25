//! Compares every fixture's outlines with its hand-written goldens.

use std::fs;

use crate::support::{fixtures, has_ext, input_of, outputs};

/// Runs one fixture and compares each markdown file, and each HTML file
/// that has a golden, byte for byte.
fn check(name: &str) {
    let dir = fixtures().join(name);
    let written = outputs(&input_of(&dir), false);
    let markdown: Vec<&(String, String)> =
        written.iter().filter(|(n, _)| has_ext(n, "md")).collect();
    assert!(!markdown.is_empty(), "{name} wrote no markdown file");
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
        if ext == "html" && !golden.exists() {
            continue;
        }
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
    base_lists => "base-lists",
    cfg_attr => "cfg-attr",
    cfg_test => "cfg-test",
    comments_attach => "comments-attach",
    comments_impls => "comments-impls",
    comments_kinds => "comments-kinds",
    consts => "consts",
    doc_forms => "doc-forms",
    dual => "dual",
    enums => "enums",
    extern_block => "extern-block",
    functions => "functions",
    impls => "impls",
    long_lines => "long-lines",
    macros => "macros",
    member_order => "member-order",
    modules => "modules",
    multi => "multi",
    names => "names",
    order_cycles => "order-cycles",
    order_levels => "order-levels",
    prompt => "prompt",
    raw_idents => "raw-idents",
    receivers => "receivers",
    reexports => "reexports",
    references => "references",
    single => "single",
    special_members => "special-members",
    struct_class => "struct-class",
    structs => "structs",
    traits => "traits",
    unions => "unions",
    unparsable => "unparsable",
}
