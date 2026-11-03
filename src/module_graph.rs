//! The module graph of each target: which of its modules use items that
//! other modules of it declare, from the observed uses of its production
//! code, and the same graph turned around, which modules use each one.

use std::collections::{BTreeMap, BTreeSet};

use crate::access::encloses;
use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::leads_items::impl_home;
use crate::model::ModId;

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
