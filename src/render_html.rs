//! Prints the document as one page of the HTML folder, with every type
//! linked: the sidebar and the page's content, which the folder's one file
//! holds packed (see `browse`). A code block prints
//! its leading doc lines as prose, then its code lines, each run of member
//! comment lines in its own box.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::document::{Document, Inline, Item, Link, Node};
use crate::layout::{Block, Line, Span, span_text};
use crate::layout_marker::Marker;
use crate::model::{Loc, TypeRef, ValueRef};

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

/// Renders the page's content, with `stamp` at the top of `<main>`. Member
/// names link to their source lines when `member_links` is given, and a name
/// that reaches another crate of the run links into that crate's page in
/// `run`.
pub(crate) fn fragment(
    doc: &Document,
    member_links: Option<&dyn Fn(Loc) -> String>,
    run: &[Anchors],
    stamp: &str,
) -> String {
    let page = Page {
        doc,
        member_links,
        run,
        foreign: None,
        ids: true,
        api: None,
    };
    let mut out = String::from("<nav>\n");
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
        page.node(&mut out, node);
    }
    out.push_str("</main>\n");
    out
}

/// One page's renderer: its document, its member links, and the other pages
/// of the run.
pub(crate) struct Page<'a> {
    pub(crate) doc: &'a Document,
    pub(crate) member_links: Option<&'a dyn Fn(Loc) -> String>,
    pub(crate) run: &'a [Anchors],
    /// The file of the document's own page when another page shows its
    /// blocks, as a facade's page does: a link inside the document then
    /// goes to that file.
    pub(crate) foreign: Option<&'a str>,
    /// Whether names take their ids, which a facade's page puts on its
    /// headings instead.
    pub(crate) ids: bool,
    /// On a facade's page, the headings of what the page shows, with the
    /// index of the document's own target: a link to an item the page
    /// shows goes to its heading.
    pub(crate) api: Option<(&'a ApiIds, usize)>,
}

/// The heading id of each item of the run that a facade's page shows, by
/// the index of the target that defines it and the item.
pub(crate) type ApiIds = BTreeMap<(usize, crate::model::Reach), String>;

impl Page<'_> {
    pub(crate) fn refs(&self) -> &[TypeRef] {
        &self.doc.refs
    }

    /// Writes the top of the page: the stamp, the title, and the crate docs.
    /// The overview holds the legend, once for every page.
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
    }

    /// Writes one node of the page's sections.
    fn node(&self, out: &mut String, node: &Node) {
        match node {
            Node::Heading {
                level,
                id,
                crumb,
                text,
            } => {
                let _ = write!(out, "<h{level} id=\"{}\">", attr(id));
                if !crumb.is_empty() {
                    let _ = write!(out, "<span class=\"dim\">{}</span>", escape(crumb));
                }
                let _ = writeln!(out, "{}</h{level}>", escape(text));
            }
            Node::Note { class, text } => {
                let _ = writeln!(out, "<p class=\"{class}\">{}</p>", escape(text));
            }
            Node::Line { class, inlines } => {
                let _ = write!(out, "<p class=\"{class}\">");
                self.inlines(out, inlines);
                out.push_str("</p>\n");
            }
            Node::Code(block) => self.pre(out, block),
            Node::Glance(block) => {
                out.push_str("<p class=\"view\">At a glance</p>\n<pre class=\"glance\"><code>");
                self.code_lines(out, &block.lines);
                out.push_str("</code></pre>\n");
            }
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
                    if let Some(via) = &source.via {
                        let _ = write!(out, " written by <code>{}</code>", escape(via));
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
            Node::List { class, items } => self.list(out, items, class),
        }
    }

    /// Writes the crate's own docs as prose, which introduces the page.
    pub(crate) fn crate_docs(&self, out: &mut String, block: &Block) {
        let doc = leading_doc(&block.lines);
        if doc == 0 {
            return;
        }
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
                Inline::Anchor { text, id, strong } => {
                    let (open, close) = if *strong { ("<b>", "</b>") } else { ("", "") };
                    let text = escape(text);
                    let _ = write!(out, "<a href=\"#{}\">{open}{text}{close}</a>", attr(id));
                }
                Inline::Cross { text, to } => self.cross(out, text, *to),
                Inline::Kill {
                    text,
                    heading_id,
                    mark,
                } => {
                    let target = mark.as_ref().unwrap_or(heading_id);
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
    /// the layout puts between members becomes a gap, an empty block as tall
    /// as the blank line. A gap before a comment box is empty, since the
    /// box's own margin already spaces it. A newline is written before a box
    /// and never after it, because a box is a block and ends its own line.
    pub(crate) fn code_lines(&self, out: &mut String, lines: &[Line]) {
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

fn source_link(link: &Link) -> String {
    format!(
        "<a href=\"{}\">{}</a>",
        attr(&link.target),
        escape(&link.text)
    )
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

/// Returns spans without the markers that open comment lines, which prose
/// leaves out.
fn unmarked(spans: &[Span]) -> Vec<Span> {
    spans
        .iter()
        .filter(|span| !matches!(span, Span::Marker(_)))
        .cloned()
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_style::STYLE;

    #[test]
    fn escapes_markup() {
        assert_eq!(escape("Vec<&'a T>"), "Vec&lt;&amp;'a T&gt;");
        assert_eq!(attr("a\"b"), "a&quot;b");
    }

    #[test]
    fn style_padding_matches_box_padding() {
        assert!(STYLE.contains(&format!("padding:.2rem {BOX_PADDING} 0")));
    }
}
