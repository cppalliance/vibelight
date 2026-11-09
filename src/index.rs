//! The index of a run, `index.md` beside the markdown outlines: one line
//! for each outline, so a reader of the folder, such as an agent, knows
//! which crates the run chose, which it followed into, and how large each
//! outline is, and then which outlines use which.

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
    /// The stems of the other outlines whose items its production code
    /// uses, each with how many distinct items, in target order.
    pub(crate) uses: Vec<(String, usize)>,
    /// The stems of the other outlines whose production code uses its
    /// items, each with how many distinct items, in target order.
    pub(crate) used_by: Vec<(String, usize)>,
}

/// Renders the HTML's section on the uses between outlines, each name
/// linked to its page, or nothing when no outline uses another.
pub(crate) fn uses_html(entries: &[Entry]) -> String {
    let mut out = String::new();
    if !crosses(entries) {
        return out;
    }
    out.push_str("<h2 id=\"uses\">Uses between outlines</h2>\n<div class=\"doc\">\n");
    out.push_str("<p>Each outline whose production code uses items of another, ");
    out.push_str("or whose items another uses, with how many distinct items, ");
    out.push_str("and its instability, <code>fan_out / (fan_in + fan_out)</code>, ");
    out.push_str("counting outlines. A use counts for the crate that declares the item ");
    out.push_str("when the using crate depends on it, else for the first of its ");
    out.push_str("dependencies in the list that re-exports the item by name, ");
    out.push_str("else for the crate that declares it.</p>\n<ul>\n");
    let link = |stem: &str| {
        let stem = crate::render_html::escape(stem);
        format!("<a href=\"{stem}.html\">{stem}</a>")
    };
    for line in entries.iter().filter_map(|entry| uses_line(entry, &link)) {
        let _ = writeln!(out, "<li>{line}</li>");
    }
    out.push_str("</ul>\n</div>\n");
    out
}
/// Renders the index, one line per entry in the order given, then the
/// uses between outlines when some outline uses another.
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
    if crosses(entries) {
        out.push_str(
            "\n## Uses between outlines\n\n\
             Each outline whose production code uses items of another, or whose items \
             another uses, with how many distinct items, and its instability, \
             `fan_out / (fan_in + fan_out)`, counting outlines. A use counts for the \
             crate that declares the item when the using crate depends on it, else for \
             the first of its dependencies in the list that re-exports the item by name, \
             else for the crate that declares it.\n\n",
        );
        let code = |stem: &str| format!("`{stem}`");
        for line in entries.iter().filter_map(|entry| uses_line(entry, &code)) {
            let _ = writeln!(out, "- {line}");
        }
    }
    out
}

/// Tells whether some outline uses another.
fn crosses(entries: &[Entry]) -> bool {
    entries.iter().any(|entry| !entry.uses.is_empty())
}

/// Returns an entry's line among the uses between outlines, or none when
/// it uses no other outline and no other uses it: its name, the outlines
/// it uses, and the outlines that use it, each with how many items and a
/// clause left out when its list is empty, then its instability, such as
/// `engine uses lua (50 items); used by app (1 item); instability 0.50`
/// when `name` writes each name as it is.
fn uses_line(entry: &Entry, name: &dyn Fn(&str) -> String) -> Option<String> {
    if entry.uses.is_empty() && entry.used_by.is_empty() {
        return None;
    }
    let mut out = name(&entry.stem);
    for (label, others) in [("uses", &entry.uses), ("used by", &entry.used_by)] {
        if others.is_empty() {
            continue;
        }
        let listed: Vec<String> = (others.iter())
            .map(|(stem, n)| {
                let noun = if *n == 1 { "item" } else { "items" };
                format!("{} ({n} {noun})", name(stem))
            })
            .collect();
        let _ = write!(out, " {label} {};", listed.join(", "));
    }
    let instability = crate::document_body::instability(entry.used_by.len(), entry.uses.len());
    let _ = write!(out, " instability {instability}");
    Some(out)
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
            uses: Vec::new(),
            used_by: Vec::new(),
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

    #[test]
    fn outlines_that_use_each_other_get_a_line_each_after_the_list() {
        let pairs = |list: &[(&str, usize)]| -> Vec<(String, usize)> {
            (list.iter())
                .map(|&(stem, n)| (stem.to_owned(), n))
                .collect()
        };
        let entry = |stem: &str, uses: &[(&str, usize)], used_by: &[(&str, usize)]| Entry {
            stem: stem.to_owned(),
            dir: format!("crates/{stem}"),
            followed: false,
            bytes: 1,
            uses: pairs(uses),
            used_by: pairs(used_by),
        };
        let entries = [
            entry("app", &[("engine", 12)], &[]),
            entry("engine", &[("lua", 50), ("types", 1)], &[("app", 12)]),
            entry("lone", &[], &[]),
            entry("lua", &[], &[("engine", 50)]),
            entry("types", &[], &[("engine", 1)]),
        ];
        let text = render(&entries);
        assert!(
            text.ends_with(
                "- [types.md](types.md): `crates/types`, selected, 1 KB\n\n\
                 ## Uses between outlines\n\n\
                 Each outline whose production code uses items of another, or whose items \
                 another uses, with how many distinct items, and its instability, \
                 `fan_out / (fan_in + fan_out)`, counting outlines. A use counts for the \
                 crate that declares the item when the using crate depends on it, else for \
                 the first of its dependencies in the list that re-exports the item by name, \
                 else for the crate that declares it.\n\n\
                 - `app` uses `engine` (12 items); instability 1.00\n\
                 - `engine` uses `lua` (50 items), `types` (1 item); used by `app` (12 items); \
                 instability 0.67\n\
                 - `lua` used by `engine` (50 items); instability 0.00\n\
                 - `types` used by `engine` (1 item); instability 0.00\n"
            ),
            "{text}"
        );
        let page = uses_html(&entries);
        assert!(
            page.starts_with(
                "<h2 id=\"uses\">Uses between outlines</h2>\n<div class=\"doc\">\n\
                 <p>Each outline whose production code uses items of another, or whose items \
                 another uses, with how many distinct items, and its instability, \
                 <code>fan_out / (fan_in + fan_out)</code>, counting outlines. A use counts for \
                 the crate that declares the item when the using crate depends on it, else for \
                 the first of its dependencies in the list that re-exports the item by name, \
                 else for the crate that declares it.</p>\n<ul>\n"
            ),
            "{page}"
        );
        assert!(
            page.contains(
                "<li><a href=\"engine.html\">engine</a> uses <a href=\"lua.html\">lua</a> \
                 (50 items), <a href=\"types.html\">types</a> (1 item); used by \
                 <a href=\"app.html\">app</a> (12 items); instability 0.67</li>\n\
                 <li><a href=\"lua.html\">lua</a> used by "
            ),
            "{page}"
        );
        assert!(page.ends_with("instability 0.00</li>\n</ul>\n</div>\n"));
        let apart = [entry("lone", &[], &[]), entry("solo", &[], &[])];
        assert!(!render(&apart).contains("##"), "nothing crosses");
        assert_eq!(uses_html(&apart), "", "nothing crosses");
    }
}
