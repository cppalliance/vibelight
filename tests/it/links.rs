//! Checks source links: their golden, their separation from everything
//! else, and that every target resolves.

use std::fs;
use std::path::Path;

use crate::support::{
    copy_dir, fixture_names, fixtures, has_ext, input_of, read_outputs, run, strip_html_links,
    strip_markdown_links,
};

#[test]
fn links_golden_matches_with_the_output_beside_the_crate() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let copy = tmp.path().join("links");
    copy_dir(&fixtures().join("links"), &copy);
    let out = copy.join("out");
    assert!(run(&copy, &out, true).status.success());
    let actual = fs::read_to_string(out.join("links.md")).expect("the outline was written");
    let expected = fs::read_to_string(copy.join("expected.md")).expect("the golden is readable");
    assert!(
        actual == expected,
        "--- expected\n{expected}\n--- actual\n{actual}"
    );
}

#[test]
fn removing_the_source_links_gives_the_no_links_output() {
    for name in fixture_names() {
        let input = input_of(&fixtures().join(&name));
        let tmp = tempfile::tempdir().expect("a temporary folder can be made");
        let with = tmp.path().join("with");
        let without = tmp.path().join("without");
        assert!(run(&input, &with, true).status.success());
        assert!(run(&input, &without, false).status.success());
        for ((file, linked), (_, plain)) in read_outputs(&with).iter().zip(&read_outputs(&without))
        {
            let stripped = if has_ext(file, "md") {
                strip_markdown_links(linked)
            } else {
                strip_html_links(linked)
            };
            assert!(
                stripped == *plain,
                "{name}/{file}: stripping links left a difference\n{stripped}"
            );
        }
    }
}

/// Returns every source link target in a markdown outline, fragment removed.
fn targets(md: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = md;
    while let Some(at) = rest.find("](") {
        let after = &rest[at + 2..];
        let end = after.find(')').expect("a link closes");
        let target = &after[..end];
        if !target.starts_with('#') {
            out.push(target.split('#').next().unwrap_or_default().to_owned());
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
        assert!(run(&crate_dir, &out, true).status.success());
        let md = fs::read_to_string(out.join("links.md")).expect("the outline was written");
        let found = targets(&md);
        assert!(found.len() > 5, "too few links in {}", out.display());
        for target in found {
            let path = Path::new(&out).join(&target);
            assert!(
                path.exists(),
                "{target} does not resolve from {}",
                out.display()
            );
            assert!(
                !target.contains('\\') && !target.contains("?\\"),
                "{target}"
            );
        }
        assert!(
            md.contains("[src/lib.rs:6]("),
            "link text is relative to the crate"
        );
    }
}
