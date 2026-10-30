//! Styled lines and blocks, and the helpers every layout step shares.

use crate::layout_marker::Marker;
use crate::model::{
    Crate, Loc, Meta, Note, RefId, Seg, Text, TypeRef, ValueRef, Vis, Where, name_order,
};
use crate::typerefs_edits::MUT;
use crate::wrap::{self, FnParts, MAX_WIDTH, ParamParts, WhereParts};

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
    /// The text that opens a comment line, which the HTML drops.
    Marker(&'static str),
    /// An attribute shown at the end of a signature, printed at half the
    /// contrast of the code around it.
    Dim(String),
    /// A type name in a comment, which links in HTML but never counts.
    DocRef(RefId),
    /// A bold label: `MACRO` or `UNSUPPORTED:`.
    Label(&'static str),
    /// A member or item name, which HTML links to its source line.
    Name(String, Loc),
    /// The name of a module-level function, constant, or static, which HTML
    /// links to its source line and gives an id.
    Value(String, Loc, ValueRef),
}

/// One line of a block.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) indent: usize,
    pub(crate) spans: Vec<Span>,
    /// An ordinary comment at the end of the line, outside the width rule.
    pub(crate) trailing: Option<String>,
}

/// A mark inside a block, which the HTML repeats as a bold line under it.
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
        Span::Key(s) | Span::Code(s) | Span::Comment(s) | Span::Name(s, _) | Span::Value(s, ..) => {
            s
        }
        Span::Marker(s) => s,
        Span::Dim(s) => s,
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
pub(crate) fn where_parts(clause: &Where) -> WhereParts {
    let mut one = code(&clause.joined());
    if let Some(Span::Code(first)) = one.first_mut() {
        if let Some(rest) = first.strip_prefix("where") {
            let rest = rest.to_owned();
            one[0] = Span::Code(rest);
            one.insert(0, Span::Key("where".to_owned()));
        }
    }
    WhereParts {
        one,
        predicates: clause.0.iter().map(code).collect(),
    }
}

pub(crate) fn key(text: &str) -> Span {
    Span::Key(text.to_owned())
}

pub(crate) fn plain(text: &str) -> Span {
    Span::Code(text.to_owned())
}

/// Returns the lines of notes at an indent; `marker` opens each doc line.
pub(crate) fn note_lines(notes: &[Note], indent: usize, marker: Marker) -> Vec<Line> {
    let mut out = Vec::new();
    for note in notes {
        match note {
            Note::Doc(lines) => {
                for text in lines {
                    let mut spans = Vec::new();
                    // A line with no segments prints the bare marker, while
                    // an empty first segment keeps the space, so a `///` line
                    // whose attribute held only whitespace prints as it did.
                    let already_indented = match text.0.first() {
                        Some(Seg::Plain(s)) => s.starts_with(char::is_whitespace),
                        Some(Seg::Ref(_)) => false,
                        None => true,
                    };
                    spans.push(Span::Marker(marker.text(already_indented)));
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
            Note::Plain(lines) => {
                out.extend(lines.iter().map(|line| plain_marker_line(line, indent)));
            }
        }
    }
    out
}

/// Returns one ordinary comment line, its `//` kept in a marker span so the
/// HTML can drop it. The line's own leading spaces stay in its text, and
/// its indent alone is the layout indent, so the line keeps the shape the
/// source gives it.
fn plain_marker_line(line: &str, indent: usize) -> Line {
    let trimmed = line.trim_start();
    let lead = &line[..line.len() - trimmed.len()];
    let mut spans = Vec::new();
    if !lead.is_empty() {
        spans.push(Span::Comment(lead.to_owned()));
    }
    match trimmed.strip_prefix("//") {
        Some(rest) => {
            spans.push(Span::Marker(Marker::PLAIN.bare));
            spans.push(Span::Comment(rest.to_owned()));
        }
        None => spans.push(Span::Comment(trimmed.to_owned())),
    }
    Line {
        indent,
        spans,
        trailing: None,
    }
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
            Span::Marker(Marker::PLAIN.spaced),
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

/// Returns the lines printed above a node: its notes, `cfg` lines, its
/// attributes when `attrs` is set, each dimmed, and marks, with the marks'
/// repeats. A function and a field set it off: their attributes close the
/// line they belong to.
pub(crate) fn above(meta: &Meta, indent: usize, loc: Loc, attrs: bool) -> (Vec<Line>, Vec<Repeat>) {
    let mut lines = note_lines(&meta.notes, indent, Marker::DOC);
    for cfg in &meta.cfgs {
        lines.push(Line {
            indent,
            spans: vec![Span::Marker(Marker::CFG.spaced), Span::Comment(cfg.clone())],
            trailing: None,
        });
    }
    for attr in meta.attrs.iter().filter(|_| attrs) {
        lines.push(Line {
            indent,
            spans: vec![Span::Dim(format!("[[{attr}]]"))],
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

/// Joins a type body's data sections, then its member sections, each set in
/// the order public, `internal:`, `private:`, with a blank line before every
/// section that has entries. A label opens a section only when its
/// visibility is not the access already in effect, which starts as `default`,
/// the access the keyword gives: public in a struct, private in a class. So a
/// visibility that holds both data and members can print its label twice,
/// as a C++ header does. Only a type uses this: a module has no enclosing
/// brace, so its items carry their own visibility line instead.
pub(crate) fn join_sections(data: Sections, members: Sections, default: Vis, block: &mut Block) {
    let labels = [
        (Vis::Public, "public:"),
        (Vis::Internal, "internal:"),
        (Vis::Private, "private:"),
    ];
    let mut current = default;
    for set in [data, members] {
        for ((vis, label), groups) in labels.into_iter().zip(set) {
            if groups.iter().all(Vec::is_empty) {
                continue;
            }
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            if vis != current {
                block.lines.push(Line {
                    indent: 0,
                    spans: vec![key(label)],
                    trailing: None,
                });
                current = vis;
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

/// Builds a function's lines, with `static` first when `is_static`. `name`
/// is the span that prints the function's name. The function's attributes
/// close the signature, in place of the semicolon, so `[[must_use]]` reads as
/// a note on the line rather than a line of its own.
pub(crate) fn fn_lines(
    krate: &Crate,
    sig: &crate::model::Sig,
    name: Span,
    trailing: Option<String>,
    indent: usize,
    is_static: bool,
    attrs: &[String],
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
    head.push(name);
    head.extend(code(&sig.generics));
    let params = sig
        .params
        .iter()
        .map(|p| ParamParts {
            notes: note_lines(&p.meta.notes, 0, Marker::DOC),
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
        where_clause: sig.where_clause.as_ref().map(where_parts),
        // An attribute closes the signature. The page needs no semicolon, so
        // a function without one ends at its return type.
        end: attributes_end(attrs),
    };
    wrap::function(&parts, indent, trailing, &krate.refs)
}

/// Returns the spans that close a signature or a field: its attributes, each
/// dimmed, or nothing when it has none. Each attribute is separated from
/// what comes before by one space.
pub(crate) fn attributes_end(attrs: &[String]) -> Vec<Span> {
    let mut spans = Vec::new();
    for attr in attrs {
        spans.push(plain(" "));
        spans.push(Span::Dim(format!("[[{attr}]]")));
    }
    spans
}

/// Returns a constant's line, showing its value only when the whole line
/// fits.
pub(crate) fn const_line(
    spans: Vec<Span>,
    value: Option<&str>,
    indent: usize,
    trailing: Option<String>,
    refs: &[TypeRef],
) -> Line {
    if let Some(value) = value {
        let mut with = spans.clone();
        with.push(plain(&format!(" = {value}")));
        if width(indent, &with, refs) <= MAX_WIDTH {
            return Line {
                indent,
                spans: with,
                trailing,
            };
        }
    }
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
