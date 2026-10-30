//! Reads the qualified paths of a text's flat tokens for the census: each
//! `a::b` path of two or more segments, a leading `::` kept, generic
//! arguments passed over, and every `use` group split into one path per
//! name it brings in. A path after a dot, a `::`, or a closing `>` is the
//! rest of something else and starts nothing.

use proc_macro2::{Delimiter, LineColumn};

use crate::census_lex::{PATH_KEYWORDS, Tok, Toks};

/// One qualified path a text writes.
pub(crate) struct Path {
    /// The segments, without `r#`.
    pub(crate) segments: Vec<String>,
    pub(crate) global: bool,
    /// Where the path starts: its first segment, or its leading `::`.
    pub(crate) at: LineColumn,
    pub(crate) in_macro: bool,
    pub(crate) in_test: bool,
}

/// The flags of the token a path starts at.
#[derive(Clone, Copy)]
struct Start {
    global: bool,
    in_macro: bool,
    in_test: bool,
}

impl Toks {
    /// Reads every qualified path outside doc attributes.
    pub(crate) fn paths(&self, hidden: &[bool], in_macro: &[bool], in_test: &[bool]) -> Vec<Path> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.toks.len() {
            i = match self.start(i).filter(|_| !hidden[i]) {
                Some(global) => {
                    let start = Start {
                        global,
                        in_macro: in_macro[i],
                        in_test: in_test[i],
                    };
                    self.scan(i, start, &mut out)
                }
                None => i + 1,
            };
        }
        out
    }

    /// Tells whether a path starts at `i`, and whether it starts with `::`:
    /// a name or a path keyword that `::` follows, with no `.`, `::`, or
    /// lifetime quote just before it, or a `::` with no name, closing `>` of
    /// generic arguments, group, or literal just before it. A `:` or the `>`
    /// of `->` or `=>` ends no path, so a `::` after one starts a path.
    fn start(&self, i: usize) -> Option<bool> {
        let before = i.checked_sub(1).and_then(|b| self.tok(b));
        if let Some((word, keyword)) = self.ident(i) {
            let pathable = !keyword || PATH_KEYWORDS.contains(&word);
            let joined = i > 0 && (self.dot(i - 1) || (i > 1 && self.colons(i - 2)));
            let lifetime = matches!(before, Some(Tok::Punct('\'', _)));
            return (self.colons(i + 1) && pathable && !joined && !lifetime).then_some(false);
        }
        if !self.colons(i) || self.ident(i + 2).is_none() {
            return None;
        }
        let joined = match before {
            Some(Tok::Ident(..)) => self
                .ident(i - 1)
                .is_some_and(|(word, keyword)| !keyword || PATH_KEYWORDS.contains(&word)),
            Some(Tok::Punct('>', _)) => !matches!(
                i.checked_sub(2).and_then(|b| self.tok(b)),
                Some(Tok::Punct('-' | '=', true))
            ),
            Some(Tok::Close | Tok::Lit) => true,
            _ => false,
        };
        (!joined).then_some(true)
    }

    /// Reads the path that starts at `i` into `out` when it has two or more
    /// segments, and returns where to go on: past the `use` group the path
    /// ends in, or else just past its start, so that the paths inside its
    /// generic arguments are read too.
    fn scan(&self, i: usize, start: Start, out: &mut Vec<Path>) -> usize {
        let mut segments = Vec::new();
        let mut k = if start.global { i + 2 } else { i };
        while let Some((word, _)) = self.ident(k) {
            segments.push(word.to_owned());
            if !self.colons(k + 1) {
                break;
            }
            k += 3;
            match self.tok(k) {
                Some(Tok::Punct('<', _)) => match self.angles_end(k) {
                    Some(end) if self.colons(end) => k = end + 2,
                    _ => break,
                },
                Some(Tok::Open(Delimiter::Brace, close)) => {
                    let close = *close;
                    self.group(k, close, &segments, start, out);
                    return close + 1;
                }
                _ => {}
            }
        }
        if segments.len() > 1 {
            out.push(path(segments, start, self.toks[i].span.start()));
        }
        i + 1
    }

    /// Returns the index past the `>` that closes the `<` at `open`,
    /// passing over each `->` and each group; `None` when the enclosing
    /// group or the statement ends first.
    fn angles_end(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut k = open;
        while let Some(flat) = self.toks.get(k) {
            match flat.tok {
                Tok::Punct('<', _) => depth += 1,
                Tok::Punct('>', _) if !matches!(self.tok(k - 1), Some(Tok::Punct('-', true))) => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(k + 1);
                    }
                }
                Tok::Open(_, close) => k = close,
                Tok::Close | Tok::Punct(';', _) => return None,
                _ => {}
            }
            k += 1;
        }
        None
    }

    /// Reads each tree of the `use` group between `open` and `close`, such
    /// as `{a, b::c, d::{e, self}, *}`, as a path after `prefix`.
    fn group(
        &self,
        open: usize,
        close: usize,
        prefix: &[String],
        start: Start,
        out: &mut Vec<Path>,
    ) {
        let mut k = open + 1;
        while k < close {
            let mut end = k;
            while end < close && !matches!(self.toks[end].tok, Tok::Punct(',', _)) {
                end = match self.toks[end].tok {
                    Tok::Open(_, inner) => inner + 1,
                    _ => end + 1,
                };
            }
            if k < end {
                let mut segments = prefix.to_vec();
                let mut at = k;
                while at < end {
                    match self.ident(at) {
                        Some((word, _)) if word != "self" => segments.push(word.to_owned()),
                        Some(_) => break,
                        None => {
                            if let Some(Tok::Open(Delimiter::Brace, inner)) = self.tok(at) {
                                self.group(at, *inner, &segments, start, out);
                                segments.clear();
                            }
                            break;
                        }
                    }
                    if !self.colons(at + 1) {
                        break;
                    }
                    at += 3;
                }
                if segments.len() > 1 {
                    out.push(path(segments, start, self.toks[k].span.start()));
                }
            }
            k = end + 1;
        }
    }
}

fn path(segments: Vec<String>, start: Start, at: LineColumn) -> Path {
    Path {
        segments,
        global: start.global,
        at,
        in_macro: start.in_macro,
        in_test: start.in_test,
    }
}

#[cfg(test)]
mod tests {
    use crate::census_lex::lex;

    fn paths_of(text: &str) -> Vec<(String, bool, bool)> {
        let lexed = lex(text, &[]).expect("it lexes");
        lexed
            .paths
            .into_iter()
            .map(|p| (p.segments.join("::"), p.global, p.in_macro))
            .collect()
    }

    #[test]
    fn paths_split_use_groups_and_skip_turbofish_and_method_paths() {
        let text = "use crate::{a::b, c::{self, d}, e as f, *};\n\
                    fn g() { let v = Vec::<u8>::new(); ::std::mem::take(&mut x); \
                    <T as Tr>::m(); foo.bar::<X>(); m!(t::Gauge::new()); \
                    let r = 0..Limit::MAX; HashMap::<K, Vec<crate::Item>>::default(); }";
        let s = |p: &str, global, in_macro| (p.to_owned(), global, in_macro);
        assert_eq!(
            paths_of(text),
            [
                s("crate::a::b", false, false),
                s("crate::c", false, false),
                s("crate::c::d", false, false),
                s("crate::e", false, false),
                s("Vec::new", false, false),
                s("std::mem::take", true, false),
                s("t::Gauge::new", false, true),
                s("Limit::MAX", false, false),
                s("HashMap::default", false, false),
                s("crate::Item", false, false),
            ]
        );
    }

    #[test]
    fn a_path_after_a_colon_or_an_arrow_starts_a_qualified_path() {
        for text in ["x: ::a::B", "-> ::a::B", "=> ::a::B"] {
            assert_eq!(paths_of(text), [("a::B".to_owned(), true, false)], "{text}");
        }
    }

    #[test]
    fn a_macro_body_path_is_a_macro_path_and_a_doc_path_is_none() {
        assert_eq!(
            paths_of("macro_rules! m { () => { $crate::run() } }"),
            [("crate::run".to_owned(), false, true)]
        );
        assert_eq!(
            paths_of("#[doc = concat!(a::b)] struct S;"),
            Vec::<(String, bool, bool)>::new()
        );
    }
}
