//! The words that the comment detectors compare: a comment's prose without
//! its markers, the lowercase words of prose, the stopwords and reason
//! words of version 1, the stem a word compares by, the phrases a text
//! holds, the identifiers a comment names explicitly, and the words of a
//! statement fingerprint.

use std::collections::BTreeSet;

use crate::census_lex::is_keyword;
use crate::leads_words::words;

/// The stopwords of version 1, sorted: English words that name nothing a
/// statement writes.
const STOPWORDS_V1: [&str; 106] = [
    "a", "about", "above", "after", "again", "all", "also", "am", "an", "and", "any", "are", "as",
    "at", "be", "been", "before", "being", "below", "between", "both", "but", "by", "can", "did",
    "do", "does", "doing", "done", "down", "during", "each", "either", "etc", "every", "few",
    "for", "from", "had", "has", "have", "having", "he", "her", "here", "his", "how", "i", "if",
    "in", "into", "is", "it", "its", "just", "me", "more", "most", "my", "now", "of", "off", "on",
    "once", "one", "only", "onto", "or", "our", "out", "over", "own", "per", "she", "some", "such",
    "than", "that", "the", "their", "them", "then", "there", "these", "they", "this", "those",
    "through", "to", "too", "under", "up", "us", "very", "via", "was", "we", "were", "what",
    "when", "where", "which", "will", "with", "you", "your",
];

/// The reason words of version 1, sorted: prose that holds one gives a
/// reason, a condition, a contrast, or a negation, which no statement
/// narrates. A word that ends in `n't` is one too.
const REASONS_V1: [&str; 33] = [
    "already",
    "although",
    "assume",
    "assumes",
    "avoid",
    "avoids",
    "because",
    "cannot",
    "could",
    "ensure",
    "ensures",
    "however",
    "instead",
    "invariant",
    "may",
    "might",
    "must",
    "never",
    "no",
    "not",
    "otherwise",
    "rather",
    "safety",
    "should",
    "since",
    "so",
    "though",
    "unless",
    "until",
    "why",
    "without",
    "would",
    "yet",
];

/// The fewest letters a stem keeps; a plain plural keeps one fewer.
const STEM_MIN: usize = 4;

/// Returns the prose of one comment line: without its `//`, `/*`, `*/`,
/// or leading `*` markers, trimmed.
pub(crate) fn prose_line(line: &str) -> &str {
    let text = line.trim();
    let text = text.strip_suffix("*/").unwrap_or(text);
    let text = match text.strip_prefix("//") {
        Some(rest) => rest.trim_start_matches(['/', '!']),
        None => match text.strip_prefix("/*") {
            Some(rest) => rest.trim_start_matches(['*', '!']),
            None => text.trim_start_matches('*'),
        },
    };
    text.trim()
}

/// Splits text at its backticks into code spans and the prose around
/// them, each with whether it is code; an unclosed backtick opens no span.
fn spans(text: &str) -> impl Iterator<Item = (bool, &str)> {
    let count = text.split('`').count();
    (text.split('`').enumerate()).map(move |(i, part)| (i % 2 == 1 && i + 1 < count, part))
}

/// Returns text with each code span in backticks removed.
pub(crate) fn without_code(text: &str) -> String {
    let prose: Vec<&str> = (spans(text).filter(|(code, _)| !code))
        .map(|(_, part)| part)
        .collect();
    prose.join(" ")
}

/// Tells whether a comment's prose reads as code that is commented out:
/// outside its code spans it starts like an item or a binding, ends with
/// `;`, `{`, or `}`, or holds `=>`.
pub(crate) fn looks_like_code(text: &str) -> bool {
    let prose = without_code(text);
    let prose = prose.trim();
    let starts =
        (["let ", "fn ", "use ", "pub ", "#["].iter()).any(|start| prose.starts_with(start));
    starts || prose.ends_with([';', '{', '}']) || prose.contains("=>")
}

/// Returns the lowercase words of prose, in order: each run of letters,
/// digits, underscores, and apostrophes, without quotes around it or a
/// possessive `'s`.
pub(crate) fn prose_words(text: &str) -> Vec<String> {
    let text = text.replace('\u{2019}', "'");
    text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '\''))
        .map(|word| word.trim_matches('\''))
        .filter(|word| !word.is_empty())
        .map(|word| {
            let word = word.to_lowercase();
            match word.strip_suffix("'s") {
                Some(base) => base.to_owned(),
                None => word,
            }
        })
        .collect()
}

/// Tells whether words hold a phrase: its words, one after another.
pub(crate) fn holds(words: &[String], phrase: &str) -> bool {
    let wanted: Vec<&str> = phrase.split(' ').collect();
    (words.windows(wanted.len())).any(|run| run.iter().zip(&wanted).all(|(a, b)| a == b))
}

/// Tells whether a lowercase word gives a reason, a condition, a
/// contrast, or a negation.
fn is_reason(word: &str) -> bool {
    REASONS_V1.binary_search(&word).is_ok() || word.ends_with("n't")
}

/// Returns the content words of a comment's prose, each once, in order:
/// its words that are no stopword, no number, and longer than one letter.
/// `None` when a word of the prose gives a reason, a condition, a
/// contrast, or a negation, which no statement narrates.
pub(crate) fn content(prose: &str) -> Option<Vec<String>> {
    let all = prose_words(prose);
    if all.iter().any(|word| is_reason(word)) {
        return None;
    }
    let mut out: Vec<String> = Vec::new();
    for word in all {
        let number = word.chars().all(|c| c.is_ascii_digit());
        let short = word.chars().count() < 2;
        let stop = STOPWORDS_V1.binary_search(&word.as_str()).is_ok();
        if !number && !short && !stop && !out.contains(&word) {
            out.push(word);
        }
    }
    Some(out)
}

/// Returns the stem a lowercase word compares by: an `ing`, `ed`, or `es`
/// ending removed, `ies` turned into `y`, a plain plural `s` removed, then a
/// final `e` removed. Each removal needs [`STEM_MIN`] letters to stay, except
/// the plural `s`, which needs one fewer and is left on a word that ends in
/// `ss`, so `parse`, `parses`, `parsed`, and `parsing` share one stem.
fn stem(word: &str) -> String {
    let long = |base: &&str| base.chars().count() >= STEM_MIN;
    if let Some(base) = word.strip_suffix("ies").filter(long) {
        return format!("{base}y");
    }
    let plural = |base: &&str| base.chars().count() + 1 >= STEM_MIN && !base.ends_with('s');
    let base = (word.strip_suffix("ing").filter(long))
        .or_else(|| word.strip_suffix("ed").filter(long))
        .or_else(|| word.strip_suffix("es").filter(long))
        .or_else(|| word.strip_suffix('s').filter(plural))
        .unwrap_or(word);
    base.strip_suffix('e')
        .filter(long)
        .unwrap_or(base)
        .to_owned()
}

/// Tells whether a code span reads as Rust that names items: a path, a
/// type, a call, or a field chain, such as `self.items.len()` or
/// `Option<&str>`, with no space, quote, slash, or single colon, and no
/// leading bracket, which a protocol token such as `[DONE]` has.
fn rusty(span: &str) -> bool {
    let shaped = (span.chars()).all(|c| c.is_alphanumeric() || "_:.()<>&*!?,".contains(c));
    let starts = span.starts_with(|c: char| c.is_alphabetic() || "_&*".contains(c));
    shaped && starts && !span.replace("::", "").contains(':')
}

/// Returns the identifiers a comment's prose names explicitly, each once
/// ignoring case, in order: every name inside a code span that reads as
/// Rust, and outside code spans every word with an underscore inside it,
/// every path, and every word with a capital right after a lowercase
/// letter, such as `HashMap`. A path counts by its last segment; a keyword,
/// a number, and a name of one letter are no names.
pub(crate) fn named(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (code, part) in spans(text) {
        if code && !rusty(part) {
            continue;
        }
        let paths = part.replace("::", "\u{0}");
        let runs = paths.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '\u{0}'));
        for run in runs.map(|run| run.trim_matches('\u{0}')) {
            let path = run.contains('\u{0}');
            let name = run.rsplit('\u{0}').next().unwrap_or(run);
            let bare = name.trim_matches('_');
            let explicit = code || path || bare.contains('_') || has_hump(bare);
            let word = name.starts_with(|c: char| c.is_alphabetic() || c == '_')
                && name.chars().count() > 1
                && name.chars().any(char::is_alphabetic);
            let lower = name.to_lowercase();
            let seen = out.iter().any(|known| known.to_lowercase() == lower);
            if explicit && word && !seen && !is_keyword(name) {
                out.push(name.to_owned());
            }
        }
    }
    out
}

/// Tells whether a word has a capital right after a lowercase letter.
fn has_hump(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    (chars.windows(2)).any(|pair| pair[0].is_lowercase() && pair[1].is_uppercase())
}

/// The words of a statement fingerprint, as the comment detectors compare
/// them.
pub(crate) struct Written<'a> {
    /// The fingerprint: lowercase identifiers and operators, sorted, each
    /// once.
    idents: &'a [String],
    /// The words of its identifiers, split at underscores.
    words: BTreeSet<String>,
    /// The stems of those words.
    stems: BTreeSet<String>,
}

impl<'a> Written<'a> {
    pub(crate) fn new(fingerprint: &'a [String]) -> Written<'a> {
        let identifiers = (fingerprint.iter())
            .filter(|entry| entry.starts_with(|c: char| c.is_alphanumeric() || c == '_'));
        let words: BTreeSet<String> = identifiers
            .flat_map(|ident| ident.split('_').filter(|w| !w.is_empty()))
            .map(str::to_owned)
            .collect();
        let stems = words.iter().map(|word| stem(word)).collect();
        Written {
            idents: fingerprint,
            words,
            stems,
        }
    }

    /// Tells whether the fingerprint holds a lowercase identifier.
    fn holds(&self, lower: &str) -> bool {
        (self.idents)
            .binary_search_by(|entry| entry.as_str().cmp(lower))
            .is_ok()
    }

    /// Tells whether the statements write an identifier a comment names:
    /// as one identifier, or as identifiers that hold each of its words, so
    /// that `items_len` matches `items.len()`.
    pub(crate) fn names(&self, name: &str) -> bool {
        let parts = words(name);
        self.holds(&name.to_lowercase())
            || (!parts.is_empty() && parts.iter().all(|part| self.words.contains(part)))
    }

    /// Tells whether the statements write a content word of a comment: as
    /// an identifier, or as a word of one with the same stem, each word of
    /// a snake case word alike.
    pub(crate) fn says(&self, word: &str) -> bool {
        let parts: Vec<&str> = word.split('_').filter(|w| !w.is_empty()).collect();
        self.holds(word)
            || (!parts.is_empty() && parts.iter().all(|part| self.stems.contains(&stem(part))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn the_word_lists_are_sorted_and_apart() {
        assert!(STOPWORDS_V1.windows(2).all(|w| w[0] < w[1]), "stopwords");
        assert!(REASONS_V1.windows(2).all(|w| w[0] < w[1]), "reasons");
        assert!(REASONS_V1.iter().all(|w| !STOPWORDS_V1.contains(w)));
    }

    #[test]
    fn comment_markers_go_and_prose_splits_into_lowercase_words() {
        assert_eq!(prose_line("  // Parse the args"), "Parse the args");
        assert_eq!(prose_line("//! inner"), "inner");
        assert_eq!(prose_line("/* one */"), "one");
        assert_eq!(prose_line(" * two"), "two");
        assert_eq!(prose_line("*/"), "");
        assert_eq!(
            prose_words("Don't read Foo's `parse_args`, 'quoted'."),
            owned(&["don't", "read", "foo", "parse_args", "quoted"])
        );
        assert_eq!(without_code("a `b` c `d"), "a   c  d");
        assert!(holds(&prose_words("Keep this in sync with"), "in sync"));
        assert!(!holds(&prose_words("async in"), "in sync"));
    }

    #[test]
    fn content_words_drop_stopwords_and_a_reason_drops_the_prose() {
        assert_eq!(
            content("Push the item onto the list, 2 times").unwrap(),
            owned(&["push", "item", "list", "times"])
        );
        assert_eq!(content("Sort first because dedup needs it"), None);
        assert_eq!(content("This isn't the end"), None);
    }

    #[test]
    fn stems_join_plural_and_tense_forms() {
        for (word, expected) in [
            ("parse", "pars"),
            ("parses", "pars"),
            ("parsed", "pars"),
            ("parsing", "pars"),
            ("entries", "entry"),
            ("entry", "entry"),
            ("lines", "line"),
            ("line", "line"),
            ("args", "arg"),
            ("class", "class"),
            ("classes", "class"),
            ("string", "string"),
            ("strings", "string"),
        ] {
            assert_eq!(stem(word), expected, "{word}");
        }
    }

    #[test]
    fn a_comment_names_backticked_snake_path_and_camel_words() {
        assert_eq!(
            named("Rebuild `index` with parse_args, then crate::load::Loader and HashMap."),
            owned(&["index", "parse_args", "Loader", "HashMap"])
        );
        assert_eq!(
            named("`self.items.len()` but not `run(\"a b\")`, `[DONE]`, `uses:`, JSON, or `T`"),
            owned(&["items", "len"])
        );
        assert_eq!(named("__bold__ and load::Loader:x"), owned(&["Loader"]));
        assert_eq!(named("Retry `try` and `box` first."), owned(&[]));
        assert!(looks_like_code("let x = run(y);"));
        assert!(looks_like_code("Some(v) => v"));
        assert!(!looks_like_code("Read `x;` first"));
    }

    #[test]
    fn a_fingerprint_writes_names_whole_or_by_their_words() {
        let fingerprint = owned(&[".", "=", "items", "len", "let", "parse_args", "total"]);
        let written = Written::new(&fingerprint);
        assert!(written.names("items_len") && written.names("parse_args"));
        assert!(written.names("ParseArgs") && !written.names("index"));
        assert!(written.says("parsed") && written.says("item") && written.says("total"));
        assert!(!written.says("count"));
    }
}
