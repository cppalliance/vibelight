//! The detector of keys shared across owned types: the collections that a
//! struct or union and the types its fields own, directly or through
//! others, key by one type, found once on the outermost owner.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_containers::Keyed;
use crate::leads_types::{Ty, leaf};
use crate::model::{DeclId, DeclKind, MemberKind, Shape, Text};

/// The fewest collections of one key for `shared-key`.
const SHARED_FROM: usize = 2;

/// The keyed collection fields of each struct and union.
type KeyedFields<'c, 'a> = BTreeMap<ItemId, Vec<Keyed<'c, 'a>>>;

/// `shared-key`: [`SHARED_FROM`] or more collections keyed by one type,
/// held by a struct or union and the types it owns, not all by itself;
/// keys that only one type holds are `parallel-collections`. A type owns
/// each struct, union, and alias of its target that a field's type names,
/// unless that type borrows or shares, and what those own in turn; an
/// enum's variants are alternatives, so an enum owns nothing. Each key is
/// found once on each outermost owner: a type that no other owner of the
/// same collections owns, or the first of types that own each other.
pub(crate) fn find(run: &AnalysisRun<'_>, keyed: &KeyedFields<'_, '_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        let edges = owns(run, t);
        let reach: Vec<BTreeSet<DeclId>> =
            (0..krate.decls.len()).map(|d| reached(&edges, d)).collect();
        let at = |d: DeclId| ItemId {
            target: t,
            item: ItemRef::Decl(d),
        };
        let held = |d: DeclId| keyed.get(&at(d)).map_or(&[][..], Vec::as_slice);
        let mut owners: BTreeMap<&str, BTreeSet<DeclId>> = BTreeMap::new();
        let mut groups: BTreeMap<(DeclId, &str), Vec<&Keyed<'_, '_>>> = BTreeMap::new();
        for (d, decl) in krate.decls.iter().enumerate() {
            if !is_owner(decl.kind) || reach[d].is_empty() {
                continue;
            }
            let mut by_key: BTreeMap<&str, Vec<&Keyed<'_, '_>>> = BTreeMap::new();
            let inner = reach[d].iter().copied().filter(|&e| e != d);
            for e in iter::once(d).chain(inner) {
                for field in held(e) {
                    by_key.entry(field.key.as_str()).or_default().push(field);
                }
            }
            for (key, group) in by_key {
                let elsewhere = group.iter().any(|k| k.field.decl != at(d));
                if group.len() >= SHARED_FROM && elsewhere {
                    owners.entry(key).or_default().insert(d);
                    groups.insert((d, key), group);
                }
            }
        }
        for (key, set) in &owners {
            for &d in set {
                let inside = |e: DeclId| {
                    e != d && reach[e].contains(&d) && (!reach[d].contains(&e) || e < d)
                };
                if set.iter().any(|&e| inside(e)) {
                    continue;
                }
                let Some(group) = groups.get(&(d, *key)) else {
                    continue;
                };
                out.push(lead(run, at(d), group));
            }
        }
    }
}

/// Builds the lead of one owner and the collections it holds of one key.
fn lead(run: &AnalysisRun<'_>, owner: ItemId, group: &[&Keyed<'_, '_>]) -> Lead {
    let shown: Vec<String> = (group.iter())
        .map(|k| {
            let holder = run.name(k.field.decl).unwrap_or_default();
            format!("`{holder}::{}: {}`", k.field.name, k.field.ty.shown())
        })
        .collect();
    let evidence = format!(
        "`{}` holds `{}` in {}",
        run.name(owner).unwrap_or_default(),
        group[0].shown,
        shown.join(", ")
    );
    let mut lead = Lead::new("shared-key", Mode::DirectSyntax, evidence).item(run, owner);
    for keyed in group {
        lead = keyed.field.add(run, lead);
    }
    lead
}

/// Tells whether a declaration of this kind can own a collection: a
/// struct or a union.
fn is_owner(kind: DeclKind) -> bool {
    matches!(kind, DeclKind::Struct | DeclKind::Union)
}

/// The pointers that share what they point at rather than own it, by the
/// last segment of their path.
const SHARED: [&str; 3] = ["Arc", "Rc", "Weak"];

/// Tells whether a type borrows or shares part of what it names: it holds
/// a reference, a raw pointer, or one of [`SHARED`] from outside the run.
fn shares(ty: &Ty) -> bool {
    let pointer = (ty.0.iter()).any(|tok| matches!(tok.code(), Some("&" | "*")));
    let shared = |name: &str| !name.starts_with('#') && SHARED.contains(&leaf(name).as_str());
    pointer || ty.names().any(shared)
}

/// Returns, for each declaration of target `t`, the structs, unions, and
/// aliases of `t` that its fields' types name, or that an alias's target
/// names; itself left out. A type that [`shares`] owns nothing, so a view
/// or a handle is no owner.
fn owns(run: &AnalysisRun<'_>, t: usize) -> Vec<BTreeSet<DeclId>> {
    let krate = &run.krates[t];
    let mut out = vec![BTreeSet::new(); krate.decls.len()];
    for (d, decl) in krate.decls.iter().enumerate() {
        let texts: Vec<&Text> = match &decl.shape {
            Shape::Named if is_owner(decl.kind) => (decl.members.iter())
                .filter_map(|member| match &member.kind {
                    MemberKind::Field(ty) => Some(ty),
                    _ => None,
                })
                .collect(),
            Shape::Tuple(params) => params.iter().map(|param| &param.text).collect(),
            Shape::Alias(text) if decl.kind == DeclKind::Alias => vec![text],
            _ => Vec::new(),
        };
        let owned = texts
            .into_iter()
            .filter(|text| !shares(&Ty::of(run, t, text, None)));
        for r in owned.flat_map(Text::refs) {
            let Some((target, owned)) = run.decl_of_ref(t, r) else {
                continue;
            };
            if target != t || owned == d {
                continue;
            }
            let kind = krate.decls[owned].kind;
            if is_owner(kind) || kind == DeclKind::Alias {
                out[d].insert(owned);
            }
        }
    }
    out
}

/// Returns the declarations that `d` owns, directly or through others.
fn reached(edges: &[BTreeSet<DeclId>], d: DeclId) -> BTreeSet<DeclId> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<DeclId> = edges[d].iter().copied().collect();
    while let Some(next) = stack.pop() {
        if seen.insert(next) {
            stack.extend(edges[next].iter().copied().filter(|e| !seen.contains(e)));
        }
    }
    seen
}
