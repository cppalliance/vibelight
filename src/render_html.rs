//! Prints the document as one self-contained HTML file with every type
//! linked.

use std::fmt::Write as _;

use crate::document::{Document, Inline, Item, Link, Node};
use crate::layout::{Block, Line, Span};
use crate::model::TypeRef;

/// The page's whole style sheet: light and dark colors follow the system.
const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1f2328;--muted:#59636e;--side:#f6f8fa;--rule:#d1d9e0;--key:#8250df;--com:#59636e;--link:#0969da;--mark:#953800;--mark-bg:#fff1e5;--target:#fff8c5}
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--muted:#9198a1;--side:#151b23;--rule:#3d444d;--key:#d2a8ff;--com:#9198a1;--link:#4493f8;--mark:#ffa657;--mark-bg:#3d2611;--target:#3a3000}}
body{margin:0;display:flex;background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif}
nav{position:sticky;top:0;align-self:flex-start;box-sizing:border-box;width:18rem;height:100vh;flex:none;overflow-y:auto;padding:1rem;background:var(--side);border-right:1px solid var(--rule);font-size:13px}
nav ul{list-style:none;margin:0;padding-left:1rem}
nav>ul{padding-left:0}
main{flex:1;min-width:0;padding:1rem 2rem}
a{color:var(--link);text-decoration:none}
a:hover{text-decoration:underline}
pre{background:var(--side);border-radius:6px;padding:.75rem 1rem;overflow-x:auto}
pre,code{font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.k{color:var(--key)}
.c{color:var(--com)}
.c a{color:inherit;text-decoration:underline}
b.m{color:var(--mark);background:var(--mark-bg);padding:0 .2em;border-radius:3px}
.source{color:var(--muted)}
a.src{color:inherit}
:target{background:var(--target)}
";

/// Renders the HTML file. Member names link to their source lines when
/// `member_links` is true.
pub(crate) fn render(
    doc: &Document,
    member_links: Option<&dyn Fn(crate::model::Loc) -> String>,
) -> String {
    let page = Page { doc, member_links };
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(out, "<title>{}</title>", escape(&doc.title));
    let _ = write!(out, "<style>\n{STYLE}</style>\n</head>\n<body>\n<nav>\n");
    page.list(&mut out, &doc.contents, "");
    out.push_str("</nav>\n<main>\n");
    let _ = writeln!(
        out,
        "<h1 id=\"{}\">{}</h1>",
        attr(&doc.id),
        escape(&doc.title)
    );
    if let Some(docs) = &doc.crate_docs {
        page.pre(&mut out, docs);
    }
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
                for (i, link) in links.iter().enumerate() {
                    if i == 1 {
                        out.push_str(", impl ");
                    } else if i > 1 {
                        out.push_str(", ");
                    }
                    out.push_str(&source_link(link));
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
            Node::List { class, items } => page.list(&mut out, items, class),
        }
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

struct Page<'a> {
    doc: &'a Document,
    member_links: Option<&'a dyn Fn(crate::model::Loc) -> String>,
}

impl Page<'_> {
    fn refs(&self) -> &[TypeRef] {
        &self.doc.refs
    }

    fn pre(&self, out: &mut String, block: &Block) {
        out.push_str("<pre><code>");
        for (i, line) in block.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            self.line(out, line);
        }
        out.push_str("</code></pre>\n");
    }

    fn line(&self, out: &mut String, line: &Line) {
        if !line.spans.is_empty() {
            out.push_str(&" ".repeat(line.indent));
        }
        self.spans(out, &line.spans);
        if let Some(trailing) = &line.trailing {
            let _ = write!(out, " <span class=\"c\">{}</span>", escape(trailing));
        }
    }

    fn spans(&self, out: &mut String, spans: &[Span]) {
        let mut in_comment = false;
        for span in spans {
            let comment = matches!(span, Span::Comment(_) | Span::DocRef(_));
            if comment && !in_comment {
                out.push_str("<span class=\"c\">");
            } else if !comment && in_comment {
                out.push_str("</span>");
            }
            in_comment = comment;
            match span {
                Span::Key(text) => {
                    let _ = write!(out, "<span class=\"k\">{}</span>", escape(text));
                }
                Span::Code(text) | Span::Comment(text) => out.push_str(&escape(text)),
                Span::Ref(id) | Span::DocRef(id) => self.reference(out, *id),
                Span::Label(label) => {
                    let _ = write!(out, "<b class=\"m\">{}</b>", escape(label));
                }
                Span::Name(text, loc) => match self.member_links {
                    Some(link) => {
                        let _ = write!(
                            out,
                            "<a class=\"src\" href=\"{}\">{}</a>",
                            attr(&link(*loc)),
                            escape(text)
                        );
                    }
                    None => out.push_str(&escape(text)),
                },
            }
        }
        if in_comment {
            out.push_str("</span>");
        }
    }

    fn reference(&self, out: &mut String, id: usize) {
        let reference = &self.refs()[id];
        match reference.target {
            Some(target) => {
                let href = &self.doc.decl_ids[target];
                let _ = write!(
                    out,
                    "<a href=\"#{}\">{}</a>",
                    attr(href),
                    escape(&reference.text)
                );
            }
            None => out.push_str(&escape(&reference.text)),
        }
    }

    fn list(&self, out: &mut String, items: &[Item], class: &str) {
        if class.is_empty() {
            out.push_str("<ul>\n");
        } else {
            let _ = writeln!(out, "<ul class=\"{class}\">");
        }
        for item in items {
            out.push_str("<li>");
            for inline in &item.inlines {
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
fn attr(text: &str) -> String {
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
