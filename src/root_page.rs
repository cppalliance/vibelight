//! The overview page of the HTML: what each crate of the run is and how
//! the crates fit together. The facades come first, each the API of a
//! crate family, then the implementation crates, grouped by the facade that
//! re-exports most of their items, then every other crate by folder, then
//! the uses between outlines. The legend closes the page, once for every
//! page.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::layout::{Block, Span, span_text};
use crate::model::TypeRef;
use crate::render_html::{attr, escape};
use crate::roles::{Role, Roles};

/// One crate of the overview.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Row {
    /// The stem of its page.
    pub(crate) stem: String,
    /// The name it shows.
    pub(crate) title: String,
    /// Its folder below the base of the run.
    pub(crate) dir: String,
    /// What it is, as HTML: the first sentence of its crate docs.
    pub(crate) summary: String,
}

/// What the overview shows: the run's name, its crates in target order
/// with their roles, how many declarations its pages give an id, and the
/// section on the uses between outlines.
pub(crate) struct Overview<'a> {
    pub(crate) name: &'a str,
    pub(crate) rows: &'a [Row],
    pub(crate) roles: &'a Roles,
    pub(crate) declarations: usize,
    pub(crate) uses: &'a str,
}

/// Renders the overview's content, with `stamp` at its top.
pub(crate) fn fragment(overview: &Overview<'_>, stamp: &str) -> String {
    let roles = &overview.roles.each;
    let count = |role: Role| roles.iter().filter(|&&r| r == role).count();
    let mut out = String::from("<main class=\"overview\">\n");
    let _ = writeln!(out, "{}", crate::stamp::html_paragraph(stamp));
    let _ = writeln!(out, "<h1 id=\"overview\">{}</h1>", escape(overview.name));
    out.push_str(
        "<p class=\"lede\">The structure of every crate here: each declaration and its \
         comments, with no function bodies. Start with a facade: it is the API that users of \
         a crate family write against, and its page shows that API in one place. The \
         implementation crates behind it are where that API is built; their pages show \
         everything, public or not.</p>\n",
    );
    let stat = |n: usize, one: &str, many: &str| {
        format!(
            "<span><b>{n}</b> {}</span>",
            if n == 1 { one } else { many }
        )
    };
    let _ = writeln!(
        out,
        "<p class=\"stats\">{}{}{}{}{}</p>",
        stat(roles.len(), "crate", "crates"),
        stat(count(Role::Facade), "facade", "facades"),
        stat(
            count(Role::Implementation),
            "implementation",
            "implementation"
        ),
        stat(count(Role::Library) + count(Role::Binary), "other", "other"),
        stat(overview.declarations, "declaration", "declarations"),
    );
    facades(&mut out, overview);
    implementations(&mut out, overview);
    others(&mut out, overview);
    out.push_str(overview.uses);
    out.push_str("<h2 id=\"legend\">Legend</h2>\n");
    out.push_str(&crate::legend::html());
    out.push_str("</main>\n");
    out
}

/// The targets of one role, by name.
fn of_role(overview: &Overview<'_>, role: Role) -> Vec<usize> {
    let mut found: Vec<usize> = (0..overview.rows.len())
        .filter(|&t| overview.roles.each[t] == role)
        .collect();
    found.sort_by(|&a, &b| overview.rows[a].title.cmp(&overview.rows[b].title));
    found
}

fn link(overview: &Overview<'_>, t: usize, class: &str) -> String {
    let row = &overview.rows[t];
    format!(
        "<a class=\"{class}\" href=\"{}.html\">{}</a>",
        attr(&row.stem),
        escape(&row.title)
    )
}

fn heading(out: &mut String, id: &str, text: &str, count: usize, hint: &str) {
    let _ = writeln!(
        out,
        "<h2 id=\"{id}\">{text} <span class=\"count\">{count}</span></h2>\n<p class=\"hint\">{hint}</p>"
    );
}

fn facades(out: &mut String, overview: &Overview<'_>) {
    let found = of_role(overview, Role::Facade);
    if found.is_empty() {
        return;
    }
    heading(
        out,
        "facades",
        "Facades",
        found.len(),
        "Crates whose public surface is mostly re-exports of other crates here. Each is the \
         public API of a crate family.",
    );
    out.push_str(
        "<table class=\"crates\">\n<tr><th>Crate</th><th>What it is</th><th>Public API</th>\
         <th>Re-exports from</th></tr>\n",
    );
    for t in found {
        let counts = &overview.roles.counts[t];
        let mut from: Vec<(usize, usize)> = counts.from.iter().map(|(&k, &n)| (k, n)).collect();
        from.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then(overview.rows[a.0].title.cmp(&overview.rows[b.0].title))
        });
        let chips: Vec<String> = from
            .into_iter()
            .map(|(k, n)| {
                let class = if overview.roles.each[k] == Role::Facade {
                    "chip facade"
                } else {
                    "chip"
                };
                let row = &overview.rows[k];
                format!(
                    "<a class=\"{class}\" href=\"{}.html\">{} <b>{n}</b></a>",
                    attr(&row.stem),
                    escape(&row.title)
                )
            })
            .collect();
        let modules = match counts.modules {
            0 => String::new(),
            1 => "<br>1 module".to_owned(),
            n => format!("<br>{n} modules"),
        };
        let _ = writeln!(
            out,
            "<tr><td>{}{}</td><td class=\"desc\">{}</td><td class=\"num\"><b>{}</b> items{modules}</td>\
             <td>{}</td></tr>",
            link(overview, t, "crate facade"),
            path(overview, t),
            description(overview, t),
            counts.items,
            chips.join(""),
        );
    }
    out.push_str("</table>\n");
}

fn implementations(out: &mut String, overview: &Overview<'_>) {
    let found = of_role(overview, Role::Implementation);
    if found.is_empty() {
        return;
    }
    heading(
        out,
        "implementation",
        "Implementation crates",
        found.len(),
        "The crates each facade's code reaches, short of other facades, grouped by the facade \
         that re-exports the most of them. Users should reach them only through it.",
    );
    out.push_str(
        "<table class=\"crates\">\n<tr><th>Crate</th><th>What it is</th>\
         <th>Reaches users as</th></tr>\n",
    );
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let facades = of_role(overview, Role::Facade);
    let rank = |f: usize| facades.iter().position(|&x| x == f).unwrap_or(usize::MAX);
    for t in found {
        let primary = (overview.roles.behind[t].first().copied())
            .or_else(|| overview.roles.fronted_by[t].keys().next().copied());
        groups
            .entry(primary.map_or(usize::MAX, rank))
            .or_default()
            .push(t);
    }
    for (place, members) in groups {
        let Some(&facade) = facades.get(place) else {
            continue;
        };
        let _ = writeln!(
            out,
            "<tr class=\"group\"><td colspan=\"3\">Behind {}</td></tr>",
            link(overview, facade, "crate facade")
        );
        for t in members {
            let reach: Vec<String> = (overview.roles.fronted_by[t].iter())
                .map(|(&f, &n)| {
                    let noun = if n == 1 { "item" } else { "items" };
                    format!("<b>{n}</b> {noun} of {}", link(overview, f, "facade"))
                })
                .collect();
            let reach = if reach.is_empty() {
                "none re-exported".to_owned()
            } else {
                reach.join("<br>")
            };
            let _ = writeln!(
                out,
                "<tr><td class=\"in\">{}</td><td class=\"desc\">{}</td><td class=\"num\">{reach}</td></tr>",
                link(overview, t, "crate implementation"),
                description(overview, t),
            );
        }
    }
    out.push_str("</table>\n");
}

fn others(out: &mut String, overview: &Overview<'_>) {
    let mut found = of_role(overview, Role::Library);
    found.extend(of_role(overview, Role::Binary));
    if found.is_empty() {
        return;
    }
    heading(
        out,
        "other",
        "Other crates",
        found.len(),
        "Libraries no facade re-exports from, and binaries, grouped by folder.",
    );
    out.push_str("<table class=\"crates\">\n<tr><th>Crate</th><th>What it is</th></tr>\n");
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for t in found {
        let dir = &overview.rows[t].dir;
        let folder = dir.rsplit_once('/').map_or(".", |(up, _)| up);
        groups.entry(format!("{folder}/")).or_default().push(t);
    }
    for (folder, mut members) in groups {
        members.sort_by(|&a, &b| overview.rows[a].title.cmp(&overview.rows[b].title));
        let _ = writeln!(
            out,
            "<tr class=\"group\"><td colspan=\"2\">{}</td></tr>",
            escape(&folder)
        );
        for t in members {
            let badge = if overview.roles.each[t] == Role::Binary {
                " <span class=\"badge\">bin</span>"
            } else {
                ""
            };
            let _ = writeln!(
                out,
                "<tr><td class=\"in\">{}{badge}</td><td class=\"desc\">{}</td></tr>",
                link(overview, t, "crate"),
                description(overview, t),
            );
        }
    }
    out.push_str("</table>\n");
}

fn path(overview: &Overview<'_>, t: usize) -> String {
    format!(
        "<span class=\"path\">{}</span>",
        escape(&overview.rows[t].dir)
    )
}

/// Returns a crate's description cell: what it is, then a warning for each
/// facade it bypasses.
fn description(overview: &Overview<'_>, t: usize) -> String {
    let mut out = overview.rows[t].summary.clone();
    let bypassed = &overview.roles.bypasses[t];
    if !bypassed.is_empty() {
        let names: Vec<String> = bypassed
            .iter()
            .map(|&i| link(overview, i, "implementation"))
            .collect();
        let mut facades: Vec<usize> = (bypassed.iter())
            .flat_map(|&i| overview.roles.behind[i].iter().copied())
            .collect();
        facades.sort_unstable();
        facades.dedup();
        let facades: Vec<String> = facades
            .into_iter()
            .map(|f| link(overview, f, "facade"))
            .collect();
        let _ = write!(
            out,
            "<span class=\"warn\">Uses {} directly, not through {}</span>",
            names.join(", "),
            facades.join(" or ")
        );
    }
    out
}

/// Returns a crate's summary as HTML: the first sentence of its crate docs,
/// without the crate's own name before a dash that opens it, link syntax
/// left out and backticked text in the code font. `names` are the names the
/// crate goes by.
pub(crate) fn summary(docs: Option<&Block>, refs: &[TypeRef], names: &[&str]) -> String {
    let Some(docs) = docs else {
        return String::new();
    };
    let mut text = String::new();
    for line in &docs.lines {
        let Some(Span::Marker(marker)) = line.spans.first() else {
            break;
        };
        if !marker.starts_with("//!") {
            break;
        }
        let piece: String = line.spans[1..].iter().map(|s| span_text(s, refs)).collect();
        let piece = piece.trim();
        if piece.is_empty() {
            if text.is_empty() {
                continue;
            }
            break;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(piece);
    }
    let text = unlinked(&text);
    let sentence = without_name(first_sentence(&text), names);
    code_html(&sentence)
}

/// Leaves out the link syntax of intra-doc links outside code: the
/// brackets, and the target in parentheses after them.
fn unlinked(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_code = false;
    while let Some(c) = chars.next() {
        match c {
            '`' => {
                in_code = !in_code;
                out.push(c);
            }
            '[' if !in_code => {}
            ']' if !in_code => {
                if chars.peek() == Some(&'(') {
                    for skipped in chars.by_ref() {
                        if skipped == ')' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Returns text through its first sentence's period, outside code.
fn first_sentence(text: &str) -> &str {
    let mut in_code = false;
    for (at, c) in text.char_indices() {
        match c {
            '`' => in_code = !in_code,
            '.' if !in_code && text[at + 1..].starts_with(' ') => return &text[..=at],
            _ => {}
        }
    }
    text
}

/// Drops a crate name and a dash that open a sentence, as in `name - what
/// it is`, and starts what remains with a capital.
fn without_name(sentence: &str, names: &[&str]) -> String {
    for name in names {
        for written in [format!("`{name}`"), (*name).to_owned()] {
            let Some(rest) = sentence.strip_prefix(&written) else {
                continue;
            };
            let rest = rest.trim_start();
            let Some(rest) = rest
                .strip_prefix('-')
                .or_else(|| rest.strip_prefix('\u{2014}'))
            else {
                continue;
            };
            let rest = rest.trim_start();
            let mut chars = rest.chars();
            return match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            };
        }
    }
    sentence.to_owned()
}

/// Escapes text, with each backticked run in the code font.
fn code_html(text: &str) -> String {
    let mut out = String::new();
    for (index, part) in text.split('`').enumerate() {
        if index % 2 == 1 {
            let _ = write!(out, "<code>{}</code>", escape(part));
        } else {
            out.push_str(&escape(part));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_summary_is_the_first_sentence_without_link_syntax_or_a_leading_name() {
        assert_eq!(
            unlinked("A [`Prompt`] runs, see [`Run::step`](Self::step) and `[T; 2]`."),
            "A `Prompt` runs, see `Run::step` and `[T; 2]`."
        );
        assert_eq!(first_sentence("One. Two."), "One.");
        assert_eq!(first_sentence("Uses `a.b` here. Then"), "Uses `a.b` here.");
        assert_eq!(
            without_name(
                "`harness-runner` - the per-run Harness.",
                &["harness-runner"]
            ),
            "The per-run Harness."
        );
        assert_eq!(
            without_name("workshop-menu - the server-owned menu.", &["workshop-menu"]),
            "The server-owned menu."
        );
        assert_eq!(
            without_name("The Engine's core.", &["engine"]),
            "The Engine's core."
        );
        assert_eq!(
            code_html("The `Run` & more"),
            "The <code>Run</code> &amp; more"
        );
    }
}
