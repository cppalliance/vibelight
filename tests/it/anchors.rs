//! Checks that every link inside each target's two files lands on an
//! anchor, and that ids never repeat.

use std::collections::BTreeSet;

use crate::support::{fixture_names, fixtures, has_ext, input_of, outputs};

/// Returns each target's markdown and HTML text, for every fixture.
fn pairs() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for name in fixture_names() {
        let dir = fixtures().join(&name);
        let written = outputs(&input_of(&dir));
        for (file, md) in written.iter().filter(|(f, _)| has_ext(f, "md")) {
            let stem = file.trim_end_matches(".md");
            let html = written
                .iter()
                .find(|(f, _)| *f == format!("{stem}.html"))
                .map_or_else(
                    || panic!("{name}: {stem}.html is missing"),
                    |(_, t)| t.clone(),
                );
            out.push((format!("{name}/{stem}"), md.clone(), html));
        }
    }
    out
}

/// Returns the inside of every tag in an HTML file. Text never holds a raw
/// `<` and attribute values never hold a raw `>`, so each tag runs from a
/// `<` to the next `>`.
fn tags(html: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find('<') {
        let after = &rest[at + 1..];
        let end = after.find('>').expect("a tag closes");
        out.push(&after[..end]);
        rest = &after[end + 1..];
    }
    out
}

/// Returns the value of a tag's `name="..."` attribute. Values never hold a
/// raw `"`, so each one ends at the next `"`.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = tag.split_once(' ')?.1;
    while let Some((key, after)) = rest.split_once("=\"") {
        let (value, tail) = after.split_once('"').expect("an attribute value closes");
        if key.trim() == name {
            return Some(value);
        }
        rest = tail;
    }
    None
}

/// Returns every `id` an HTML file defines, in document order.
fn ids(html: &str) -> Vec<&str> {
    tags(html)
        .into_iter()
        .filter_map(|tag| attribute(tag, "id"))
        .collect()
}

/// Returns the id each in-page link points at, in document order.
fn link_targets(html: &str) -> Vec<&str> {
    tags(html)
        .into_iter()
        .filter(|tag| tag.starts_with("a "))
        .filter_map(|tag| attribute(tag, "href")?.strip_prefix('#'))
        .collect()
}

#[test]
fn ids_and_links_are_read_only_from_tags() {
    let html = "<h1 id=\"a\"></h1><code>&lt;a id=\"b\" href=\"#c\"&gt;</code><a href=\"#a\"></a>";
    assert_eq!(ids(html), ["a"]);
    assert_eq!(link_targets(html), ["a"]);
}

#[test]
fn ids_are_unique_and_every_link_lands() {
    for (target, md, html) in pairs() {
        let found = ids(&html);
        let defined: BTreeSet<&str> = found.iter().copied().collect();
        assert_eq!(defined.len(), found.len(), "{target}: an id repeats");
        for id in link_targets(&html) {
            assert!(defined.contains(id), "{target}: HTML link to missing #{id}");
        }
        let anchors: BTreeSet<&str> = md
            .lines()
            .filter_map(|l| {
                l.strip_prefix("<a id=\"")
                    .and_then(|r| r.strip_suffix("\"></a>"))
            })
            .collect();
        let mut rest = md.as_str();
        while let Some(at) = rest.find("](#") {
            let after = &rest[at + 3..];
            let end = after.find(')').expect("a markdown link closes");
            assert!(
                anchors.contains(&after[..end]),
                "{target}: markdown link to missing #{}",
                &after[..end]
            );
            rest = &after[end..];
        }
    }
}

const VALUE_KINDS: [&str; 3] = ["fn.", "const.", "static."];

fn is_value_id(id: &str) -> bool {
    VALUE_KINDS.iter().any(|kind| id.starts_with(kind))
}

#[test]
fn every_link_to_a_function_constant_or_static_lands_on_an_id() {
    let mut linked = 0;
    for (target, _, html) in pairs() {
        let defined: BTreeSet<&str> = ids(&html).into_iter().collect();
        for id in link_targets(&html).into_iter().filter(|id| is_value_id(id)) {
            assert!(
                defined.contains(id),
                "{target}: HTML link to missing value #{id}"
            );
            linked += 1;
        }
    }
    assert!(linked >= 10, "the fixtures hold only {linked} value links");
}

#[test]
fn only_the_html_file_holds_ids_for_functions_constants_and_statics() {
    let mut defined = 0;
    for (target, md, html) in pairs() {
        assert!(
            !md.contains("id=\"fn.") && !md.contains("id=\"const.") && !md.contains("id=\"static."),
            "{target}: the markdown holds a value anchor"
        );
        assert!(
            !md.contains("](#fn.") && !md.contains("](#const.") && !md.contains("](#static."),
            "{target}: the markdown links to a value"
        );
        defined += ids(&html).into_iter().filter(|id| is_value_id(id)).count();
    }
    assert!(
        defined >= 10,
        "the fixtures define only {defined} value ids"
    );
}

#[test]
fn markdown_never_puts_a_link_inside_a_fence() {
    for (target, md, _) in pairs() {
        let mut fence: Option<&str> = None;
        for line in md.lines() {
            match fence {
                Some(marks) if line == marks => fence = None,
                Some(_) => assert!(
                    !line.contains("](#"),
                    "{target}: link inside a fence: {line}"
                ),
                None if line.len() >= 3 && line.chars().all(|c| c == '`') => fence = Some(line),
                None => {}
            }
        }
    }
}

#[test]
fn html_escapes_angle_brackets_and_ampersands() {
    let written = outputs(&input_of(&fixtures().join("names")));
    let html = &written
        .iter()
        .find(|(f, _)| f == "names.html")
        .expect("names.html is written")
        .1;
    assert!(html.contains("Option&lt;Vec&lt;"));
    assert!(html.contains(")]&amp; 'a;"));
    assert!(!html.contains("Option<Vec<"));
}
