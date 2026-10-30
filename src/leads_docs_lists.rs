//! The doc detectors that compare what a doc lists or counts with the
//! declaration: a list that names some of an enum's variants but not all,
//! and a written count of variants or fields that differs from the
//! declared one.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::leads::{Lead, Mode};
use crate::leads_docs::{Doc, Docs, Node};
use crate::leads_docs_refs::intra_path;
use crate::leads_docs_text::{Kind, Phrase, Prose};
use crate::leads_types::Ty;
use crate::leads_words::words;
use crate::model::{DeclId, Shape};
use crate::typerefs::strip_raw;

/// The fewest variants an enum has for `partial-doc-list`.
const MIN_VARIANTS: usize = 3;

/// The fewest variants one sentence names to be a list, after a word of
/// [`COMPLETE_CUES`].
const MIN_LISTED: usize = 2;

/// The fewest variants one sentence names to be a list without such a
/// word.
const MIN_LISTED_BARE: usize = 3;

/// Version 1 of the phrases that make a sentence's names a whole list.
const COMPLETE_CUES: [Phrase; 3] = [
    &[&["either", "only"]],
    &[&["one", "any", "each", "all"], &["of"]],
    &[&["is", "are"], &["one"]],
];

/// Version 1 of the words that make a sentence's names examples or
/// exceptions rather than a list.
const EXAMPLE_CUES: [&str; 12] = [
    "besides",
    "e.g",
    "eg",
    "example",
    "except",
    "including",
    "includes",
    "instance",
    "like",
    "other",
    "such",
    "unlike",
];

/// The number words a count may be written as, from two.
const NUMBER_WORDS: [&str; 19] = [
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
];

/// The largest count written in digits that a count claim reads.
const MAX_COUNT: usize = 99;

/// Version 1 of the nouns that count an enum's variants.
const VARIANT_NOUNS: [&str; 17] = [
    "cases",
    "categories",
    "classes",
    "flavors",
    "flavours",
    "forms",
    "kinds",
    "levels",
    "modes",
    "options",
    "outcomes",
    "shapes",
    "sorts",
    "states",
    "types",
    "values",
    "variants",
];

/// Version 1 of the nouns that count a struct's fields.
const FIELD_NOUNS: [&str; 4] = ["components", "fields", "members", "parts"];

/// The words that may stand between a count and its noun.
const COUNT_ADJECTIVES: [&str; 6] = [
    "different",
    "distinct",
    "known",
    "possible",
    "supported",
    "valid",
];

/// The most words that stand between a count and its noun.
const COUNT_GAP: usize = 2;

/// The words before a number that make it no exact count.
const HEDGES: [&str; 13] = [
    "about", "around", "first", "last", "least", "most", "nearly", "next", "other", "over", "than",
    "to", "under",
];

/// Finds the leads of lists and counts.
pub(crate) fn find(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    for doc in &docs.all {
        partial_doc_list(run, docs, doc, out);
        if let Node::Decl(d) = doc.node {
            doc_count_mismatch(run, docs, d, doc, out);
        }
    }
}

/// Tells whether sentence `s` holds a word of [`EXAMPLE_CUES`].
fn cued(prose: &Prose<'_>, s: usize) -> bool {
    (prose.toks.iter())
        .any(|t| t.sentence == s && t.kind == Kind::Word && EXAMPLE_CUES.contains(&t.text.as_str()))
}

/// `partial-doc-list`: a sentence of an enum's own doc, or of the doc of a
/// field whose type names the enum, that lists its variants: [`MIN_LISTED`]
/// or more after a word of [`COMPLETE_CUES`], or [`MIN_LISTED_BARE`] or
/// more, with no example or exception cue, when the doc names at least
/// half of the enum's variants but not all. A code span names a variant
/// as `Enum::Variant`, as `Self::Variant` in the enum's own doc, or bare.
/// A function's doc that names some variants is left out, since it most
/// often names the ones that the function returns or handles.
fn partial_doc_list(run: &AnalysisRun<'_>, docs: &Docs<'_>, doc: &Doc<'_>, out: &mut Vec<Lead>) {
    let t = doc.item.target;
    let krate = &run.krates[t];
    let variants = |d: DeclId| match &krate.decls[d].shape {
        Shape::Enum(variants) if variants.len() >= MIN_VARIANTS => Some(variants),
        _ => None,
    };
    let mut context: Vec<DeclId> = Vec::new();
    let add = |ty: &Ty, context: &mut Vec<DeclId>| {
        for name in ty.names() {
            let found = docs
                .decl(name)
                .filter(|&(at, d)| at == t && variants(d).is_some());
            if let Some((_, d)) = found.filter(|(_, d)| !context.contains(d)) {
                context.push(d);
            }
        }
    };
    let owner = match doc.node {
        Node::Decl(d) => Some(d).filter(|&d| variants(d).is_some()),
        Node::Field(f) => {
            add(&docs.catalog.fields[f].ty, &mut context);
            None
        }
        Node::Module | Node::Variant(..) | Node::Func(_) | Node::Value => return,
    };
    context.extend(owner);
    let mut mentions: BTreeMap<DeclId, BTreeMap<usize, BTreeSet<usize>>> = BTreeMap::new();
    for tok in doc.prose.toks.iter().filter(|t| t.kind == Kind::Code) {
        let Some(path) = intra_path(&tok.text) else {
            continue;
        };
        let name = &path[path.len() - 1];
        let candidates: Vec<DeclId> = match path.len().checked_sub(2).map(|k| path[k].as_str()) {
            Some("Self") => owner.into_iter().collect(),
            Some(enum_name) => (0..krate.decls.len())
                .filter(|&d| strip_raw(&krate.decls[d].name) == enum_name && variants(d).is_some())
                .collect(),
            None => context.clone(),
        };
        for d in candidates {
            let listed =
                variants(d).and_then(|vs| vs.iter().position(|v| strip_raw(&v.name) == *name));
            if let Some(v) = listed {
                let sentences = mentions.entry(d).or_default();
                sentences.entry(tok.sentence).or_default().insert(v);
            }
        }
    }
    for (d, sentences) in mentions {
        let Some(all) = variants(d) else {
            continue;
        };
        let seen: BTreeSet<usize> = sentences.values().flatten().copied().collect();
        let listed = sentences.iter().find(|&(&s, set)| {
            let whole = (COMPLETE_CUES.iter()).any(|cue| {
                (doc.prose.matches(cue).iter()).any(|m| doc.prose.toks[m.start].sentence == s)
            });
            set.len() >= MIN_LISTED
                && (set.len() >= MIN_LISTED_BARE || whole)
                && !cued(&doc.prose, s)
        });
        let most = seen.len() * 2 >= all.len();
        let Some((&s, _)) = listed.filter(|_| most && seen.len() < all.len()) else {
            continue;
        };
        let shown = |pick: &dyn Fn(usize) -> bool| -> String {
            let names = (0..all.len()).filter(|&v| pick(v));
            let names: Vec<String> = names.map(|v| format!("`{}`", all[v].name)).collect();
            names.join(", ")
        };
        let evidence = format!(
            "\"{}\" names {} of the {} variants of `{}`, leaving out {}",
            doc.prose.sentence(s),
            shown(&|v| seen.contains(&v)),
            all.len(),
            krate.decls[d].name,
            shown(&|v| !seen.contains(&v))
        );
        let at = ItemId::decl(t, d);
        let lead = Lead::new("partial-doc-list", Mode::DirectSyntax, evidence).item(run, at);
        out.push(doc.add(run, lead));
    }
}

/// Returns the count a word writes: a number word from two, or digits
/// from 2 to [`MAX_COUNT`].
fn count_of(word: &str) -> Option<usize> {
    let spelled = NUMBER_WORDS.iter().position(|w| *w == word).map(|p| p + 2);
    spelled.or_else(|| (word.parse::<usize>().ok()).filter(|n| (2..=MAX_COUNT).contains(n)))
}

/// `doc-count-mismatch`: an enum's or a struct's own doc that writes a
/// count before a noun that counts its variants or fields, such as "the
/// three kinds", which differs from the declared count. Adjectives such
/// as "possible" and the words of the type's name may stand between, and
/// a count after a hedge such as "at least" is no claim.
fn doc_count_mismatch(
    run: &AnalysisRun<'_>,
    docs: &Docs<'_>,
    d: DeclId,
    doc: &Doc<'_>,
    out: &mut Vec<Lead>,
) {
    let t = doc.item.target;
    let decl = &run.krates[t].decls[d];
    let name: Vec<String> = words(&strip_raw(&decl.name));
    let at = ItemId::decl(t, d);
    let owned = |list: &[&str]| -> Vec<String> { list.iter().map(|n| (*n).to_owned()).collect() };
    let (nouns, count, what) = match &decl.shape {
        Shape::Enum(variants) => {
            let mut nouns = owned(&VARIANT_NOUNS);
            nouns.extend(name.last().map(|head| format!("{head}s")));
            (nouns, variants.len(), "variants")
        }
        Shape::Named | Shape::Tuple(_) => {
            let fields = docs.catalog.fields_of(at, None).count();
            (owned(&FIELD_NOUNS), fields, "fields")
        }
        Shape::Unit | Shape::Trait { .. } | Shape::Alias(_) => return,
    };
    let toks = &doc.prose.toks;
    for (i, tok) in toks.iter().enumerate() {
        let Some(n) = (tok.kind == Kind::Word)
            .then(|| count_of(&tok.text))
            .flatten()
        else {
            continue;
        };
        let word =
            |k: usize| (toks.get(k)).filter(|t| t.kind == Kind::Word && t.sentence == tok.sentence);
        let hedged =
            (i.checked_sub(1).and_then(word)).is_some_and(|t| HEDGES.contains(&t.text.as_str()));
        let mut k = i + 1;
        while k - i <= COUNT_GAP
            && word(k).is_some_and(|t| {
                COUNT_ADJECTIVES.contains(&t.text.as_str()) || name.contains(&t.text)
            })
        {
            k += 1;
        }
        if hedged || n == count || word(k).is_none_or(|t| !nouns.contains(&t.text)) {
            continue;
        }
        let said = doc.prose.words(i..k + 1);
        let evidence = format!(
            "\"{said}\" in \"{}\", but it declares {count} {what}",
            doc.prose.sentence(tok.sentence)
        );
        out.push(doc.add(
            run,
            Lead::new("doc-count-mismatch", Mode::DirectSyntax, evidence),
        ));
        return;
    }
}
