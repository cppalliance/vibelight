//! Tests the module graph turned around: each module's users, each with
//! the edge from it, so its item count and its resolved flag stay.

use std::collections::BTreeMap;

use crate::model::ModId;
use crate::module_graph::{ModEdge, used_by};

/// Returns the maps of `n` modules, where each `(m, other, items,
/// resolved)` puts an edge under `other` in module `m`'s map.
fn maps(n: usize, edges: &[(ModId, ModId, usize, bool)]) -> Vec<BTreeMap<ModId, ModEdge>> {
    let mut out = vec![BTreeMap::new(); n];
    for &(m, other, items, resolved) in edges {
        out[m].insert(other, ModEdge { items, resolved });
    }
    out
}

#[test]
fn each_edge_shows_under_the_module_it_uses_with_its_item_count() {
    let graph = maps(4, &[(1, 2, 3, true), (1, 3, 1, false), (2, 3, 2, true)]);
    let users = used_by(&graph);
    let turned = maps(4, &[(2, 1, 3, true), (3, 1, 1, false), (3, 2, 2, true)]);
    assert_eq!(users, turned);
    let fan_in: Vec<usize> = users.iter().map(BTreeMap::len).collect();
    let fan_out: Vec<usize> = graph.iter().map(BTreeMap::len).collect();
    assert_eq!((fan_in, fan_out), (vec![0, 0, 1, 2], vec![0, 2, 1, 0]));
}

#[test]
fn modules_that_use_each_other_are_each_others_users() {
    let graph = maps(3, &[(1, 2, 4, true), (2, 1, 1, false)]);
    let turned = maps(3, &[(1, 2, 1, false), (2, 1, 4, true)]);
    assert_eq!(used_by(&graph), turned);
}

#[test]
fn a_graph_with_no_edges_gives_no_module_a_user() {
    assert_eq!(used_by(&maps(3, &[])), maps(3, &[]));
    assert_eq!(used_by(&[]), maps(0, &[]));
}
