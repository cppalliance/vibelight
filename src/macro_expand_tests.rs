//! Tests the macro look-through: which definition an invocation names,
//! which invocations stay marks and why, and where the items an expansion
//! writes land.

use crate::analysis_tests::{fixture, run};
use crate::collect_body_facts::tests::collected;
use crate::leads;
use crate::leads_tests::MANIFEST;
use crate::model::Crate;

/// A macro that writes one unit struct of the name it is given.
const MAKE: &str = "macro_rules! make { ($name:ident) => { pub struct $name; }; }\n";

/// Returns each declaration's path and the real line it maps to, sorted.
fn decls(krate: &Crate) -> Vec<(String, u32)> {
    let mut out: Vec<(String, u32)> = (0..krate.decls.len())
        .map(|d| {
            let line = krate.real_loc(krate.decls[d].loc).line;
            (krate.decl_path(d).join("::"), line)
        })
        .collect();
    out.sort();
    out
}

/// Returns the crate's warnings as stderr prints them, less the prefix.
fn warnings(krate: &Crate) -> Vec<String> {
    (krate.warnings.iter())
        .map(|w| format!("{}:{}: {}", w.file, w.line, w.message))
        .collect()
}

#[test]
fn an_invocation_takes_the_nearest_definition_around_it() {
    let lib = "macro_rules! make { () => { pub struct Outer; }; }\n\
               mod inner {\n    macro_rules! make { () => { pub struct Inner; }; }\n    \
               make!();\n}\nmake!();\n";
    let (_tmp, krate) = collected(lib);
    let expected = [("Outer".to_owned(), 6), ("inner::Inner".to_owned(), 4)];
    assert_eq!(decls(&krate), expected);
    assert_eq!(warnings(&krate), Vec::<String>::new());
    let d = (0..krate.decls.len()).find(|&d| krate.decls[d].name == "Inner");
    let file = krate.decls[d.unwrap()].loc.file;
    assert_eq!(krate.via(file), Some("make!"));
    assert_eq!(krate.files[file].rel, "src/lib.rs");
}

#[test]
fn definitions_in_two_sibling_modules_leave_their_parent_s_invocation_a_mark() {
    let lib = "mod a {\n    macro_rules! make { () => { pub struct A; }; }\n}\n\
               mod b {\n    macro_rules! make { () => { pub struct B; }; }\n}\nmake!();\n";
    let (_tmp, krate) = collected(lib);
    assert_eq!(decls(&krate), []);
    let why = "the crate defines it 2 times, and no module around the invocation does";
    assert_eq!(
        warnings(&krate),
        [format!("src/lib.rs:7: macro `make!` not expanded: {why}")]
    );
}

#[test]
fn an_expansion_that_writes_a_module_or_a_definition_that_does_not_parse_warns() {
    let lib = "macro_rules! hide { () => { mod hidden {} }; }\nhide!();\n\
               macro_rules! odd { ($x:ident) => {}; junk }\nodd!(a);\n";
    let (_tmp, krate) = collected(lib);
    let odd = "unsupported: a rule that is not `(matcher) => {transcriber}`";
    assert_eq!(
        warnings(&krate),
        [
            "src/lib.rs:2: macro `hide!` not expanded: the expansion writes a `mod` item"
                .to_owned(),
            format!("src/lib.rs:4: macro `odd!` not expanded: {odd}"),
        ]
    );
    assert_eq!(krate.modules.len(), 1, "no module joins the tree");
}

#[test]
fn a_lead_names_the_macro_at_its_invocation_and_twins_inside_one_expansion_drop() {
    let lib = "macro_rules! twins {\n    () => {\n        \
               pub struct Left { pub x: u8, pub y: String }\n        \
               pub struct Right { pub y: String, pub x: u8 }\n    };\n}\ntwins!();\n\
               pub struct Up { pub top: u8, pub tag: String }\n\
               pub struct Down { pub tag: String, pub top: u8 }\n\
               mod inner {\n    macro_rules! helper { () => { pub fn assist() {} }; }\n    \
               helper!();\n}\n";
    let fx = fixture(&[MANIFEST, ("src/lib.rs", lib)]);
    let run = run(&fx);
    let found = leads::find(&run, &[]).leads;
    leads::validate(&found, &run).unwrap();
    let twins: Vec<Vec<String>> = (found.iter())
        .filter(|lead| lead.id == "same-shape")
        .map(|lead| lead.subjects.iter().map(|s| s.path.join("::")).collect())
        .collect();
    assert_eq!(twins, [["Down", "Up"]]);
    let off = found.iter().find(|lead| lead.id == "pub-off-surface");
    let off = off.expect("the macro's function is off the surface");
    let evidence = &off.evidence;
    assert!(evidence.ends_with("; written by `helper!`"), "{evidence}");
    assert_eq!((off.locs.len(), off.locs[0].line), (1, 12));
}

#[test]
fn test_only_and_foreign_invocations_stay_marks_and_a_cfg_reaches_the_items() {
    let lib = format!(
        "{MAKE}#[cfg(test)]\nmake!(Tested);\nother::make!(Foreign);\n\
         #[cfg(feature = \"x\")]\nmake!(Gated);\n"
    );
    let (_tmp, krate) = collected(&lib);
    assert_eq!(decls(&krate), [("Gated".to_owned(), 6)]);
    assert_eq!(krate.decls[0].meta.cfgs, ["feature = \"x\""]);
    assert_eq!(warnings(&krate), Vec::<String>::new());
}
