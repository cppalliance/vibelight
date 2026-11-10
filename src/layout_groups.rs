//! The groups of a type body or a module block: sortable entries, the
//! sections they print in, labeled by visibility, and the blank lines
//! between them.

use std::collections::BTreeMap;

use crate::access::Access;
use crate::layout::{Block, Line, Repeat, Span, key, line_text};
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

/// Joins a type body's or a module block's groups: the unlabeled groups
/// `first`, then each set of labeled sections, data first and then
/// members, every section opened by its label, such as `pub(crate):`, with
/// a blank line before it. A visibility that holds both data and members
/// prints its label twice.
pub(crate) fn join_sections(first: Vec<Vec<Entry>>, sets: [Sections; 2], block: &mut Block) {
    if first.iter().any(|group| !group.is_empty()) {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        join_groups(first, block);
    }
    for set in sets {
        for ((_, spelling), groups) in set {
            if groups.iter().all(Vec::is_empty) {
                continue;
            }
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            block.lines.push(label_line(&spelling));
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
    use crate::layout::plain;

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
        join_sections(Vec::new(), [sections, Sections::new()], &mut block);
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
