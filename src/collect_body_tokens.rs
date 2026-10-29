//! Scans a body's tokens for analysis, macro arguments included: its clone
//! fingerprint, method-call names, literals, the identifiers inside macro
//! arguments, and the words of a statement a comment goes with.

use std::collections::BTreeSet;

use proc_macro2::{Delimiter, Literal, Spacing, TokenStream, TokenTree};

use crate::model_body::Fingerprint;
use crate::typerefs::strip_raw;

/// The keywords, which canonical tokens keep as written; `true` and
/// `false` are literals.
const KEYWORDS: [&str; 36] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
    "where", "while",
];

/// The version of the shingle hash, its first input byte. A change to how
/// tokens become canonical changes it.
const SHINGLE_VERSION: u8 = 1;
/// How many canonical tokens one shingle holds.
const SHINGLE: usize = 5;
/// The fewest canonical tokens a body has shingles for.
const MIN_TOKENS: usize = 20;

/// One token, with each group flattened into its delimiters.
#[derive(Clone, Debug)]
enum Tok {
    Ident(String),
    /// A punctuation character, and whether another one follows at once.
    Punct(char, bool),
    Lit(Literal),
    Open(Delimiter),
    Close(Delimiter),
}

/// What a body's tokens hold.
pub(crate) struct Scan {
    pub(crate) fingerprint: Fingerprint,
    /// Method-call names, sorted, repeats kept.
    pub(crate) methods: Vec<String>,
    /// The identifiers inside macro arguments, sorted, each once.
    pub(crate) macro_idents: Vec<String>,
    /// String literal values, sorted, each once.
    pub(crate) strings: Vec<String>,
    /// Number literals as base 10 digits, sorted, each once.
    pub(crate) numbers: Vec<String>,
    /// How often macro arguments name each parameter, in the order of
    /// `params`.
    pub(crate) param_uses: Vec<usize>,
}

/// Scans the tokens of a body's text; text that does not lex has none.
pub(crate) fn scan(text: &str, params: &[String]) -> Scan {
    let toks = tokens(text);
    let (canonical, in_macro) = canonical(&toks);
    let mut methods = Vec::new();
    let mut idents = BTreeSet::new();
    let mut strings = BTreeSet::new();
    let mut numbers = BTreeSet::new();
    let mut param_uses = vec![0; params.len()];
    for (i, tok) in toks.iter().enumerate() {
        match tok {
            Tok::Ident(name) if in_macro[i] => {
                let name = strip_raw(name);
                if let Some(at) = params.iter().position(|p| *p == name) {
                    param_uses[at] += 1;
                }
                idents.insert(name);
            }
            Tok::Punct('.', false) => methods.extend(method_after(&toks, i)),
            Tok::Lit(literal) => match syn::Lit::new(literal.clone()) {
                syn::Lit::Str(s) => {
                    strings.insert(s.value());
                }
                syn::Lit::Int(n) => {
                    numbers.insert(n.base10_digits().to_owned());
                }
                syn::Lit::Float(f) => {
                    numbers.insert(f.base10_digits().to_owned());
                }
                _ => {}
            },
            _ => {}
        }
    }
    methods.sort();
    Scan {
        fingerprint: fingerprint(&canonical),
        methods,
        macro_idents: idents.into_iter().collect(),
        strings: strings.into_iter().collect(),
        numbers: numbers.into_iter().collect(),
        param_uses,
    }
}

/// Returns the words of a statement's text for a comment's fingerprint:
/// each identifier in lowercase and each run of punctuation, but no
/// literal, `,`, `;`, or lifetime quote.
pub(crate) fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut run = String::new();
    for tok in tokens(text) {
        if let Tok::Punct(c, joint) = tok {
            if c == '\'' {
                end_run(&mut out, &mut run);
                continue;
            }
            run.push(c);
            if !joint {
                end_run(&mut out, &mut run);
            }
            continue;
        }
        end_run(&mut out, &mut run);
        if let Tok::Ident(name) = tok {
            out.push(strip_raw(&name).to_lowercase());
        }
    }
    end_run(&mut out, &mut run);
    out
}

fn end_run(out: &mut Vec<String>, run: &mut String) {
    let run = std::mem::take(run);
    if !matches!(run.as_str(), "" | "," | ";") {
        out.push(run);
    }
}

/// Returns the canonical tokens, each macro invocation folded into
/// `MACRO`, and for each token whether it is inside macro arguments.
fn canonical(toks: &[Tok]) -> (Vec<String>, Vec<bool>) {
    let closes = closes(toks);
    let mut canonical = Vec::new();
    let mut in_macro = vec![false; toks.len()];
    let mut i = 0;
    while i < toks.len() {
        if let Some(end) = macro_end(toks, &closes, i) {
            canonical.push("MACRO".to_owned());
            in_macro[i + 2..=end].fill(true);
            i = end + 1;
        } else {
            canonical.extend(canonical_token(&toks[i]));
            i += 1;
        }
    }
    (canonical, in_macro)
}

/// Lexes text into flat tokens, without doc attributes, which is what
/// doc comments lex to; text that does not lex gives none.
fn tokens(text: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    if let Ok(stream) = text.parse::<TokenStream>() {
        flatten(stream, &mut out);
    }
    out
}

fn flatten(stream: TokenStream, out: &mut Vec<Tok>) {
    let trees: Vec<TokenTree> = stream.into_iter().collect();
    let mut i = 0;
    while i < trees.len() {
        if let Some(len) = doc_attribute(&trees[i..]) {
            i += len;
            continue;
        }
        match &trees[i] {
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let marked = delimiter != Delimiter::None;
                if marked {
                    out.push(Tok::Open(delimiter));
                }
                flatten(group.stream(), out);
                if marked {
                    out.push(Tok::Close(delimiter));
                }
            }
            TokenTree::Ident(ident) => out.push(Tok::Ident(ident.to_string())),
            TokenTree::Punct(punct) => out.push(Tok::Punct(
                punct.as_char(),
                punct.spacing() == Spacing::Joint,
            )),
            TokenTree::Literal(literal) => out.push(Tok::Lit(literal.clone())),
        }
        i += 1;
    }
}

/// Returns how many trees a doc attribute at the start of `trees` takes:
/// `#`, an optional `!`, and a bracket group that starts with `doc`.
fn doc_attribute(trees: &[TokenTree]) -> Option<usize> {
    let Some(TokenTree::Punct(hash)) = trees.first() else {
        return None;
    };
    if hash.as_char() != '#' {
        return None;
    }
    let bang = matches!(trees.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '!');
    let at = 1 + usize::from(bang);
    let Some(TokenTree::Group(group)) = trees.get(at) else {
        return None;
    };
    let first = group.stream().into_iter().next();
    let doc = matches!(first, Some(TokenTree::Ident(ident)) if ident == "doc");
    (group.delimiter() == Delimiter::Bracket && doc).then_some(at + 1)
}

/// Returns, for each open delimiter, the index of its close; any other
/// token maps to itself.
fn closes(toks: &[Tok]) -> Vec<usize> {
    let mut out: Vec<usize> = (0..toks.len()).collect();
    let mut open = Vec::new();
    for (i, tok) in toks.iter().enumerate() {
        match tok {
            Tok::Open(_) => open.push(i),
            Tok::Close(_) => {
                if let Some(start) = open.pop() {
                    out[start] = i;
                }
            }
            _ => {}
        }
    }
    out
}

/// Returns where a macro invocation that starts at `at` ends: a name that
/// is not a keyword, a `!`, and a delimited group.
fn macro_end(toks: &[Tok], closes: &[usize], at: usize) -> Option<usize> {
    let Tok::Ident(name) = &toks[at] else {
        return None;
    };
    let bang = matches!(toks.get(at + 1), Some(Tok::Punct('!', false)));
    let group = matches!(toks.get(at + 2), Some(Tok::Open(_)));
    (!KEYWORDS.contains(&name.as_str()) && bang && group).then(|| closes[at + 2])
}

/// Returns the name of the method a `.` at `dot` calls: an identifier
/// that is not a keyword, an optional turbofish, and parentheses. The
/// second dot of `..` calls nothing.
fn method_after(toks: &[Tok], dot: usize) -> Option<String> {
    if dot > 0 && matches!(toks[dot - 1], Tok::Punct('.', true)) {
        return None;
    }
    let Some(Tok::Ident(name)) = toks.get(dot + 1) else {
        return None;
    };
    if KEYWORDS.contains(&name.as_str()) {
        return None;
    }
    let mut at = dot + 2;
    let turbofish = matches!(toks.get(at), Some(Tok::Punct(':', true)))
        && matches!(toks.get(at + 1), Some(Tok::Punct(':', _)))
        && matches!(toks.get(at + 2), Some(Tok::Punct('<', _)));
    if turbofish {
        at = angles_end(toks, at + 2)?;
    }
    matches!(toks.get(at), Some(Tok::Open(Delimiter::Parenthesis))).then(|| strip_raw(name))
}

/// Returns the index after the `>` that closes the `<` at `open`, passing
/// over the `>` of each `->`.
fn angles_end(toks: &[Tok], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, tok) in toks.iter().enumerate().skip(open) {
        match tok {
            Tok::Punct('<', _) => depth += 1,
            Tok::Punct('>', _) if !matches!(toks[i - 1], Tok::Punct('-', true)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Returns a token's canonical form; the delimiters of an invisible group
/// have none.
fn canonical_token(tok: &Tok) -> Option<String> {
    let text = match tok {
        Tok::Ident(name) if name == "true" || name == "false" => "LIT",
        Tok::Ident(name) => KEYWORDS
            .iter()
            .find(|keyword| **keyword == name.as_str())
            .copied()
            .unwrap_or("ID"),
        Tok::Punct(c, _) => return Some(c.to_string()),
        Tok::Lit(_) => "LIT",
        Tok::Open(delimiter) => match delimiter {
            Delimiter::Parenthesis => "(",
            Delimiter::Brace => "{",
            Delimiter::Bracket => "[",
            Delimiter::None => return None,
        },
        Tok::Close(delimiter) => match delimiter {
            Delimiter::Parenthesis => ")",
            Delimiter::Brace => "}",
            Delimiter::Bracket => "]",
            Delimiter::None => return None,
        },
    };
    Some(text.to_owned())
}

/// Builds the fingerprint of a body's canonical tokens.
fn fingerprint(canonical: &[String]) -> Fingerprint {
    let mut shingles: Vec<u64> = if canonical.len() < MIN_TOKENS {
        Vec::new()
    } else {
        canonical.windows(SHINGLE).map(shingle).collect()
    };
    shingles.sort_unstable();
    shingles.dedup();
    Fingerprint {
        tokens: canonical.len(),
        shingles,
    }
}

/// Hashes one shingle with version 1 of the shingle hash: FNV-1a 64 over
/// the version byte, then each token's bytes, each followed by `0xFF`,
/// which no UTF-8 text holds.
fn shingle(tokens: &[String]) -> u64 {
    let bytes = tokens
        .iter()
        .flat_map(|token| token.bytes().chain(std::iter::once(0xFF)));
    fnv1a64(std::iter::once(SHINGLE_VERSION).chain(bytes))
}

/// The 64-bit FNV-1a hash.
fn fnv1a64(bytes: impl IntoIterator<Item = u8>) -> u64 {
    bytes.into_iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn fnv1a64_matches_the_published_vectors() {
        assert_eq!(fnv1a64(*b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(*b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(*b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn a_shingle_hashes_the_version_and_its_tokens() {
        let tokens = owned(&["let", "ID", "=", "LIT", ";"]);
        assert_eq!(shingle(&tokens), 0xa562_97b4_dc3a_99ac);
    }

    #[test]
    fn canonical_tokens_drop_comments_and_docs_and_fold_macros() {
        let text = "{ let x = foo!(a, b); /// doc\n /* c */ x.len() == r#type || !(y) }";
        let (found, in_macro) = canonical(&tokens(text));
        let expected = "{ let ID = MACRO ; ID . ID ( ) = = ID | | ! ( ID ) }";
        assert_eq!(found.join(" "), expected);
        assert_eq!(in_macro.iter().filter(|inside| **inside).count(), 5);
        assert_eq!(scan(text, &[]).fingerprint.tokens, 21);
    }

    #[test]
    fn bodies_of_one_shape_share_shingles_and_short_ones_have_none() {
        let a = "{ let total = items.iter().map(|x| x * 2).sum::<u32>(); total + 1 }";
        let b = "{ let sum = values.iter().map(|v| v * 9).sum::<u64>(); sum + 7 }";
        let c = "{ let sum = values.iter().filter(|v| v > 9).count(); sum }";
        let (a, b, c) = (scan(a, &[]), scan(b, &[]), scan(c, &[]));
        assert!(a.fingerprint.tokens >= MIN_TOKENS);
        assert!(
            !a.fingerprint.shingles.is_empty(),
            "a long body has shingles"
        );
        assert_eq!(a.fingerprint, b.fingerprint);
        assert_ne!(a.fingerprint.shingles, c.fingerprint.shingles);
        let short = scan("{ f(x) }", &[]);
        assert_eq!(short.fingerprint.tokens, 6);
        assert!(
            short.fingerprint.shingles.is_empty(),
            "too short for shingles"
        );
        let mut sorted = a.fingerprint.shingles.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted, a.fingerprint.shingles);
    }

    #[test]
    fn method_names_come_from_tokens_and_macro_arguments() {
        let text = "{ a.len() + b.iter().map(f).count() + c.collect::<Vec<fn() -> u8>>() \
                    + (0..n()).sum() + x.y + d.r#match() + w.await; format!(\"{}\", z.to_string()) }";
        let found = scan(text, &[]).methods;
        let expected = [
            "collect",
            "count",
            "iter",
            "len",
            "map",
            "match",
            "sum",
            "to_string",
        ];
        assert_eq!(found, owned(&expected));
    }

    #[test]
    fn literals_and_macro_identifiers_are_kept_and_parameters_counted() {
        let text = "{ let s = \"a\"; let t = r#\"b\"#; let n = 0x10 + 1_000u32; \
                    println!(\"{} {}\", x, 2.5f64); y }";
        let params = owned(&["x", "y"]);
        let found = scan(text, &params);
        assert_eq!(found.strings, owned(&["a", "b", "{} {}"]));
        assert_eq!(found.numbers, owned(&["1000", "16", "2.5"]));
        assert_eq!(found.macro_idents, owned(&["x"]));
        assert_eq!(found.param_uses, [1, 0]);
    }

    #[test]
    fn statement_words_keep_identifiers_and_operators() {
        let found = words("self.count += step * 2; // note\nlet x: &'a T = y;");
        let expected = [
            "self", ".", "count", "+=", "step", "*", "let", "x", ":", "&", "a", "t", "=", "y",
        ];
        assert_eq!(found, owned(&expected));
    }

    #[test]
    fn text_that_does_not_lex_has_no_tokens() {
        let found = scan("{ \"open", &[]);
        assert_eq!(found.fingerprint, Fingerprint::default());
        assert!(words("\"open").is_empty(), "an open string does not lex");
    }
}
