//! The module graph of each target: which of its modules use items that
//! other modules of it declare, from the observed uses of its production
//! code, and the same graph turned around, which modules use each one.
//! Beside it, the crate graph of the run: which targets use the items of
//! which others, each use counted for the item's own target or for a
//! dependency that re-exports the item by name.

use std::collections::{BTreeMap, BTreeSet};

use crate::access::encloses;
use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef, impl_home};
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::model::{ItemKind, ModId};

/// The items one module uses from another.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ModEdge {
    /// How many distinct items the uses name.
    pub(crate) items: usize,
    /// Whether a resolved use backs the edge, not only qualified paths of
    /// the census.
    pub(crate) resolved: bool,
}

/// For each target and module, the other modules whose items it uses. A
/// module and one inside it never share an edge.
pub(crate) type Graph = Vec<Vec<BTreeMap<ModId, ModEdge>>>;

/// Builds each target's module graph from the run's observed uses: the
/// uses in production code of the item's own target, doc links left out.
pub(crate) fn build(run: &AnalysisRun<'_>) -> Graph {
    type Used = (BTreeSet<ItemRef>, bool);
    let mut edges: Vec<BTreeMap<(ModId, ModId), Used>> = vec![BTreeMap::new(); run.krates.len()];
    for u in &run.uses {
        let t = u.item.target;
        let inside = run.census.sources[u.source].target == Some(t);
        if u.kind == UseKind::Doc || u.context != Context::Production || !inside {
            continue;
        }
        let Some(from) = u.module else {
            continue;
        };
        let to = home(run, u.item);
        let krate = &run.krates[t];
        if encloses(krate, from, to) || encloses(krate, to, from) {
            continue;
        }
        let entry = edges[t].entry((from, to)).or_default();
        entry.0.insert(u.item.item);
        entry.1 |= !matches!(u.kind, UseKind::Qualified(_));
    }
    (edges.into_iter().zip(run.krates))
        .map(|(edges, krate)| {
            let mut out = vec![BTreeMap::new(); krate.modules.len()];
            for ((from, to), (used, resolved)) in edges {
                let edge = ModEdge {
                    items: used.len(),
                    resolved,
                };
                out[from].insert(to, edge);
            }
            out
        })
        .collect()
}

/// Returns the module that declares an item: a module's own, a member's
/// or a part's declaration's, and an impl member's impl's home.
fn home(run: &AnalysisRun<'_>, item: ItemId) -> ModId {
    let krate = &run.krates[item.target];
    match item.item {
        ItemRef::Module(m) => m,
        ItemRef::Decl(d) | ItemRef::Own(d, _) | ItemRef::Variant(d, _) => krate.decls[d].module,
        ItemRef::Value(at) => at.module,
        ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _) => impl_home(run, item.target, i),
    }
}

/// Returns one target's module graph turned around: for each module, the
/// modules that use its items, each with the edge from it.
pub(crate) fn used_by(graph: &[BTreeMap<ModId, ModEdge>]) -> Vec<BTreeMap<ModId, ModEdge>> {
    let mut out = vec![BTreeMap::new(); graph.len()];
    for (from, edges) in graph.iter().enumerate() {
        for (&to, &edge) in edges {
            out[to].insert(from, edge);
        }
    }
    out
}

/// Builds the run's crate graph from its observed uses: for each target,
/// the other targets whose items its production code uses, each with how
/// many distinct items, doc links left out. A use counts for the item's
/// own target when the using target depends on it, or else for the first
/// dependency in target order that re-exports the item by name, directly
/// or through other re-exports, and for the item's own target when none
/// does.
pub(crate) fn crates(run: &AnalysisRun<'_>) -> Vec<BTreeMap<usize, usize>> {
    let reexporters = reexporters(run);
    let mut used: Vec<BTreeMap<usize, BTreeSet<ItemId>>> = vec![BTreeMap::new(); run.krates.len()];
    for u in &run.uses {
        let Some(from) = run.census.sources[u.source].target else {
            continue;
        };
        if from == u.item.target || u.kind == UseKind::Doc || u.context != Context::Production {
            continue;
        }
        let to = credited(run, &reexporters, from, u.item);
        used[from].entry(to).or_default().insert(u.item);
    }
    (used.into_iter())
        .map(|to| to.into_iter().map(|(t, items)| (t, items.len())).collect())
        .collect()
}

/// Returns the target that a use in target `s` of an item of another
/// target counts for.
fn credited(
    run: &AnalysisRun<'_>,
    reexporters: &BTreeMap<ItemId, BTreeSet<usize>>,
    s: usize,
    item: ItemId,
) -> usize {
    let depends = |t: usize| run.analysis_deps[s].values().any(|&d| d == t);
    if depends(item.target) {
        return item.target;
    }
    (reexporters.get(&owner(run, item)).into_iter().flatten())
        .copied()
        .find(|&r| depends(r))
        .unwrap_or(item.target)
}

/// Returns the item whose re-export brings an item along: the declaration
/// of a member that has one, and any other item itself.
fn owner(run: &AnalysisRun<'_>, item: ItemId) -> ItemId {
    let decl = |d| ItemId::decl(item.target, d);
    match item.item {
        ItemRef::Own(d, _) | ItemRef::Variant(d, _) => decl(d),
        ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _) => {
            run.attached[item.target][i].map_or(item, decl)
        }
        ItemRef::Module(_) | ItemRef::Decl(_) | ItemRef::Value(_) => item,
    }
}

/// Returns, for each item that a re-export reaches, directly or through
/// other re-exports, the targets whose modules hold such a re-export.
fn reexporters(run: &AnalysisRun<'_>) -> BTreeMap<ItemId, BTreeSet<usize>> {
    let mut out: BTreeMap<ItemId, BTreeSet<usize>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for item in krate.modules.iter().flat_map(|module| &module.items) {
            let ItemKind::Reexport { text, .. } = &item.kind else {
                continue;
            };
            for found in text.refs().filter_map(|r| run.item_of_ref(t, r)) {
                out.entry(found).or_default().insert(t);
            }
        }
    }
    out
}
