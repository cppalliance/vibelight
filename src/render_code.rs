//! The page's code blocks: a block's leading doc lines as prose, then its
//! code lines as Rust writes them, comments and all.

use crate::layout::{Block, Line, Span, span_text};
use crate::render_html::Page;

impl Page<'_> {
    pub(crate) fn pre(&self, out: &mut String, block: &Block) {
        let doc = leading_doc(&block.lines);
        if doc > 0 {
            // The prose path uses the same tick scan as a code line, so a
            // backticked name in a doc comment prints in the code font.
            let spans = |out: &mut String, spans: &[Span]| self.ticked(out, &unmarked(spans));
            let text_of = |span: &Span| span_text(span, self.refs()).to_owned();
            crate::render_prose::block(
                out,
                &block.lines[..doc],
                &crate::render_prose::InlineRenderer {
                    spans: &spans,
                    text_of: &text_of,
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

    /// Writes a block's code lines, one per line, a blank line as an empty
    /// one.
    pub(crate) fn code_lines(&self, out: &mut String, lines: &[Line]) {
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            self.line(out, line);
        }
    }
}

/// Returns spans without the markers that open comment lines, which prose
/// leaves out.
pub(crate) fn unmarked(spans: &[Span]) -> Vec<Span> {
    spans
        .iter()
        .filter(|span| !matches!(span, Span::Marker(_)))
        .cloned()
        .collect()
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
