//! The doc detectors that match the words of one doc: conditions on a
//! field's state, words that tell an earlier design, excuses for a
//! visibility, errors called impossible, and claims about a collection's
//! order, uniqueness, or keys. Each phrase list is versioned.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::leads::{Lead, Mode};
use crate::leads_comments_words::prose_line;
use crate::leads_docs::{Doc, Docs, Node};
use crate::leads_docs_text::{Kind, Phrase, Prose, START, identifiers, read_except};
use crate::leads_items::handles_err;
use crate::leads_types::{Ty, leaf};
use crate::model::{Loc, Vis};

/// The fields of one declaration or variant that hang on one sibling,
/// named by the sibling.
type Owner = (ItemId, Option<usize>, String);

/// Finds what the words of each doc say about a field's state, history,
/// visibility, errors, and collection invariants.
pub(crate) fn find(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    let mut conditions: BTreeMap<Owner, Vec<(&Doc<'_>, String)>> = BTreeMap::new();
    for doc in &docs.all {
        history_words(run, doc, out);
        visibility_excuse(run, doc, out);
        if let Node::Field(f) = doc.node {
            let field = &docs.catalog.fields[f];
            if let Some((sibling, evidence)) = conditional_field(docs, f, doc) {
                let owner = (field.decl, field.variant, sibling);
                conditions.entry(owner).or_default().push((doc, evidence));
            }
            container_invariant_claim(run, docs, f, doc, out);
        }
    }
    for found in conditions.into_values() {
        let shown: Vec<&str> = found
            .iter()
            .map(|(_, evidence)| evidence.as_str())
            .collect();
        let mut lead = Lead::new("conditional-field", Mode::DirectSyntax, shown.join("; "));
        for (doc, _) in &found {
            lead = doc.add(run, lead);
        }
        out.push(lead);
    }
    impossible_error(run, docs, out);
}

/// Version 1 of the history words: phrases that tell an earlier design.
/// `no longer` counts only before a word that tells use, and `legacy` only
/// after `for`, or before `callers`, `code`, `shim`, or `support`, since both
/// also tell runtime state or an outside system's history.
const HISTORY: [Phrase; 9] = [
    &[&["formerly"]],
    &[&["was", "were"], &["renamed"]],
    &[&["replaces", "replaced"], &["the"], &["old"]],
    &[&["used"], &["to"], &["be"]],
    &[&["no"], &["longer"], &NO_LONGER],
    &[START, &["previously"]],
    &[&["was", "were", "had", "has", "have"], &["previously"]],
    &[&["for"], &["legacy"]],
    &[&["legacy"], &["callers", "code", "shim", "support"]],
];

/// The words after `no longer` that tell an item's use.
const NO_LONGER: [&str; 9] = [
    "called",
    "emitted",
    "needed",
    "produced",
    "read",
    "required",
    "supported",
    "used",
    "written",
];

/// The words that name a visibility.
const VISIBILITY_WORDS: [&str; 6] = [
    "exported",
    "exposed",
    "pub",
    "public",
    "re-exported",
    "visible",
];

/// Version 1 of the visibility excuses.
const EXCUSES: [Phrase; 8] = [
    &[&VISIBILITY_WORDS, &["only"], &["so", "for", "because"]],
    &[&["only"], &VISIBILITY_WORDS, &["so", "for", "because"]],
    &[&VISIBILITY_WORDS, &["so"], &["that"]],
    &[
        &VISIBILITY_WORDS,
        &["for"],
        &["test", "tests", "testing", "benches"],
    ],
    &[&["only"], &["for"], &["test", "tests", "testing"]],
    &[&["for"], &["internal"], &["use"]],
    &[&["internal"], &["use"], &["only"]],
    &[
        &["not"],
        &["part"],
        &["of"],
        &["the"],
        &["public"],
        &["api", "surface"],
    ],
];

/// Version 2 of the phrases that call an error impossible. A word such as
/// `invariant` names the condition an error reports, not its absence, so
/// it is none of them.
const IMPOSSIBLE: [Phrase; 6] = [
    &[&["cannot", "can't"], &["occur", "happen"]],
    &[
        &["should", "must", "will"],
        &["never"],
        &["occur", "happen"],
    ],
    &[&["never"], &["occurs", "happens"]],
    &[&["should"], &["not"], &["occur", "happen"]],
    &[&["unreachable"]],
    &[&["impossible"]],
];

/// Version 1 of the words that give a field's state a condition.
const CONDITIONS: [Phrase; 4] = [
    &[&["when"]],
    &[&["only"]],
    &[&["unless"]],
    &[&["empty"], &["for"]],
];

/// The most tokens between a condition and a sibling field's name written
/// as a plain word, as in "Local kind only".
const SIBLING_REACH: usize = 3;

/// Version 1 of the container claims: sorted, unique, FIFO, and keyed.
const CLAIMS: [(&str, &[Phrase]); 4] = [
    (
        "sorted",
        &[
            &[&["sorted", "ascending", "descending"]],
            &[&["ordered"], &["by"]],
        ],
    ),
    (
        "unique",
        &[
            &[&["unique", "deduplicated"]],
            &[&["no"], &["duplicates"]],
            &[&["at"], &["most"], &["once"]],
            &[&["each"], &["once"]],
        ],
    ),
    (
        "FIFO",
        &[&[&["fifo"]], &[&["first"], &["in"], &["first"], &["out"]]],
    ),
    ("keyed", &[&[&["keyed", "indexed"], &["by", "on"]]]),
];

/// `history-words`: a doc that says a phrase of [`HISTORY`].
fn history_words(run: &AnalysisRun<'_>, doc: &Doc<'_>, out: &mut Vec<Lead>) {
    if let Some(found) = doc.prose.first(&HISTORY) {
        let lead = Lead::new("history-words", Mode::DirectSyntax, doc.prose.said(found));
        out.push(doc.add(run, lead));
    }
}

/// `visibility-excuse`: the doc of a node that is not private says a
/// phrase of [`EXCUSES`].
fn visibility_excuse(run: &AnalysisRun<'_>, doc: &Doc<'_>, out: &mut Vec<Lead>) {
    if doc.vis == Vis::Private {
        return;
    }
    if let Some(found) = doc.prose.first(&EXCUSES) {
        let evidence = format!(
            "{} on a `{}` node",
            doc.prose.said(found),
            vis_word(doc.vis)
        );
        let lead = Lead::new("visibility-excuse", Mode::DirectSyntax, evidence);
        out.push(doc.add(run, lead));
    }
}

/// Returns how a visibility reads.
fn vis_word(vis: Vis) -> &'static str {
    match vis {
        Vis::Public => "pub",
        Vis::Internal => "pub(crate)",
        Vis::Private => "private",
    }
}

/// `impossible-error`: a function that returns a `Result` and says that
/// its error cannot occur, by a phrase of [`IMPOSSIBLE`] in its doc's
/// `# Errors` section, or else in a comment of its body when the body
/// builds no `Err` and matches no `Err` arm; a comment beside an `Err` is
/// `infallible-comment-in-fallible-body`'s. The doc of an error type or
/// variant describes a failure, not a function's contract, and is left
/// out.
fn impossible_error(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    let mut documented: BTreeMap<usize, &Doc<'_>> = BTreeMap::new();
    for doc in &docs.all {
        if let Node::Func(f) = doc.node {
            documented.entry(f).or_insert(doc);
        }
    }
    let first = |prose: &Prose<'_>, errors_only: bool| {
        (IMPOSSIBLE.iter())
            .flat_map(|phrase| prose.matches(phrase))
            .filter(|m| !errors_only || prose.section(m.start) == Some("errors"))
            .min_by_key(|m| (m.start, m.end))
            .map(|found| prose.said(found))
    };
    for (f, func) in docs.catalog.funcs.iter().enumerate() {
        if !func.ret.as_ref().is_some_and(Ty::is_result) {
            continue;
        }
        let lead = |evidence| Lead::new("impossible-error", Mode::DirectSyntax, evidence);
        if let Some(said) = documented.get(&f).and_then(|doc| first(&doc.prose, true)) {
            out.push(lead(said).item(run, func.item));
            continue;
        }
        let t = func.item.target;
        let Some(body) = run.body_of(func.item).map(|b| &run.krates[t].bodies[b]) else {
            continue;
        };
        if handles_err(&body.facts) {
            continue;
        }
        for comment in &body.facts.comments {
            let lines: Vec<&str> = comment.text.iter().map(|line| prose_line(line)).collect();
            let text = lines.join(" ");
            if let Some(said) = first(&read_except(&text, Vec::new()), false) {
                let at = Loc {
                    file: body.loc.file,
                    line: comment.line,
                };
                let found = lead(format!("{said} in a body comment"));
                out.push(found.item(run, func.item).at(t, at));
                break;
            }
        }
    }
}

/// `conditional-field`: an `Option`, `bool`, or collection field whose doc
/// says a phrase of [`CONDITIONS`] in a sentence that names a sibling
/// field, in a code span or as a word within [`SIBLING_REACH`] tokens of
/// the phrase, so that the field's state hangs on another field's. A
/// condition on outside input alone, such as "`None` when the user gave
/// none", is how an optional value is documented. Returns the sibling and
/// the evidence; the fields of one owner that hang on one sibling make one
/// lead.
fn conditional_field(docs: &Docs<'_>, f: usize, doc: &Doc<'_>) -> Option<(String, String)> {
    let field = &docs.catalog.fields[f];
    let held = field.ty.deref();
    if held.option().is_none() && !held.is_bool() && !held.is_collection() {
        return None;
    }
    let siblings: Vec<&str> = (docs.catalog.fields_of(field.decl, field.variant))
        .filter(|other| other.named && other.name != field.name)
        .map(|other| other.name.as_str())
        .collect();
    let prose = &doc.prose;
    let sibling = |found: &Range<usize>| {
        let sentence = prose.toks[found.start].sentence;
        let near = found.start.saturating_sub(SIBLING_REACH)..found.end + SIBLING_REACH;
        (prose.toks.iter().enumerate())
            .filter(|(_, tok)| tok.sentence == sentence)
            .find_map(|(i, tok)| {
                let mut names = match tok.kind {
                    Kind::Word if near.contains(&i) => vec![tok.text.as_str()],
                    Kind::Code => identifiers(&tok.text).collect(),
                    Kind::Word | Kind::Punct => Vec::new(),
                };
                names.retain(|name| siblings.contains(name));
                names.first().copied()
            })
    };
    let found = (CONDITIONS.iter())
        .flat_map(|phrase| prose.matches(phrase))
        .filter_map(|found| Some((sibling(&found)?, found)))
        .min_by_key(|(_, found)| (found.start, found.end));
    let (name, found) = found?;
    let evidence = format!(
        "{}: {}, which names `{name}`",
        field.ty.typed(&field.name),
        prose.said(found)
    );
    Some((name.to_owned(), evidence))
}

/// `container-invariant-claim`: a collection field whose doc claims an
/// order, uniqueness, FIFO, or keys that its type does not keep by itself.
fn container_invariant_claim(
    run: &AnalysisRun<'_>,
    docs: &Docs<'_>,
    f: usize,
    doc: &Doc<'_>,
    out: &mut Vec<Lead>,
) {
    let field = &docs.catalog.fields[f];
    let Some(kept) = keeps(&field.ty) else {
        return;
    };
    let mut claimed = Vec::new();
    for ((claim, phrases), kept) in CLAIMS.iter().zip(kept) {
        if let Some(found) = doc.prose.first(phrases).filter(|_| !kept) {
            claimed.push(format!("{claim} (\"{}\")", doc.prose.words(found)));
        }
    }
    if !claimed.is_empty() {
        let evidence = format!(
            "{} claims {}",
            field.ty.typed(&field.name),
            claimed.join(", ")
        );
        let lead = Lead::new("container-invariant-claim", Mode::DirectSyntax, evidence);
        out.push(doc.add(run, lead));
    }
}

/// Returns, for a collection type, which of the claims of [`CLAIMS`] it
/// keeps by itself; `None` for a type that is no collection.
fn keeps(ty: &Ty) -> Option<[bool; 4]> {
    let core = ty.core();
    if !core.is_collection() {
        return None;
    }
    let head = core
        .generic()
        .map_or_else(|| "Vec".to_owned(), |(head, _)| leaf(head));
    Some(match head.as_str() {
        "BTreeMap" => [true, true, false, true],
        "HashMap" | "IndexMap" => [false, true, false, true],
        "BTreeSet" => [true, true, false, false],
        "HashSet" | "IndexSet" => [false, true, false, false],
        "BinaryHeap" => [true, false, false, false],
        "VecDeque" => [false, false, true, false],
        _ => [false; 4],
    })
}
