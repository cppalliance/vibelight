//! The files that make the HTML folder one page that works from `file:`.
//! Cursor's browser drops every click on a link to a `file:` URL, so the
//! folder holds one page, `index.html`, which loads `vibelight.js`. The
//! script shows each outline's page from its data file, which it loads with
//! a script tag, since a script tag is no navigation, and records the page
//! in the address's fragment. `search.js` lists every declaration the pages
//! give an id, for the script's search box.

use std::fmt::Write as _;

use crate::render_html::Anchors;
use crate::render_style::STYLE;

/// The script that the page loads, written beside it.
pub(crate) const SCRIPT: &str = include_str!("browse.js");

/// The file name of the script.
pub(crate) const SCRIPT_FILE: &str = "vibelight.js";

/// The file name of the search index.
pub(crate) const SEARCH_FILE: &str = "search.js";

/// The stem of the index's data file, which the page shows first.
pub(crate) const INDEX: &str = "index";

/// How a data file starts, which is how a run knows a `.js` file in its
/// folder is its own.
pub(crate) const DATA_START: &str = "vibelightPage(";

/// The name of the data file of the page whose stem is `stem`.
pub(crate) fn data_file(stem: &str) -> String {
    format!("{stem}.page.js")
}

/// Writes the one HTML page of the folder: the style sheet and the script,
/// and a line for a browser that runs no script. `stamp` goes in its first
/// line.
pub(crate) fn shell(stamp: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", crate::stamp::html_comment(stamp));
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>Outlines</title>\n<style>\n");
    out.push_str(STYLE);
    out.push_str("</style>\n");
    let _ = writeln!(out, "<script src=\"{SCRIPT_FILE}\" defer></script>");
    out.push_str("</head>\n<body>\n<noscript>The outlines need JavaScript: ");
    out.push_str("each page is a script file beside this one.</noscript>\n");
    out.push_str("</body>\n</html>\n");
    out
}

/// Writes the data file of a page: one call that gives the script the
/// page's title and its content. The content goes in a template literal line
/// by line, so the stamp line stays a line of its own and a run that changes
/// only the stamp leaves the file alone.
pub(crate) fn data(stem: &str, title: &str, content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 128);
    let _ = writeln!(out, "{DATA_START}{}, {}, `", json(stem), json(title));
    out.push_str(&template(content));
    out.push_str("`);\n");
    out
}

/// Writes the search index: the stem of each page, and every id of a
/// declaration, function, constant, or static, with the index of its page,
/// then the headings of each facade's API, which `api` lists by page.
pub(crate) fn search(run: &[Anchors], api: &[Vec<String>]) -> String {
    let mut out = String::from("vibelightSearch({\"pages\":[");
    for (i, page) in run.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json(page.file.trim_end_matches(".html")));
    }
    out.push_str("],\"ids\":[");
    let mut first = true;
    for (i, page) in run.iter().enumerate() {
        let ids = page.ids.iter().chain(page.values.values());
        let headings = api.get(i).into_iter().flatten();
        for id in ids.filter(|id| !id.is_empty()).chain(headings) {
            if !first {
                out.push(',');
            }
            first = false;
            let _ = write!(out, "\n[{i},{}]", json(id));
        }
    }
    out.push_str("\n]});\n");
    out
}

/// Quotes text as a JSON string, which is also a JavaScript string.
fn json(text: &str) -> String {
    serde_json::Value::String(text.to_owned()).to_string()
}

/// Escapes text for the inside of a template literal: the backslash, the
/// backtick, and the `${` that opens a substitution.
fn template(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::model::ValueRef;
    use crate::stamp;

    fn content(time: &str, body: &str) -> String {
        format!("<main>\n{}\n{body}</main>\n", stamp::html_paragraph(time))
    }

    #[test]
    fn the_data_holds_the_title_and_the_content_as_a_template() {
        assert_eq!(
            data(
                "a-b",
                "a & b",
                &content("2026-01-01T00:00:00Z", "<p>x</p>\n")
            ),
            "vibelightPage(\"a-b\", \"a & b\", `\n<main>\n\
             <p class=\"stamp\">Updated <time datetime=\"2026-01-01T00:00:00Z\">\
             2026-01-01T00:00:00Z</time></p>\n<p>x</p>\n</main>\n`);\n"
        );
    }

    #[test]
    fn the_template_escapes_the_backslash_the_backtick_and_a_substitution() {
        assert_eq!(
            template("a\\b `c` ${d} $e {f}"),
            "a\\\\b \\`c\\` \\${d} $e {f}"
        );
        let data = data("t", "t", "<code>`${x}\\</code>\n");
        assert!(data.contains("<code>\\`\\${x}\\\\</code>\n"), "{data}");
    }

    #[test]
    fn a_run_that_changes_only_the_stamp_leaves_the_data_alone() {
        let first = data("t", "t", &content("2026-01-01T00:00:00Z", "<p>x</p>\n"));
        let later = data("t", "t", &content("2027-06-30T12:00:00Z", "<p>x</p>\n"));
        assert_ne!(first, later);
        assert_eq!(stamp::mask(&first), stamp::mask(&later));
        let edited = data("t", "t", &content("2027-06-30T12:00:00Z", "<p>y</p>\n"));
        assert_ne!(stamp::mask(&first), stamp::mask(&edited));
    }

    #[test]
    fn the_shell_loads_the_style_and_the_script_and_holds_no_page() {
        let page = shell("2026-01-01T00:00:00Z");
        assert!(page.starts_with("<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n"));
        assert!(page.contains(&format!(
            "</style>\n<script src=\"{SCRIPT_FILE}\" defer></script>\n"
        )));
        assert!(page.contains("<body>\n<noscript>"));
        assert!(!page.contains("<main>"));
    }

    #[test]
    fn the_search_index_lists_each_page_and_its_ids() {
        let mut values = BTreeMap::new();
        values.insert(ValueRef { module: 0, item: 0 }, "fn.b.run".to_owned());
        let run = [
            Anchors {
                file: "a.html".to_owned(),
                ids: vec!["type.a.A".to_owned()],
                values: BTreeMap::new(),
            },
            Anchors {
                file: "b.html".to_owned(),
                ids: Vec::new(),
                values,
            },
        ];
        assert_eq!(
            search(&run, &[vec!["api.Run".to_owned()]]),
            "vibelightSearch({\"pages\":[\"a\",\"b\"],\"ids\":[\n[0,\"type.a.A\"],\n[0,\"api.Run\"],\
             \n[1,\"fn.b.run\"]\n]});\n"
        );
    }

    #[test]
    fn the_script_names_the_files_and_the_functions_that_the_data_calls() {
        assert!(SCRIPT.contains("window.vibelightPage = function"));
        assert!(SCRIPT.contains("window.vibelightSearch = function"));
        assert!(SCRIPT.contains("'.page.js'"));
        assert!(SCRIPT.contains(SEARCH_FILE));
        assert_eq!(data_file("a-b"), "a-b.page.js");
        assert!(data("t", "t", "").starts_with(DATA_START));
    }
}
