//! Styled lines and blocks, and the helpers every layout step shares.

use crate::layout_marker::Marker;
use crate::model::{Crate, Loc, Meta, Note, Receiver, RefId, Seg, Text, TypeRef, ValueRef, Where};
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

/// Turns text into code spans, as Rust spells it.
pub(crate) fn code(text: &Text) -> Vec<Span> {
    text.rust()
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

/// Returns a visibility as written and the space after it, or nothing for
/// a node with no visibility keyword.
pub(crate) fn vis_spans(vis: &str) -> Vec<Span> {
    if vis.is_empty() {
        Vec::new()
    } else {
        vec![key(vis), plain(" ")]
    }
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

/// Returns the lines printed above a node: its notes, its `cfg`s and its
/// attributes as dim `#[...]` lines, and its marks, with the marks' repeats.
pub(crate) fn above(meta: &Meta, indent: usize, loc: Loc) -> (Vec<Line>, Vec<Repeat>) {
    let mut lines = note_lines(&meta.notes, indent, Marker::DOC);
    let attrs = meta
        .cfgs
        .iter()
        .map(|cfg| format!("#[cfg({cfg})]"))
        .chain(meta.attrs.iter().map(|attr| format!("#[{attr}]")));
    lines.extend(attrs.map(|text| Line {
        indent,
        spans: vec![Span::Dim(text)],
        trailing: None,
    }));
    let mut repeats = Vec::new();
    for mark in &meta.marks {
        let (line, repeat) = mark_line("MACRO", mark, indent, Some(loc));
        lines.push(line);
        repeats.push(repeat);
    }
    (lines, repeats)
}

/// Builds a function's lines as Rust declares it, ending in `;`: its
/// visibility `vis` as written, its qualifiers, `fn`, and `name`, the span
/// that prints the function's name, then its receiver and parameters.
pub(crate) fn fn_lines(
    krate: &Crate,
    sig: &crate::model::Sig,
    vis: &str,
    name: Span,
    trailing: Option<String>,
    indent: usize,
) -> Vec<Line> {
    let mut head = vis_spans(vis);
    for qual in &sig.quals {
        head.push(key(crate::render_slop_text::qualifier(qual)));
        head.push(plain(" "));
    }
    head.extend([key("fn"), plain(" "), name]);
    head.extend(code(&sig.generics));
    let mut params = Vec::new();
    if let Receiver::Shared(text) | Receiver::Mutable(text) = &sig.receiver {
        params.push(ParamParts {
            notes: Vec::new(),
            text: receiver(text),
        });
    }
    params.extend(sig.params.iter().map(|p| ParamParts {
        notes: note_lines(&p.meta.notes, 0, Marker::DOC),
        text: code(&p.text),
    }));
    let parts = FnParts {
        head,
        params,
        ret: sig.ret.as_ref().map(code),
        where_clause: sig.where_clause.as_ref().map(where_parts),
        end: vec![plain(";")],
    };
    wrap::function(&parts, indent, trailing, &krate.refs)
}

/// Returns a reference receiver as written, such as `&'a mut self`, with
/// `mut` and `self` as keywords.
fn receiver(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            spans.push(plain(" "));
        }
        if let Some(before) = word.strip_suffix("self") {
            if !before.is_empty() {
                spans.push(plain(before));
            }
            spans.push(key("self"));
        } else if word == "mut" {
            spans.push(key("mut"));
        } else {
            spans.push(plain(word));
        }
    }
    spans
}

/// Returns a constant's line, ending in `;`, showing its value only when the
/// whole line fits.
pub(crate) fn const_line(
    spans: Vec<Span>,
    value: Option<&str>,
    indent: usize,
    trailing: Option<String>,
    refs: &[TypeRef],
) -> Line {
    let end = |mut spans: Vec<Span>| {
        spans.push(plain(";"));
        spans
    };
    if let Some(value) = value {
        let mut with = spans.clone();
        with.push(plain(&format!(" = {value}")));
        let with = end(with);
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
        spans: end(spans),
        trailing,
    }
}
