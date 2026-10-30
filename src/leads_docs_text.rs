//! The prose of a doc text as the doc detectors read it: its words and
//! code spans in written order, each with its sentence and the heading
//! above it, outside code blocks, headings, and reference definitions, and
//! the explicit links it writes, each with its destination, and how a
//! detector matches phrases in it and quotes it. `leads_docs_scan.rs`
//! reads it. A passage, a run of lines up to a blank line, that holds one
//! of the config's boilerplate texts is left unread by every doc detector.

use std::borrow::Cow;
use std::ops::Range;

#[cfg(test)]
pub(crate) use crate::leads_docs_scan::read;
pub(crate) use crate::leads_docs_scan::read_except;

/// The most characters a quote of prose keeps.
pub(crate) const QUOTE_CHARS: usize = 160;

/// What one token of prose is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Letters, digits, and underscores, with an apostrophe, a hyphen, or
    /// a dot kept between two of them, such as `can't` or `2.5`.
    Word,
    /// A code span.
    Code,
    /// One character that is neither of those nor white space.
    Punct,
}

/// One token of prose.
#[derive(Clone, Debug)]
pub(crate) struct Tok {
    pub(crate) kind: Kind,
    /// A word in lowercase, a code span's content, or the character.
    pub(crate) text: String,
    /// The bytes of the doc text it is written at, a code span's
    /// backticks included.
    pub(crate) span: Range<usize>,
    /// Its sentence: a paragraph, a list item, and a heading each start a
    /// new one.
    pub(crate) sentence: usize,
    /// The heading above it, as an index into [`Prose::headings`].
    pub(crate) section: Option<usize>,
}

/// An explicit link: `[label](dest)`, `[label][name]`, `[label][]`, or
/// `[label]`.
#[derive(Clone, Debug)]
pub(crate) struct Link {
    /// The bytes of the whole link.
    pub(crate) span: Range<usize>,
    /// The label's tokens, as indexes into [`Prose::toks`].
    pub(crate) label: Range<usize>,
    /// The destination as written: inline, from the reference definition
    /// the link names, or else that name or the label itself.
    pub(crate) dest: String,
    /// Whether the link is `[label]` alone.
    pub(crate) shortcut: bool,
}

/// The prose of one doc text.
#[derive(Clone, Debug)]
pub(crate) struct Prose<'a> {
    pub(crate) doc: &'a str,
    pub(crate) toks: Vec<Tok>,
    /// Each heading's text, in lowercase.
    pub(crate) headings: Vec<String>,
    pub(crate) links: Vec<Link>,
    /// The bytes of each code block between its fences.
    pub(crate) blocks: Vec<Range<usize>>,
    /// The bytes of each boilerplate passage, left unread.
    pub(crate) ignored: Vec<Range<usize>>,
}

/// One word of a phrase: the lowercase words that may stand there. A
/// phrase whose first word is [`START`] matches only where a sentence
/// starts.
pub(crate) type Phrase = &'static [&'static [&'static str]];

/// The mark of a phrase that matches only where a sentence starts.
pub(crate) const START: &[&str] = &["^"];

impl Prose<'_> {
    /// Tells whether token `i` starts its sentence.
    pub(crate) fn starts(&self, i: usize) -> bool {
        i == 0 || self.toks[i - 1].sentence != self.toks[i].sentence
    }

    /// Returns the heading above token `i`.
    pub(crate) fn section(&self, i: usize) -> Option<&str> {
        let heading = self.toks.get(i)?.section?;
        self.headings.get(heading).map(String::as_str)
    }

    /// Returns each match of `phrase`: its words one after another in one
    /// sentence, with commas between them skipped, as the range of its
    /// tokens.
    pub(crate) fn matches(&self, phrase: Phrase) -> Vec<Range<usize>> {
        self.each(phrase).collect()
    }

    fn each(&self, phrase: Phrase) -> impl Iterator<Item = Range<usize>> + '_ {
        let (anchored, words) = match phrase.split_first() {
            Some((first, rest)) if *first == START => (true, rest),
            _ => (false, phrase),
        };
        (0..self.toks.len())
            .filter(move |&s| !anchored || self.starts(s))
            .filter_map(move |s| Some(s..self.match_at(s, words)?))
    }

    fn match_at(&self, s: usize, words: &[&[&str]]) -> Option<usize> {
        let sentence = self.toks.get(s)?.sentence;
        let mut k = s;
        for (n, set) in words.iter().enumerate() {
            while n > 0 && self.toks.get(k).is_some_and(|t| t.text == ",") {
                k += 1;
            }
            let tok = self.toks.get(k)?;
            let fits = tok.kind == Kind::Word && tok.sentence == sentence;
            if !fits || !set.contains(&tok.text.as_str()) {
                return None;
            }
            k += 1;
        }
        Some(k)
    }

    /// Returns the first match of any of `phrases`, by where it starts.
    pub(crate) fn first(&self, phrases: &[Phrase]) -> Option<Range<usize>> {
        let found = phrases.iter().filter_map(|phrase| self.each(phrase).next());
        found.min_by_key(|m| (m.start, m.end))
    }

    /// Returns the words of a match, joined by spaces.
    pub(crate) fn words(&self, found: Range<usize>) -> String {
        let words: Vec<&str> = (self.toks[found].iter())
            .filter(|t| t.kind == Kind::Word)
            .map(|t| t.text.as_str())
            .collect();
        words.join(" ")
    }

    /// Returns a quote of sentence `s`.
    pub(crate) fn sentence(&self, s: usize) -> String {
        let mut toks = self.toks.iter().filter(|t| t.sentence == s);
        let Some(first) = toks.next() else {
            return String::new();
        };
        let last = toks.next_back().unwrap_or(first);
        quote(&self.doc[first.span.start..last.span.end])
    }

    /// Returns a match's words and a quote of its sentence, as evidence.
    pub(crate) fn said(&self, found: Range<usize>) -> String {
        let sentence = self.toks[found.start].sentence;
        format!(
            "\"{}\" in \"{}\"",
            self.words(found),
            self.sentence(sentence)
        )
    }

    /// Returns the doc text without its boilerplate passages.
    pub(crate) fn kept(&self) -> Cow<'_, str> {
        without(self.doc, &self.ignored)
    }
}

/// Returns the bytes of each passage of a doc text, a run of lines up to a
/// blank line, that holds one of `texts`, with runs of white space
/// compared as one space, in order.
pub(crate) fn boilerplate(doc: &str, texts: &[String]) -> Vec<Range<usize>> {
    let collapse = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let wanted: Vec<String> = (texts.iter().map(|text| collapse(text)))
        .filter(|text| !text.is_empty())
        .collect();
    let mut passages = Vec::new();
    let (mut from, mut at): (Option<usize>, usize) = (None, 0);
    for line in doc.split('\n') {
        if line.trim().is_empty() {
            passages.extend(from.take().map(|start| start..at.saturating_sub(1)));
        } else {
            from.get_or_insert(at);
        }
        at += line.len() + 1;
    }
    passages.extend(from.map(|start| start..doc.len()));
    passages.retain(|passage| {
        let text = collapse(&doc[passage.clone()]);
        wanted.iter().any(|wanted| text.contains(wanted.as_str()))
    });
    passages
}

/// Returns a doc text without the bytes of `passages`, which are sorted.
pub(crate) fn without<'d>(doc: &'d str, passages: &[Range<usize>]) -> Cow<'d, str> {
    if passages.is_empty() {
        return Cow::Borrowed(doc);
    }
    let mut out = String::new();
    let mut at = 0;
    for passage in passages {
        out.push_str(&doc[at..passage.start]);
        at = passage.end;
    }
    out.push_str(&doc[at..]);
    Cow::Owned(out)
}

/// Returns text with its white space collapsed, cut at [`QUOTE_CHARS`].
pub(crate) fn quote(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= QUOTE_CHARS {
        return collapsed;
    }
    let cut: String = collapsed.chars().take(QUOTE_CHARS).collect();
    format!("{}...", cut.trim_end())
}

/// Returns the runs of letters, digits, and underscores in a text.
pub(crate) fn identifiers(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown<'p>(prose: &'p Prose<'_>) -> Vec<(usize, Option<&'p str>, String)> {
        (prose.toks.iter().enumerate())
            .map(|(i, t)| {
                let text = match t.kind {
                    Kind::Code => format!("`{}`", t.text),
                    _ => t.text.clone(),
                };
                (t.sentence, prose.section(i), text)
            })
            .collect()
    }

    #[test]
    fn prose_splits_into_sentences_and_sections_outside_code_blocks() {
        let doc = "Reads `a`, e.g. the file. Then\nstops.\n\n```\nlet x = 1;\n```\n\
                   # Errors\n\n- `path` is gone\n- it can't be read";
        let prose = read(doc);
        let s = |n: usize, section: Option<&'static str>, text: &str| (n, section, text.to_owned());
        assert_eq!(
            shown(&prose),
            [
                s(0, None, "reads"),
                s(0, None, "`a`"),
                s(0, None, ","),
                s(0, None, "e.g"),
                s(0, None, "."),
                s(0, None, "the"),
                s(0, None, "file"),
                s(0, None, "."),
                s(1, None, "then"),
                s(1, None, "stops"),
                s(1, None, "."),
                s(2, Some("errors"), "`path`"),
                s(2, Some("errors"), "is"),
                s(2, Some("errors"), "gone"),
                s(3, Some("errors"), "it"),
                s(3, Some("errors"), "can't"),
                s(3, Some("errors"), "be"),
                s(3, Some("errors"), "read"),
            ]
        );
        assert_eq!(&doc[prose.blocks[0].clone()], "let x = 1;");
        assert_eq!(prose.sentence(1), "Then stops.");
    }

    #[test]
    fn links_keep_their_labels_as_prose_and_find_their_destinations() {
        let doc = "See [`Parser`], [the docs](https://x.org \"t\"), [run][r], and\n\
                   [`a` b][]. [^1] and [ ] are no links.\n\n[r]: crate::run\n[`a` b]: Z";
        let prose = read(doc);
        let links: Vec<(&str, &str, bool)> = (prose.links.iter())
            .map(|l| (&doc[l.span.clone()], l.dest.as_str(), l.shortcut))
            .collect();
        assert_eq!(
            links,
            [
                ("[`Parser`]", "`Parser`", true),
                ("[the docs](https://x.org \"t\")", "https://x.org", false),
                ("[run][r]", "crate::run", false),
                ("[`a` b][]", "Z", false),
            ]
        );
        assert_eq!(prose.words(prose.links[1].label.clone()), "the docs");
        let found = |phrase: Phrase| -> Vec<(usize, usize)> {
            (prose.matches(phrase).into_iter())
                .map(|m| (m.start, m.end))
                .collect()
        };
        assert_eq!(found(&[START, &["see"]]), [(0, 1)]);
        assert_eq!(found(&[START, &["and"]]), []);
        assert_eq!(found(&[&["docs"], &["run"]]), [(4, 7)], "commas skip");
    }
}
