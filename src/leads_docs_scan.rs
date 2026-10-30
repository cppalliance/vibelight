//! Reads the prose of a doc text: its paragraphs, list items, headings,
//! code blocks, and reference definitions line by line, then each
//! paragraph's words, code spans, and links character by character. A
//! paragraph or a code block inside an ignored passage is left unread.

use std::ops::Range;

use crate::census_docs::fence;
use crate::leads_docs_text::{Kind, Link, Prose, Tok};

/// The words after which a dot ends no sentence.
const ABBREVIATIONS: [&str; 6] = ["cf", "e.g", "etc", "i.e", "vs", "viz"];

/// Reads a doc text's prose.
#[cfg(test)]
pub(crate) fn read(doc: &str) -> Prose<'_> {
    read_except(doc, Vec::new())
}

/// Reads a doc text's prose outside the `ignored` passages, given as
/// sorted byte ranges.
pub(crate) fn read_except(doc: &str, ignored: Vec<Range<usize>>) -> Prose<'_> {
    let inside = |range: &Range<usize>| {
        (ignored.iter()).any(|skip| skip.start <= range.start && range.end <= skip.end)
    };
    let mut reader = Reader {
        prose: Prose {
            doc,
            toks: Vec::new(),
            headings: Vec::new(),
            links: Vec::new(),
            blocks: Vec::new(),
            ignored: Vec::new(),
        },
        sentence: 0,
        started: false,
        section: None,
        names: Vec::new(),
    };
    let mut definitions: Vec<(String, String)> = Vec::new();
    let mut open: Option<(char, usize, usize)> = None;
    let mut paragraph: Option<usize> = None;
    let mut start = 0;
    for line in doc.split('\n') {
        let next = start + line.len() + 1;
        if let Some((mark, count, from)) = open {
            let closes =
                fence(line).is_some_and(|(m, n, info)| m == mark && n >= count && info.is_empty());
            if closes {
                let block = from..start.saturating_sub(1).max(from);
                if !inside(&block) {
                    reader.prose.blocks.push(block);
                }
                open = None;
            }
            start = next;
            continue;
        }
        let opened = fence(line);
        let heading = heading(line);
        let definition = definition(line).filter(|_| paragraph.is_none());
        if opened.is_none() && heading.is_none() && definition.is_none() && !line.trim().is_empty()
        {
            paragraph.get_or_insert(start);
            start = next;
            continue;
        }
        if let Some(from) = paragraph.take() {
            let range = from..start.saturating_sub(1);
            if !inside(&range) {
                reader.paragraph(range);
            }
        }
        if let Some((mark, count, _)) = opened {
            open = Some((mark, count, next.min(doc.len())));
        } else if let Some(title) = heading {
            reader.stop();
            reader.prose.headings.push(title.to_lowercase());
            reader.section = Some(reader.prose.headings.len() - 1);
        } else if let Some((name, dest)) = definition {
            definitions.push((name.to_lowercase(), dest.to_owned()));
        }
        start = next;
    }
    if let Some(range) = paragraph.map(|from| from..doc.len()) {
        if !inside(&range) {
            reader.paragraph(range);
        }
    }
    if let Some(block) = open.map(|(_, _, from)| from..doc.len()) {
        if !inside(&block) {
            reader.prose.blocks.push(block);
        }
    }
    let mut prose = reader.prose;
    prose.ignored = ignored;
    for (link, name) in prose.links.iter_mut().zip(reader.names) {
        if let Some(name) = name {
            let defined = definitions.iter().find(|(n, _)| *n == name.to_lowercase());
            link.dest = defined.map_or(name, |(_, dest)| dest.clone());
        }
    }
    prose
}

/// Returns a heading line's text.
fn heading(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    let rest = &trimmed[hashes..];
    let shaped = (1..=6).contains(&hashes) && (rest.is_empty() || rest.starts_with([' ', '\t']));
    shaped.then(|| rest.trim().trim_end_matches('#').trim())
}

/// Returns a reference definition's name and destination, as in
/// `[name]: dest` or `[name]: dest "title"`; a line where words follow the
/// destination is prose. A definition never interrupts a paragraph.
fn definition(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    if line.len() - trimmed.len() > 3 || !trimmed.starts_with('[') {
        return None;
    }
    let close = trimmed.find("]:")?;
    let name = &trimmed[1..close];
    let rest = trimmed[close + 2..].trim_start();
    let dest = rest.split_whitespace().next()?;
    let title = rest[dest.len()..].trim_start();
    let titled = title.is_empty() || title.starts_with(['"', '\'', '(']);
    let dest = dest.trim_start_matches('<').trim_end_matches('>');
    (titled && !name.is_empty() && !name.contains(['[', ']'])).then_some((name, dest))
}

/// Returns how many bytes a list item's or a quote's marker takes at the
/// start of a line, and whether a list item starts there.
fn marker(text: &str) -> (usize, bool) {
    let line = text.split('\n').next().unwrap_or_default();
    let indent = line.len() - line.trim_start().len();
    let rest = &line[indent..];
    if rest.starts_with('>') {
        return (indent + 1, false);
    }
    let spaced = |at: usize| rest[at..].starts_with([' ', '\t']);
    if rest.starts_with(['-', '*', '+']) && spaced(1) {
        return (indent + 2, true);
    }
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if (1..=3).contains(&digits) && rest[digits..].starts_with(['.', ')']) && spaced(digits + 1) {
        return (indent + digits + 2, true);
    }
    (indent, false)
}

/// Counts the bytes `b` at and after `at`, before `end`.
fn run_of(doc: &str, at: usize, end: usize, b: u8) -> usize {
    doc.as_bytes()[at..end]
        .iter()
        .take_while(|&&c| c == b)
        .count()
}

/// Finds the next run of exactly `n` backticks at or after `from`.
fn close_run(doc: &str, from: usize, end: usize, n: usize) -> Option<usize> {
    let mut i = from;
    while i < end {
        if doc.as_bytes()[i] == b'`' {
            let len = run_of(doc, i, end, b'`');
            if len == n {
                return Some(i);
            }
            i += len;
        } else {
            i += 1;
        }
    }
    None
}

/// Finds the byte that closes the bracket at `open`, skipping code spans
/// and escaped characters.
fn matching(doc: &str, open: usize, end: usize, left: u8, right: u8) -> Option<usize> {
    let bytes = doc.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < end {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'`' => {
                let n = run_of(doc, i, end, b'`');
                i = close_run(doc, i + n, end, n).map_or(i + n, |close| close + n);
                continue;
            }
            b if b == left => depth += 1,
            b if b == right => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Reads prose token by token.
struct Reader<'a> {
    prose: Prose<'a>,
    sentence: usize,
    /// Whether the current sentence has a token.
    started: bool,
    section: Option<usize>,
    /// For each link, the reference it names when it has no inline
    /// destination.
    names: Vec<Option<String>>,
}

impl Reader<'_> {
    fn push(&mut self, kind: Kind, text: String, span: Range<usize>) {
        self.prose.toks.push(Tok {
            kind,
            text,
            span,
            sentence: self.sentence,
            section: self.section,
        });
        self.started = true;
    }

    /// Ends the current sentence.
    fn stop(&mut self) {
        if self.started {
            self.sentence += 1;
            self.started = false;
        }
    }

    fn paragraph(&mut self, range: Range<usize>) {
        self.stop();
        self.scan(range, true);
        self.stop();
    }

    fn scan(&mut self, range: Range<usize>, mut line_start: bool) {
        let doc = self.prose.doc;
        let mut i = range.start;
        while i < range.end {
            if line_start {
                line_start = false;
                let (skip, item) = marker(&doc[i..range.end]);
                if item {
                    self.stop();
                }
                i += skip;
                continue;
            }
            let Some(c) = doc[i..range.end].chars().next() else {
                break;
            };
            let len = c.len_utf8();
            match c {
                '\n' => {
                    line_start = true;
                    i += 1;
                }
                c if c.is_whitespace() => i += len,
                '`' => i = self.code(i, range.end),
                '[' => i = self.link(i, range.end),
                c if c.is_alphanumeric() || c == '_' => i = self.word(i, range.end),
                '\\' => {
                    let escaped = doc[i + 1..range.end].chars().next();
                    let skip = escaped
                        .filter(char::is_ascii_punctuation)
                        .map_or(0, char::len_utf8);
                    let text = doc[i + 1..i + 1 + skip].to_owned();
                    self.push(Kind::Punct, text, i..i + 1 + skip);
                    i += 1 + skip;
                }
                _ => {
                    self.push(Kind::Punct, c.to_string(), i..i + len);
                    i += len;
                    let spaced = doc[i..range.end]
                        .chars()
                        .next()
                        .is_none_or(char::is_whitespace);
                    if matches!(c, '.' | '!' | '?') && spaced && !self.abbreviated() {
                        self.stop();
                    }
                }
            }
        }
    }

    /// Tells whether the dot just read ends an abbreviation.
    fn abbreviated(&self) -> bool {
        let toks = &self.prose.toks;
        let before = toks.len().checked_sub(2).and_then(|i| toks.get(i));
        before.is_some_and(|t| t.kind == Kind::Word && ABBREVIATIONS.contains(&t.text.as_str()))
    }

    fn code(&mut self, i: usize, end: usize) -> usize {
        let doc = self.prose.doc;
        let n = run_of(doc, i, end, b'`');
        let Some(close) = close_run(doc, i + n, end, n) else {
            self.push(Kind::Punct, "`".repeat(n), i..i + n);
            return i + n;
        };
        let content = doc[i + n..close].replace('\n', " ");
        let padded = content.len() >= 2
            && content.starts_with(' ')
            && content.ends_with(' ')
            && !content.trim().is_empty();
        let content = if padded {
            content[1..content.len() - 1].to_owned()
        } else {
            content
        };
        self.push(Kind::Code, content, i..close + n);
        close + n
    }

    fn word(&mut self, i: usize, end: usize) -> usize {
        let doc = self.prose.doc;
        let mut stop = i;
        let mut chars = doc[i..end].char_indices().peekable();
        while let Some((at, c)) = chars.next() {
            let inner = matches!(c, '\'' | '\u{2019}' | '-' | '.')
                && chars.peek().is_some_and(|&(_, n)| n.is_alphanumeric());
            if !(c.is_alphanumeric() || c == '_' || inner) {
                break;
            }
            stop = i + at + c.len_utf8();
        }
        self.push(Kind::Word, doc[i..stop].to_lowercase(), i..stop);
        stop
    }

    fn link(&mut self, i: usize, end: usize) -> usize {
        let doc = self.prose.doc;
        let close = matching(doc, i, end, b'[', b']');
        let label_text = close.map(|close| &doc[i + 1..close]);
        let Some((close, label_text)) = close
            .zip(label_text)
            .filter(|(_, text)| !text.starts_with('^') && !text.trim().is_empty())
        else {
            self.push(Kind::Punct, "[".to_owned(), i..i + 1);
            return i + 1;
        };
        let first = self.prose.toks.len();
        self.scan(i + 1..close, false);
        let label = first..self.prose.toks.len();
        let after = close + 1;
        let next = (after < end).then(|| doc.as_bytes()[after]);
        let inline = (next == Some(b'(')).then(|| matching(doc, after, end, b'(', b')'));
        let named = (next == Some(b'[')).then(|| matching(doc, after, end, b'[', b']'));
        let (stop, dest, name, shortcut) = match (inline.flatten(), named.flatten()) {
            (Some(paren), _) => {
                let inner = doc[after + 1..paren].split_whitespace().next();
                let dest = inner.unwrap_or_default();
                let dest = dest.trim_start_matches('<').trim_end_matches('>');
                (paren + 1, dest.to_owned(), None, false)
            }
            (None, Some(bracket)) => {
                let name = doc[after + 1..bracket].trim();
                let name = if name.is_empty() { label_text } else { name };
                (bracket + 1, String::new(), Some(name), false)
            }
            (None, None) => (after, String::new(), Some(label_text), true),
        };
        self.prose.links.push(Link {
            span: i..stop,
            label,
            dest,
            shortcut,
        });
        self.names.push(name.map(|n| n.trim().to_owned()));
        stop
    }
}
