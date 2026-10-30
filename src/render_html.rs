//! Prints the document as one self-contained HTML file with every type
//! linked.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::document::{Document, Inline, Item, Link, Node};
use crate::layout::{Block, Line, Span, span_text};
use crate::model::{Loc, TypeRef, ValueRef};
use crate::render_style::STYLE;

/// The id of the checkbox that hides the member comments, which the sidebar
/// label and the style sheet both name.
const TOGGLE_ID: &str = "vibelight-comments";

/// The least indentation a comment line needs to be a member's comment and so
/// go in a box: a comment at indent 0 states the type's own contract, such as
/// `// internal` or `// impl Trait`, and stays visible.
const MEMBER_COMMENT_INDENT: usize = 4;

/// The box's left and right padding, which its margin subtracts so the
/// comment's text starts on the member's column. It matches the `padding` in
/// the box's rule in `render_style`.
const BOX_PADDING: &str = ".9rem";

/// A page of the run that other pages link into: its file name, the anchor
/// id of each of its declarations, and the id of each of its functions,
/// constants, and statics.
#[derive(Debug, Clone)]
pub(crate) struct Anchors {
    pub(crate) file: String,
    pub(crate) ids: Vec<String>,
    pub(crate) values: BTreeMap<ValueRef, String>,
}

/// Renders the HTML file, with `stamp` in a comment on its first line and
/// at the top of `<main>`. Member names link to their source lines when
/// `member_links` is given, and a name that reaches another crate of the
/// run links into that crate's page in `run`.
pub(crate) fn render(
    doc: &Document,
    member_links: Option<&dyn Fn(Loc) -> String>,
    run: &[Anchors],
    stamp: &str,
) -> String {
    let page = Page {
        doc,
        member_links,
        run,
    };
    let mut out = String::new();
    let _ = writeln!(out, "{}", crate::stamp::html_comment(stamp));
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(out, "<title>{}</title>", escape(&doc.title));
    out.push_str("<style>\n");
    out.push_str(STYLE);
    out.push_str("</style>\n</head>\n<body>\n");
    // The first element of the page: a checkbox the sidebar's label toggles,
    // so hiding the member comments needs no script.
    let _ = writeln!(
        out,
        "<input type=\"checkbox\" id=\"{TOGGLE_ID}\" class=\"toggle\" aria-label=\"Hide member \
         comments\">"
    );
    out.push_str("<nav>\n");
    let _ = writeln!(
        out,
        "<label for=\"{TOGGLE_ID}\" class=\"button\"><span class=\"on\">Show comments</span>\
         <span class=\"off\">Hide comments</span></label>"
    );
    page.list(&mut out, &doc.contents, "");
    out.push_str("</nav>\n<main>\n");
    page.head(&mut out, stamp);
    let _ = writeln!(
        out,
        "<p class=\"kill\"><b class=\"m\">MACROS:</b> {}</p>",
        escape(&doc.kill_summary)
    );
    if !doc.kill.is_empty() {
        page.list(&mut out, &doc.kill, "kill");
    }
    for node in &doc.sections {
        match node {
            Node::Heading { level, id, text } => {
                let _ = writeln!(
                    out,
                    "<h{level} id=\"{}\">{}</h{level}>",
                    attr(id),
                    escape(text)
                );
            }
            Node::Code(block) => page.pre(&mut out, block),
            Node::Source(links) => {
                out.push_str("<p class=\"source\">Source: ");
                for (i, source) in links.iter().enumerate() {
                    if i == 1 {
                        out.push_str(", impl ");
                    } else if i > 1 {
                        out.push_str(", ");
                    }
                    out.push_str(&source_link(&source.link));
                    if !source.members.is_empty() {
                        let _ = write!(out, " ({})", escape(&source.members.join(", ")));
                    }
                }
                out.push_str("</p>\n");
            }
            Node::Mark {
                label,
                text,
                source,
                id,
            } => {
                match id {
                    Some(id) => {
                        let _ = write!(out, "<p class=\"mark\" id=\"{}\">", attr(id));
                    }
                    None => out.push_str("<p class=\"mark\">"),
                }
                let _ = write!(out, "<b class=\"m\">{}</b> {}", escape(label), escape(text));
                if let Some(link) = source {
                    let _ = write!(
                        out,
                        " <span class=\"src\">(source: {})</span>",
                        source_link(link)
                    );
                }
                out.push_str("</p>\n");
            }
            Node::Rule => out.push_str("<hr>\n"),
            Node::List { class, items } => page.list(&mut out, items, class),
        }
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

/// One page's renderer: its document, its member links, and the other pages
/// of the run.
pub(crate) struct Page<'a> {
    pub(crate) doc: &'a Document,
    pub(crate) member_links: Option<&'a dyn Fn(Loc) -> String>,
    pub(crate) run: &'a [Anchors],
}

impl Page<'_> {
    fn refs(&self) -> &[TypeRef] {
        &self.doc.refs
    }

    /// Writes the top of the page: the stamp, the title, the crate docs, and
    /// the legend.
    fn head(&self, out: &mut String, stamp: &str) {
        let _ = writeln!(out, "{}", crate::stamp::html_paragraph(stamp));
        let _ = writeln!(
            out,
            "<h1 id=\"{}\">{}</h1>",
            attr(&self.doc.id),
            escape(&self.doc.title)
        );
        if let Some(docs) = &self.doc.crate_docs {
            self.crate_docs(out, docs);
        }
        if !self.doc.legend.is_empty() {
            out.push_str("<h2 id=\"legend\">Legend</h2>\n");
            self.list(out, &self.doc.legend, "legend");
        }
    }

    /// Writes the crate's own docs: prose that the comment toggle leaves
    /// alone, since it introduces the page rather than documenting a member.
    fn crate_docs(&self, out: &mut String, block: &Block) {
        let doc = leading_doc(&block.lines);
        if doc == 0 {
            return;
        }
        let spans = |out: &mut String, spans: &[Span]| self.ticked(out, spans);
        let text_of = |span: &Span| span_text(span, self.refs()).to_owned();
        crate::render_prose::block(
            out,
            &block.lines[..doc],
            &crate::render_prose::Inline {
                spans: &spans,
                text_of: &text_of,
                hideable: false,
            },
        );
    }

    fn pre(&self, out: &mut String, block: &Block) {
        let doc = leading_doc(&block.lines);
        if doc > 0 {
            // The prose path uses the same tick scan as a code line, so a
            // backticked name in a doc comment prints in the code font.
            let spans = |out: &mut String, spans: &[Span]| self.ticked(out, spans);
            let text_of = |span: &Span| span_text(span, self.refs()).to_owned();
            crate::render_prose::block(
                out,
                &block.lines[..doc],
                &crate::render_prose::Inline {
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
    /// the layout puts between members becomes the box's own margin when a
    /// comment line follows it, so it leaves with the box when the box is
    /// hidden; before a code line it is written as one newline. A newline is
    /// written before a box and never after it, because a box is a block and
    /// ends its own line: that is what leaves two members adjacent once their
    /// comments are hidden.
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
                    // joins them and the box's `white-space: pre` keeps it.
                    out.push('\n');
                    self.comment_line(out, line);
                } else {
                    if !first && !after_box {
                        out.push('\n');
                    }
                    // The blanks seen become the box's own margin, so the gap
                    // they stood for is hidden with the box.
                    //
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
                // The box is an inline-block, so it does not end its own line
                // the way a block does: a newline after it is what keeps the
                // next code line clear of the box.
                out.push_str("</div>\n");
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
            for _ in 0..blanks {
                out.push('\n');
            }
            blanks = 0;
            after_box = false;
            self.line(out, line);
            first = false;
        }
        if open_box {
            out.push_str("</div>");
        }
    }

    /// Writes a boxed member comment line's text, whose spacing the box's own
    /// `white-space: pre` keeps. The box's margin and padding put the text on
    /// the member's column, so any whitespace the comment marker and the
    /// source comment left at the start is dropped rather than pushing the
    /// first word off that column.
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
        self.ticked(out, &trimmed);
    }

    fn list(&self, out: &mut String, items: &[Item], class: &str) {
        if class.is_empty() {
            out.push_str("<ul>\n");
        } else {
            let _ = writeln!(out, "<ul class=\"{class}\">");
        }
        for item in items {
            out.push_str("<li>");
            self.inlines(out, &item.inlines);
            if item.children.is_empty() {
                out.push_str("</li>\n");
            } else {
                out.push('\n');
                self.list(out, &item.children, "");
                out.push_str("</li>\n");
            }
        }
        out.push_str("</ul>\n");
    }

    fn inlines(&self, out: &mut String, inlines: &[Inline]) {
        for inline in inlines {
            match inline {
                Inline::Text(text) => out.push_str(&escape(text)),
                Inline::Bold(text) => {
                    let _ = write!(out, "<b class=\"m\">{}</b>", escape(text));
                }
                Inline::Code(spans) => {
                    out.push_str("<code>");
                    self.spans(out, spans);
                    out.push_str("</code>");
                }
                Inline::Anchor { text, id } => {
                    let _ = write!(out, "<a href=\"#{}\">{}</a>", attr(id), escape(text));
                }
                Inline::Kill {
                    text,
                    heading,
                    mark,
                } => {
                    let target = mark.as_ref().unwrap_or(heading);
                    let _ = write!(out, "<a href=\"#{}\">{}</a>", attr(target), escape(text));
                }
                Inline::Source(links) => {
                    let links: Vec<String> = links.iter().map(source_link).collect();
                    let _ = write!(
                        out,
                        " <span class=\"src\">(source: {})</span>",
                        links.join(", ")
                    );
                }
                Inline::Break => out.push_str("<br>"),
            }
        }
    }
}

fn source_link(link: &Link) -> String {
    format!(
        "<a href=\"{}\">{}</a>",
        attr(&link.target),
        escape(&link.text)
    )
}

/// Returns how many of a block's leading lines are doc comment lines, the
/// run the page renders as prose. A block that opens with anything else, a
/// mark or code, has no prose run.
fn leading_doc(lines: &[Line]) -> usize {
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

/// Escapes text for HTML content.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escapes text for an attribute value in double quotes.
pub(crate) fn attr(text: &str) -> String {
    escape(text).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(escape("Vec<&'a T>"), "Vec&lt;&amp;'a T&gt;");
        assert_eq!(attr("a\"b"), "a&quot;b");
    }
}
