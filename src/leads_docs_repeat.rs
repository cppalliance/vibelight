//! The doc detectors that compare a doc's words: with the names of what it
//! documents, and with the words of every other doc of the run, through
//! an inverted index of word shingles rather than a comparison of every
//! pair.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_docs::{Doc, Docs, Node, value_type};
use crate::leads_docs_text::{Kind, Prose, identifiers, quote};
use crate::leads_docs_words::{is_stopword, singular};
use crate::leads_types::{Tok, Ty, leaf};
use crate::leads_words::words;
use crate::model::Shape;

/// How many normalized words one shingle of `repeated-doc-text` holds.
const SHINGLE_WORDS: usize = 8;

/// The fewest content words a doc has for `doc-echo`.
const ECHO_MIN_TERMS: usize = 3;

/// The offset and prime of 64-bit FNV-1a, which hashes shingles.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Finds the leads of the doc text that one doc compares with names or
/// with other docs.
pub(crate) fn find(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    for doc in &docs.all {
        doc_echo(run, docs, doc, out);
    }
    repeated_doc_text(run, docs, out);
}

/// Returns the content words of a doc for `doc-echo`: each word, and each
/// identifier of a code span, as its lowercase singular words, without
/// stopwords.
fn echo_terms(prose: &Prose<'_>) -> Vec<String> {
    let mut out = Vec::new();
    for tok in &prose.toks {
        let parts: Vec<&str> = match tok.kind {
            Kind::Word if is_stopword(&tok.text) => continue,
            Kind::Word => vec![&prose.doc[tok.span.clone()]],
            Kind::Code => identifiers(&tok.text).collect(),
            Kind::Punct => continue,
        };
        for word in parts.into_iter().flat_map(words) {
            if !is_stopword(&word) {
                out.push(singular(&word));
            }
        }
    }
    out
}

/// Adds the lowercase singular words of a name.
fn add_name(vocab: &mut BTreeSet<String>, name: &str) {
    for word in identifiers(name).flat_map(words) {
        vocab.insert(singular(&word));
    }
}

/// Adds the words of each name and keyword in a type.
fn add_ty(vocab: &mut BTreeSet<String>, ty: &Ty) {
    for tok in &ty.0 {
        match tok {
            Tok::Name(name) => add_name(vocab, &leaf(name)),
            Tok::Code(code) => add_name(vocab, code),
        }
    }
}

/// Returns the words a doc may echo: those of its node's path, signature,
/// fields, variants, and types.
fn vocabulary(run: &AnalysisRun<'_>, docs: &Docs<'_>, doc: &Doc<'_>) -> BTreeSet<String> {
    let mut vocab = BTreeSet::new();
    for segment in doc.path(run) {
        add_name(&mut vocab, &segment);
    }
    let t = doc.item.target;
    if doc.item.item == ItemRef::Module(0) {
        add_name(&mut vocab, &run.targets[t].crate_name);
    }
    let catalog = &docs.catalog;
    let fields = |decl: ItemId, variant: Option<usize>, vocab: &mut BTreeSet<String>| {
        for field in catalog.fields_of(decl, variant) {
            add_name(vocab, &field.name);
            add_ty(vocab, &field.ty);
        }
    };
    match doc.node {
        Node::Func(f) => {
            let func = &catalog.funcs[f];
            if func.has_receiver() {
                vocab.insert("self".to_owned());
            }
            for arg in &func.args {
                add_name(&mut vocab, arg.name.as_deref().unwrap_or_default());
                add_ty(&mut vocab, &arg.ty);
            }
            if let Some(ret) = &func.ret {
                add_ty(&mut vocab, ret);
            }
            add_ty(&mut vocab, &Ty::of(run, t, &func.sig.generics, None));
        }
        Node::Field(f) => add_ty(&mut vocab, &catalog.fields[f].ty),
        Node::Decl(d) => {
            let decl = &run.krates[t].decls[d];
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            fields(at, None, &mut vocab);
            match &decl.shape {
                Shape::Enum(variants) => {
                    for (v, variant) in variants.iter().enumerate() {
                        add_name(&mut vocab, &variant.name);
                        fields(at, Some(v), &mut vocab);
                    }
                }
                Shape::Alias(target) => add_ty(&mut vocab, &Ty::of(run, t, target, None)),
                _ => {}
            }
            for member in &decl.members {
                add_name(&mut vocab, &member.name);
            }
        }
        Node::Variant(d, v) => {
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            fields(at, Some(v), &mut vocab);
        }
        Node::Value => {
            if let Some(ty) = value_type(run, doc.item) {
                add_ty(&mut vocab, &Ty::of(run, t, ty, None));
            }
        }
        Node::Module => {}
    }
    vocab
}

/// `doc-echo`: a doc of prose alone, with no code block, code span, or
/// link, and at least [`ECHO_MIN_TERMS`] content words, every one of them
/// a word of its node's path, signature, fields, or types. A code span or
/// a link names something beside the node, such as the value a field
/// holds.
fn doc_echo(run: &AnalysisRun<'_>, docs: &Docs<'_>, doc: &Doc<'_>, out: &mut Vec<Lead>) {
    let coded = doc.prose.toks.iter().any(|tok| tok.kind == Kind::Code);
    if coded || !doc.prose.blocks.is_empty() || !doc.prose.links.is_empty() {
        return;
    }
    let terms = echo_terms(&doc.prose);
    if terms.len() < ECHO_MIN_TERMS {
        return;
    }
    let vocab = vocabulary(run, docs, doc);
    if terms.iter().all(|term| vocab.contains(term)) {
        let evidence = format!(
            "every content word of \"{}\" is in its name, signature, or types",
            quote(&doc.prose.kept())
        );
        out.push(doc.add(run, Lead::new("doc-echo", Mode::DirectSyntax, evidence)));
    }
}

/// Returns the normalized words of a doc for shingles: each word, and
/// each identifier of a code span, in lowercase without stopwords, with
/// the index of the token it comes from.
fn shingle_terms(prose: &Prose<'_>) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, tok) in prose.toks.iter().enumerate() {
        match tok.kind {
            Kind::Word if !is_stopword(&tok.text) => out.push((tok.text.clone(), i)),
            Kind::Code => {
                for ident in identifiers(&tok.text) {
                    let lower = ident.to_lowercase();
                    if !is_stopword(&lower) {
                        out.push((lower, i));
                    }
                }
            }
            Kind::Word | Kind::Punct => {}
        }
    }
    out
}

/// Hashes the words of one shingle with FNV-1a 64, a separator after each.
fn hash(window: &[(String, usize)]) -> u64 {
    let mut h = FNV_OFFSET;
    for (word, _) in window {
        for b in word.bytes().chain([0xff]) {
            h ^= u64::from(b);
            h = h.wrapping_mul(FNV_PRIME);
        }
    }
    h
}

/// Returns the root of a doc's set, flattening the path to it.
fn root(roots: &mut [usize], d: usize) -> usize {
    let mut r = d;
    while roots[r] != r {
        r = roots[r];
    }
    let mut at = d;
    while roots[at] != r {
        let next = roots[at];
        roots[at] = r;
        at = next;
    }
    r
}

/// `repeated-doc-text`: one normalized shingle of [`SHINGLE_WORDS`] words
/// in the docs of two or more nodes; the docs of one path and kind of node,
/// such as one function under two `cfg`s, are one site. Docs linked by
/// shared shingles form one lead with every site; its evidence is the
/// longest passage that the shingle shared by the most sites extends to.
fn repeated_doc_text(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    let terms: Vec<Vec<(String, usize)>> = (docs.all.iter())
        .map(|doc| shingle_terms(&doc.prose))
        .collect();
    let sites: Vec<(usize, Vec<String>, u8)> = (docs.all.iter())
        .map(|doc| (doc.item.target, doc.path(run), doc.node.rank()))
        .collect();
    let mut index: BTreeMap<u64, Vec<(usize, usize)>> = BTreeMap::new();
    for (d, list) in terms.iter().enumerate() {
        for (i, window) in list.windows(SHINGLE_WORDS).enumerate() {
            let places = index.entry(hash(window)).or_default();
            if places.last().is_none_or(|&(last, _)| last != d) {
                places.push((d, i));
            }
        }
    }
    index.retain(|_, places| places.len() > 1);
    let same = |a: (usize, usize), b: (usize, usize)| {
        let words = |(d, i): (usize, usize)| terms[d][i..i + SHINGLE_WORDS].iter().map(|t| &t.0);
        words(a).eq(words(b))
    };
    let mut roots: Vec<usize> = (0..docs.all.len()).collect();
    let mut shared: Vec<(usize, Vec<(usize, usize)>)> = Vec::new();
    for places in index.values() {
        let places: Vec<(usize, usize)> = (places.iter().copied())
            .filter(|&p| same(p, places[0]))
            .collect();
        let distinct: BTreeSet<&(usize, Vec<String>, u8)> =
            places.iter().map(|&(d, _)| &sites[d]).collect();
        if distinct.len() < 2 {
            continue;
        }
        for &(d, _) in &places[1..] {
            let (a, b) = (root(&mut roots, places[0].0), root(&mut roots, d));
            roots[a.max(b)] = a.min(b);
        }
        shared.push((distinct.len(), places));
    }
    let mut best: BTreeMap<usize, (usize, Vec<(usize, usize)>)> = BTreeMap::new();
    for (count, places) in shared {
        let r = root(&mut roots, places[0].0);
        let wider = (best.get(&r))
            .is_none_or(|(most, at)| count > *most || (count == *most && places[0] < at[0]));
        if wider {
            best.insert(r, (count, places));
        }
    }
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for d in 0..docs.all.len() {
        members.entry(root(&mut roots, d)).or_default().push(d);
    }
    for (r, (count, places)) in best {
        let evidence = format!("\"{}\" in {count} docs", passage(docs, &terms, &places));
        let mut lead = Lead::new("repeated-doc-text", Mode::DirectSyntax, evidence);
        for &d in &members[&r] {
            lead = docs.all[d].add(run, lead);
        }
        out.push(lead);
    }
}

/// Returns the passage that one shingle at each of `places` extends to
/// while the words before and after it stay the same at every place, as
/// the first place writes it.
fn passage(docs: &Docs<'_>, terms: &[Vec<(String, usize)>], places: &[(usize, usize)]) -> String {
    let mut starts: Vec<(usize, usize)> = places.to_vec();
    let mut len = SHINGLE_WORDS;
    let agree = |starts: &[(usize, usize)], step: &dyn Fn(usize) -> Option<usize>| {
        let mut found =
            (starts.iter()).map(|&(d, i)| step(i).and_then(|k| terms[d].get(k)).map(|t| &t.0));
        let first = found.next().flatten();
        first.is_some() && found.all(|word| word == first)
    };
    while agree(&starts, &|i| i.checked_sub(1)) {
        for start in &mut starts {
            start.1 -= 1;
        }
        len += 1;
    }
    while agree(&starts, &|i| Some(i + len)) {
        len += 1;
    }
    let (d, i) = starts[0];
    let prose = &docs.all[d].prose;
    let first = &prose.toks[terms[d][i].1];
    let last = &prose.toks[terms[d][i + len - 1].1];
    quote(&prose.doc[first.span.start..last.span.end])
}
