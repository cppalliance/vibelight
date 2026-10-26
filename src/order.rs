//! Orders declarations by how they name each other (rules 23 to 30).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::layout::{Block, Span};
use crate::model::{Crate, DeclId, Foreign, ItemKind, ModId, Seg, Sig, Text, path_order};
/// The document order of the declarations.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Order {
    /// Level 1 first, each level in its final order.
    pub(crate) levels: Vec<Vec<DeclId>>,
    /// Declarations that name nothing and that nothing names, in path order.
    pub(crate) unconnected: Vec<DeclId>,
    /// Test-support declarations, which leave the levels, in path order.
    pub(crate) test_support: Vec<DeclId>,
    /// What names each declaration.
    pub(crate) namers: Vec<Namers>,
}

/// What names one declaration: the declarations whose blocks name it and
/// the modules whose items name it, each in path order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Namers {
    pub(crate) decls: Vec<DeclId>,
    pub(crate) modules: Vec<ModId>,
}

/// Compares declarations in path order (rule 30); declarations that share
/// a path compare by their rendered text.
pub(crate) fn path_cmp(krate: &Crate, texts: &[String], a: DeclId, b: DeclId) -> Ordering {
    path_order(&krate.decl_path(a), &krate.decl_path(b))
        .then_with(|| texts[a].cmp(&texts[b]))
        .then(a.cmp(&b))
}

/// Returns each block's printed text, for ordering declarations that share
/// a path.
pub(crate) fn block_texts(krate: &Crate, blocks: &[Block]) -> Vec<String> {
    blocks
        .iter()
        .map(|block| {
            block
                .lines
                .iter()
                .map(|line| crate::layout::line_text(line, &krate.refs))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect()
}

/// Orders the declarations from the blocks laid out for them.
pub(crate) fn order(krate: &Crate, blocks: &[Block]) -> Order {
    let texts = block_texts(krate, blocks);
    let texts = texts.as_slice();
    let path_cmp = |a: DeclId, b: DeclId| path_cmp(krate, texts, a, b);
    let n = krate.decls.len();
    let mentions = block_mentions(krate, blocks);
    let by_modules = module_mentions(krate);
    let Split {
        out,
        connected,
        mut unconnected,
        mut test_support,
    } = split(krate, &mentions, &by_modules);
    test_support.sort_by(|&a, &b| path_cmp(a, b));
    unconnected.sort_by(|&a, &b| path_cmp(a, b));
    let used_by = namers(krate, &mentions, by_modules, &path_cmp);

    let groups = strongly_connected(n, &out, &connected);
    let mut group_of = vec![usize::MAX; n];
    for (g, members) in groups.iter().enumerate() {
        for &d in members {
            group_of[d] = g;
        }
    }
    let mut namers: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); groups.len()];
    for &d in &connected {
        for &t in &out[d] {
            if group_of[t] != group_of[d] {
                namers[group_of[t]].insert(group_of[d]);
            }
        }
    }
    let level = levels(&namers);
    let max_level = level.iter().copied().max().unwrap_or(0);

    let mut placed: Vec<DeclId> = Vec::new();
    let mut result = Vec::new();
    for current in 1..=max_level {
        let mut at_level: Vec<usize> = (0..groups.len()).filter(|&g| level[g] == current).collect();
        if current == 1 {
            let reach: BTreeMap<usize, usize> = at_level
                .iter()
                .map(|&g| (g, reach(&groups[g], &out)))
                .collect();
            at_level.sort_by(|&a, &b| {
                reach[&b].cmp(&reach[&a]).then_with(|| {
                    path_cmp(
                        first_by_path(&groups[a], &path_cmp),
                        first_by_path(&groups[b], &path_cmp),
                    )
                })
            });
        } else {
            let seen = first_seen(&placed, &mentions);
            let position = |g: usize| {
                groups[g]
                    .iter()
                    .filter_map(|d| seen.get(d))
                    .min()
                    .copied()
                    .unwrap_or(usize::MAX)
            };
            at_level.sort_by(|&a, &b| {
                position(a).cmp(&position(b)).then_with(|| {
                    path_cmp(
                        first_by_path(&groups[a], &path_cmp),
                        first_by_path(&groups[b], &path_cmp),
                    )
                })
            });
        }
        let mut row = Vec::new();
        for g in at_level {
            let mut remaining: Vec<DeclId> = groups[g].clone();
            while !remaining.is_empty() {
                let seen = first_seen(&placed, &mentions);
                remaining.sort_by(|&a, &b| match (seen.get(&a), seen.get(&b)) {
                    (Some(x), Some(y)) => x.cmp(y),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => path_cmp(a, b),
                });
                let next = remaining.remove(0);
                placed.push(next);
                row.push(next);
            }
        }
        result.push(row);
    }
    Order {
        levels: result,
        unconnected,
        test_support,
        namers: used_by,
    }
}

/// Lists what names each declaration, each list in path order.
fn namers(
    krate: &Crate,
    mentions: &[Vec<DeclId>],
    by_modules: Vec<Vec<ModId>>,
    path_cmp: &impl Fn(DeclId, DeclId) -> Ordering,
) -> Vec<Namers> {
    let mut out: Vec<Namers> = by_modules
        .into_iter()
        .map(|mut modules| {
            modules.sort_by(|&a, &b| {
                path_order(&krate.modules[a].path, &krate.modules[b].path).then(a.cmp(&b))
            });
            Namers {
                decls: Vec::new(),
                modules,
            }
        })
        .collect();
    for (d, targets) in mentions.iter().enumerate() {
        for &t in targets {
            if !out[t].decls.contains(&d) {
                out[t].decls.push(d);
            }
        }
    }
    for namers in &mut out {
        namers.decls.sort_by(|&a, &b| path_cmp(a, b));
    }
    out
}

/// The declarations sorted into the three places they print, before
/// ordering.
struct Split {
    /// The naming edges among the declarations that take part in the
    /// levels.
    out: Vec<BTreeSet<DeclId>>,
    /// The declarations that name something or that something names.
    connected: Vec<DeclId>,
    unconnected: Vec<DeclId>,
    /// Test-support declarations, which leave the levels. The names they
    /// use still keep a type out of Unconnected.
    test_support: Vec<DeclId>,
}

fn split(krate: &Crate, mentions: &[Vec<DeclId>], by_modules: &[Vec<ModId>]) -> Split {
    let n = krate.decls.len();
    let support = |d: DeclId| krate.decls[d].test_support;
    let mut named = vec![false; n];
    for targets in mentions {
        for &t in targets {
            named[t] = true;
        }
    }
    for (d, modules) in by_modules.iter().enumerate() {
        named[d] |= !modules.is_empty();
    }
    let out = mentions
        .iter()
        .enumerate()
        .map(|(d, m)| {
            m.iter()
                .copied()
                .filter(|&t| !support(d) && !support(t))
                .collect()
        })
        .collect();
    let in_use = |d: DeclId| !mentions[d].is_empty() || named[d];
    Split {
        out,
        connected: (0..n).filter(|&d| !support(d) && in_use(d)).collect(),
        unconnected: (0..n).filter(|&d| !support(d) && !in_use(d)).collect(),
        test_support: (0..n).filter(|&d| support(d)).collect(),
    }
}

/// Lists the declarations each block names (rule 24), in the order it
/// names them.
fn block_mentions(krate: &Crate, blocks: &[Block]) -> Vec<Vec<DeclId>> {
    blocks
        .iter()
        .enumerate()
        .map(|(own, block)| {
            block
                .lines
                .iter()
                .flat_map(|line| &line.spans)
                .filter_map(|span| match span {
                    Span::Ref(id) => krate.refs[*id]
                        .target
                        .filter(|&t| t != own && !krate.refs[*id].comment),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

/// Lists, for each declaration, the modules whose items name it: a free
/// function, a constant, a static, an extern item, or a re-export. Such a
/// type is in use, so it is never Unconnected, but a module is not a
/// declaration, so it gives no level. Only the items' own text counts, not
/// their comments.
pub(crate) fn module_mentions(krate: &Crate) -> Vec<Vec<ModId>> {
    let mut out: Vec<Vec<ModId>> = vec![Vec::new(); krate.decls.len()];
    for (m, module) in krate.modules.iter().enumerate() {
        for item in &module.items {
            for text in item_texts(&item.kind) {
                for seg in &text.0 {
                    if let Seg::Ref(id) = seg {
                        if let Some(t) = krate.refs[*id].target {
                            if !out[t].contains(&m) {
                                out[t].push(m);
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Returns the texts of a module item that can name types: a function's
/// signature, a constant's or static's type, and a re-export's path.
pub(crate) fn item_texts(kind: &ItemKind) -> Vec<&Text> {
    match kind {
        ItemKind::Fn(sig)
        | ItemKind::Foreign {
            item: Foreign::Fn(sig),
            ..
        } => sig_texts(sig),
        ItemKind::Const { ty, .. }
        | ItemKind::Static { ty, .. }
        | ItemKind::Foreign {
            item: Foreign::Static { ty, .. },
            ..
        } => vec![ty],
        ItemKind::Reexport { text, .. } => vec![text],
        ItemKind::Foreign { .. } | ItemKind::Mark(_) | ItemKind::Unsupported(_) => Vec::new(),
    }
}

fn sig_texts(sig: &Sig) -> Vec<&Text> {
    let mut texts = vec![&sig.generics];
    texts.extend(sig.params.iter().map(|p| &p.text));
    texts.extend(sig.ret.iter());
    texts.extend(sig.where_clause.iter());
    texts
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

fn first_by_path(members: &[DeclId], cmp: &impl Fn(DeclId, DeclId) -> Ordering) -> DeclId {
    members
        .iter()
        .copied()
        .min_by(|&a, &b| cmp(a, b))
        .unwrap_or(0)
}

/// Counts the declarations a group reaches through naming, its own members
/// left out.
fn reach(members: &[DeclId], out: &[BTreeSet<DeclId>]) -> usize {
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

/// Gives each group a level: 1 when nothing names it, otherwise one more
/// than the highest level of any group that names it.
fn levels(namers: &[BTreeSet<usize>]) -> Vec<usize> {
    fn visit(g: usize, namers: &[BTreeSet<usize>], level: &mut [usize]) -> usize {
        if level[g] > 0 {
            return level[g];
        }
        let value = 1 + namers[g]
            .iter()
            .map(|&h| visit(h, namers, level))
            .max()
            .unwrap_or(0);
        level[g] = value;
        value
    }
    let mut level = vec![0usize; namers.len()];
    for g in 0..namers.len() {
        visit(g, namers, &mut level);
    }
    level
}

/// Finds the strongly connected components among `nodes` (Tarjan).
fn strongly_connected(n: usize, out: &[BTreeSet<DeclId>], nodes: &[DeclId]) -> Vec<Vec<DeclId>> {
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

    fn graph(edges: &[(usize, usize)], n: usize) -> Vec<BTreeSet<usize>> {
        let mut out = vec![BTreeSet::new(); n];
        for &(a, b) in edges {
            out[a].insert(b);
        }
        out
    }

    #[test]
    fn diamond_puts_the_shared_type_below_both_namers() {
        let out = graph(&[(0, 1), (0, 2), (1, 2)], 3);
        let groups = strongly_connected(3, &out, &[0, 1, 2]);
        assert_eq!(groups.len(), 3);
        let mut group_of = [0; 3];
        for (g, m) in groups.iter().enumerate() {
            group_of[m[0]] = g;
        }
        let mut namers = vec![BTreeSet::new(); 3];
        for (a, targets) in out.iter().enumerate() {
            for &b in targets {
                namers[group_of[b]].insert(group_of[a]);
            }
        }
        let level = levels(&namers);
        assert_eq!(
            [level[group_of[0]], level[group_of[1]], level[group_of[2]]],
            [1, 2, 3]
        );
    }

    #[test]
    fn cycles_form_one_group_and_reach_skips_members() {
        let out = graph(&[(0, 1), (1, 0), (1, 2)], 3);
        let groups = strongly_connected(3, &out, &[0, 1, 2]);
        assert!(groups.contains(&vec![0, 1]));
        assert_eq!(reach(&[0, 1], &out), 1);
    }
}
