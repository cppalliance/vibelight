//! Checks source links: their golden, that stripping them for the other
//! goldens leaves none behind, and that every target resolves.

use std::fs;
use std::path::Path;

use crate::support::{
    copy_dir, fixture_names, fixtures, has_ext, input_of, read_plain, read_run, run,
};

#[test]
fn links_golden_matches_with_the_output_beside_the_crate() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let copy = tmp.path().join("links");
    copy_dir(&fixtures().join("links"), &copy);
    let out = copy.join("out");
    assert!(run(&copy, &out).status.success());
    for (written, golden) in [
        (out.join("links.md"), "expected.md"),
        (out.join("html").join("links.html"), "expected.html"),
    ] {
        let actual = fs::read_to_string(&written).expect("the outline was written");
        let expected = fs::read_to_string(copy.join(golden)).expect("the golden is readable");
        assert!(
            actual == expected,
            "{golden}\n--- expected\n{expected}\n--- actual\n{actual}"
        );
    }
}

#[test]
fn stripping_the_links_leaves_no_source_link_in_either_format() {
    let mut linked = 0;
    for name in fixture_names() {
        let tmp = tempfile::tempdir().expect("a temporary folder can be made");
        let out = tmp.path().join("out");
        assert!(
            run(&input_of(&fixtures().join(&name)), &out)
                .status
                .success()
        );
        for (_, text) in read_run(&out) {
            if text.contains("Source: [") || text.contains("class=\"src\" href=") {
                linked += 1;
            }
        }
        for (file, text) in read_plain(&out) {
            for leftover in [
                "Source: [",
                " (source: [",
                "<p class=\"source\">",
                "class=\"src\" href=",
                "(source: <a",
            ] {
                assert!(!text.contains(leftover), "{name}/{file} keeps {leftover}");
            }
        }
    }
    assert!(linked > 50, "only {linked} files had source links");
}

/// Returns every source link target in an outline, fragment removed:
/// `[text](target)` links in markdown, and `href` values in HTML, less the
/// links inside the page and to other pages of the run.
fn targets(file: &str, page: &str) -> Vec<String> {
    let (open, close) = if has_ext(file, "html") {
        ("href=\"", '"')
    } else {
        ("](", ')')
    };
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(at) = rest.find(open) {
        let after = &rest[at + open.len()..];
        let end = after.find(close).expect("a link closes");
        let target = after[..end].split('#').next().unwrap_or_default();
        if !target.is_empty() && !has_ext(target, "html") {
            out.push(target.to_owned());
        }
        rest = &after[end..];
    }
    out
}

#[test]
fn targets_resolve_from_inside_beside_and_above_the_crate() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let crate_dir = tmp.path().join("work").join("links");
    copy_dir(&fixtures().join("links"), &crate_dir);
    let places = [
        crate_dir.join("out"),
        tmp.path().join("work").join("beside"),
        tmp.path().join("work"),
    ];
    for out in places {
        assert!(run(&crate_dir, &out).status.success());
        for (file, folder) in [("links.md", out.clone()), ("links.html", out.join("html"))] {
            let page = fs::read_to_string(folder.join(file)).expect("the outline was written");
            let found = targets(file, &page);
            assert!(found.len() > 5, "too few links in {}", folder.display());
            for target in found {
                assert!(
                    Path::new(&folder).join(&target).exists(),
                    "{target} does not resolve from {}",
                    folder.display()
                );
                assert!(
                    !target.contains('\\') && !target.contains("?\\"),
                    "{target}"
                );
            }
            if file == "links.md" {
                assert!(
                    page.contains("[src/lib.rs:6]("),
                    "link text is relative to the crate"
                );
            }
        }
    }
}
