//! Prints the document as one self-contained HTML file with every type
//! linked.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::document::{Document, Inline, Item, Link, Node};
use crate::layout::{Block, Span, span_text};
use crate::model::{Loc, TypeRef, ValueRef};
use crate::render_code::leading_doc;
use crate::render_style::STYLE;

/// The id of the checkbox that hides the member comments, which the
/// checkbox's `id` and the sidebar label's `for` both name.
const TOGGLE_ID: &str = "vibelight-comments";

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
    pub(crate) fn refs(&self) -> &[TypeRef] {
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
            &crate::render_prose::InlineRenderer {
                spans: &spans,
                text_of: &text_of,
                hideable: false,
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
                Inline::Anchor { text, id } => {
                    let _ = write!(out, "<a href=\"#{}\">{}</a>", attr(id), escape(text));
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(escape("Vec<&'a T>"), "Vec&lt;&amp;'a T&gt;");
        assert_eq!(attr("a\"b"), "a&quot;b");
    }
}
