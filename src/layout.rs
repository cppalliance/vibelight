//! Styled lines and blocks, and the helpers every layout step shares.

use crate::model::{Crate, Loc, Meta, Note, RefId, Seg, Text, TypeRef, Vis, name_order};
use crate::typerefs_edits::MUT;
use crate::wrap::{self, FnParts, MAX_WIDTH, ParamParts};

/// One styled run of a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Span {
    /// A keyword the outline prints, such as `struct` or `static`.
    Key(String),
    /// Code text.
    Code(String),
    /// A type name in code, which counts for naming and links in HTML.
    Ref(RefId),
    /// Comment text, including `// cfg:` lines.
    Comment(String),
    /// A type name in a comment, which links in HTML but never counts.
    DocRef(RefId),
    /// A bold label: `MACRO` or `UNSUPPORTED:`.
    Label(&'static str),
    /// A member or item name, which HTML links to its source line.
    Name(String, Loc),
}

/// One line of a block.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) indent: usize,
    pub(crate) spans: Vec<Span>,
    /// An ordinary comment at the end of the line, outside the width rule.
    pub(crate) trailing: Option<String>,
}

/// A mark inside a block, which markdown repeats as a bold line under it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Repeat {
    pub(crate) label: &'static str,
    pub(crate) text: String,
    pub(crate) loc: Option<Loc>,
}

/// A fenced block of code lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Block {
    pub(crate) lines: Vec<Line>,
    pub(crate) repeats: Vec<Repeat>,
}

/// One sortable entry of a group: a member, a variant, or an item.
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) lines: Vec<Line>,
    pub(crate) repeats: Vec<Repeat>,
}

/// Returns the printed text of a span.
pub(crate) fn span_text<'a>(span: &'a Span, refs: &'a [TypeRef]) -> &'a str {
    match span {
        Span::Key(s) | Span::Code(s) | Span::Comment(s) | Span::Name(s, _) => s,
        Span::Ref(id) | Span::DocRef(id) => &refs[*id].text,
        Span::Label(label) => label,
    }
}

/// Returns a line's printed text without its indent or trailing comment.
pub(crate) fn line_text(line: &Line, refs: &[TypeRef]) -> String {
    line.spans.iter().map(|s| span_text(s, refs)).collect()
}

/// Returns a line's width: its indent and its text, in characters.
pub(crate) fn width(indent: usize, spans: &[Span], refs: &[TypeRef]) -> usize {
    indent
        + spans
            .iter()
            .map(|s| span_text(s, refs).chars().count())
            .sum::<usize>()
}

/// Turns text into code spans.
pub(crate) fn code(text: &Text) -> Vec<Span> {
    text.0
        .iter()
        .map(|seg| match seg {
            Seg::Plain(s) => Span::Code(s.clone()),
            Seg::Ref(id) => Span::Ref(*id),
        })
        .collect()
}

/// Turns a `where` clause into spans with `where` as a keyword.
pub(crate) fn where_spans(text: &Text) -> Vec<Span> {
    let mut spans = code(text);
    if let Some(Span::Code(first)) = spans.first_mut() {
        if let Some(rest) = first.strip_prefix("where") {
            let rest = rest.to_owned();
            spans[0] = Span::Code(rest);
            spans.insert(0, Span::Key("where".to_owned()));
        }
    }
    spans
}

pub(crate) fn key(text: &str) -> Span {
    Span::Key(text.to_owned())
}

pub(crate) fn plain(text: &str) -> Span {
    Span::Code(text.to_owned())
}

/// Returns the lines of notes at an indent; doc comments use `prefix`.
pub(crate) fn note_lines(notes: &[Note], indent: usize, prefix: &str) -> Vec<Line> {
    let mut out = Vec::new();
    for note in notes {
        match note {
            Note::Doc(lines) => {
                for text in lines {
                    let mut spans = Vec::new();
                    let starts_bare = match text.0.first() {
                        Some(Seg::Plain(s)) => !s.starts_with(char::is_whitespace),
                        Some(Seg::Ref(_)) => true,
                        None => false,
                    };
                    spans.push(Span::Comment(if starts_bare {
                        format!("{prefix} ")
                    } else {
                        prefix.to_owned()
                    }));
                    for seg in &text.0 {
                        spans.push(match seg {
                            Seg::Plain(s) => Span::Comment(s.clone()),
                            Seg::Ref(id) => Span::DocRef(*id),
                        });
                    }
                    out.push(Line {
                        indent,
                        spans,
                        trailing: None,
                    });
                }
            }
            Note::Plain(lines) => out.extend(lines.iter().map(|line| Line {
                indent,
                spans: vec![Span::Comment(line.clone())],
                trailing: None,
            })),
        }
    }
    out
}

/// Returns a mark line, `// MACRO text`, and its repeat.
pub(crate) fn mark_line(
    label: &'static str,
    text: &str,
    indent: usize,
    loc: Option<Loc>,
) -> (Line, Repeat) {
    let line = Line {
        indent,
        spans: vec![
            Span::Comment("// ".to_owned()),
            Span::Label(label),
            Span::Comment(format!(" {text}")),
        ],
        trailing: None,
    };
    let repeat = Repeat {
        label,
        text: text.to_owned(),
        loc,
    };
    (line, repeat)
}

/// Returns the lines printed above a node: its notes, `cfg` lines, and
/// marks, with the marks' repeats.
pub(crate) fn above(meta: &Meta, indent: usize, loc: Loc) -> (Vec<Line>, Vec<Repeat>) {
    let mut lines = note_lines(&meta.notes, indent, "///");
    for cfg in &meta.cfgs {
        lines.push(Line {
            indent,
            spans: vec![Span::Comment(format!("// cfg: {cfg}"))],
            trailing: None,
        });
    }
    let mut repeats = Vec::new();
    for mark in &meta.marks {
        let (line, repeat) = mark_line("MACRO", mark, indent, Some(loc));
        lines.push(line);
        repeats.push(repeat);
    }
    (lines, repeats)
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

/// Returns empty sections of `groups` groups each.
pub(crate) fn sections(groups: usize) -> Sections {
    std::array::from_fn(|_| (0..groups).map(|_| Vec::new()).collect())
}

/// Returns the index of a visibility's section.
pub(crate) fn section(vis: Vis) -> usize {
    match vis {
        Vis::Public => 0,
        Vis::Internal => 1,
        Vis::Private => 2,
    }
}

/// Joins the sections that have entries in the order public, `internal:`,
/// `private:`, a blank line and the label before each. The first section
/// leaves its label out when its visibility is `default`, the section a
/// reader assumes: public in a struct or a module, private in a class.
pub(crate) fn join_sections(sections: Sections, default: Vis, block: &mut Block) {
    let labels = [
        (Vis::Public, "public:"),
        (Vis::Internal, "internal:"),
        (Vis::Private, "private:"),
    ];
    let mut first = true;
    for ((vis, label), groups) in labels.into_iter().zip(sections) {
        if groups.iter().all(Vec::is_empty) {
            continue;
        }
        if !(first && vis == default) {
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            block.lines.push(Line {
                indent: 0,
                spans: vec![key(label)],
                trailing: None,
            });
        }
        join_groups(groups, block);
        first = false;
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

/// Builds a function's lines, with `static` first when `is_static`.
pub(crate) fn fn_lines(
    krate: &Crate,
    sig: &crate::model::Sig,
    loc: Loc,
    trailing: Option<String>,
    indent: usize,
    is_static: bool,
) -> Vec<Line> {
    let mut head = Vec::new();
    if is_static {
        head.push(key("static"));
        head.push(plain(" "));
    }
    for qual in &sig.quals {
        head.push(key(qual));
        head.push(plain(" "));
    }
    head.push(Span::Name(sig.name.clone(), loc));
    head.extend(code(&sig.generics));
    let params = sig
        .params
        .iter()
        .map(|p| ParamParts {
            notes: note_lines(&p.meta.notes, 0, "///"),
            text: code(&p.text),
        })
        .collect();
    let after = if sig.receiver == crate::model::Receiver::Mutable {
        vec![plain(" "), key(MUT)]
    } else {
        Vec::new()
    };
    let parts = FnParts {
        head,
        params,
        after,
        ret: sig.ret.as_ref().map(code),
        where_clause: sig.where_clause.as_ref().map(where_spans),
        end: vec![plain(";")],
    };
    wrap::function(&parts, indent, trailing, &krate.refs)
}

/// Returns a constant's line, showing its value only when the whole line
/// fits.
pub(crate) fn const_line(
    mut spans: Vec<Span>,
    value: Option<&str>,
    indent: usize,
    trailing: Option<String>,
    refs: &[TypeRef],
) -> Line {
    if let Some(value) = value {
        let mut with = spans.clone();
        with.push(plain(&format!(" = {value};")));
        if width(indent, &with, refs) <= MAX_WIDTH {
            return Line {
                indent,
                spans: with,
                trailing,
            };
        }
    }
    spans.push(plain(";"));
    Line {
        indent,
        spans,
        trailing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
