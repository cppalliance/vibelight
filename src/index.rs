//! The index of a run, `index.md` beside the markdown outlines: one line
//! for each outline, so a reader of the folder, such as an agent, knows
//! which crates the run chose, which it followed into, and how large each
//! outline is.

use std::fmt::Write as _;

/// The index's file name in the markdown folder.
pub(crate) const FILE: &str = "index.md";

/// One outline of the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The output stem, shared by the markdown file and the HTML page.
    pub(crate) stem: String,
    /// The crate's folder below the base of the run, with forward slashes.
    pub(crate) dir: String,
    /// True when the run reached the crate through a path dependency.
    pub(crate) followed: bool,
    /// The size of the markdown outline.
    pub(crate) bytes: usize,
}

/// Renders `html/index.html`: one linked line per page of the run, so the
/// HTML folder has a place to start.
pub(crate) fn render_html(entries: &[Entry], stamp: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", crate::stamp::html_comment(stamp));
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>Outlines</title>\n<style>\n");
    out.push_str(crate::render_style::STYLE);
    out.push_str("</style>\n</head>\n<body>\n<main>\n");
    let _ = writeln!(out, "{}", crate::stamp::html_paragraph(stamp));
    out.push_str("<h1>Outlines</h1>\n<div class=\"doc\">\n");
    out.push_str("<p>One page for each target of the run, each with its crate's folder, ");
    out.push_str("whether the run selected the crate or followed a path dependency to it, ");
    out.push_str("and the size of its markdown outline.</p>\n<ul>\n");
    for entry in entries {
        let how = if entry.followed {
            "followed"
        } else {
            "selected"
        };
        let _ = writeln!(
            out,
            "<li><a href=\"{stem}.html\">{stem}</a> <code>{dir}</code>, {how}, {kb} KB \
             of markdown</li>",
            stem = crate::render_html::escape(&entry.stem),
            dir = crate::render_html::escape(&entry.dir),
            kb = entry.bytes.div_ceil(1024),
        );
    }
    out.push_str("</ul>\n</div>\n</main>\n</body>\n</html>\n");
    out
}

/// Renders the index, one line per entry in the order given.
pub(crate) fn render(entries: &[Entry]) -> String {
    let mut out = String::from(
        "# Outlines\n\n\
         Each markdown outline of the run: its crate's folder, whether the run \
         selected the crate or followed a path dependency to it, and its size. \
         Each has an HTML page of the same name in `html`.\n\n",
    );
    for entry in entries {
        let how = if entry.followed {
            "followed"
        } else {
            "selected"
        };
        let _ = writeln!(
            out,
            "- [{stem}.md]({stem}.md): `{dir}`, {how}, {kb} KB",
            stem = entry.stem,
            dir = entry.dir,
            kb = entry.bytes.div_ceil(1024),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_outline_gets_one_line_with_its_folder_its_role_and_its_size() {
        let entry = |stem: &str, dir: &str, followed, bytes| Entry {
            stem: stem.to_owned(),
            dir: dir.to_owned(),
            followed,
            bytes,
        };
        let text = render(&[
            entry("alpha", "crates/alpha", false, 1),
            entry("delta", "crates/delta", true, 2049),
        ]);
        assert!(text.starts_with("# Outlines\n\n"), "{text}");
        assert!(text.ends_with(
            "\n\n- [alpha.md](alpha.md): `crates/alpha`, selected, 1 KB\n\
             - [delta.md](delta.md): `crates/delta`, followed, 3 KB\n"
        ));
    }
}
