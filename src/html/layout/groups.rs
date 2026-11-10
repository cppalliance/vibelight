//! The groups of a type body or a module block: sortable entries, the
//! sections they print in, labeled by visibility, and the blank lines
//! between them.

use std::collections::BTreeMap;

use crate::access::Access;
use crate::html::layout::{Block, Line, Repeat, Span, key, line_text};
use crate::model::{Meta, TypeRef, name_order};

/// One sortable entry of a group: a member, a variant, or an item.
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) lines: Vec<Line>,
    pub(crate) repeats: Vec<Repeat>,
}

/// Sorts entries by name, then by their printed text.
pub(crate) fn sort_entries(entries: &mut [Entry], refs: &[TypeRef]) {
    let text = |e: &Entry| -> String {
        e.lines
            .iter()
            .map(|l| line_text(l, refs))
            .collect::<Vec<_>>()
            .join("\n")
    };
    entries.sort_by(|a, b| name_order(&a.name, &b.name).then_with(|| text(a).cmp(&text(b))));
}

/// Joins the groups of a type body or a module block in order, with one
/// blank line between every two entries.
pub(crate) fn join_groups(groups: Vec<Vec<Entry>>, block: &mut Block) {
    let mut members = Vec::new();
    for entry in groups.into_iter().flatten() {
        block.repeats.extend(entry.repeats);
        members.push(entry.lines);
    }
    block.lines.extend(spaced(members));
}

/// A section's label: the rank its written visibility sorts by, wider
/// first, then the spelling it prints, such as `pub(crate)`.
pub(crate) type Label = (usize, String);

/// The labeled sections of a type body or a module block, in print order,
/// each holding its groups.
pub(crate) type Sections = BTreeMap<Label, Vec<Vec<Entry>>>;

/// Returns a node's label: the rank its stamp gives, and its visibility as
/// written, `pub(self)` for none. A node analysis did not stamp ranks by
/// its spelling alone.
pub(crate) fn label(meta: &Meta) -> Label {
    let spelling = if meta.vis.is_empty() {
        "pub(self)"
    } else {
        meta.vis.as_str()
    };
    let guess = || match spelling {
        "pub" => 0,
        "pub(crate)" => 1,
        "pub(self)" => usize::MAX,
        _ => 2,
    };
    let rank = meta.access.as_ref().map_or_else(guess, |stamp| stamp.rank);
    (rank, spelling.to_owned())
}

/// Adds an entry to group `group` of the section `label`, whose groups
/// number `per`.
pub(crate) fn add(sections: &mut Sections, label: Label, per: usize, group: usize, entry: Entry) {
    let groups = (sections.entry(label)).or_insert_with(|| (0..per).map(|_| Vec::new()).collect());
    groups[group].push(entry);
}

/// Returns the dim line that says how far a node reaches when that is less
/// than its visibility gives, unless its container's note, `shown`, says
/// the same.
pub(crate) fn reaches_line(meta: &Meta, indent: usize, shown: Option<Access>) -> Option<Line> {
    let stamp = meta.access.as_ref()?;
    let reach = stamp.reaches.as_ref().filter(|_| stamp.note() != shown)?;
    Some(Line {
        indent,
        spans: vec![Span::Dim(format!("reaches {reach}"))],
        trailing: None,
    })
}

/// Which of its sections' labels a block prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Labels<'a> {
    /// Every section opens with its label, as in a module's block.
    Every,
    /// A section opens with its label only where its visibility differs
    /// from the one in force, as in the body of a C++ class. The value is
    /// the visibility in force at the opening brace: `pub` in a `struct`,
    /// `pub(self)` in a `class`, and none in any other body, whose first
    /// label then prints.
    Changes(Option<&'a str>),
}

/// Joins a type body's or a module block's groups: the unlabeled groups
/// `first`, then each set of labeled sections, data first and then
/// members, with a blank line before each section. `labels` says which
/// sections open with their label, such as `pub(crate):`.
pub(crate) fn join_sections(
    first: Vec<Vec<Entry>>,
    sets: [Sections; 2],
    labels: Labels<'_>,
    block: &mut Block,
) {
    if first.iter().any(|group| !group.is_empty()) {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        join_groups(first, block);
    }
    let mut in_force = match labels {
        Labels::Every => None,
        Labels::Changes(start) => start.map(str::to_owned),
    };
    for set in sets {
        for ((_, spelling), groups) in set {
            if groups.iter().all(Vec::is_empty) {
                continue;
            }
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            let changes = in_force.as_deref() != Some(spelling.as_str());
            if labels == Labels::Every || changes {
                block.lines.push(label_line(&spelling));
            }
            in_force = Some(spelling);
            join_groups(groups, block);
        }
    }
}

/// Returns the line that opens a section: its label, such as `pub(crate):`.
fn label_line(spelling: &str) -> Line {
    Line {
        spans: vec![key(&format!("{spelling}:"))],
        ..Line::default()
    }
}

/// Whether a line is a section's label, as `label_line` writes it: one
/// keyword that ends in a colon. No other line of a block is that.
pub(crate) fn is_label(line: &Line) -> bool {
    matches!(line.spans.as_slice(), [Span::Key(text)] if text.ends_with(':'))
}

/// Whether a line is the brace that opens a block's body: `{` alone, at no
/// indent. The first member of a `struct` or a `class` follows it with no
/// label between.
pub(crate) fn is_open(line: &Line) -> bool {
    line.indent == 0 && matches!(line.spans.as_slice(), [Span::Code(text)] if text == "{")
}

/// Joins the lines of each member of a list, one blank line between every
/// two members.
pub(crate) fn spaced(members: impl IntoIterator<Item = Vec<Line>>) -> Vec<Line> {
    let mut out = Vec::new();
    for lines in members {
        if !out.is_empty() {
            out.push(Line::default());
        }
        out.extend(lines);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::layout::plain;

    fn lines(texts: &[&str]) -> Vec<Line> {
        texts
            .iter()
            .map(|text| Line {
                indent: 4,
                spans: vec![plain(text)],
                trailing: None,
            })
            .collect()
    }

    fn text(lines: &[Line]) -> Vec<String> {
        lines.iter().map(|l| line_text(l, &[])).collect()
    }

    #[test]
    fn every_member_of_a_list_gets_one_blank_line_between() {
        let members = vec![
            lines(&["/// Horizontal position.", "x: i32,"]),
            lines(&["/// Vertical position.", "y: i32,"]),
            lines(&["z: i32,"]),
            lines(&["w: i32,"]),
        ];
        let expected = [
            "/// Horizontal position.",
            "x: i32,",
            "",
            "/// Vertical position.",
            "y: i32,",
            "",
            "z: i32,",
            "",
            "w: i32,",
        ];
        assert_eq!(text(&spaced(members)), expected);
    }

    #[test]
    fn only_the_line_that_opens_a_section_is_a_label() {
        let entry = Entry {
            name: String::new(),
            lines: lines(&["x: u8,"]),
            repeats: Vec::new(),
        };
        let mut sections = Sections::new();
        add(&mut sections, (1, "pub(crate)".to_owned()), 1, 0, entry);
        let mut block = Block::default();
        let sets = [sections, Sections::new()];
        join_sections(Vec::new(), sets, Labels::Every, &mut block);
        assert_eq!(text(&block.lines), ["pub(crate):", "x: u8,"]);
        assert!(is_label(&block.lines[0]));
        assert!(!is_label(&block.lines[1]));
        let keyword_then_text = Line {
            spans: vec![key("implements:"), plain(" Named for Run")],
            ..Line::default()
        };
        assert!(!is_label(&keyword_then_text));
        let keyword_alone = Line {
            spans: vec![key("where")],
            ..Line::default()
        };
        assert!(!is_label(&keyword_alone));
        assert!(!is_label(&Line::default()), "a blank line is no label");
    }

    /// Joins a body's data, `pub` and then any `rest`, and its members,
    /// `pub` and then private, as `labels` says.
    fn joined(rest: &[&str], labels: Labels<'_>) -> Vec<String> {
        let entry = |text: &str| Entry {
            name: String::new(),
            lines: lines(&[text]),
            repeats: Vec::new(),
        };
        let private = || (usize::MAX, "pub(self)".to_owned());
        let public = || (0, "pub".to_owned());
        let mut data = Sections::new();
        add(&mut data, public(), 1, 0, entry("x: u8,"));
        for field in rest {
            add(&mut data, private(), 1, 0, entry(field));
        }
        let mut members = Sections::new();
        add(&mut members, public(), 1, 0, entry("get()"));
        add(&mut members, private(), 1, 0, entry("check()"));
        let mut block = Block::default();
        join_sections(Vec::new(), [data, members], labels, &mut block);
        text(&block.lines)
    }

    #[test]
    fn a_label_prints_only_where_the_visibility_changes() {
        let with = |head: &[&'static str]| [head, &["get()", "", "pub(self):", "check()"]].concat();
        assert_eq!(
            joined(&[], Labels::Changes(Some("pub"))),
            with(&["x: u8,", ""]),
            "`pub` is in force at the brace and still in force at the first member"
        );
        assert_eq!(
            joined(&[], Labels::Changes(None)),
            with(&["pub:", "x: u8,", ""]),
            "with nothing in force the first label prints"
        );
        assert_eq!(
            joined(&[], Labels::Changes(Some("pub(self)"))),
            with(&["pub:", "x: u8,", ""])
        );
        assert_eq!(
            joined(&["y: u8,"], Labels::Changes(Some("pub"))),
            with(&["x: u8,", "", "pub(self):", "y: u8,", "", "pub:"]),
            "the members' `pub` follows private data, so its label prints"
        );
    }

    #[test]
    fn a_block_that_prints_every_label_repeats_one() {
        assert_eq!(
            joined(&[], Labels::Every),
            [
                "pub:",
                "x: u8,",
                "",
                "pub:",
                "get()",
                "",
                "pub(self):",
                "check()"
            ]
        );
    }

    #[test]
    fn only_a_brace_alone_at_no_indent_opens_a_body() {
        let brace = |indent| Line {
            indent,
            spans: vec![plain("{")],
            trailing: None,
        };
        assert!(is_open(&brace(0)));
        assert!(!is_open(&brace(4)), "a struct-like variant's brace");
        let closing = Line {
            spans: vec![plain("}")],
            ..Line::default()
        };
        assert!(!is_open(&closing));
        assert!(!is_open(&Line::default()));
    }

    #[test]
    fn a_module_block_gets_one_blank_line_between_entries_in_and_across_groups() {
        let entry = |texts: &[&str]| Entry {
            name: String::new(),
            lines: lines(texts),
            repeats: Vec::new(),
        };
        let groups = vec![
            vec![entry(&["using a::B;"]), entry(&["using a::C;"])],
            Vec::new(),
            vec![entry(&["constexpr LIMIT: u8 = 4;"])],
            vec![
                entry(&["run(", "input: &str) -> bool;"]),
                entry(&["stop();"]),
            ],
        ];
        let mut block = Block::default();
        join_groups(groups, &mut block);
        let expected = [
            "using a::B;",
            "",
            "using a::C;",
            "",
            "constexpr LIMIT: u8 = 4;",
            "",
            "run(",
            "input: &str) -> bool;",
            "",
            "stop();",
        ];
        assert_eq!(text(&block.lines), expected);
    }
}
