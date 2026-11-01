//! The page's code blocks: a block's leading doc lines as prose, then its
//! code lines, each run of member comment lines in its own box.

use std::fmt::Write as _;

use crate::layout::{Block, Line, Span, span_text};
use crate::layout_marker::Marker;
use crate::render_html::Page;

/// The least indentation a comment line needs to be a member's comment and so
/// go in a box: a comment at indent 0 states the type's own contract, such as
/// `// internal` or `// impl Trait`, and stays visible.
const MEMBER_COMMENT_INDENT: usize = 4;

/// The box's left and right padding, which its margin subtracts so the
/// comment's text starts on the member's column. It matches the `padding` in
/// the box's rule in `render_style`.
const BOX_PADDING: &str = ".9rem";

impl Page<'_> {
    pub(crate) fn pre(&self, out: &mut String, block: &Block) {
        let doc = leading_doc(&block.lines);
        if doc > 0 {
            // The prose path uses the same tick scan as a code line, so a
            // backticked name in a doc comment prints in the code font.
            let spans = |out: &mut String, spans: &[Span]| self.ticked(out, spans);
            let text_of = |span: &Span| span_text(span, self.refs()).to_owned();
            crate::render_prose::block(
                out,
                &block.lines[..doc],
                &crate::render_prose::InlineRenderer {
                    spans: &spans,
                    text_of: &text_of,
                    hideable: true,
                },
            );
        }
        let rest = &block.lines[doc..];
        if rest.is_empty() {
            return;
        }
        out.push_str("<pre><code>");
        self.code_lines(out, rest);
        out.push_str("</code></pre>\n");
    }

    /// Writes a block's code lines, each run of member comment lines in its
    /// own box.
    ///
    /// A blank line is never written into the flow as its own text. The one
    /// the layout puts between members becomes a gap, an empty block that is
    /// as tall as the blank line while comments show and half that while they
    /// are hidden. A gap before a comment box is empty while comments show,
    /// since the box's own margin already spaces it. A newline is written
    /// before a box and never after it, because a box is a block and ends its
    /// own line: that leaves nothing behind once a box is hidden, so the
    /// first member sits directly under the brace.
    fn code_lines(&self, out: &mut String, lines: &[Line]) {
        let mut first = true;
        let mut open_box = false;
        // True once a box closed: the block ended that line, so the next line
        // needs no newline of its own.
        let mut after_box = false;
        // Blank lines seen since the last line written.
        let mut blanks = 0usize;
        for line in lines {
            let blank = line.spans.is_empty() && line.trailing.is_none();
            let boxed = !blank
                && line.indent >= MEMBER_COMMENT_INDENT
                && matches!(line.spans.first(), Some(Span::Marker(_)));
            if boxed {
                if open_box {
                    // Inside a box the lines are one text run, so a newline
                    // joins them, and the box's `white-space: normal` turns
                    // that newline, and any blank comment line, into one
                    // space: the comment reflows as one paragraph.
                    out.push('\n');
                    self.comment_line(out, line);
                } else {
                    if !first && !after_box {
                        out.push('\n');
                    }
                    gaps(out, blanks, "gap gap-box");
                    // The box's left edge sits one padding-width left of the
                    // member's column, so the comment's text starts exactly
                    // where the declaration's text does. `ch` is the code
                    // font's own character width, which is the unit the
                    // layout indents in, so the box names the code font and
                    // never the page's proportional one.
                    let _ = write!(
                        out,
                        "<div class=\"mc\" style=\"margin-left:calc({}ch - {})\">",
                        line.indent, BOX_PADDING
                    );
                    open_box = true;
                    self.comment_line(out, line);
                }
                blanks = 0;
                after_box = false;
                first = false;
                continue;
            }
            if open_box {
                out.push_str("</div>");
                open_box = false;
                after_box = true;
            }
            if blank {
                blanks += 1;
                continue;
            }
            if !first && !after_box {
                out.push('\n');
            }
            gaps(out, blanks, "gap");
            blanks = 0;
            after_box = false;
            self.line(out, line);
            first = false;
        }
        if open_box {
            out.push_str("</div>");
        }
    }

    /// Writes a boxed member comment line's text, which the box's own
    /// `white-space: normal` reflows with the lines around it. The box's
    /// margin and padding put the text on the member's column, so any
    /// whitespace the comment marker and the source comment left at the start
    /// is dropped rather than pushing the first word off that column.
    fn comment_line(&self, out: &mut String, line: &Line) {
        let mut started = false;
        let mut trimmed: Vec<Span> = Vec::new();
        for span in &line.spans {
            if started {
                trimmed.push(span.clone());
                continue;
            }
            match span {
                Span::Marker(_) => {}
                Span::Comment(text) => match text.trim_start() {
                    "" => {}
                    rest => {
                        started = true;
                        trimmed.push(Span::Comment(rest.to_owned()));
                    }
                },
                Span::DocRef(_) | Span::Ref(_) => {
                    started = true;
                    trimmed.push(span.clone());
                }
                other => {
                    started = true;
                    trimmed.push(other.clone());
                }
            }
        }
        // A `cfg` line is a condition and a `# Errors` line is a heading, so
        // neither runs into the prose: each keeps its own line inside the
        // reflowed paragraph.
        let cfg = matches!(line.spans.first(), Some(Span::Marker(m)) if *m == Marker::CFG.spaced);
        let heading = matches!(trimmed.first(), Some(Span::Comment(text)) if is_heading(text));
        let class = if cfg {
            "cfg"
        } else if heading {
            "sec"
        } else {
            ""
        };
        if !class.is_empty() {
            let _ = write!(out, "<span class=\"{class}\">");
        }
        self.ticked(out, &trimmed);
        if !class.is_empty() {
            out.push_str("</span>");
        }
    }
}

/// Tells whether a comment line is a heading such as `# Errors`: one to six
/// `#` and then a space.
fn is_heading(text: &str) -> bool {
    let rest = text.trim_start_matches('#');
    let hashes = text.len() - rest.len();
    (1..=6).contains(&hashes) && rest.starts_with(' ')
}

/// Writes `count` gaps of one class: the empty blocks that stand for the
/// blank lines of a block, so the style sheet sets their height.
fn gaps(out: &mut String, count: usize, class: &str) {
    for _ in 0..count {
        let _ = write!(out, "<span class=\"{class}\"></span>");
    }
}

/// Returns how many of a block's leading lines are doc comment lines, the
/// run the page renders as prose. A block that opens with anything else, a
/// mark or code, has no prose run.
pub(crate) fn leading_doc(lines: &[Line]) -> usize {
    lines
        .iter()
        .take_while(|line| {
            matches!(
                line.spans.first(),
                Some(Span::Marker(mark)) if mark.starts_with("///") || mark.starts_with("//!")
            )
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_style::STYLE;

    #[test]
    fn style_padding_matches_box_padding() {
        assert!(STYLE.contains(&format!("padding:.2rem {BOX_PADDING} 0")));
    }
}
