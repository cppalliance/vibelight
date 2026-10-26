//! Prints the document as markdown: no links inside fences, marks outside.

use std::fmt::Write as _;

use crate::document::{Document, Inline, Item, Link, Node};
use crate::layout::{Block, Line, span_text};
use crate::model::TypeRef;

/// Renders the markdown file, with `stamp` on its first line.
pub(crate) fn render(doc: &Document, stamp: &str) -> String {
    let mut out = String::new();
    let _ = write!(out, "{}\n\n", crate::stamp::markdown(stamp));
    let _ = write!(out, "<a id=\"{}\"></a>\n# {}\n", doc.id, doc.title);
    if let Some(docs) = &doc.crate_docs {
        out.push('\n');
        fence(&mut out, docs, &doc.refs);
    }
    out.push_str("\n<a id=\"legend\"></a>\n## Legend\n\n");
    list(&mut out, &doc.legend, 0, &doc.refs);
    out.push('\n');
    list(&mut out, &doc.contents, 0, &doc.refs);
    if let Some(summary) = &doc.kill_summary {
        out.push('\n');
        let _ = writeln!(out, "**MACROS:** {summary}");
        list(&mut out, &doc.kill, 0, &doc.refs);
    }
    for node in &doc.sections {
        out.push('\n');
        match node {
            Node::Heading { level, id, text } => {
                let _ = writeln!(out, "<a id=\"{id}\"></a>");
                let _ = writeln!(out, "{} {text}", "#".repeat(usize::from(*level)));
            }
            Node::Code(block) => fence(&mut out, block, &doc.refs),
            Node::Source(links) => {
                out.push_str("Source: ");
                for (i, source) in links.iter().enumerate() {
                    if i == 1 {
                        out.push_str(", impl ");
                    } else if i > 1 {
                        out.push_str(", ");
                    }
                    out.push_str(&link_text(&source.link));
                    if !source.members.is_empty() {
                        let _ = write!(out, " ({})", source.members.join(", "));
                    }
                }
                out.push('\n');
            }
            Node::Mark {
                label,
                text,
                source,
                ..
            } => {
                let _ = write!(out, "**{label}** {text}");
                if let Some(link) = source {
                    let _ = write!(out, " (source: {})", link_text(link));
                }
                out.push('\n');
            }
            Node::List { items, .. } => list(&mut out, items, 0, &doc.refs),
        }
    }
    out
}

fn link_text(link: &Link) -> String {
    format!("[{}]({})", link.text, link.target)
}

/// Prints a fenced block whose fence is longer than any backtick run that
/// starts a line inside it.
fn fence(out: &mut String, block: &Block, refs: &[TypeRef]) {
    let lines: Vec<String> = block
        .lines
        .iter()
        .map(|line| print_line(line, refs))
        .collect();
    let longest = lines
        .iter()
        .map(|l| l.trim_start().chars().take_while(|&c| c == '`').count())
        .max()
        .unwrap_or(0);
    let marks = "`".repeat((longest + 1).max(3));
    let _ = writeln!(out, "{marks}");
    for line in lines {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(out, "{marks}");
}

/// Prints one block line: its indent, its text, and any trailing comment.
pub(crate) fn print_line(line: &Line, refs: &[TypeRef]) -> String {
    let body: String = line.spans.iter().map(|s| span_text(s, refs)).collect();
    let mut text = if body.is_empty() {
        String::new()
    } else {
        format!("{}{body}", " ".repeat(line.indent))
    };
    if let Some(trailing) = &line.trailing {
        text.push(' ');
        text.push_str(trailing);
    }
    text
}

fn list(out: &mut String, items: &[Item], depth: usize, refs: &[TypeRef]) {
    let indent = "  ".repeat(depth);
    for item in items {
        let _ = write!(out, "{indent}- ");
        for inline in &item.inlines {
            match inline {
                Inline::Text(text) => out.push_str(text),
                Inline::Bold(text) => {
                    let _ = write!(out, "**{text}**");
                }
                Inline::Code(spans) => {
                    let text: String = spans.iter().map(|s| span_text(s, refs)).collect();
                    out.push_str(&code_span(&text));
                }
                Inline::Anchor { text, id }
                | Inline::Kill {
                    text, heading: id, ..
                } => {
                    let _ = write!(out, "[{text}](#{id})");
                }
                Inline::Source(links) => {
                    let links: Vec<String> = links.iter().map(link_text).collect();
                    let _ = write!(out, " (source: {})", links.join(", "));
                }
                Inline::Break => {
                    let _ = write!(out, "\\\n{indent}  ");
                }
            }
        }
        out.push('\n');
        list(out, &item.children, depth + 1, refs);
    }
}

/// Wraps text in a code span whose backtick run is longer than any run
/// inside it.
fn code_span(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        if c == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let ticks = "`".repeat(longest + 1);
    if text.starts_with('`') || text.ends_with('`') {
        format!("{ticks} {text} {ticks}")
    } else {
        format!("{ticks}{text}{ticks}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Span;

    #[test]
    fn fences_outgrow_backtick_runs_at_line_starts() {
        let block = Block {
            lines: vec![
                Line {
                    indent: 0,
                    spans: vec![Span::Comment("/// ```".to_owned())],
                    trailing: None,
                },
                Line {
                    indent: 0,
                    spans: vec![Span::Comment("````".to_owned())],
                    trailing: None,
                },
            ],
            repeats: Vec::new(),
        };
        let mut out = String::new();
        fence(&mut out, &block, &[]);
        assert_eq!(out, "`````\n/// ```\n````\n`````\n");
    }

    #[test]
    fn code_spans_outgrow_inner_backticks() {
        assert_eq!(code_span("a"), "`a`");
        assert_eq!(code_span("see `x`"), "`` see `x` ``");
        assert_eq!(code_span("`x"), "`` `x ``");
    }
}
