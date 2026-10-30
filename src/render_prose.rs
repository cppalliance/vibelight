//! Renders a comment block as prose: paragraphs, bullets, subheadings, and
//! fenced code. The HTML page uses it for a type's or a module's doc
//! comment, where flowing text reads better than a wall of comment lines.
//!
//! The lines arrive as [`Span`]s, so a type name inside a comment still
//! links. Every span the renderer writes is passed to the caller's inline
//! renderer, which decides how it prints.

use crate::layout::{Line, Span};

/// How the prose renderer reaches the page's own inline renderer.
pub(crate) struct InlineRenderer<'a> {
    /// Writes one line's spans as inline HTML, with no comment coloring.
    pub(crate) spans: &'a dyn Fn(&mut String, &[Span]),
    /// Returns one span's printed text, references resolved.
    pub(crate) text_of: &'a dyn Fn(&Span) -> String,
    /// True for a declaration's doc comment, which the page's comment toggle
    /// hides; the crate's own docs pass false, since they introduce the page.
    pub(crate) hideable: bool,
}

/// Writes `lines` as prose.
pub(crate) fn block(out: &mut String, lines: &[Line], inline: &InlineRenderer<'_>) {
    let mut paragraph: Vec<Vec<Span>> = Vec::new();
    let mut bullets: Vec<Vec<Vec<Span>>> = Vec::new();
    let mut fence: Option<String> = None;
    let mut in_bullet = false;
    if inline.hideable {
        out.push_str("<div class=\"doc mc-doc\">\n");
    } else {
        out.push_str("<div class=\"doc\">\n");
    }

    for line in lines {
        let text = plain(line, inline);
        let trimmed = text.trim();

        if let Some(mark) = &fence {
            if closes(trimmed, mark) {
                out.push_str("</code></pre>\n");
                fence = None;
            } else {
                (inline.spans)(out, &line.spans);
                out.push('\n');
            }
            continue;
        }

        if let Some(opening) = fence_open(trimmed) {
            flush(out, &mut paragraph, &mut bullets, inline);
            in_bullet = false;
            out.push_str("<pre><code>");
            fence = Some(opening);
            continue;
        }

        if trimmed.is_empty() {
            flush(out, &mut paragraph, &mut bullets, inline);
            in_bullet = false;
            continue;
        }

        if let Some(words) = trimmed.strip_prefix("## ") {
            flush(out, &mut paragraph, &mut bullets, inline);
            in_bullet = false;
            subheading(out, words);
            continue;
        }

        if let Some(words) = trimmed.strip_prefix("# ") {
            flush(out, &mut paragraph, &mut bullets, inline);
            in_bullet = false;
            subheading(out, words);
            continue;
        }

        let lead = text.chars().count() - text.trim_start().chars().count();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            flush_paragraph(out, &mut paragraph, inline);
            bullets.push(vec![without(line, lead + 2, inline)]);
            in_bullet = true;
            continue;
        }

        let body = without(line, lead, inline);
        if in_bullet {
            if let Some(last) = bullets.last_mut() {
                last.push(body);
                continue;
            }
        }
        paragraph.push(body);
    }

    flush(out, &mut paragraph, &mut bullets, inline);
    if fence.is_some() {
        out.push_str("</code></pre>\n");
    }
    out.push_str("</div>\n");
}

/// Writes a markdown heading as the page's smallest heading.
fn subheading(out: &mut String, words: &str) {
    out.push_str("<h4>");
    out.push_str(&crate::render_html::escape(words.trim()));
    out.push_str("</h4>\n");
}

/// Returns a line's printed text with its markers dropped, which is what
/// decides how the line classifies.
fn plain(line: &Line, inline: &InlineRenderer<'_>) -> String {
    line.spans
        .iter()
        .filter(|span| !matches!(span, Span::Marker(_)))
        .map(|span| (inline.text_of)(span))
        .collect()
}

/// Returns the line's spans with the first `skip` characters of text
/// dropped, which removes a bullet's mark or the line's own indentation.
fn without(line: &Line, skip: usize, inline: &InlineRenderer<'_>) -> Vec<Span> {
    let mut left = skip;
    let mut out: Vec<Span> = Vec::new();
    for span in &line.spans {
        match span {
            Span::Marker(_) => {}
            Span::DocRef(id) => {
                if left == 0 {
                    out.push(Span::DocRef(*id));
                }
            }
            Span::Ref(id) => {
                if left == 0 {
                    out.push(Span::Ref(*id));
                }
            }
            other => {
                let text = (inline.text_of)(other);
                let count = text.chars().count();
                if left >= count {
                    left -= count;
                    continue;
                }
                let kept: String = text.chars().skip(left).collect();
                left = 0;
                if !kept.is_empty() {
                    out.push(Span::Comment(kept));
                }
            }
        }
    }
    out
}

/// Tells whether a line closes the fence `mark`.
fn closes(trimmed: &str, mark: &str) -> bool {
    let Some(first) = mark.chars().next() else {
        return false;
    };
    !trimmed.is_empty() && trimmed.chars().all(|c| c == first) && trimmed.len() >= mark.len()
}

/// Returns the fence a line opens, when it opens one: three or more
/// backticks or tildes and nothing after them.
fn fence_open(trimmed: &str) -> Option<String> {
    let mark = trimmed.chars().next().filter(|&c| c == '`' || c == '~')?;
    let count = trimmed.chars().take_while(|&c| c == mark).count();
    (count >= 3 && trimmed[count * mark.len_utf8()..].trim().is_empty())
        .then(|| mark.to_string().repeat(count))
}

/// Ends a paragraph and a bullet list.
fn flush(
    out: &mut String,
    paragraph: &mut Vec<Vec<Span>>,
    bullets: &mut Vec<Vec<Vec<Span>>>,
    inline: &InlineRenderer<'_>,
) {
    flush_paragraph(out, paragraph, inline);
    flush_bullets(out, bullets, inline);
}

/// Writes the paragraph being built, if any.
fn flush_paragraph(out: &mut String, paragraph: &mut Vec<Vec<Span>>, inline: &InlineRenderer<'_>) {
    if paragraph.is_empty() {
        return;
    }
    out.push_str("<p>");
    for (index, line) in std::mem::take(paragraph).iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        (inline.spans)(out, line);
    }
    out.push_str("</p>\n");
}

/// Writes the bullet list being built, if any.
fn flush_bullets(out: &mut String, bullets: &mut Vec<Vec<Vec<Span>>>, inline: &InlineRenderer<'_>) {
    if bullets.is_empty() {
        return;
    }
    out.push_str("<ul>\n");
    for bullet in std::mem::take(bullets) {
        out.push_str("<li>");
        for (index, line) in bullet.iter().enumerate() {
            if index > 0 {
                out.push(' ');
            }
            (inline.spans)(out, line);
        }
        out.push_str("</li>\n");
    }
    out.push_str("</ul>\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_heading_escapes_markup() {
        let mut out = String::new();
        subheading(&mut out, " Vec<&T> ");
        assert_eq!(out, "<h4>Vec&lt;&amp;T&gt;</h4>\n");
    }
}
