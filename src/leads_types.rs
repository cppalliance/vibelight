//! The types that detectors compare. A type is a list of tokens in its
//! Rust spelling, each name standing for what it reaches, with no lifetime
//! but `'static`, so that two spellings of one type compare equal. What a
//! type is made of is read in `leads_types_shape.rs`.

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::model::{RefId, Seg, Text};
use crate::prelude::item_path;

/// One token of a compared type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Tok {
    /// A name, as what it reaches: `#3::parse::Args` for an item of target
    /// 3, `std::string::String` for an item outside the run, `$T` for a
    /// generic parameter, `Self`, or the path as written when nothing
    /// resolves.
    Name(String),
    /// A keyword, a number, a `'static` lifetime, or punctuation.
    Code(String),
}

impl Tok {
    pub(crate) fn code(&self) -> Option<&str> {
        match self {
            Tok::Code(code) => Some(code),
            Tok::Name(_) => None,
        }
    }
}

/// A type, a generic parameter list, or a `where` clause, as compared
/// tokens.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Ty(pub(crate) Vec<Tok>);

/// Returns what reference `r` of target `t` names, as a [`Tok::Name`]
/// holds it; a name that reaches `own` is `Self`.
pub(crate) fn identity(run: &AnalysisRun<'_>, t: usize, r: RefId, own: Option<ItemId>) -> String {
    if let Some(item) = run.item_of_ref(t, r) {
        if Some(item) == own {
            return "Self".to_owned();
        }
        return item_name(run, item);
    }
    let reference = &run.krates[t].refs[r];
    if let Some(path) = reference.outside().and_then(item_path) {
        return path;
    }
    let written = reference.path.join("::");
    match reference.path.first() {
        Some(first) if reference.generics.contains(first) => format!("${written}"),
        _ => written,
    }
}

/// Returns an item of the run as a [`Tok::Name`] holds it.
pub(crate) fn item_name(run: &AnalysisRun<'_>, item: ItemId) -> String {
    format!("#{}::{}", item.target, run.path(item).join("::"))
}

/// Returns the last segment of a name as [`Tok::Name`] holds it, without
/// a generic parameter's mark.
pub(crate) fn leaf(name: &str) -> String {
    let last = name.rsplit("::").next().unwrap_or(name);
    last.strip_prefix('$').unwrap_or(last).to_owned()
}

impl Ty {
    /// Reads a text of target `t` in its Rust spelling.
    pub(crate) fn of(run: &AnalysisRun<'_>, t: usize, text: &Text, own: Option<ItemId>) -> Ty {
        Ty::segs(run, t, text.rust(), own)
    }

    /// Reads segments of target `t`.
    pub(crate) fn segs(run: &AnalysisRun<'_>, t: usize, segs: &[Seg], own: Option<ItemId>) -> Ty {
        let mut toks = Vec::new();
        for seg in segs {
            match seg {
                Seg::Plain(text) => lex(text, &mut toks),
                Seg::Ref(r) => toks.push(Tok::Name(identity(run, t, *r, own))),
            }
        }
        Ty(tidy(&toks))
    }

    /// Builds a type of one name.
    pub(crate) fn name(name: &str) -> Ty {
        Ty(vec![Tok::Name(name.to_owned())])
    }

    /// Returns the tokens as one string, each name as what it reaches.
    pub(crate) fn key(&self) -> String {
        self.render(str::to_owned)
    }

    /// Returns the tokens as a reader sees them, each name as its last
    /// segment.
    pub(crate) fn shown(&self) -> String {
        self.render(leaf)
    }

    /// Returns a field or a parameter as the evidence shows it, such as
    /// `` `names: Vec<String>` ``.
    pub(crate) fn typed(&self, name: &str) -> String {
        format!("`{name}: {}`", self.shown())
    }

    fn render(&self, name: impl Fn(&str) -> String) -> String {
        let mut out = String::new();
        let mut wordy = false;
        for tok in &self.0 {
            let (text, word) = match tok {
                Tok::Name(n) => (name(n), true),
                Tok::Code(c) => (c.clone(), c.starts_with(|c: char| is_word(c) || c == '\'')),
            };
            if wordy && word {
                out.push(' ');
            }
            out.push_str(&text);
            wordy = word;
        }
        out
    }
}

/// Tells whether a character can be part of an identifier or a number.
pub(crate) fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Appends the tokens of plain type text, leaving out every lifetime but
/// `'static`.
fn lex(text: &str, out: &mut Vec<Tok>) {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        let start = i;
        if c.is_whitespace() {
            i += 1;
        } else if is_word(c) || c == '\'' {
            i += 1;
            while chars.get(i).is_some_and(|&c| is_word(c)) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if !word.starts_with('\'') || word == "'static" {
                out.push(Tok::Code(word));
            }
        } else {
            let pair: String = chars[i..chars.len().min(i + 2)].iter().collect();
            i += if pair == "::" || pair == "->" { 2 } else { 1 };
            out.push(Tok::Code(chars[start..i].iter().collect()));
        }
    }
}

/// Drops what leaving lifetimes out leaves behind: empty angle brackets,
/// and a comma or a `+` with nothing on one side.
fn tidy(toks: &[Tok]) -> Vec<Tok> {
    let mut out: Vec<Tok> = Vec::with_capacity(toks.len());
    for (i, tok) in toks.iter().enumerate() {
        let next = toks.get(i + 1).and_then(Tok::code);
        let prev = out.last().and_then(Tok::code);
        let lone = match tok.code() {
            Some(">") if prev == Some("<") => {
                out.pop();
                continue;
            }
            Some(",") => {
                matches!(prev, Some("<" | "(" | "[" | ",")) || matches!(next, Some(">" | "]"))
            }
            Some("+") => {
                out.is_empty()
                    || i + 1 == toks.len()
                    || matches!(prev, Some("<" | "(" | "," | "+" | ":"))
                    || matches!(next, Some(">" | ")" | "]" | ","))
            }
            _ => false,
        };
        if !lone {
            out.push(tok.clone());
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Builds a type from pieces: one with `::` or a leading `$` is a
    /// name, and any other is plain text.
    pub(crate) fn ty(pieces: &[&str]) -> Ty {
        let mut out = Vec::new();
        for piece in pieces {
            if piece.contains("::") || piece.starts_with('$') {
                out.push(Tok::Name((*piece).to_owned()));
            } else {
                lex(piece, &mut out);
            }
        }
        Ty(tidy(&out))
    }

    #[test]
    fn lifetimes_leave_no_trace_but_static_stays() {
        let map = ty(&[
            "&'a ",
            "std::collections::BTreeMap",
            "<'a, ",
            "std::string::String",
            ">",
        ]);
        assert_eq!(map.shown(), "&BTreeMap<String>");
        assert_eq!(ty(&["#0::Iter", "<'_>"]).shown(), "Iter");
        assert_eq!(
            ty(&["&'static ", "std::primitive::str"]).shown(),
            "&'static str"
        );
        let bounded = ty(&["Box<dyn ", "#0::Run", " + 'a + ", "std::marker::Send", ">"]);
        assert_eq!(bounded.shown(), "Box<dyn Run+Send>");
        assert_eq!(
            bounded.key(),
            "Box<dyn #0::Run+std::marker::Send>",
            "a key keeps what each name reaches"
        );
    }
}
