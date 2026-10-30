//! Checks source locations and links: their golden, that normalizing the
//! other goldens leaves no output folder or source line behind, and that
//! every location and link resolves. The markdown's header `root` leads
//! from the markdown folder to the crate, each location string names a
//! file below it, and nothing else in the markdown depends on where the
//! run writes; the HTML holds each link in an `href`.

use std::fs;

use crate::support::{
    copy_dir, fixture_names, fixtures, has_ext, input_of, locations, read_plain, read_run, root_of,
    run, split_location,
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
        for (file, text) in read_run(&out) {
            assert!(!text.contains("\"url\":"), "{name}/{file} has a url");
            if text.contains("\"root\":\"") || text.contains("class=\"src\" href=") {
                linked += 1;
            }
        }
        for (file, text) in read_plain(&out) {
            for leftover in [
                "\"root\":",
                "<p class=\"source\">",
                "class=\"src\" href=",
                "(source: <a",
            ] {
                assert!(!text.contains(leftover), "{name}/{file} keeps {leftover}");
            }
            let lines = text
                .match_indices("\"line\":")
                .map(|(at, key)| at + key.len());
            for at in lines {
                assert!(
                    text[at..].starts_with('0'),
                    "{name}/{file} keeps a source line"
                );
            }
            if has_ext(&file, "md") {
                for loc in locations(&text) {
                    assert!(loc.ends_with(":0"), "{name}/{file} keeps {loc}");
                }
            }
        }
    }
    assert!(linked > 50, "only {linked} files had source links");
}

/// Returns every source link target in an HTML page, fragment removed,
/// less the links inside the page and to other pages of the run.
fn targets(page: &str) -> Vec<String> {
    let open = "href=\"";
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(at) = rest.find(open) {
        let after = &rest[at + open.len()..];
        let end = after.find('"').expect("a link closes");
        let target = after[..end].split('#').next().unwrap_or_default();
        if !target.is_empty() && !has_ext(target, "html") {
            out.push(target.to_owned());
        }
        rest = &after[end..];
    }
    out
}

/// Returns an outline with its header's `root` taken out.
fn rootless(md: &str) -> String {
    let root = serde_json::to_string(&root_of(md)).expect("a root is JSON");
    let root = format!(",\"root\":{root}");
    md.replacen(&root, "", 1)
}

#[test]
fn locations_and_links_resolve_from_inside_beside_and_above_the_crate() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let crate_dir = tmp.path().join("work").join("links");
    copy_dir(&fixtures().join("links"), &crate_dir);
    let places = [
        crate_dir.join("out"),
        tmp.path().join("work").join("beside"),
        tmp.path().join("work"),
    ];
    let mut roots = Vec::new();
    let mut markdown = Vec::new();
    for out in places {
        assert!(run(&crate_dir, &out).status.success());
        let md = fs::read_to_string(out.join("links.md")).expect("the outline was written");
        let root = root_of(&md);
        assert!(!root.contains('\\'), "{root}");
        let found = locations(&md);
        assert!(found.len() > 5, "too few locations in {}", out.display());
        for loc in found {
            let (stem, file) = split_location(&loc);
            assert_eq!(stem, None, "{loc}");
            assert!(
                out.join(&root).join(file).is_file(),
                "{root} and {loc} do not resolve from {}",
                out.display()
            );
        }
        assert!(
            md.contains("\"loc\":\"src/lib.rs:6\""),
            "a location's file is relative to the crate"
        );
        let html = out.join("html");
        let page = fs::read_to_string(html.join("links.html")).expect("the page was written");
        let found = targets(&page);
        assert!(found.len() > 5, "too few links in {}", html.display());
        for target in found {
            assert!(
                html.join(&target).exists(),
                "{target} does not resolve from {}",
                html.display()
            );
            assert!(
                !target.contains('\\') && !target.contains("?\\"),
                "{target}"
            );
        }
        roots.push(root);
        markdown.push(rootless(&md));
    }
    assert_eq!(roots, ["..", "../links", "links"]);
    assert!(
        markdown.iter().all(|md| *md == markdown[0]),
        "only the header's root depends on where the run writes"
    );
}
