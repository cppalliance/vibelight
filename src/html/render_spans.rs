//! The page's inline renderer: a line's spans to HTML, with a code span for
//! each matched pair of backticks, a link for each type name, and a link to
//! the source line for each member name.

use std::fmt::Write as _;

use crate::html::layout::{Line, Span};
use crate::model::{CrossRef, Loc, Reach};

use crate::html::render_html::{Page, attr, escape};

impl Page<'_> {
    /// Writes one line of a block: its indent, its spans, and its trailing
    /// comment.
    pub(crate) fn line(&self, out: &mut String, line: &Line) {
        if !line.spans.is_empty() {
            out.push_str(&" ".repeat(line.indent));
        }
        // The marker's own text carried the space between it and the
        // comment, and the marker is not printed, so that one space is added
        // back here.
        if let Some(Span::Marker(marker)) = line.spans.first() {
            if marker.ends_with(' ') {
                out.push(' ');
            }
        }
        self.ticked(out, &line.spans);
        if let Some(trailing) = &line.trailing {
            let _ = write!(out, " <span class=\"c\">{}</span>", escape(trailing));
        }
    }

    /// Writes a line's spans, turning each matched pair of backticks in a
    /// comment into a code span. A run of ticks opens one only when none is
    /// open, and closes it only when its length equals the open run's, as a
    /// code span in a doc comment does. A line with no comment spans takes
    /// the plain path.
    pub(crate) fn ticked(&self, out: &mut String, spans: &[Span]) {
        if !spans
            .iter()
            .any(|span| matches!(span, Span::Comment(_) | Span::DocRef(_)))
        {
            self.spans(out, spans);
            return;
        }
        let mut open: Option<usize> = None;
        for span in spans {
            match span {
                Span::Marker(marker) => Self::piece(out, false, &escape(marker_word(marker))),
                Span::Comment(text) => {
                    let mut rest = text.as_str();
                    while let Some(at) = rest.find('`') {
                        let run = rest[at..].chars().take_while(|&c| c == '`').count();
                        Self::piece(out, open.is_some(), &escape(&rest[..at]));
                        match open {
                            None => open = Some(run),
                            Some(length) if length == run => open = None,
                            Some(_) => Self::piece(out, true, &"`".repeat(run)),
                        }
                        rest = &rest[at + run..];
                    }
                    Self::piece(out, open.is_some(), &escape(rest));
                }
                other => {
                    let mut html = String::new();
                    self.spans(&mut html, std::slice::from_ref(other));
                    Self::piece(out, open.is_some(), &html);
                }
            }
        }
    }

    /// Writes one piece of a comment line, inside a code span when `code` is
    /// set and inside the comment color otherwise.
    fn piece(out: &mut String, code: bool, html: &str) {
        if html.is_empty() {
            return;
        }
        if code {
            let _ = write!(out, "<code>{html}</code>");
        } else {
            let _ = write!(out, "<span class=\"c\">{html}</span>");
        }
    }

    /// Writes a run of spans a span at a time, each comment run wrapped in
    /// the comment color.
    pub(crate) fn spans(&self, out: &mut String, spans: &[Span]) {
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
                Span::Marker(marker) => {
                    let word = marker_word(marker);
                    if !word.is_empty() {
                        let _ = write!(out, "<span class=\"c\">{}</span>", escape(word));
                    }
                }
                Span::Dim(text) => {
                    let _ = write!(out, "<span class=\"dim\">{}</span>", escape(text));
                }
                Span::Code(text) | Span::Comment(text) => out.push_str(&escape(text)),
                Span::Ref(id) | Span::DocRef(id) => self.reference(out, *id),
                Span::Label(label) => {
                    let _ = write!(out, "<b class=\"m\">{}</b>", escape(label));
                }
                Span::Name(text, loc) => self.name(out, text, *loc, None),
                Span::Value(text, loc, value) => {
                    let id =
                        (self.doc.value_ids.get(value).map(String::as_str)).filter(|_| self.ids);
                    self.name(out, text, *loc, id);
                }
                Span::Fn(text, loc, id) => self.name(out, text, *loc, id.as_deref()),
            }
        }
        if in_comment {
            out.push_str("</span>");
        }
    }

    /// Writes a member or item name: a link to its source line when links
    /// are on, otherwise a span, with its id when it has one.
    fn name(&self, out: &mut String, text: &str, loc: Loc, id: Option<&str>) {
        let id = id
            .filter(|id| !id.is_empty())
            .map_or_else(String::new, |id| format!(" id=\"{}\"", attr(id)));
        match self.member_links {
            Some(link) => {
                let _ = write!(
                    out,
                    "<a class=\"src\" href=\"{}\"{id}>{}</a>",
                    attr(&link(loc)),
                    escape(text)
                );
            }
            None => {
                let _ = write!(out, "<span class=\"src\"{id}>{}</span>", escape(text));
            }
        }
    }

    /// Writes a type name: a link into this page, a link into another
    /// crate's page, or plain text when it names nothing of the run. An
    /// item whose id is empty is one its page leaves out, as a facade's
    /// page leaves out what is not public.
    fn reference(&self, out: &mut String, id: usize) {
        let reference = &self.doc.refs[id];
        let reach = match (reference.decl(), reference.value()) {
            (Some(target), _) => Some(Reach::Decl(target)),
            (None, Some(value)) => Some(Reach::Value(value)),
            (None, None) => None,
        };
        if let (Some(reach), Some((_, own))) = (reach, self.api) {
            if self.api_link(out, &reference.text, own, reach) {
                return;
            }
        }
        let local = match (reference.decl(), reference.value()) {
            (Some(target), _) => self.doc.decl_ids.get(target),
            (None, Some(value)) => self.doc.value_ids.get(&value),
            (None, None) => None,
        };
        if local.is_some_and(String::is_empty) {
            out.push_str(&escape(&reference.text));
            return;
        }
        match (local, reference.foreign) {
            (Some(href), _) => match self.foreign {
                Some(file) => {
                    let _ = write!(
                        out,
                        "<a class=\"cross\" href=\"{}#{}\">{}</a>",
                        attr(file),
                        attr(href),
                        escape(&reference.text)
                    );
                }
                None => {
                    let _ = write!(
                        out,
                        "<a href=\"#{}\">{}</a>",
                        attr(href),
                        escape(&reference.text)
                    );
                }
            },
            (None, Some(to)) => self.cross(out, &reference.text, to),
            (None, None) => out.push_str(&escape(&reference.text)),
        }
    }

    /// Writes `text` as a link to the heading of an item a facade's page
    /// shows, when this is that page and it shows the item, and says
    /// whether it did.
    fn api_link(&self, out: &mut String, text: &str, krate: usize, reach: Reach) -> bool {
        let Some(id) = self.api.and_then(|(api, _)| api.get(&(krate, reach))) else {
            return false;
        };
        let _ = write!(out, "<a href=\"#{}\">{}</a>", attr(id), escape(text));
        true
    }

    /// Writes `text` as a link into the page of the other crate of the run
    /// that holds `to`, or as plain text when the run holds no page or
    /// anchor for it.
    pub(crate) fn cross(&self, out: &mut String, text: &str, to: CrossRef) {
        if self.api_link(out, text, to.krate, to.reach) {
            return;
        }
        let target = self.run.get(to.krate).and_then(|page| {
            let anchor = match to.reach {
                Reach::Decl(decl) => page.ids.get(decl)?,
                Reach::Value(value) => page.values.get(&value)?,
            };
            (!anchor.is_empty()).then_some((&page.file, anchor))
        });
        match target {
            Some((file, anchor)) => {
                let _ = write!(
                    out,
                    "<a class=\"cross\" href=\"{}#{}\">{}</a>",
                    attr(file),
                    attr(anchor),
                    escape(text)
                );
            }
            None => out.push_str(&escape(text)),
        }
    }
}

/// Returns the word a comment marker carries, which the page prints:
/// `cfg: ` for `// cfg: `, and nothing for the bare `//`, `///`, and `//!`,
/// whose slashes the page leaves out.
fn marker_word(marker: &str) -> &str {
    marker.trim_start_matches(['/', '!']).trim_start()
}
