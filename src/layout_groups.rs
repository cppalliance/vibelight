//! The groups of a type body or a module block: sortable entries, the
//! sections they print in, and the blank lines between them.

use crate::layout::{Block, Line, Repeat, key, line_text};
use crate::model::{TypeRef, Vis, name_order};

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

/// The groups of a body's or a module block's sections: public, internal,
/// and private, in print order.
pub(crate) type Sections = [Vec<Vec<Entry>>; 3];

/// Returns empty sections of `per_section` groups each.
pub(crate) fn empty_sections(per_section: usize) -> Sections {
    std::array::from_fn(|_| (0..per_section).map(|_| Vec::new()).collect())
}

/// Returns the index of a visibility's section: public, internal, or
/// private.
pub(crate) fn section(vis: Vis) -> usize {
    usize::from(vis.rank())
}

/// Joins a type body's data sections, then its member sections, each set in
/// the order public, `internal:`, `private:`, with a blank line before every
/// section that has entries. A label opens a section only when its
/// visibility is not the access already in effect, which starts as `default`,
/// the access the keyword gives: public in a struct, private in a class. So a
/// visibility that holds both data and members can print its label twice,
/// as a C++ header does. Only a type uses this: a module has no enclosing
/// brace, so its items carry their own visibility line instead.
pub(crate) fn join_sections(data: Sections, members: Sections, default: Vis, block: &mut Block) {
    let labels = ["public:", "internal:", "private:"];
    let mut current = section(default);
    for set in [data, members] {
        for ((index, label), groups) in labels.into_iter().enumerate().zip(set) {
            if groups.iter().all(Vec::is_empty) {
                continue;
            }
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            if index != current {
                block.lines.push(Line {
                    spans: vec![key(label)],
                    ..Line::default()
                });
                current = index;
            }
            join_groups(groups, block);
        }
    }
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
