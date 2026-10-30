//! Lexes one source text for the census: each name it writes, with how it
//! writes the name, whether it is inside macro arguments, and whether it
//! is inside a test-only span, and each qualified path, which
//! `census_paths` reads from the same flat tokens. Doc attributes,
//! keywords, and lifetimes give no names. This scan keeps doc-attribute
//! tokens and flags them, and flags `macro_rules!` bodies as macro tokens;
//! it shares only the keyword table with `collect_body_tokens`.

use std::fmt::Write as _;
use std::ops::Range;

use proc_macro2::{Delimiter, LineColumn, Spacing, Span, TokenStream, TokenTree};

use crate::census::Form;
use crate::census_paths::Path;

/// Rust's strict and reserved keywords, sorted for a binary search. A
/// keyword never names an item, but a raw identifier such as `r#type` does.
const KEYWORDS: [&str; 51] = [
    "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
    "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// The word that starts a macro definition, which names no item either.
const MACRO_RULES: &str = "macro_rules";

/// The keywords that may start a path.
pub(crate) const PATH_KEYWORDS: [&str; 4] = ["Self", "crate", "self", "super"];

/// What one text writes, in source order.
pub(crate) struct Lexed {
    /// The text of every identifier, one after another, without `r#`.
    pub(crate) words: String,
    pub(crate) names: Vec<Name>,
    pub(crate) paths: Vec<Path>,
}

/// One name a text writes.
pub(crate) struct Name {
    /// The name, as a range of `Lexed::words`.
    pub(crate) text: Range<usize>,
    pub(crate) form: Form,
    pub(crate) in_macro: bool,
    pub(crate) in_test: bool,
}

/// One token, with each group flattened into its delimiters.
pub(crate) enum Tok {
    /// An identifier, as a range of the words without `r#`, and whether it
    /// is a keyword, which a raw identifier never is.
    Ident(Range<usize>, bool),
    /// A punctuation character, and whether another one follows at once.
    Punct(char, bool),
    Lit,
    /// An open delimiter and the index of its close.
    Open(Delimiter, usize),
    Close,
}

pub(crate) struct Flat {
    pub(crate) tok: Tok,
    pub(crate) span: Span,
}

/// The flat tokens of one text and the words their identifiers index.
pub(crate) struct Toks {
    pub(crate) toks: Vec<Flat>,
    words: String,
}

/// Lexes a text; `None` when it does not lex. A token that starts inside
/// one of `tests`, sorted spans that do not overlap, is in test code.
pub(crate) fn lex(text: &str, tests: &[(LineColumn, LineColumn)]) -> Option<Lexed> {
    let stream: TokenStream = text.parse().ok()?;
    let mut toks = Toks {
        toks: Vec::new(),
        words: String::new(),
    };
    toks.flatten(stream);
    let hidden = toks.doc_flags();
    let in_macro = toks.macro_flags();
    let in_test = toks.test_flags(tests);
    let mut names = Vec::new();
    for (i, flat) in toks.toks.iter().enumerate() {
        if let (Tok::Ident(text, _), Some(form), false) = (&flat.tok, toks.form(i), hidden[i]) {
            names.push(Name {
                text: text.clone(),
                form,
                in_macro: in_macro[i],
                in_test: in_test[i],
            });
        }
    }
    let paths = toks.paths(&hidden, &in_macro, &in_test);
    Some(Lexed {
        words: toks.words,
        names,
        paths,
    })
}

/// Tells whether a word, as written, is a keyword.
pub(crate) fn is_keyword(word: &str) -> bool {
    KEYWORDS.binary_search(&word).is_ok()
}

impl Toks {
    /// Flattens a stream, taking each group apart without copying it.
    fn flatten(&mut self, stream: TokenStream) {
        for tree in stream {
            let span = tree.span();
            let tok = match tree {
                TokenTree::Group(group) => {
                    let open = self.toks.len();
                    let delimiter = group.delimiter();
                    self.toks.push(Flat {
                        tok: Tok::Open(delimiter, open),
                        span: group.span_open(),
                    });
                    let close_span = group.span_close();
                    let inner = group.stream();
                    drop(group);
                    self.flatten(inner);
                    let close = self.toks.len();
                    self.toks[open].tok = Tok::Open(delimiter, close);
                    self.toks.push(Flat {
                        tok: Tok::Close,
                        span: close_span,
                    });
                    continue;
                }
                TokenTree::Ident(ident) => {
                    let start = self.words.len();
                    let _ = write!(self.words, "{ident}");
                    let raw = self.words[start..].starts_with("r#");
                    if raw {
                        self.words.replace_range(start..start + 2, "");
                    }
                    let keyword = !raw && is_keyword(&self.words[start..]);
                    Tok::Ident(start..self.words.len(), keyword)
                }
                TokenTree::Punct(punct) => {
                    Tok::Punct(punct.as_char(), punct.spacing() == Spacing::Joint)
                }
                TokenTree::Literal(_) => Tok::Lit,
            };
            self.toks.push(Flat { tok, span });
        }
    }

    pub(crate) fn tok(&self, i: usize) -> Option<&Tok> {
        self.toks.get(i).map(|flat| &flat.tok)
    }

    /// Returns the identifier at `i` as written without `r#`, and whether
    /// it is a keyword.
    pub(crate) fn ident(&self, i: usize) -> Option<(&str, bool)> {
        match self.tok(i)? {
            Tok::Ident(text, keyword) => Some((&self.words[text.clone()], *keyword)),
            _ => None,
        }
    }

    /// Tells whether `::` starts at `i`.
    pub(crate) fn colons(&self, i: usize) -> bool {
        matches!(self.tok(i), Some(Tok::Punct(':', true)))
            && matches!(self.tok(i + 1), Some(Tok::Punct(':', _)))
    }

    /// Tells whether the token at `i` is a lone `.`, not the second dot of
    /// `..`.
    pub(crate) fn dot(&self, i: usize) -> bool {
        let range = i > 0 && matches!(self.tok(i - 1), Some(Tok::Punct('.', true)));
        matches!(self.tok(i), Some(Tok::Punct('.', _))) && !range
    }

    /// Returns, for each token, whether it is part of a doc attribute: `#`,
    /// an optional `!`, and a bracket group that starts with `doc`.
    fn doc_flags(&self) -> Vec<bool> {
        let mut flags = vec![false; self.toks.len()];
        for i in 0..self.toks.len() {
            if !matches!(self.tok(i), Some(Tok::Punct('#', _))) {
                continue;
            }
            let at = i + 1 + usize::from(matches!(self.tok(i + 1), Some(Tok::Punct('!', _))));
            if let Some(Tok::Open(Delimiter::Bracket, close)) = self.tok(at) {
                if self.ident(at + 1).is_some_and(|(word, _)| word == "doc") {
                    flags[i..=*close].fill(true);
                }
            }
        }
        flags
    }

    /// Returns, for each token, whether it is inside a macro invocation's
    /// arguments or a `macro_rules!` body. Each token is marked at most
    /// once, since an invocation inside another one lies inside its group.
    fn macro_flags(&self) -> Vec<bool> {
        let mut flags = vec![false; self.toks.len()];
        let mut done = 0;
        for i in 0..self.toks.len() {
            if let Some((open, close)) = self.macro_group(i) {
                let from = (open + 1).max(done);
                if from < close {
                    flags[from..close].fill(true);
                }
                done = done.max(close);
            }
        }
        flags
    }

    /// Returns the group that a macro invocation at `i` opens, as the
    /// indexes of its delimiters: a name that is not a keyword, `!`, and a
    /// group, or `macro_rules!`, a name, and a group.
    fn macro_group(&self, i: usize) -> Option<(usize, usize)> {
        let (word, keyword) = self.ident(i)?;
        let rules = word == MACRO_RULES;
        if keyword || !matches!(self.tok(i + 1), Some(Tok::Punct('!', false))) {
            return None;
        }
        let open = if rules { i + 3 } else { i + 2 };
        match self.tok(open) {
            Some(Tok::Open(_, close)) => Some((open, *close)),
            _ => None,
        }
    }

    /// Returns, for each token, whether it starts inside one of `tests`.
    /// Only the bounds of each span are looked up, since tokens are in
    /// source order.
    fn test_flags(&self, tests: &[(LineColumn, LineColumn)]) -> Vec<bool> {
        let mut flags = vec![false; self.toks.len()];
        for (start, end) in tests {
            let from = self.toks.partition_point(|flat| flat.span.start() < *start);
            let to = self.toks.partition_point(|flat| flat.span.start() < *end);
            flags[from..to.max(from)].fill(true);
        }
        flags
    }

    /// Returns how the identifier at `i` writes a name, or `None` for a
    /// keyword, a lifetime, or a token that is no identifier.
    fn form(&self, i: usize) -> Option<Form> {
        let (word, keyword) = self.ident(i)?;
        let lifetime = i > 0 && matches!(self.tok(i - 1), Some(Tok::Punct('\'', _)));
        if keyword || word == MACRO_RULES || lifetime {
            return None;
        }
        if i > 0 && self.dot(i - 1) {
            let called = matches!(self.tok(i + 1), Some(Tok::Open(Delimiter::Parenthesis, _)))
                || self.colons(i + 1);
            return Some(if called { Form::Method } else { Form::Field });
        }
        let invoked = matches!(self.tok(i + 1), Some(Tok::Punct('!', false)))
            && matches!(self.tok(i + 2), Some(Tok::Open(..)));
        Some(if invoked { Form::Macro } else { Form::Plain })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(text: &str) -> Vec<(String, Form, bool)> {
        let lexed = lex(text, &[]).expect("it lexes");
        let words = &lexed.words;
        lexed
            .names
            .into_iter()
            .map(|n| (words[n.text].to_owned(), n.form, n.in_macro))
            .collect()
    }

    #[test]
    fn keywords_are_sorted_for_the_binary_search() {
        assert!(KEYWORDS.windows(2).all(|w| w[0] < w[1]));
        assert!(PATH_KEYWORDS.iter().all(|k| is_keyword(k)));
    }

    #[test]
    fn a_census_scan_records_gen_as_a_method_name() {
        let found = names("fn f() { r.gen(); }");
        assert!(
            found
                .iter()
                .any(|(n, form, _)| n == "gen" && *form == Form::Method)
        );
    }

    #[test]
    fn names_keep_their_form_and_macro_flag_without_keywords_docs_or_lifetimes() {
        let text = "/// Gauge in a doc\nfn f<'a>(x: &'a Gauge) -> u8 { x.len() + x.level \
                    + x.iter().count::<u8>() + m!(Gauge::new(), y.size()) + r#type }";
        let found = names(text);
        let s = str::to_owned;
        assert_eq!(
            found,
            [
                (s("f"), Form::Plain, false),
                (s("x"), Form::Plain, false),
                (s("Gauge"), Form::Plain, false),
                (s("u8"), Form::Plain, false),
                (s("x"), Form::Plain, false),
                (s("len"), Form::Method, false),
                (s("x"), Form::Plain, false),
                (s("level"), Form::Field, false),
                (s("x"), Form::Plain, false),
                (s("iter"), Form::Method, false),
                (s("count"), Form::Method, false),
                (s("u8"), Form::Plain, false),
                (s("m"), Form::Macro, false),
                (s("Gauge"), Form::Plain, true),
                (s("new"), Form::Plain, true),
                (s("y"), Form::Plain, true),
                (s("size"), Form::Method, true),
                (s("type"), Form::Plain, false),
            ]
        );
    }

    #[test]
    fn macro_bodies_are_macro_tokens_test_spans_mark_tokens_and_bad_text_is_none() {
        let found = names("macro_rules! twice { ($x:expr) => { $crate::run($x) } }");
        assert!(found.iter().any(|(n, _, inside)| n == "twice" && !inside));
        assert!(found.iter().any(|(n, _, inside)| n == "run" && *inside));
        let at = |line, column| LineColumn { line, column };
        let text = "fn a() { x::y() }\nfn b() { z::w() }\n";
        let lexed = lex(text, &[(at(2, 0), at(2, 17))]).expect("it lexes");
        let tested: Vec<(&str, bool)> = (lexed.names.iter())
            .map(|n| (&lexed.words[n.text.clone()], n.in_test))
            .collect();
        assert_eq!(
            tested,
            [
                ("a", false),
                ("x", false),
                ("y", false),
                ("b", true),
                ("z", true),
                ("w", true)
            ]
        );
        let paths: Vec<(u32, bool)> = (lexed.paths.iter())
            .map(|p| (u32::try_from(p.at.line).unwrap_or(0), p.in_test))
            .collect();
        assert_eq!(paths, [(1, false), (2, true)]);
        assert!(lex("fn f() { \"open", &[]).is_none());
    }
}
