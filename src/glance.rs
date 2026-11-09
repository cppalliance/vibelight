//! The compact view of a module's items, which the HTML prints above the
//! documented view of the same items: every declaration and item as Rust
//! writes it, without comments, attributes, or blank lines, its one-line
//! fields and variants packed onto as few lines as the width allows.

use crate::layout::{Line, Span, line_text, plain, width};
use crate::model::TypeRef;
use crate::wrap::MAX_WIDTH;

/// Returns the compact lines of a block: its code lines less every comment
/// line but the one that names the traits a type implements, every
/// attribute and `reaches` line, every blank line, and every trailing
/// comment, with runs of short lines that end in a comma packed.
pub(crate) fn compact(lines: &[Line], refs: &[TypeRef]) -> Vec<Line> {
    let kept: Vec<Line> = lines
        .iter()
        .filter(|line| kept(line))
        .map(|line| Line {
            indent: line.indent,
            spans: line.spans.iter().map(without_id).collect(),
            trailing: None,
        })
        .collect();
    pack(kept, refs)
}

/// Returns a span as the compact view prints it: the name of a function,
/// constant, or static links to its source like any member's name, and
/// leaves its id to the documented view, so no id repeats on the page.
fn without_id(span: &Span) -> Span {
    match span {
        Span::Value(text, loc, _) => Span::Name(text.clone(), *loc),
        other => other.clone(),
    }
}

/// Joins the compact lines of several blocks, one blank line between two
/// blocks.
pub(crate) fn joined<'a>(
    blocks: impl IntoIterator<Item = &'a [Line]>,
    refs: &[TypeRef],
) -> Vec<Line> {
    let mut out = Vec::new();
    for lines in blocks {
        let lines = compact(lines, refs);
        if lines.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(Line::default());
        }
        out.extend(lines);
    }
    out
}

fn kept(line: &Line) -> bool {
    match line.spans.first() {
        None | Some(Span::Dim(_)) => false,
        Some(Span::Marker(_)) => {
            matches!(line.spans.get(1), Some(Span::Comment(text)) if text == "implements ")
        }
        Some(_) => true,
    }
}

/// Packs each run of lines of one indent that end in a comma onto as few
/// lines as fit within the width, joined by a space.
fn pack(lines: Vec<Line>, refs: &[TypeRef]) -> Vec<Line> {
    let packable = |line: &Line| line.indent > 0 && line_text(line, refs).ends_with(',');
    let mut out: Vec<Line> = Vec::new();
    let mut open = false;
    for line in lines {
        if open && packable(&line) {
            if let Some(last) = out.last_mut().filter(|last| last.indent == line.indent) {
                let mut spans = last.spans.clone();
                spans.push(plain(" "));
                spans.extend(line.spans.iter().cloned());
                if width(last.indent, &spans, refs) <= MAX_WIDTH {
                    last.spans = spans;
                    continue;
                }
            }
        }
        open = packable(&line);
        out.push(line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::key;
    use crate::layout_marker::Marker;

    fn line(indent: usize, spans: Vec<Span>) -> Line {
        Line {
            indent,
            spans,
            trailing: None,
        }
    }

    fn text(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|l| format!("{}{}", " ".repeat(l.indent), line_text(l, &[])))
            .collect()
    }

    #[test]
    fn comments_attributes_and_blank_lines_go_and_short_fields_pack() {
        let lines = vec![
            line(
                0,
                vec![
                    Span::Marker(Marker::DOC.spaced),
                    Span::Comment("A point.".to_owned()),
                ],
            ),
            line(0, vec![Span::Dim("#[derive(Debug)]".to_owned())]),
            line(0, vec![key("pub struct"), plain(" Point {")]),
            line(
                4,
                vec![
                    Span::Marker(Marker::DOC.spaced),
                    Span::Comment("Across.".to_owned()),
                ],
            ),
            line(4, vec![plain("pub x: i32,")]),
            Line::default(),
            line(4, vec![plain("pub y: i32,")]),
            line(4, vec![plain("label: String,")]),
            line(0, vec![plain("}")]),
            Line::default(),
            line(0, vec![key("impl"), plain(" Point {")]),
            line(4, vec![plain("pub fn new() -> Point;")]),
            line(4, vec![plain("pub fn len(&self) -> usize;")]),
            line(0, vec![plain("}")]),
            line(
                0,
                vec![
                    Span::Marker(Marker::PLAIN.spaced),
                    Span::Comment("implements ".to_owned()),
                    Span::Comment("Display".to_owned()),
                ],
            ),
        ];
        assert_eq!(
            text(&compact(&lines, &[])),
            [
                "pub struct Point {",
                "    pub x: i32, pub y: i32, label: String,",
                "}",
                "impl Point {",
                "    pub fn new() -> Point;",
                "    pub fn len(&self) -> usize;",
                "}",
                "// implements Display",
            ]
        );
    }

    #[test]
    fn a_run_wider_than_the_width_wraps_onto_more_lines() {
        let field = |name: &str| line(4, vec![plain(&format!("pub {name}: SomeLongTypeName,"))]);
        let lines: Vec<Line> = ["first", "second", "third", "fourth"].map(field).to_vec();
        let packed = compact(&lines, &[]);
        assert_eq!(packed.len(), 2, "{:?}", text(&packed));
        assert!(
            packed
                .iter()
                .all(|l| width(l.indent, &l.spans, &[]) <= MAX_WIDTH)
        );
    }
}
