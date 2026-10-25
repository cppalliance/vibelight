//! Holds each source file's text and turns spans into normalized slices.

use std::ops::Range;

use proc_macro2::{LineColumn, Span};

/// One `.rs` file: its crate-relative path and its text with LF line endings.
#[derive(Debug)]
pub(crate) struct SourceFile {
    /// The path relative to the crate directory, with forward slashes.
    pub(crate) rel: String,
    /// The text, with CRLF turned into LF and any byte-order mark removed.
    pub(crate) text: String,
    /// The byte offset where each line starts.
    line_starts: Vec<usize>,
}

impl SourceFile {
    /// Builds a source file from raw file text.
    pub(crate) fn new(rel: String, raw: &str) -> SourceFile {
        let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
        let text = raw.replace("\r\n", "\n");
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        SourceFile {
            rel,
            text,
            line_starts,
        }
    }

    /// Converts a line (1-based) and character column (0-based) to a byte
    /// offset, clamped to the text.
    pub(crate) fn offset(&self, at: LineColumn) -> usize {
        let Some(&start) = self.line_starts.get(at.line.saturating_sub(1)) else {
            return self.text.len();
        };
        self.text[start..]
            .char_indices()
            .nth(at.column)
            .map_or(self.text.len(), |(i, _)| start + i)
    }

    /// Returns the byte offset where a span starts.
    pub(crate) fn start(&self, span: Span) -> usize {
        self.offset(span.start())
    }

    /// Returns the byte offset where a span ends.
    pub(crate) fn end(&self, span: Span) -> usize {
        self.offset(span.end())
    }

    /// Returns the 1-based line holding a byte offset.
    pub(crate) fn line_of(&self, offset: usize) -> u32 {
        let index = self.line_starts.partition_point(|&s| s <= offset);
        u32::try_from(index.max(1)).unwrap_or(u32::MAX)
    }

    /// Returns the text of a byte range with its comments removed and its
    /// whitespace normalized.
    pub(crate) fn clean(&self, range: Range<usize>) -> String {
        let range = range.start.min(self.text.len())..range.end.min(self.text.len());
        if range.start >= range.end {
            return String::new();
        }
        let mut text = String::with_capacity(range.len());
        let mut at = range.start;
        for comment in scan(&self.text, range.clone()) {
            text.push_str(&self.text[at..comment.range.start]);
            text.push(' ');
            at = comment.range.end;
        }
        text.push_str(&self.text[at..range.end]);
        normalize(&text)
    }
}

/// Collapses whitespace runs to one space, removes whitespace just inside
/// `(`, `[`, and `<` and just before `)`, `]`, `>`, and `,`, and drops a
/// trailing comma before a closing bracket. The comma of a one-element tuple
/// such as `(T,)` stays, because it is part of the type.
pub(crate) fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            space = !out.is_empty();
            continue;
        }
        if matches!(c, ')' | ']' | '>' | '}')
            && out.ends_with(',')
            && !(c == ')' && is_one_tuple(&out))
        {
            out.pop();
        }
        if space && !out.ends_with(['(', '[', '<']) && !matches!(c, ')' | ']' | '>' | ',') {
            out.push(' ');
        }
        space = false;
        out.push(c);
    }
    out
}

/// Tells whether `out`, which ends in a comma, is inside a parenthesized
/// list whose only comma is that last one.
fn is_one_tuple(out: &str) -> bool {
    let body: Vec<char> = out[..out.len() - 1].chars().collect();
    let mut depth = 0usize;
    for (i, &c) in body.iter().enumerate().rev() {
        let arrow = c == '>' && i > 0 && body[i - 1] == '-';
        match c {
            ')' | ']' | '}' => depth += 1,
            '>' if !arrow => depth += 1,
            '(' if depth == 0 => return true,
            '(' | '[' | '{' | '<' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return false,
            _ => {}
        }
    }
    false
}

/// One comment found in source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Comment {
    /// The comment's bytes, from its first slash to its end (a line comment
    /// stops before its newline).
    pub(crate) range: Range<usize>,
    /// True for doc comments (`///`, `//!`, `/** */`, `/*! */`), which syn
    /// already reports as attributes.
    pub(crate) doc: bool,
}

/// Finds every comment in `range`, skipping string, character, and raw
/// string literals. `range` must start outside any literal or comment.
pub(crate) fn scan(text: &str, range: Range<usize>) -> Vec<Comment> {
    let bytes = text.as_bytes();
    let end = range.end.min(bytes.len());
    let mut found = Vec::new();
    let mut i = range.start;
    while i < end {
        let rest = &text[i..];
        let c = bytes[i];
        if rest.starts_with("//") {
            let stop = rest.find('\n').map_or(text.len(), |n| i + n).min(end);
            let doc =
                (rest.starts_with("///") && !rest.starts_with("////")) || rest.starts_with("//!");
            found.push(Comment {
                range: i..stop,
                doc,
            });
            i = stop;
        } else if rest.starts_with("/*") {
            let stop = block_comment_end(text, i).min(end);
            let doc =
                (rest.starts_with("/**") && !rest.starts_with("/***") && !rest.starts_with("/**/"))
                    || rest.starts_with("/*!");
            found.push(Comment {
                range: i..stop,
                doc,
            });
            i = stop;
        } else if c == b'"' {
            i = string_end(bytes, i + 1);
        } else if c == b'\'' {
            i = quote_end(text, i);
        } else if c.is_ascii_alphabetic() || c == b'_' {
            i = word_end(text, i);
        } else {
            i += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    found
}

/// Returns the offset just past a block comment that starts at `start`,
/// honoring nesting.
fn block_comment_end(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut i = start;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    bytes.len()
}

/// Returns the offset just past the closing quote of a string whose body
/// starts at `i`.
fn string_end(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Skips a character literal or a lifetime starting at the quote at `i`.
fn quote_end(text: &str, i: usize) -> usize {
    let bytes = text.as_bytes();
    let Some(next) = text[i + 1..].chars().next() else {
        return text.len();
    };
    if next == '\\' {
        let mut j = i + 3;
        while j < bytes.len() && bytes[j] != b'\'' && bytes[j] != b'\n' {
            j += 1;
        }
        return (j + 1).min(text.len());
    }
    let after = i + 1 + next.len_utf8();
    if bytes.get(after) == Some(&b'\'') {
        return after + 1;
    }
    i + 1
}

/// Skips an identifier, a keyword, or a literal prefix such as `r#"`, `b"`,
/// or `br#"`, along with the literal it starts.
fn word_end(text: &str, i: usize) -> usize {
    let bytes = text.as_bytes();
    let mut j = i;
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
        j += 1;
    }
    let word = &text[i..j];
    let next = bytes.get(j).copied();
    match (word, next) {
        ("r" | "br" | "cr", Some(b'"' | b'#')) => raw_string_end(bytes, j).unwrap_or(j),
        ("b" | "c", Some(b'"')) => string_end(bytes, j + 1),
        ("b", Some(b'\'')) => quote_end(text, j),
        _ => j,
    }
}

/// Skips a raw string whose hashes or quote start at `i`, or returns `None`
/// when the text is a raw identifier such as `r#type`.
fn raw_string_end(bytes: &[u8], i: usize) -> Option<usize> {
    let mut hashes = 0;
    let mut j = i;
    while bytes.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if bytes.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    while j < bytes.len() {
        if bytes[j] == b'"'
            && bytes[j + 1..]
                .iter()
                .take(hashes)
                .filter(|&&b| b == b'#')
                .count()
                == hashes
        {
            return Some(j + 1 + hashes);
        }
        j += 1;
    }
    Some(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comments(text: &str) -> Vec<&str> {
        scan(text, 0..text.len())
            .into_iter()
            .map(|c| &text[c.range])
            .collect()
    }

    #[test]
    fn offsets_count_characters_not_bytes() {
        let file = SourceFile::new("a.rs".into(), "fn é() {}\nstruct Ü;\n");
        let at = |line, column| file.offset(LineColumn { line, column });
        assert_eq!(&file.text[at(1, 3)..], "é() {}\nstruct Ü;\n");
        assert_eq!(&file.text[at(1, 4)..], "() {}\nstruct Ü;\n");
        assert_eq!(&file.text[at(2, 7)..], "Ü;\n");
        assert_eq!(&file.text[at(2, 8)..], ";\n");
        assert_eq!(at(9, 0), file.text.len());
    }

    #[test]
    fn crlf_and_bom_are_removed() {
        let file = SourceFile::new("a.rs".into(), "\u{feff}a\r\nb\r\n");
        assert_eq!(file.text, "a\nb\n");
        assert_eq!(file.line_of(0), 1);
        assert_eq!(file.line_of(2), 2);
    }

    #[test]
    fn normalize_collapses_and_trims() {
        assert_eq!(normalize("Vec< Block >"), "Vec<Block>");
        assert_eq!(
            normalize("(\n    input: &str,\n    execution: &str,\n)"),
            "(input: &str, execution: &str)"
        );
        assert_eq!(normalize("HashMap<\n    K,\n    V,\n>"), "HashMap<K, V>");
        assert_eq!(normalize("[ u8 ; 32 ]"), "[u8 ; 32]");
        assert_eq!(
            normalize("Box<dyn Fn(u32)\n    -> u32 + Send>"),
            "Box<dyn Fn(u32) -> u32 + Send>"
        );
        assert_eq!(normalize("C {\n    x: T,\n}"), "C { x: T }");
        assert_eq!(normalize("T: A\n    + B"), "T: A + B");
        assert_eq!(normalize("a ,b"), "a,b");
        assert_eq!(normalize("(\n    u32,\n)"), "(u32,)");
        assert_eq!(normalize("(fn() -> u8,)"), "(fn() -> u8,)");
        assert_eq!(normalize("(a, (b,),)"), "(a, (b,))");
        assert_eq!(normalize("{N}"), "{N}");
    }

    #[test]
    fn scan_skips_literals_and_nests_blocks() {
        let text = "let s = \"// not\"; // yes\nlet c = '/'; /* a /* b */ c */ x";
        assert_eq!(comments(text), ["// yes", "/* a /* b */ c */"]);
        let raw = "r#\"// no\"# r\"/* no */\" br##\"x\"## // tail";
        assert_eq!(comments(raw), ["// tail"]);
        let lifetimes = "fn f<'a>(x: &'a str) -> char { '\\'' } // end";
        assert_eq!(comments(lifetimes), ["// end"]);
        let raw_ident = "r#type // after";
        assert_eq!(comments(raw_ident), ["// after"]);
    }

    #[test]
    fn scan_marks_doc_comments() {
        let text = "/// d\n//// o\n//! i\n/** b */\n/*** o */\n/**/\n/*! i */\n// o";
        let kinds: Vec<bool> = scan(text, 0..text.len()).iter().map(|c| c.doc).collect();
        assert_eq!(kinds, [true, false, true, true, false, false, true, false]);
    }

    #[test]
    fn clean_lifts_comments_out_of_slices() {
        let file = SourceFile::new("a.rs".into(), "Vec</* inner */ u32>\n");
        assert_eq!(file.clean(0..20), "Vec<u32>");
        let file = SourceFile::new("a.rs".into(), "Option<\n    u32, // trailing\n>");
        assert_eq!(file.clean(0..file.text.len()), "Option<u32>");
    }
}
