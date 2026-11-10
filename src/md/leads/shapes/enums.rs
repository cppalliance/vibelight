//! The detectors of enums whose variant names overlap: an enum whose every
//! variant name is in enums with more variants, and two enums that differ
//! in a few names. This file also holds the grouping of the run's enums by
//! their names that `mirror-enum` reads.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::md::leads::{Lead, Mode};
use crate::model::Shape;
use crate::typerefs::strip_raw;

/// The fewest variant names of the smaller enum for `contained-enum`, and
/// of the names that two enums share for `near-mirror-enum`.
const SHARED_FROM: usize = 3;

/// The most names that only one of two enums has for `near-mirror-enum`,
/// whatever their size.
const APART: usize = 2;

/// For `near-mirror-enum`, the names that only one of two enums has may
/// also number up to one in this many of all their names.
const APART_ONE_IN: usize = 5;

/// The most names that evidence lists for what an enum adds; more are
/// counted.
const LISTED: usize = 3;

/// A set of variant names, and the enums of the run that have it, in run
/// order.
struct NameSet {
    names: BTreeSet<String>,
    enums: Vec<ItemId>,
}

/// Finds the leads of enums whose variant names overlap.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let mut merged: BTreeMap<BTreeSet<String>, Vec<ItemId>> = BTreeMap::new();
    for (names, enums) in name_sets(run) {
        let names: BTreeSet<String> = names.into_iter().collect();
        if names.len() >= SHARED_FROM {
            merged.entry(names).or_default().extend(enums);
        }
    }
    let sets: Vec<NameSet> = (merged.into_iter())
        .map(|(names, mut enums)| {
            enums.sort();
            NameSet { names, enums }
        })
        .collect();
    let mut index: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (s, set) in sets.iter().enumerate() {
        for name in &set.names {
            index.entry(name.as_str()).or_default().push(s);
        }
    }
    let reported = contained_enum(run, &sets, &index, out);
    near_mirror_enum(run, &sets, &index, &reported, out);
}

/// Returns every enum of the run under its variant names, sorted, each
/// without its `r#`.
pub(crate) fn name_sets(run: &AnalysisRun<'_>) -> BTreeMap<Vec<String>, Vec<ItemId>> {
    let mut groups: BTreeMap<Vec<String>, Vec<ItemId>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let Shape::Enum(variants) = &decl.shape else {
                continue;
            };
            let mut names: Vec<String> = variants.iter().map(|v| strip_raw(&v.name)).collect();
            names.sort();
            groups.entry(names).or_default().push(ItemId::decl(t, d));
        }
    }
    groups
}

/// `contained-enum`: the enums of at least [`SHARED_FROM`] variants whose
/// every variant name is a variant name of enums with more variants
/// anywhere in the run, one lead per set of names with every enum of the
/// set and every enum that holds it. Returns the sets it reports.
fn contained_enum(
    run: &AnalysisRun<'_>,
    sets: &[NameSet],
    index: &BTreeMap<&str, Vec<usize>>,
    out: &mut Vec<Lead>,
) -> Vec<usize> {
    let mut reported = Vec::new();
    for (s, small) in sets.iter().enumerate() {
        let Some(first) = small.names.first() else {
            continue;
        };
        let mut bigs: Vec<(ItemId, &NameSet)> = Vec::new();
        for &b in &index[first.as_str()] {
            let big = &sets[b];
            if big.names.len() > small.names.len() && small.names.is_subset(&big.names) {
                bigs.extend(big.enums.iter().map(|&at| (at, big)));
            }
        }
        if bigs.is_empty() {
            continue;
        }
        bigs.sort_by_key(|&(at, _)| at);
        reported.push(s);
        let adds: Vec<String> = (bigs.iter())
            .map(|&(at, big)| adding(run, at, big, &small.names))
            .collect();
        let smalls: Vec<String> = (small.enums.iter())
            .map(|&at| format!("`{}`", run.path(at).join("::")))
            .collect();
        let evidence = format!(
            "variants {} of {} in {}",
            joined(&small.names),
            smalls.join(", "),
            adds.join(", ")
        );
        let lead = Lead::new("contained-enum", Mode::DirectSyntax, evidence);
        let lead = (small.enums.iter()).fold(lead, |lead, &at| lead.item(run, at));
        out.push((bigs.into_iter()).fold(lead, |lead, (at, _)| lead.item(run, at)));
    }
    reported
}

/// `near-mirror-enum`: two enums anywhere in the run that share at least
/// [`SHARED_FROM`] variant names, neither inside the other, where the
/// names only one of them has number at most [`APART`], or at most one in
/// [`APART_ONE_IN`] of all their names, one lead per pair of sets with
/// every enum of either. A pair whose shared names hold a set that
/// `contained-enum` reports is left to that lead.
fn near_mirror_enum(
    run: &AnalysisRun<'_>,
    sets: &[NameSet],
    index: &BTreeMap<&str, Vec<usize>>,
    reported: &[usize],
    out: &mut Vec<Lead>,
) {
    for (i, one) in sets.iter().enumerate() {
        let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
        for name in &one.names {
            for &j in index[name.as_str()].iter().filter(|&&j| j > i) {
                *counts.entry(j).or_default() += 1;
            }
        }
        for (j, count) in counts {
            let other = &sets[j];
            let all = one.names.len() + other.names.len() - count;
            let apart = all - count;
            let nested = count == one.names.len().min(other.names.len());
            if count < SHARED_FROM || nested || (apart > APART && apart * APART_ONE_IN > all) {
                continue;
            }
            let shared: BTreeSet<String> = one.names.intersection(&other.names).cloned().collect();
            if reported.iter().any(|&k| sets[k].names.is_subset(&shared)) {
                continue;
            }
            let mut both: Vec<(ItemId, &NameSet)> = (one.enums.iter().map(|&at| (at, one)))
                .chain(other.enums.iter().map(|&at| (at, other)))
                .collect();
            both.sort_by_key(|&(at, _)| at);
            let adds: Vec<String> = (both.iter())
                .map(|&(at, set)| adding(run, at, set, &shared))
                .collect();
            let evidence = format!("variants {} in {}", joined(&shared), adds.join(", "));
            let lead = Lead::new("near-mirror-enum", Mode::DirectSyntax, evidence);
            out.push((both.into_iter()).fold(lead, |lead, (at, _)| lead.item(run, at)));
        }
    }
}

/// Returns names joined by `, `.
fn joined(names: &BTreeSet<String>) -> String {
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    names.join(", ")
}

/// Returns an enum of a set by its path below its crate's root, and the
/// names of the set beyond `shared`: up to [`LISTED`] of them, or how
/// many.
fn adding(run: &AnalysisRun<'_>, at: ItemId, set: &NameSet, shared: &BTreeSet<String>) -> String {
    let extra: Vec<&str> = set.names.difference(shared).map(String::as_str).collect();
    let adds = if extra.len() > LISTED {
        format!("{} more", extra.len())
    } else {
        extra.join(", ")
    };
    format!("`{}` (adds {adds})", run.path(at).join("::"))
}
