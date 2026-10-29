//! Reads ordinary comments from the gaps between nodes and builds notes.

use std::fs;
use std::ops::Range;
use std::path::Path;

use syn::{Attribute, Expr, Meta};

use crate::attrs::string_value;
use crate::flatten::{Piece, flatten};
use crate::model::{ModId, Note, Text, TypeRef, Warning};
use crate::source::{Comment, SourceFile, scan};
use crate::typerefs::{range_of, strip_raw};

/// The ordinary comments of one file, each used at most once.
pub(crate) struct Pool {
    comments: Vec<Comment>,
    taken: Vec<bool>,
}

impl Pool {
    pub(crate) fn new(file: &SourceFile) -> Pool {
        let comments: Vec<Comment> = scan(&file.text, 0..file.text.len())
            .into_iter()
            .filter(|c| !c.doc)
            .collect();
        let taken = vec![false; comments.len()];
        Pool { comments, taken }
    }

    /// Takes every unused comment that starts inside `range`.
    pub(crate) fn take(&mut self, range: Range<usize>) -> Vec<Comment> {
        let first = self
            .comments
            .partition_point(|c| c.range.start < range.start);
        let mut out = Vec::new();
        for i in first..self.comments.len() {
            if self.comments[i].range.start >= range.end {
                break;
            }
            if !self.taken[i] {
                self.taken[i] = true;
                out.push(self.comments[i].clone());
            }
        }
        out
    }
}

/// The comments rule 20 assigns to the children of one scope.
#[derive(Debug, Default)]
pub(crate) struct Attached {
    /// Comments above each child.
    pub(crate) leading: Vec<Vec<Comment>>,
    /// Comments at the end of each child's last line.
    pub(crate) trailing: Vec<Vec<Comment>>,
    /// Comments with nothing after them in the scope.
    pub(crate) dangling: Vec<Comment>,
}

/// Assigns the comments in the gaps of `scope` to its `children`, which are
/// sorted and do not overlap.
pub(crate) fn attach(
    pool: &mut Pool,
    file: &SourceFile,
    scope: Range<usize>,
    children: &[Range<usize>],
) -> Attached {
    let n = children.len();
    let mut out = Attached {
        leading: vec![Vec::new(); n],
        trailing: vec![Vec::new(); n],
        dangling: Vec::new(),
    };
    let mut previous = scope.start;
    let gap_ends = children
        .iter()
        .map(|c| c.start)
        .chain(std::iter::once(scope.end));
    for (k, gap_end) in gap_ends.enumerate() {
        let mut rest = Vec::new();
        for comment in pool.take(previous..gap_end.max(previous)) {
            if k > 0 && is_trailing(file, previous, &comment) {
                out.trailing[k - 1].push(comment);
            } else {
                rest.push(comment);
            }
        }
        match children.get(k) {
            Some(child) => {
                out.leading[k] = rest;
                previous = child.end.max(previous);
            }
            None => out.dangling = rest,
        }
    }
    out
}

/// Tells whether a comment sits on the line where a node ends, with only
/// spaces and separators between them, and ends on that line.
fn is_trailing(file: &SourceFile, node_end: usize, comment: &Comment) -> bool {
    let between = &file.text[node_end.min(comment.range.start)..comment.range.start];
    between.chars().all(|c| matches!(c, ' ' | '\t' | ',' | ';'))
        && !file.text[comment.range.clone()].contains('\n')
}

/// Builds notes for nodes of one file.
pub(crate) struct NoteCx<'a> {
    pub(crate) file: &'a SourceFile,
    pub(crate) refs: &'a mut Vec<TypeRef>,
    pub(crate) module: ModId,
    pub(crate) generics: &'a [String],
    pub(crate) crate_dir: &'a Path,
    pub(crate) warnings: &'a mut Vec<Warning>,
}

impl NoteCx<'_> {
    /// Merges doc attributes and ordinary comments into notes, in the order
    /// they were written.
    pub(crate) fn notes(&mut self, attrs: &[Attribute], comments: Vec<Comment>) -> Vec<Note> {
        enum Event {
            Doc(Vec<String>),
            Plain(Vec<String>),
        }
        let mut events: Vec<(usize, Event)> = Vec::new();
        for attr in attrs {
            if let Some(lines) = self.doc_lines(attr) {
                events.push((range_of(attr, self.file).start, Event::Doc(lines)));
            }
        }
        for comment in comments {
            events.push((
                comment.range.start,
                Event::Plain(comment_lines(self.file, &comment)),
            ));
        }
        events.sort_by_key(|(at, _)| *at);
        let mut notes = Vec::new();
        let mut doc: Vec<String> = Vec::new();
        for (_, event) in events {
            match event {
                Event::Doc(lines) => doc.extend(lines),
                Event::Plain(lines) => {
                    if !doc.is_empty() {
                        let note = self.doc_note(&std::mem::take(&mut doc));
                        notes.extend(note);
                    }
                    if let Some(Note::Plain(previous)) = notes.last_mut() {
                        previous.extend(lines);
                    } else {
                        notes.push(Note::Plain(lines));
                    }
                }
            }
        }
        if !doc.is_empty() {
            notes.extend(self.doc_note(&doc));
        }
        notes
    }

    /// Returns the lines of one doc attribute, or `None` when the attribute
    /// is not a doc comment.
    fn doc_lines(&mut self, attr: &Attribute) -> Option<Vec<String>> {
        match doc_text(attr, self.file, self.crate_dir)? {
            DocText::Written(lines) | DocText::Included(lines) => Some(lines),
            DocText::Unreadable(path) => {
                let line = self.file.line_of(range_of(attr, self.file).start);
                self.warnings.push(Warning {
                    file: self.file.rel.clone(),
                    line,
                    message: format!("cannot read doc file {path}"),
                });
                None
            }
        }
    }

    /// Flattens doc lines into a note, turning names from code spans and
    /// intra-doc links into references.
    fn doc_note(&mut self, lines: &[String]) -> Option<Note> {
        let flat = flatten(lines);
        if flat.is_empty() {
            return None;
        }
        let mut out = Vec::with_capacity(flat.len());
        for line in flat {
            let mut text = Text::default();
            for piece in line {
                match piece {
                    Piece::Text(t) => text.push_str(&t),
                    Piece::Name { text: shown, path } => {
                        let global = path.starts_with("::");
                        let segments = path
                            .trim_start_matches("::")
                            .split("::")
                            .map(strip_raw)
                            .collect();
                        let id = self.refs.len();
                        self.refs.push(TypeRef {
                            text: shown,
                            path: segments,
                            global,
                            module: self.module,
                            generics: self.generics.to_vec(),
                            comment: true,
                            may_be_value: true,
                            target: None,
                            value: None,
                            canonical: None,
                            written: None,
                            foreign: None,
                        });
                        text.push_ref(id);
                    }
                }
            }
            if let Some(crate::model::Seg::Plain(last)) = text.0.last_mut() {
                let trimmed = last.trim_end().len();
                last.truncate(trimmed);
                if last.is_empty() {
                    text.0.pop();
                }
            }
            out.push(text);
        }
        Some(Note::Doc(out))
    }
}

/// The text of one doc attribute.
pub(crate) enum DocText {
    /// Lines written in the source, each still with the space after `///`.
    Written(Vec<String>),
    /// The lines of the file an `include_str!` names.
    Included(Vec<String>),
    /// The path, as written, of a file an `include_str!` names that cannot
    /// be read.
    Unreadable(String),
}

/// Returns the text of one doc attribute, or `None` when the attribute is
/// not a doc comment.
pub(crate) fn doc_text(attr: &Attribute, file: &SourceFile, crate_dir: &Path) -> Option<DocText> {
    let Meta::NameValue(nv) = &attr.meta else {
        return None;
    };
    if !nv.path.is_ident("doc") {
        return None;
    }
    if let Some(value) = string_value(&nv.value) {
        let start = range_of(attr, file).start;
        let source = &file.text[start..];
        if source.starts_with("/**") || source.starts_with("/*!") {
            return Some(DocText::Written(beautify(&value)));
        }
        return Some(DocText::Written(
            value.split('\n').map(str::to_owned).collect(),
        ));
    }
    let Expr::Macro(call) = &nv.value else {
        return None;
    };
    if !call.mac.path.is_ident("include_str") {
        return None;
    }
    let literal: syn::LitStr = call.mac.parse_body().ok()?;
    let dir = file.rel.rfind('/').map_or("", |i| &file.rel[..i]);
    let path = crate_dir.join(dir).join(literal.value());
    Some(match fs::read_to_string(&path) {
        Ok(text) => DocText::Included(
            text.replace("\r\n", "\n")
                .lines()
                .map(str::to_owned)
                .collect(),
        ),
        Err(_) => DocText::Unreadable(literal.value()),
    })
}

/// Returns an ordinary comment's lines. A block comment's continuation
/// lines lose their leading whitespace, except one space before a leading
/// `*`, the way rustfmt re-indents them, so reformatting the source never
/// changes the output.
pub(crate) fn comment_lines(file: &SourceFile, comment: &Comment) -> Vec<String> {
    let text = &file.text[comment.range.clone()];
    text.split('\n')
        .enumerate()
        .map(|(i, line)| {
            let line = line.trim_end();
            if i == 0 {
                return line.to_owned();
            }
            match line.find(|c: char| !c.is_whitespace()) {
                Some(at) if at > 0 && line[at..].starts_with('*') => {
                    format!(" {}", &line[at..])
                }
                Some(at) => line[at..].to_owned(),
                None => String::new(),
            }
        })
        .collect()
}

/// Turns the text of a `/** */` doc comment into lines: a blank first or
/// last line goes, and a leading `*` on every line is removed.
pub(crate) fn beautify(value: &str) -> Vec<String> {
    let mut lines: Vec<&str> = value.split('\n').collect();
    let mut from = 1;
    if lines.len() > 1 && lines[0].trim().is_empty() {
        lines.remove(0);
        from = 0;
    }
    if lines.len() > 1 && lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    let rest = lines.get(from..).unwrap_or(&[]);
    let starred = !rest.is_empty()
        && rest.iter().all(|l| {
            let t = l.trim_start();
            t.starts_with('*') || t.is_empty()
        });
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line = if starred && i >= from {
                let t = line.trim_start();
                t.strip_prefix('*').unwrap_or(t)
            } else {
                line
            };
            line.trim_end().to_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beautify_block_docs() {
        assert_eq!(
            beautify(" A block doc comment\n * over two lines. "),
            [" A block doc comment", " over two lines."]
        );
        assert_eq!(
            beautify("\n * A starred block doc.\n *\n * With a paragraph.\n "),
            [" A starred block doc.", "", " With a paragraph."]
        );
        assert_eq!(beautify(" One line. "), [" One line."]);
    }

    #[test]
    fn gaps_give_leading_trailing_and_dangling() {
        let text = "a, // t\n// lead\nb,\n// end\n";
        let file = SourceFile::new("a.rs".into(), text);
        let mut pool = Pool::new(&file);
        let children = [0..1, 16..17];
        let att = attach(&mut pool, &file, 0..text.len(), &children);
        let show = |c: &Vec<Comment>| -> Vec<String> {
            c.iter().map(|c| text[c.range.clone()].to_owned()).collect()
        };
        assert_eq!(show(&att.trailing[0]), ["// t"]);
        assert_eq!(show(&att.leading[1]), ["// lead"]);
        assert_eq!(show(&att.dangling), ["// end"]);
        assert_eq!(pool.take(0..text.len()).len(), 0);
    }

    #[test]
    fn block_comments_keep_relative_indent() {
        let text = "    /* first\n     * second */\n";
        let file = SourceFile::new("a.rs".into(), text);
        let comment = scan(text, 0..text.len()).remove(0);
        assert_eq!(comment_lines(&file, &comment), ["/* first", " * second */"]);
        let text = "/* A block comment\n   kept as written. */\n";
        let file = SourceFile::new("b.rs".into(), text);
        let comment = scan(text, 0..text.len()).remove(0);
        assert_eq!(
            comment_lines(&file, &comment),
            ["/* A block comment", "kept as written. */"]
        );
    }
}
