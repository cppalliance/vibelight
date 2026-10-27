//! The pieces of the Rust spelling, which the markdown prints: text as Rust
//! writes it, attribute lines, and the lines of a function, a constant,
//! and a `where` clause. A signature stays on one line whatever its width,
//! since its reader is a model.

use crate::layout::{Line, Repeat, Span, key, mark_line, note_lines, width};
use crate::model::{Loc, Meta, Note, Seg, Sig, Text, TypeRef, Where};

/// The widest a constant's line may be and still show its value.
const VALUE_WIDTH: usize = 100;

/// Turns text into code spans in its Rust spelling.
pub(crate) fn code(text: &Text) -> Vec<Span> {
    text.rust()
        .iter()
        .map(|seg| match seg {
            Seg::Plain(s) => Span::Code(s.clone()),
            Seg::Ref(id) => Span::Ref(*id),
        })
        .collect()
}

pub(crate) fn plain(text: &str) -> Span {
    Span::Code(text.to_owned())
}

pub(crate) fn line(indent: usize, spans: Vec<Span>) -> Line {
    Line {
        indent,
        spans,
        trailing: None,
    }
}

/// Returns a visibility as written with the space after it, or nothing.
pub(crate) fn vis(meta: &Meta) -> String {
    if meta.vis.is_empty() {
        String::new()
    } else {
        format!("{} ", meta.vis)
    }
}

/// Returns the lines above a node: its doc comments, its attribute lines,
/// and its macro marks, with their repeats.
pub(crate) fn above(meta: &Meta, indent: usize, loc: Loc) -> (Vec<Line>, Vec<Repeat>) {
    let mut lines = note_lines(&meta.notes, indent, "///");
    lines.extend(attr_lines(&[], meta, indent));
    let mut repeats = Vec::new();
    for mark in &meta.marks {
        let (mark_line, repeat) = mark_line("MACRO", mark, indent, Some(loc));
        lines.push(mark_line);
        repeats.push(repeat);
    }
    (lines, repeats)
}

/// Returns one `#[cfg(...)]` line per condition, a `#[derive(...)]` line
/// when `derives` names a trait, and one line per attribute that shapes an
/// API or its wire format.
pub(crate) fn attr_lines(derives: &[String], meta: &Meta, indent: usize) -> Vec<Line> {
    let mut lines: Vec<Line> = meta
        .cfgs
        .iter()
        .map(|cfg| line(indent, vec![plain(&format!("#[cfg({cfg})]"))]))
        .collect();
    if !derives.is_empty() {
        let text = format!("#[derive({})]", derives.join(", "));
        lines.push(line(indent, vec![plain(&text)]));
    }
    lines.extend(
        meta.attrs
            .iter()
            .map(|attr| line(indent, vec![plain(&format!("#[{attr}]"))])),
    );
    lines
}

/// Returns a `where` clause on lines of its own: `where`, then each
/// predicate, indented 4 more, every one but the last ending in a comma and
/// the last in `end`.
pub(crate) fn where_lines(clause: &Where, indent: usize, end: &str) -> Vec<Line> {
    let mut lines = vec![line(indent, vec![key("where")])];
    let count = clause.0.len();
    for (index, predicate) in clause.0.iter().enumerate() {
        let mut spans = code(predicate);
        spans.push(plain(if index + 1 < count { "," } else { end }));
        lines.push(line(indent + 4, spans));
    }
    lines
}

/// Spells a qualifier the way Rust writes it.
fn qualifier(text: &str) -> &str {
    if text == "constexpr" { "const" } else { text }
}

/// Returns a function's lines, ending in `;`: on one line, or with one
/// parameter per line when a parameter has comments, and a `where` clause
/// on lines of its own.
pub(crate) fn fn_lines(sig: &Sig, vis: &str, indent: usize, trailing: Option<String>) -> Vec<Line> {
    let mut head = vec![plain(vis)];
    for qual in &sig.quals {
        head.push(key(qualifier(qual)));
        head.push(plain(" "));
    }
    head.push(key("fn"));
    head.push(plain(&format!(" {}", sig.name)));
    head.extend(code(&sig.generics));
    let mut params: Vec<(&[Note], Vec<Span>)> = Vec::new();
    if let Some(receiver) = &sig.self_text {
        params.push((&[], vec![plain(receiver)]));
    }
    params.extend(
        sig.params
            .iter()
            .map(|p| (p.meta.notes.as_slice(), code(&p.text))),
    );
    let mut tail = vec![plain(")")];
    if let Some(ret) = &sig.ret {
        tail.push(plain(" -> "));
        tail.extend(code(ret));
    }
    if sig.where_clause.is_none() {
        tail.push(plain(";"));
    }
    let mut lines = Vec::new();
    if params.iter().all(|(notes, _)| notes.is_empty()) {
        let mut spans = head;
        spans.push(plain("("));
        for (index, (_, param)) in params.into_iter().enumerate() {
            if index > 0 {
                spans.push(plain(", "));
            }
            spans.extend(param);
        }
        spans.extend(tail);
        lines.push(line(indent, spans));
    } else {
        head.push(plain("("));
        lines.push(line(indent, head));
        for (notes, mut param) in params {
            lines.extend(note_lines(notes, indent + 4, "///"));
            param.push(plain(","));
            lines.push(line(indent + 4, param));
        }
        lines.push(line(indent, tail));
    }
    if let Some(clause) = &sig.where_clause {
        lines.extend(where_lines(clause, indent, ";"));
    }
    if let Some(last) = lines.last_mut() {
        last.trailing = trailing;
    }
    lines
}

/// Returns a constant or static's line: its head, then its value, or
/// ` = ...` when the value makes the line wider than `VALUE_WIDTH`, or
/// only `;` when no value is shown.
pub(crate) fn value_line(
    mut spans: Vec<Span>,
    value: Option<&str>,
    indent: usize,
    trailing: Option<String>,
    refs: &[TypeRef],
) -> Line {
    if let Some(value) = value {
        let text = format!(" = {value};");
        let wide = width(indent, &spans, refs) + text.chars().count() > VALUE_WIDTH;
        spans.push(plain(if wide { " = ...;" } else { &text }));
    } else {
        spans.push(plain(";"));
    }
    Line {
        indent,
        spans,
        trailing,
    }
}
