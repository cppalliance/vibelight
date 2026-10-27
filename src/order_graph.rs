//! The naming graph: its strongly connected groups, how far each group
//! reaches, and the dependency order in which the groups are placed.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::model::DeclId;

/// What placing the groups reads: the groups, the group of each
/// declaration, and the targets each declaration names, in the order its
/// block names them.
pub(crate) struct Graph<'a> {
    pub(crate) groups: &'a [Vec<DeclId>],
    pub(crate) group_of: &'a [usize],
    pub(crate) mentions: &'a [Vec<DeclId>],
}

impl Graph<'_> {
    /// Lists the other groups that `members` of group `g` name, each once,
    /// in the order they are first named.
    fn named_groups(&self, g: usize, members: &[DeclId]) -> Vec<usize> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for &d in members {
            for &t in &self.mentions[d] {
                let h = self.group_of[t];
                if h != g && h != usize::MAX && seen.insert(h) {
                    out.push(h);
                }
            }
        }
        out
    }
}

/// Places every group in dependency order and returns the declarations in
/// that order. A group that no other group names is a root, and the roots
/// go in `root_cmp` order. After a group is placed, each group it names
/// loses one unplaced namer, and a group with none left is placed at once,
/// followed by the groups it names in the same way. So a group with one
/// namer follows it directly, a group with several follows the last of
/// them, and every namer is above the groups it names.
pub(crate) fn place(
    graph: &Graph<'_>,
    root_cmp: &impl Fn(usize, usize) -> Ordering,
    path_cmp: &impl Fn(DeclId, DeclId) -> Ordering,
) -> Vec<DeclId> {
    let count = graph.groups.len();
    let mut left = vec![0usize; count];
    for (g, members) in graph.groups.iter().enumerate() {
        for h in graph.named_groups(g, members) {
            left[h] += 1;
        }
    }
    let mut roots: Vec<usize> = (0..count).filter(|&g| left[g] == 0).collect();
    roots.sort_by(|&a, &b| root_cmp(a, b));
    let mut placer = Placer {
        graph,
        path_cmp,
        left,
        placed: Vec::new(),
        emitted: vec![false; count],
    };
    for root in roots {
        placer.emit(root);
    }
    // Every group has a root above it, so nothing is left; a group that is
    // would still be placed, in path order, rather than lost.
    let mut rest: Vec<usize> = (0..count).filter(|&g| !placer.emitted[g]).collect();
    rest.sort_by(|&a, &b| {
        path_cmp(
            first_by_path(&graph.groups[a], path_cmp),
            first_by_path(&graph.groups[b], path_cmp),
        )
    });
    for g in rest {
        placer.emit(g);
    }
    placer.placed
}

/// The state of one placement.
struct Placer<'a, P: Fn(DeclId, DeclId) -> Ordering> {
    graph: &'a Graph<'a>,
    /// Settles the members of a group nothing has named yet.
    path_cmp: &'a P,
    /// How many namers of each group are not placed yet.
    left: Vec<usize>,
    placed: Vec<DeclId>,
    emitted: Vec<bool>,
}

impl<P: Fn(DeclId, DeclId) -> Ordering> Placer<'_, P> {
    fn emit(&mut self, g: usize) {
        if self.emitted[g] {
            return;
        }
        self.emitted[g] = true;
        let members = self.place_members(g);
        for h in self.graph.named_groups(g, &members) {
            self.left[h] = self.left[h].saturating_sub(1);
            if self.left[h] == 0 {
                self.emit(h);
            }
        }
    }

    /// Places the members of one group, each next one the one first named
    /// by the blocks placed so far, and returns them in that order.
    fn place_members(&mut self, g: usize) -> Vec<DeclId> {
        let mut remaining = self.graph.groups[g].clone();
        let mut members = Vec::new();
        while !remaining.is_empty() {
            let seen = first_seen(&self.placed, self.graph.mentions);
            remaining.sort_by(|&a, &b| match (seen.get(&a), seen.get(&b)) {
                (Some(x), Some(y)) => x.cmp(y),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => (self.path_cmp)(a, b),
            });
            let next = remaining.remove(0);
            self.placed.push(next);
            members.push(next);
        }
        members
    }
}

/// Returns the member that comes first in path order.
pub(crate) fn first_by_path(
    members: &[DeclId],
    cmp: &impl Fn(DeclId, DeclId) -> Ordering,
) -> DeclId {
    members
        .iter()
        .copied()
        .min_by(|&a, &b| cmp(a, b))
        .unwrap_or(0)
}

/// Maps each declaration to the position of its first mention when the
/// placed blocks are read top to bottom.
fn first_seen(placed: &[DeclId], mentions: &[Vec<DeclId>]) -> BTreeMap<DeclId, usize> {
    let mut seen = BTreeMap::new();
    let mut position = 0;
    for &d in placed {
        for &t in &mentions[d] {
            seen.entry(t).or_insert(position);
            position += 1;
        }
    }
    seen
}

/// Counts the declarations a group reaches through naming, its own members
/// left out.
pub(crate) fn reach(members: &[DeclId], out: &[BTreeSet<DeclId>]) -> usize {
    let own: BTreeSet<DeclId> = members.iter().copied().collect();
    let mut seen: BTreeSet<DeclId> = own.clone();
    let mut stack: Vec<DeclId> = members.to_vec();
    while let Some(d) = stack.pop() {
        for &t in &out[d] {
            if seen.insert(t) {
                stack.push(t);
            }
        }
    }
    seen.difference(&own).count()
}

/// Finds the strongly connected components among `nodes` (Tarjan).
pub(crate) fn strongly_connected(
    n: usize,
    out: &[BTreeSet<DeclId>],
    nodes: &[DeclId],
) -> Vec<Vec<DeclId>> {
    struct State {
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<usize>,
        next: usize,
        groups: Vec<Vec<usize>>,
    }
    fn connect(v: usize, out: &[BTreeSet<DeclId>], s: &mut State) {
        s.index[v] = Some(s.next);
        s.low[v] = s.next;
        s.next += 1;
        s.stack.push(v);
        s.on_stack[v] = true;
        for &w in &out[v] {
            match s.index[w] {
                None => {
                    connect(w, out, s);
                    s.low[v] = s.low[v].min(s.low[w]);
                }
                Some(index) if s.on_stack[w] => s.low[v] = s.low[v].min(index),
                Some(_) => {}
            }
        }
        if Some(s.low[v]) == s.index[v] {
            let mut group = Vec::new();
            while let Some(w) = s.stack.pop() {
                s.on_stack[w] = false;
                group.push(w);
                if w == v {
                    break;
                }
            }
            group.sort_unstable();
            s.groups.push(group);
        }
    }
    let mut state = State {
        index: vec![None; n],
        low: vec![0; n],
        on_stack: vec![false; n],
        stack: Vec::new(),
        next: 0,
        groups: Vec::new(),
    };
    for &v in nodes {
        if state.index[v].is_none() {
            connect(v, out, &mut state);
        }
    }
    state.groups
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Places `nodes` of a graph with `n` declarations and the given
    /// naming edges, which name their targets in the order listed. Roots go
    /// by reach, then by number, and declarations compare by number.
    fn sequence(n: usize, edges: &[(usize, usize)], nodes: &[usize]) -> Vec<usize> {
        sequence_by(n, edges, nodes, &|a, b| a.cmp(&b))
    }

    /// Like `sequence`, with declarations compared by `path_cmp`.
    fn sequence_by(
        n: usize,
        edges: &[(usize, usize)],
        nodes: &[usize],
        path_cmp: &impl Fn(usize, usize) -> Ordering,
    ) -> Vec<usize> {
        let mut mentions = vec![Vec::new(); n];
        let mut out = vec![BTreeSet::new(); n];
        for &(a, b) in edges {
            mentions[a].push(b);
            out[a].insert(b);
        }
        let groups = strongly_connected(n, &out, nodes);
        let mut group_of = vec![usize::MAX; n];
        for (g, members) in groups.iter().enumerate() {
            for &d in members {
                group_of[d] = g;
            }
        }
        let graph = Graph {
            groups: &groups,
            group_of: &group_of,
            mentions: &mentions,
        };
        let reaches: Vec<usize> = groups.iter().map(|m| reach(m, &out)).collect();
        let first = |g: usize| groups[g].iter().copied().min().unwrap();
        let root_cmp =
            |a: usize, b: usize| reaches[b].cmp(&reaches[a]).then(first(a).cmp(&first(b)));
        place(&graph, &root_cmp, path_cmp)
    }

    #[test]
    fn a_chain_goes_in_order() {
        assert_eq!(sequence(3, &[(0, 1), (1, 2)], &[0, 1, 2]), [0, 1, 2]);
    }

    #[test]
    fn a_type_with_one_user_follows_it_directly() {
        let edges = [(0, 1), (0, 2), (1, 3)];
        assert_eq!(sequence(4, &edges, &[0, 1, 2, 3]), [0, 1, 3, 2]);
    }

    #[test]
    fn a_type_shared_by_a_parent_and_its_child_follows_the_child() {
        let edges = [(0, 1), (0, 2), (1, 2)];
        assert_eq!(sequence(3, &edges, &[0, 1, 2]), [0, 1, 2]);
        let edges = [(0, 2), (0, 1), (1, 2)];
        assert_eq!(sequence(3, &edges, &[0, 1, 2]), [0, 1, 2]);
    }

    #[test]
    fn a_shared_type_follows_the_last_of_its_users_however_deep() {
        let edges = [(0, 1), (0, 3), (1, 2), (2, 3)];
        assert_eq!(sequence(4, &edges, &[0, 1, 2, 3]), [0, 1, 2, 3]);
    }

    #[test]
    fn a_diamond_puts_the_base_after_both_branches() {
        let edges = [(0, 1), (0, 2), (1, 3), (2, 3)];
        assert_eq!(sequence(4, &edges, &[0, 1, 2, 3]), [0, 1, 2, 3]);
    }

    #[test]
    fn roots_go_by_reach_then_by_number() {
        let edges = [(0, 1), (2, 3), (2, 4), (5, 6)];
        let nodes = [0, 1, 2, 3, 4, 5, 6];
        assert_eq!(sequence(7, &edges, &nodes), [2, 3, 4, 0, 1, 5, 6]);
    }

    #[test]
    fn siblings_follow_the_order_their_owner_names_them() {
        let edges = [(0, 2), (0, 1)];
        assert_eq!(sequence(3, &edges, &[0, 1, 2]), [0, 2, 1]);
    }

    #[test]
    fn a_cycle_is_placed_together_in_the_order_it_is_first_named() {
        let edges = [(0, 2), (2, 1), (1, 2)];
        assert_eq!(sequence(3, &edges, &[0, 1, 2]), [0, 2, 1]);
    }

    #[test]
    fn a_cycle_at_the_top_is_a_root_that_reaches_least() {
        let edges = [(0, 1), (1, 0), (2, 3)];
        assert_eq!(sequence(4, &edges, &[0, 1, 2, 3]), [2, 3, 0, 1]);
    }

    #[test]
    fn the_members_of_a_cycle_nothing_names_go_by_path_not_by_number() {
        let edges = [(0, 1), (1, 0)];
        let by_path = |a: usize, b: usize| b.cmp(&a);
        assert_eq!(sequence_by(2, &edges, &[0, 1], &by_path), [1, 0]);
        assert_eq!(sequence(2, &edges, &[0, 1]), [0, 1]);
    }

    #[test]
    fn cycles_form_one_group_and_reach_skips_members() {
        let mut out = vec![BTreeSet::new(); 3];
        for (a, b) in [(0, 1), (1, 0), (1, 2)] {
            out[a].insert(b);
        }
        let groups = strongly_connected(3, &out, &[0, 1, 2]);
        assert!(groups.contains(&vec![0, 1]));
        assert_eq!(reach(&[0, 1], &out), 1);
    }
}
