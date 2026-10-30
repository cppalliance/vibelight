//! The detectors of body comments: a comment whose words its statement
//! already writes, a comment that names identifiers its next statements do
//! not write, a comment that ties code to code elsewhere, a comment that
//! claims an invariant or a cost, and a papercut in a comment or a doc.
//! The two that compare a comment with its code compare words, never
//! meaning. This file holds the group's registry and the body comment
//! detectors; its siblings hold the words they compare and the docs.

use proc_macro2::LineColumn;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_comments_docs;
use crate::leads_comments_words::{
    Written, content, holds, looks_like_code, named, prose_line, prose_words, without_code,
};
use crate::model::Loc;
use crate::model_body::{BodyCommentFact, Owner, Placement};
use crate::source::SourceFile;

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "comment-statement-overlap",
    "comment-name-absent-next-statement",
    "sync-comment",
    "claim-comment",
    "papercut",
];

/// The ids whose leads rest on what the run does not observe: none, since
/// each lead here rests on the words a comment and its code write.
pub(crate) const ABSENT: &[&str] = &[];

/// The fewest content words a narrating comment has.
const NARRATION_MIN_WORDS: usize = 2;

/// The most content words a narrating comment has; a longer one explains.
const NARRATION_MAX_WORDS: usize = 8;

/// The share of a comment's content words, in percent, that its
/// statements write when the comment narrates them.
const NARRATION_PERCENT: usize = 80;

/// The phrases of a comment that ties code to code elsewhere.
const SYNC_PHRASES: [&str; 4] = ["in sync", "mirror of", "mirrors", "must match"];

/// The words and phrases of a comment that claims an invariant or a
/// safety argument, beside a cost such as `O(1)`.
const CLAIM_PHRASES: [&str; 4] = ["fifo", "safe because", "sorted", "unique"];

/// For each claim word, the starts of the statement words that make the
/// claim where the comment stands, so that the comment narrates it.
const CLAIM_MADE: [(&str, &[&str]); 2] = [
    ("sorted", &["binaryheap", "btreemap", "btreeset", "sort"]),
    ("unique", &["btreeset", "dedup", "hashset", "unique"]),
];

/// The most characters inside the parentheses of a cost such as `O(n)`.
const COST_CHARS: usize = 12;

/// The words and phrases of a papercut.
pub(crate) const PAPERCUT_PHRASES: [&str; 8] = [
    "fixme",
    "for now",
    "hack",
    "hacky",
    "todo",
    "work around",
    "workaround",
    "workarounds",
];

/// The most characters of a comment or doc line that evidence shows.
const SHOWN_CHARS: usize = 72;

/// One comment of a body as the detectors read it: one block comment, or
/// line comments on consecutive lines that go with one statement in one
/// place.
struct Remark<'a> {
    /// The item whose body holds it; an anonymous `const _` is its module.
    item: ItemId,
    /// Where its first line is.
    loc: Loc,
    /// Its lines as written.
    lines: Vec<&'a str>,
    placement: Placement,
    /// The words of the statement it goes with, and of the next two for a
    /// comment above one.
    fingerprint: &'a [String],
    /// The values of its body's string literals, sorted, each once.
    strings: &'a [String],
}

impl Remark<'_> {
    /// Returns its prose: its lines without their markers, joined by
    /// spaces.
    fn prose(&self) -> String {
        let lines: Vec<&str> = (self.lines.iter().map(|line| prose_line(line)))
            .filter(|line| !line.is_empty())
            .collect();
        lines.join(" ")
    }

    /// Starts a lead on the comment: its item, located where the item and
    /// the comment are written.
    fn lead(&self, run: &AnalysisRun<'_>, id: &'static str, mode: Mode, evidence: String) -> Lead {
        (Lead::new(id, mode, evidence).item(run, self.item)).at(self.item.target, self.loc)
    }

    /// Returns as evidence shows it the first of its lines whose prose
    /// passes `test`, or its first line when none does.
    fn shown_where(&self, test: impl Fn(&str) -> bool) -> String {
        let first = self.lines.first().copied().unwrap_or_default();
        let line = (self.lines.iter().copied()).find(|line| test(prose_line(line)));
        shown(line.unwrap_or(first))
    }

    /// Returns as evidence shows it the first of its lines that holds one
    /// of `phrases` outside its code spans.
    fn shown_with(&self, phrases: &[&str]) -> String {
        self.shown_where(|prose| !hits(&prose_words(&without_code(prose)), phrases).is_empty())
    }
}

/// Returns a line as evidence shows it: trimmed, and cut after
/// [`SHOWN_CHARS`] characters with `...`.
pub(crate) fn shown(line: &str) -> String {
    let line = line.trim();
    if line.chars().count() <= SHOWN_CHARS {
        return line.to_owned();
    }
    let cut: String = line.chars().take(SHOWN_CHARS).collect();
    format!("{}...", cut.trim_end())
}

/// Returns phrases in quotes, joined by `, `.
pub(crate) fn quoted(phrases: &[&str]) -> String {
    let quoted: Vec<String> = phrases.iter().map(|p| format!("\"{p}\"")).collect();
    quoted.join(", ")
}

/// Returns every body comment of the run, in target, body, and source
/// order. A comment in the root module's anonymous `const _` has no item
/// to name and is left out.
fn remarks<'a>(run: &AnalysisRun<'a>) -> Vec<Remark<'a>> {
    let mut out: Vec<Remark<'a>> = Vec::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (b, body) in krate.bodies.iter().enumerate() {
            let item = match (run.body_items[t][b], &body.owner) {
                (Some(item), _) => item,
                (None, Owner::Anon(m)) if !krate.modules[*m].path.is_empty() => ItemRef::Module(*m),
                (None, _) => continue,
            };
            let item = ItemId { target: t, item };
            let file = &krate.files[body.file];
            let mut previous: Option<&BodyCommentFact> = None;
            for fact in &body.facts.comments {
                let joins = previous.is_some_and(|before| continues(file, before, fact));
                match out.last_mut().filter(|_| joins) {
                    Some(remark) => remark.lines.extend(fact.text.iter().map(String::as_str)),
                    None => out.push(Remark {
                        item,
                        loc: Loc {
                            file: body.file,
                            line: fact.line,
                        },
                        lines: fact.text.iter().map(String::as_str).collect(),
                        placement: fact.placement,
                        fingerprint: &fact.fingerprint,
                        strings: &body.facts.strings,
                    }),
                }
                previous = Some(fact);
            }
        }
    }
    out
}

/// Tells whether a body comment continues the line comment before it: a
/// line comment alone on the next line that goes with the same statement
/// in the same place, after one alone on its line.
fn continues(file: &SourceFile, before: &BodyCommentFact, fact: &BodyCommentFact) -> bool {
    let alone = |f: &BodyCommentFact| {
        let line = usize::try_from(f.line).unwrap_or(usize::MAX);
        let start = file.offset(LineColumn { line, column: 0 });
        let text = file.text[start..].split('\n').next().unwrap_or_default();
        let first = f.text.first().map_or("", String::as_str);
        f.text.len() == 1 && first.starts_with("//") && text.trim_start().starts_with(first)
    };
    fact.line == before.line + 1
        && (fact.placement, fact.statement) == (before.placement, before.statement)
        && alone(before)
        && alone(fact)
}

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    for remark in remarks(run) {
        let prose = remark.prose();
        let words = prose_words(&without_code(&prose));
        narration(run, &remark, &prose, out);
        absent_names(run, &remark, &prose, out);
        let sync = hits(&words, &SYNC_PHRASES);
        if !sync.is_empty() {
            let line = remark.shown_with(&sync);
            let evidence = format!("{} in \"{line}\"", quoted(&sync));
            out.push(remark.lead(run, "sync-comment", Mode::DirectSyntax, evidence));
        }
        claims(run, &remark, &prose, &words, out);
        let papercuts = hits(&words, &PAPERCUT_PHRASES);
        if !papercuts.is_empty() {
            let line = remark.shown_with(&papercuts);
            let evidence = format!("{} in \"{line}\"", quoted(&papercuts));
            out.push(remark.lead(run, "papercut", Mode::DirectSyntax, evidence));
        }
    }
    leads_comments_docs::find(run, out);
}

/// Returns the phrases that words hold, in list order.
pub(crate) fn hits<'p>(words: &[String], phrases: &[&'p str]) -> Vec<&'p str> {
    (phrases.iter().copied())
        .filter(|phrase| holds(words, phrase))
        .collect()
}

/// `comment-statement-overlap`: a comment above or after a statement,
/// with no reason word and not code that is commented out, whose content
/// words its statements write, by identifier or by stem: at least
/// [`NARRATION_PERCENT`] percent of at least [`NARRATION_MIN_WORDS`] and at
/// most [`NARRATION_MAX_WORDS`] words.
fn narration(run: &AnalysisRun<'_>, remark: &Remark<'_>, prose: &str, out: &mut Vec<Lead>) {
    let placed = matches!(remark.placement, Placement::Leading | Placement::Trailing);
    if !placed || looks_like_code(prose) {
        return;
    }
    let Some(words) = content(prose) else {
        return;
    };
    if !(NARRATION_MIN_WORDS..=NARRATION_MAX_WORDS).contains(&words.len()) {
        return;
    }
    let written = Written::new(remark.fingerprint);
    let shared: Vec<&str> = (words.iter())
        .filter(|word| written.says(word))
        .map(String::as_str)
        .collect();
    if shared.len() * 100 < NARRATION_PERCENT * words.len() {
        return;
    }
    let evidence = format!(
        "{} of {} words of \"{}\" are in its statement: {}",
        shared.len(),
        words.len(),
        remark.shown_where(|_| true),
        shared.join(", ")
    );
    out.push(remark.lead(run, "comment-statement-overlap", Mode::Lexical, evidence));
}

/// `comment-name-absent-next-statement`: a comment above a statement of
/// at most [`NARRATION_MAX_WORDS`] content words, with no reason word and
/// not code that is commented out, that names identifiers explicitly,
/// none of which that statement or the next two write, and none of which
/// is a string literal of the body, such as a JSON key. A comment that
/// gives a reason, or a longer one, explains what lies elsewhere.
fn absent_names(run: &AnalysisRun<'_>, remark: &Remark<'_>, prose: &str, out: &mut Vec<Lead>) {
    let short = content(prose).is_some_and(|words| words.len() <= NARRATION_MAX_WORDS);
    if remark.placement != Placement::Leading || !short || looks_like_code(prose) {
        return;
    }
    let names = named(prose);
    let written = Written::new(remark.fingerprint);
    let literal = |name: &String| remark.strings.binary_search(name).is_ok();
    if names.is_empty()
        || names
            .iter()
            .any(|name| written.names(name) || literal(name))
    {
        return;
    }
    let line = remark.shown_where(|prose| !named(prose).is_empty());
    let names: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
    let evidence = format!(
        "\"{line}\" names {}, which its next statements do not write",
        names.join(", ")
    );
    out.push(remark.lead(
        run,
        "comment-name-absent-next-statement",
        Mode::Lexical,
        evidence,
    ));
}

/// `claim-comment`: a comment that claims a cost such as `O(1)`, or holds
/// a claim word or phrase that its statements do not make where it
/// stands, such as `sorted` above no sort.
fn claims(
    run: &AnalysisRun<'_>,
    remark: &Remark<'_>,
    prose: &str,
    words: &[String],
    out: &mut Vec<Lead>,
) {
    let mut claimed: Vec<&str> = Vec::new();
    for phrase in hits(words, &CLAIM_PHRASES) {
        let made = (CLAIM_MADE.iter().find(|(word, _)| *word == phrase)).is_some_and(|(_, by)| {
            (remark.fingerprint.iter()).any(|entry| by.iter().any(|start| entry.starts_with(start)))
        });
        if !made {
            claimed.push(phrase);
        }
    }
    let costs = cost(prose);
    let mut found: Vec<&str> = costs.as_deref().into_iter().collect();
    found.extend(&claimed);
    if found.is_empty() {
        return;
    }
    let line = remark.shown_where(|line| {
        (costs.is_some() && cost(line).is_some())
            || !hits(&prose_words(&without_code(line)), &claimed).is_empty()
    });
    let evidence = format!("{} in \"{line}\"", quoted(&found));
    out.push(remark.lead(run, "claim-comment", Mode::DirectSyntax, evidence));
}

/// Returns the first cost a text claims, backticks ignored: a capital `O`
/// after no letter, digit, or underscore, then parentheses around at most
/// [`COST_CHARS`] characters with a letter or digit among them, such as
/// `O(1)` or `O(n log n)`.
fn cost(text: &str) -> Option<String> {
    let text = text.replace('`', "");
    for (i, _) in text.match_indices("O(") {
        let before = text[..i].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let rest = &text[i + 2..];
        let Some(close) = rest.find([')', '(']) else {
            continue;
        };
        let inside = &rest[..close];
        let fits = rest[close..].starts_with(')') && inside.chars().count() <= COST_CHARS;
        if fits && inside.chars().any(char::is_alphanumeric) {
            return Some(format!("O({inside})"));
        }
    }
    None
}
