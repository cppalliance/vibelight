//! Prints the document as one self-contained HTML file with every type
//! linked.

use std::fmt::Write as _;

use crate::document::{Document, Inline, Item, Link, Node};
use crate::layout::{Block, Line, Span};
use crate::model::TypeRef;

/// The page's whole style sheet: light and dark colors follow the system.
const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1f2328;--muted:#59636e;--side:#f6f8fa;--rule:#d1d9e0;--key:#8250df;--com:#59636e;--link:#0969da;--cross:#8a5c00;--mark:#953800;--mark-bg:#fff1e5;--target:#fff8c5}
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--muted:#9198a1;--side:#151b23;--rule:#3d444d;--key:#d2a8ff;--com:#9198a1;--link:#4493f8;--cross:#e3b341;--mark:#ffa657;--mark-bg:#3d2611;--target:#3a3000}}
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
a.cross{color:var(--cross)}
b.m{color:var(--mark);background:var(--mark-bg);padding:0 .2em;border-radius:3px}
.source{color:var(--muted)}
a.src{color:inherit}
span.src{cursor:default}
:target{background:var(--target)}
";

/// A page of the run that other pages link into: its file name and the
/// anchor id of each of its declarations.
#[derive(Debug, Clone)]
pub(crate) struct Anchors {
    pub(crate) file: String,
    pub(crate) ids: Vec<String>,
}

/// Renders the HTML file. Member names link to their source lines when
/// `member_links` is given, and a name that reaches another crate of the
/// run links into that crate's page in `run`.
pub(crate) fn render(
    doc: &Document,
    member_links: Option<&dyn Fn(crate::model::Loc) -> String>,
    run: &[Anchors],
) -> String {
    let page = Page {
        doc,
        member_links,
        run,
    };
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
    run: &'a [Anchors],
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
                    None => {
                        let _ = write!(out, "<span class=\"src\">{}</span>", escape(text));
                    }
                },
            }
        }
        if in_comment {
            out.push_str("</span>");
        }
    }

    fn reference(&self, out: &mut String, id: usize) {
        let reference = &self.refs()[id];
        let other = reference.foreign.and_then(|f| {
            let page = self.run.get(f.krate)?;
            Some((&page.file, page.ids.get(f.decl)?))
        });
        match (reference.target, other) {
            (Some(target), _) => {
                let href = &self.doc.decl_ids[target];
                let _ = write!(
                    out,
                    "<a href=\"#{}\">{}</a>",
                    attr(href),
                    escape(&reference.text)
                );
            }
            (None, Some((file, anchor))) => {
                let _ = write!(
                    out,
                    "<a class=\"cross\" href=\"{}#{}\">{}</a>",
                    attr(file),
                    attr(anchor),
                    escape(&reference.text)
                );
            }
            (None, None) => out.push_str(&escape(&reference.text)),
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

    #[test]
    fn the_style_sheet_colors_cross_crate_links_in_both_themes() {
        let dark = STYLE.find("@media (prefers-color-scheme:dark)").unwrap();
        let (light, dark) = STYLE.split_at(dark);
        assert!(
            light.contains("--cross:#8a5c00"),
            "a darker gold when light"
        );
        let dark_line = dark.lines().next().unwrap();
        assert!(
            dark_line.contains("--cross:#e3b341"),
            "a lighter yellow when dark"
        );
        assert!(STYLE.contains("\na.cross{color:var(--cross)}\n"));
        assert!(STYLE.contains("\nspan.src{cursor:default}\n"));
        assert!(
            STYLE.find("a.cross").unwrap() > STYLE.find(".c a").unwrap(),
            "a cross-crate link stays yellow inside a comment"
        );
        assert!(!STYLE.contains("url(") && !STYLE.contains("@import"));
    }

    #[test]
    fn a_foreign_name_links_into_its_page_and_member_names_are_spans_without_links() {
        use crate::model::{CrossRef, Loc};

        let refs = vec![TypeRef {
            text: "Gauge".into(),
            path: vec!["Gauge".into()],
            global: false,
            module: 0,
            generics: Vec::new(),
            comment: false,
            target: None,
            canonical: Some(vec!["b".into(), "Gauge".into()]),
            written: None,
            foreign: Some(CrossRef { krate: 1, decl: 0 }),
        }];
        let line = Line {
            indent: 4,
            spans: vec![
                Span::Name("gauge".into(), Loc { file: 0, line: 1 }),
                Span::Code(": ".into()),
                Span::Ref(0),
            ],
            trailing: None,
        };
        let doc = Document {
            title: "a".into(),
            id: "crate.a".into(),
            crate_docs: None,
            contents: Vec::new(),
            kill_summary: "none".into(),
            kill: Vec::new(),
            sections: vec![Node::Code(Block {
                lines: vec![line],
                repeats: Vec::new(),
            })],
            refs,
            decl_ids: Vec::new(),
        };
        let run = [
            Anchors {
                file: "a.html".into(),
                ids: Vec::new(),
            },
            Anchors {
                file: "b.html".into(),
                ids: vec!["type.b.parts.Gauge".into()],
            },
        ];
        let html = render(&doc, None, &run);
        assert!(
            html.contains(
                "<span class=\"src\">gauge</span>: \
                 <a class=\"cross\" href=\"b.html#type.b.parts.Gauge\">Gauge</a>"
            ),
            "{html}"
        );
    }
}
