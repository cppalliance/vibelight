//! The detector of cloned bodies: function bodies whose sets of canonical
//! token shingles mostly agree. An inverted index holds each body's rarest
//! shingles, enough of them that two bodies over the threshold share one,
//! so candidates come from shared shingles and no pass compares every body
//! with every other. A shingle that very many bodies hold is boilerplate
//! and finds no candidate.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::leads::{Lead, Mode};
use crate::model_body::BodyKind;

/// The least overlap of two bodies' shingle sets, as a percentage of their
/// union. Two sets over it have sizes within this percentage of each other,
/// and two compared bodies must have token counts within it too.
const CLONE_PERCENT: usize = 80;

/// The fewest distinct shingles of a compared body. A body that repeats
/// one short pattern, such as a table of `match` arms, has few, and any
/// two such bodies overlap.
const CLONE_MIN_SHINGLES: usize = 32;

/// The most bodies that one shingle may be in and still find candidates.
const COMMON_SHINGLE: usize = 64;

/// One compared body.
struct Doc<'a> {
    item: ItemId,
    shingles: &'a [u64],
    tokens: usize,
}

/// One clone group: its bodies, and the least and the most overlap, in
/// percent, of the pairs that join them.
struct Group {
    docs: Vec<usize>,
    low: usize,
    high: usize,
}

/// `body-clone`: function bodies of [`CLONE_MIN_SHINGLES`] or more distinct
/// shingles whose shingle sets overlap by [`CLONE_PERCENT`] or more of
/// their union, joined into groups through the pairs that do.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let docs = docs(run);
    let pairs = pairs(&docs);
    for group in groups(docs.len(), &pairs) {
        let overlap = if group.low == group.high {
            format!("{}%", group.low)
        } else {
            format!("{}% to {}%", group.low, group.high)
        };
        let evidence = format!(
            "{} function bodies, joined by token-shingle overlaps of {overlap}",
            group.docs.len()
        );
        let mut lead = Lead::new("body-clone", Mode::DirectSyntax, evidence);
        for d in group.docs {
            lead = lead.item(run, docs[d].item);
        }
        out.push(lead);
    }
}

/// Returns the compared bodies: each function body of an item with
/// [`CLONE_MIN_SHINGLES`] or more distinct shingles, in target and body
/// order.
fn docs<'a>(run: &AnalysisRun<'a>) -> Vec<Doc<'a>> {
    let mut out = Vec::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (b, body) in krate.bodies.iter().enumerate() {
            let function = matches!(
                body.kind,
                BodyKind::Fn
                    | BodyKind::Method
                    | BodyKind::TraitImplMethod
                    | BodyKind::TraitDefault
            );
            let print = &body.facts.fingerprint;
            let long = print.shingles.len() >= CLONE_MIN_SHINGLES;
            if let (true, Some(item)) = (function && long, run.body_items[t][b]) {
                out.push(Doc {
                    item: ItemId { target: t, item },
                    shingles: &print.shingles,
                    tokens: print.tokens,
                });
            }
        }
    }
    out
}

/// Tells whether two sizes are within [`CLONE_PERCENT`] of each other.
fn close(a: usize, b: usize) -> bool {
    100 * a.min(b) >= CLONE_PERCENT * a.max(b)
}

/// Returns how many of a set's rarest shingles to index: one more than a
/// set of `n` may lack and still overlap another by [`CLONE_PERCENT`].
fn prefix_len(n: usize) -> usize {
    n + 1 - (CLONE_PERCENT * n).div_ceil(100)
}

/// Counts the elements that two sorted sets share.
fn shared(left: &[u64], right: &[u64]) -> usize {
    let (mut l, mut r, mut both) = (0, 0, 0);
    while let (Some(x), Some(y)) = (left.get(l), right.get(r)) {
        match x.cmp(y) {
            Ordering::Less => l += 1,
            Ordering::Greater => r += 1,
            Ordering::Equal => {
                both += 1;
                l += 1;
                r += 1;
            }
        }
    }
    both
}

/// Returns the entries of a list sorted by key whose key is `key`.
fn keyed<T: Copy>(list: &[(u64, T)], key: u64) -> &[(u64, T)] {
    let start = list.partition_point(|&(k, _)| k < key);
    let end = list.partition_point(|&(k, _)| k <= key);
    &list[start..end]
}

/// Returns each pair of compared bodies whose shingle sets overlap by
/// [`CLONE_PERCENT`] or more, with that overlap in percent, first body
/// first. Each body's rarest shingles, its prefix, go in the index, and a
/// body's candidates are the later bodies of close size whose prefixes
/// share a shingle with its own.
fn pairs(docs: &[Doc<'_>]) -> Vec<(usize, usize, usize)> {
    let mut every: Vec<u64> = docs
        .iter()
        .flat_map(|d| d.shingles.iter().copied())
        .collect();
    every.sort_unstable();
    let mut bodies: Vec<(u64, usize)> = Vec::new();
    for s in every {
        match bodies.last_mut() {
            Some((last, n)) if *last == s => *n += 1,
            _ => bodies.push((s, 1)),
        }
    }
    let prefixes: Vec<Vec<u64>> = (docs.iter())
        .map(|doc| {
            let mut rare: Vec<(usize, u64)> = (doc.shingles.iter())
                .map(|&s| (keyed(&bodies, s).first().map_or(0, |&(_, n)| n), s))
                .collect();
            rare.sort_unstable();
            rare.truncate(prefix_len(doc.shingles.len()));
            (rare.into_iter())
                .filter(|&(n, _)| n <= COMMON_SHINGLE)
                .map(|(_, s)| s)
                .collect()
        })
        .collect();
    let mut index: Vec<(u64, usize)> = (prefixes.iter().enumerate())
        .flat_map(|(d, prefix)| prefix.iter().map(move |&s| (s, d)))
        .collect();
    index.sort_unstable();
    let mut out = Vec::new();
    let mut candidates = Vec::new();
    for (a, prefix) in prefixes.iter().enumerate() {
        let doc = &docs[a];
        candidates.clear();
        for &s in prefix {
            let later = (keyed(&index, s).iter())
                .map(|&(_, d)| d)
                .filter(|&d| d > a);
            candidates.extend(later.filter(|&b| {
                close(doc.shingles.len(), docs[b].shingles.len())
                    && close(doc.tokens, docs[b].tokens)
            }));
        }
        candidates.sort_unstable();
        candidates.dedup();
        for &b in &candidates {
            let both = shared(doc.shingles, docs[b].shingles);
            let union = doc.shingles.len() + docs[b].shingles.len() - both;
            if 100 * both >= CLONE_PERCENT * union {
                out.push((a, b, 100 * both / union));
            }
        }
    }
    out
}

/// Returns the root of a body's group, halving the path to it.
fn root(parent: &mut [usize], mut d: usize) -> usize {
    while parent[d] != d {
        parent[d] = parent[parent[d]];
        d = parent[d];
    }
    d
}

/// Joins `n` bodies into groups through the pairs, each group by its first
/// body, its bodies ascending.
fn groups(n: usize, pairs: &[(usize, usize, usize)]) -> Vec<Group> {
    let mut parent: Vec<usize> = (0..n).collect();
    for &(a, b, _) in pairs {
        let (x, y) = (root(&mut parent, a), root(&mut parent, b));
        parent[x.max(y)] = x.min(y);
    }
    let mut groups: BTreeMap<usize, Group> = BTreeMap::new();
    for &(a, _, percent) in pairs {
        let group = groups.entry(root(&mut parent, a)).or_insert(Group {
            docs: Vec::new(),
            low: percent,
            high: percent,
        });
        group.low = group.low.min(percent);
        group.high = group.high.max(percent);
    }
    for d in 0..n {
        let top = root(&mut parent, d);
        if let Some(group) = groups.get_mut(&top) {
            group.docs.push(d);
        }
    }
    groups.into_values().collect()
}
