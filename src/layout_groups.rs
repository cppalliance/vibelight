//! The groups of a type body, an impl, or a module block: sortable entries,
//! the sections they print in, one per visibility, the widest first, and the
//! blank lines between them.

use std::collections::BTreeMap;

use crate::access::Access;
use crate::layout::{Block, Line, Repeat, Span, line_text};
use crate::model::{Meta, TypeRef, name_order};

/// One sortable entry of a group: a member, a variant, or an item.
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) lines: Vec<Line>,
    pub(crate) repeats: Vec<Repeat>,
}

/// Sorts entries by name, then by their printed text, so moving code
/// between files or impls changes no outline.
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

/// Joins the groups of a body or a module block in order, with a blank line
/// between two groups, and between two entries of a group as `spaced` puts
/// one.
pub(crate) fn join_groups(groups: Vec<Vec<Entry>>, block: &mut Block) {
    let groups = groups.into_iter().filter(|group| !group.is_empty());
    for (i, group) in groups.enumerate() {
        let mut members = Vec::new();
        for entry in group {
            block.repeats.extend(entry.repeats);
            members.push(entry.lines);
        }
        if i > 0 {
            block.lines.push(Line::default());
        }
        block.lines.extend(spaced(members));
    }
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

/// Joins a body's or a module block's groups: the groups `first`, which have
/// no visibility of their own, then each section of `sections`, the widest
/// visibility first, with a blank line before each. Every entry spells its
/// own visibility, so a section needs no label.
pub(crate) fn join_sections(first: Vec<Vec<Entry>>, sections: Sections, block: &mut Block) {
    if first.iter().any(|group| !group.is_empty()) {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        join_groups(first, block);
    }
    for groups in sections.into_values() {
        if groups.iter().all(Vec::is_empty) {
            continue;
        }
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        join_groups(groups, block);
    }
}

/// Joins the lines of each member of a list, with a blank line between two
/// members when either takes more than one line, as a member with a comment
/// or an attribute above it does, so one-line members stack as rustfmt
/// leaves them.
pub(crate) fn spaced(members: impl IntoIterator<Item = Vec<Line>>) -> Vec<Line> {
    let mut out = Vec::new();
    let mut last_tall = false;
    for lines in members {
        let tall = lines.len() > 1;
        if !out.is_empty() && (tall || last_tall) {
            out.push(Line::default());
        }
        last_tall = tall;
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
    fn a_blank_line_sets_off_each_member_that_takes_more_than_one_line() {
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
            "w: i32,",
        ];
        assert_eq!(text(&spaced(members)), expected);
    }

    #[test]
    fn a_module_block_sets_its_groups_apart_and_stacks_one_line_entries() {
        let entry = |texts: &[&str]| Entry {
            name: String::new(),
            lines: lines(texts),
            repeats: Vec::new(),
        };
        let groups = vec![
            vec![entry(&["pub use a::B;"]), entry(&["pub use a::C;"])],
            Vec::new(),
            vec![entry(&["pub const LIMIT: u8 = 4;"])],
            vec![
                entry(&["pub fn run(", "input: &str) -> bool;"]),
                entry(&["pub fn stop();"]),
            ],
        ];
        let mut block = Block::default();
        join_groups(groups, &mut block);
        let expected = [
            "pub use a::B;",
            "pub use a::C;",
            "",
            "pub const LIMIT: u8 = 4;",
            "",
            "pub fn run(",
            "input: &str) -> bool;",
            "",
            "pub fn stop();",
        ];
        assert_eq!(text(&block.lines), expected);
    }
}
