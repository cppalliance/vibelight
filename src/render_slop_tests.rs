//! Tests the markdown outline: small crates rendered the way a run renders
//! them, the schema check on outlines that break it, and the schema check
//! on every markdown golden, the inline `ROOTED` golden, and, on request,
//! every outline in a folder.

use std::fs;
use std::path::{Path, PathBuf};

use crate::analysis_tests::{fixture, run};
use crate::render_slop::{AnalysisTarget, RenderInput, render};
use crate::render_slop_check::{Mode, check};
use crate::root::TargetKind;

/// Renders the outline of a one-file library, without links.
fn outline(lib: &str) -> String {
    let fixture = fixture(&[("src/lib.rs", lib)]);
    let run = run(&fixture);
    let leads = crate::leads::find(&run);
    let input = RenderInput {
        analysis_target: AnalysisTarget {
            run: &run,
            target: 0,
        },
        leads: &leads,
        stamp: "2026-01-01T00:00:00Z",
        target_kind: TargetKind::Lib,
        shown_dir: ".".to_owned(),
        link_bases: &[],
    };
    let text = render(&input).unwrap();
    check(&text, Mode::Raw).unwrap_or_else(|why| panic!("{why}\n{text}"));
    text
}

/// Returns the outline's lines of one kind.
fn kind<'t>(text: &'t str, k: &str) -> Vec<&'t str> {
    let open = format!("{{\"k\":\"{k}\",");
    text.lines().filter(|l| l.starts_with(&open)).collect()
}

#[test]
fn a_library_prints_its_header_modules_and_one_record_per_item() {
    let text = outline(
        "//! The crate.\n\n/// A point.\n#[derive(Debug, Clone)]\npub struct Point {\n    \
         /// Across.\n    pub x: i32,\n    y: i32,\n}\n\nimpl Point {\n    /// Makes one.\n    \
         pub fn new(x: i32) -> Self {\n        // keep y at zero\n        Point { x, y: 0 }\n    \
         }\n}\n\nimpl std::fmt::Display for Point {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) \
         -> std::fmt::Result {\n        write!(f, \"secret text\")\n    }\n}\n",
    );
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "*Updated 2026-01-01T00:00:00Z*");
    assert!(lines[1].starts_with("{\"k\":\"vibelight-outline\",\"schema\":1,\"crate\":\""));
    assert!(
        lines[1].ends_with("\",\"target\":\"lib\"}"),
        "no folder: {}",
        lines[1]
    );
    assert!(lines[2].starts_with("{\"k\":\"modules\",\"items\":[{\"path\":\"crate\",\"from\":"));
    let structs = kind(&text, "struct");
    assert_eq!(structs.len(), 1);
    assert!(structs[0].contains("\"rust\":\"pub struct Point { pub x: i32, y: i32 }\""));
    assert!(structs[0].contains("\"derives\":[\"Clone\",\"Debug\"]"));
    let fields = kind(&text, "field");
    assert_eq!(fields.len(), 1, "only the commented field has a record");
    assert!(fields[0].contains("\"path\":\"Point::x\""));
    let fns = kind(&text, "fn");
    assert!(
        fns.iter()
            .any(|l| l.contains("\"sig\":\"pub fn new(x: i32) -> Self\""))
    );
    let fmt = fns
        .iter()
        .find(|l| l.contains("\"path\":\"Point::fmt\""))
        .unwrap();
    assert!(
        fmt.contains("\"impl\":\"impl std::fmt::Display for Point\""),
        "{fmt}"
    );
    assert!(text.contains("\"comments\":[{\"text\":\"// keep y at zero\",\"line\":14}]"));
    assert!(!text.contains("secret text"), "no body code prints");
    let docs = kind(&text, "doc");
    assert!(docs.contains(&"{\"k\":\"doc\",\"path\":\"Point\",\"text\":\"A point.\"}"));
    let after = lines
        .iter()
        .position(|l| l.contains("\"text\":\"A point.\""))
        .unwrap();
    assert!(lines[after - 1].starts_with("{\"k\":\"struct\",\"path\":\"Point\","));
    assert_eq!(kind(&text, "impl").len(), 2);
}

#[test]
fn a_trait_prints_its_header_and_each_item_as_a_record() {
    let text = outline(
        "/// Reads.\npub trait Reads: Clone {\n    /// The item.\n    type Item;\n    \
         const N: usize = 3;\n    fn read(&self) -> Self::Item;\n    fn twice(&self) -> usize { Self::N * 2 }\n}\n",
    );
    let traits = kind(&text, "trait");
    assert_eq!(
        traits,
        [
            "{\"k\":\"trait\",\"path\":\"Reads\",\"rust\":\"pub trait Reads: Clone\",\"loc\":{\"file\":\"src/lib.rs\",\"line\":2}}"
        ]
    );
    assert!(text.contains("{\"k\":\"alias\",\"path\":\"Reads::Item\",\"rust\":\"type Item\""));
    assert!(text.contains(
        "{\"k\":\"const\",\"path\":\"Reads::N\",\"rust\":\"const N: usize\",\"value\":\"3\""
    ));
    let twice = kind(&text, "fn")
        .into_iter()
        .find(|l| l.contains("twice"))
        .unwrap();
    assert!(twice.contains("\"body\":{\"statements\":1,"), "{twice}");
}

#[test]
fn signatures_and_shapes_stay_on_one_line_with_where_clauses_inline() {
    let text = outline(
        "pub enum Shape<T> where T: Copy {\n    Circle { r: T },\n    Square(T),\n    Dot = 3,\n}\n\n\
         pub fn pick<T>(\n    a: T,\n    b: T,\n) -> T\nwhere\n    T: PartialOrd,\n{\n    a\n}\n",
    );
    assert!(text.contains(
        "\"rust\":\"pub enum Shape<T> where T: Copy { Circle { r: T }, Square(T), Dot = 3 }\""
    ));
    assert!(text.contains("\"sig\":\"pub fn pick<T>(a: T, b: T) -> T where T: PartialOrd\""));
}

/// A small outline that meets the schema, to break line by line.
fn sample() -> String {
    outline("/// A.\npub struct A;\n\n/// Runs.\npub fn run(a: u8, b: u8, c: u8) {}\n")
}

#[test]
fn the_check_rejects_every_kind_of_broken_outline() {
    let good = sample();
    check(&good, Mode::Raw).unwrap();
    let broken = |from: &str, to: &str| {
        assert!(good.contains(from), "{from}");
        good.replacen(from, to, 1)
    };
    let cases = [
        (
            broken(
                "\"rust\":\"pub struct A\"",
                "\"rust\":\"pub struct A\",\"x\":1",
            ),
            "unknown",
        ),
        (broken(",\"rust\":\"pub struct A\"", ""), "missing"),
        (broken("*\n", "*\r\n"), "CR"),
        (broken("*\n", "*\n\n"), "blank"),
        (broken("\"schema\":1", "\"schema\":2"), "schema"),
        (broken("{\"k\":\"modules\"", "{\"k\":\"modules\" "), "form"),
        (good.replace("\"line\":2", "\"line\":0"), "raw line"),
    ];
    for (text, what) in cases {
        assert!(check(&text, Mode::Raw).is_err(), "{what} passed:\n{text}");
    }
    let mut lines: Vec<&str> = good.lines().collect();
    lines.swap(1, 2);
    let swapped = format!("{}\n", lines.join("\n"));
    assert!(check(&swapped, Mode::Raw).is_err(), "order passed");
    let mut lines: Vec<&str> = good.lines().collect();
    let last = lines.len() - 1;
    let lead = lines
        .iter()
        .position(|l| l.starts_with("{\"k\":\"lead\""))
        .unwrap();
    lines.swap(lead, last);
    let late = format!("{}\n", lines.join("\n"));
    assert!(check(&late, Mode::Raw).is_err(), "a late lead passed");
    let lead = kind(&good, "lead")[0];
    let twice = lead.replacen("\"subjects\":[\"run\"", "\"subjects\":[\"run\",\"run\"", 1);
    assert!(twice != lead, "{lead}");
    assert!(
        check(&good.replace(lead, &twice), Mode::Raw).is_err(),
        "repeat passed"
    );
    let modules = good.lines().nth(2).unwrap();
    let shifted = modules.replacen("\"from\":", "\"from\":1", 1);
    assert!(
        check(&good.replace(modules, &shifted), Mode::Raw).is_err(),
        "range passed"
    );
    let unsorted = good.replace(
        "\"subjects\"",
        "\"limits\":[\"macros\",\"cfg-selection\"],\"subjects\"",
    );
    assert!(check(&unsorted, Mode::Raw).is_err(), "unsorted passed");
    assert!(
        check(&good, Mode::NormalizedGolden).is_err(),
        "a raw line passed as golden"
    );
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("md"))
}

/// Returns every markdown golden under the fixtures, each with the mode its
/// check takes: `links` keeps its source lines and links.
fn goldens() -> Vec<(PathBuf, Mode)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let in_expected = dir.file_name().is_some_and(|n| n == "expected");
            if path.is_dir() {
                stack.push(path);
            } else if is_markdown(&path) && (name.starts_with("expected") || in_expected) {
                let links = dir.file_name().is_some_and(|n| n == "links");
                let mode = if links {
                    Mode::Raw
                } else {
                    Mode::NormalizedGolden
                };
                out.push((path, mode));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn every_markdown_golden_meets_the_schema() {
    let found = goldens();
    assert_eq!(
        found.len(),
        59,
        "52 targets, links, and six workspace outlines"
    );
    for (path, mode) in found {
        let text = fs::read_to_string(&path).unwrap();
        check(&text, mode).unwrap_or_else(|why| panic!("{}: {why}", path.display()));
    }
}

#[test]
fn the_inline_rooted_golden_meets_the_schema() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("it")
        .join("workspaces.rs");
    let file = syn::parse_file(&fs::read_to_string(path).unwrap()).unwrap();
    let rooted = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Const(c) if c.ident == "ROOTED" => match &*c.expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => None,
            },
            _ => None,
        })
        .expect("workspaces.rs holds ROOTED");
    check(&rooted, Mode::NormalizedGolden).unwrap();
}

/// Checks every outline in the folder `VIBELIGHT_CHECK_DIR` names, as a
/// run wrote them: `cargo test outlines_in_a_folder -- --ignored`.
#[test]
#[ignore = "reads the folder that VIBELIGHT_CHECK_DIR names"]
fn outlines_in_a_folder_meet_the_schema() {
    let dir = PathBuf::from(std::env::var("VIBELIGHT_CHECK_DIR").unwrap());
    let mut checked = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if is_markdown(&path) && name != "index.md" {
            let text = fs::read_to_string(&path).unwrap();
            check(&text, Mode::Raw).unwrap_or_else(|why| panic!("{name}: {why}"));
            checked += 1;
        }
    }
    assert!(checked > 0, "the folder holds no outline");
}
