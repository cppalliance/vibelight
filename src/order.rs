//! Orders declarations by how they name each other: a type follows its only
//! user directly, and a type with several users follows the last of them.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::layout::{Block, Span};
use crate::model::{
    BodyOwner, Crate, DeclId, Foreign, ItemKind, MemberKind, ModId, Seg, Shape, Sig, Text,
    VariantFields, path_order,
};
use crate::order_graph::{Graph, first_by_path, place, reach, strongly_connected};

/// The document order of the declarations.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Order {
    /// Every connected production declaration in dependency order: the
    /// roots by reach, each followed by the types it owns.
    pub(crate) sequence: Vec<DeclId>,
    /// Declarations that name nothing and that nothing names, in path order.
    pub(crate) unconnected: Vec<DeclId>,
    /// Test-support declarations, which leave the sequence, in path order.
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
    let named: Vec<Vec<DeclId>> = mentions
        .iter()
        .enumerate()
        .map(|(d, targets)| {
            declared_mentions(krate, d)
                .into_iter()
                .chain(targets.iter().copied())
                .filter(|t| out[d].contains(t))
                .collect()
        })
        .collect();
    let reaches: Vec<usize> = groups.iter().map(|members| reach(members, &out)).collect();
    let graph = Graph {
        groups: &groups,
        group_of: &group_of,
        mentions: &named,
    };
    let root_cmp = |a: usize, b: usize| {
        reaches[b].cmp(&reaches[a]).then_with(|| {
            path_cmp(
                first_by_path(&groups[a], &path_cmp),
                first_by_path(&groups[b], &path_cmp),
            )
        })
    };
    Order {
        sequence: place(&graph, &root_cmp, &path_cmp),
        unconnected,
        test_support,
        namers: used_by,
    }
}
/// Lists the declarations that the fields, the variants, the alias target,
/// or the supertraits of `d` name, in declared order. The parts of a type
/// follow it in this order, ahead of the types its signatures name, which
/// the printed block would mention first when its methods print before its
/// private fields.
fn declared_mentions(krate: &Crate, d: DeclId) -> Vec<DeclId> {
    let decl = &krate.decls[d];
    let mut texts: Vec<&Text> = decl
        .members
        .iter()
        .filter_map(|m| match &m.kind {
            MemberKind::Field(ty) => Some(ty),
            _ => None,
        })
        .collect();
    match &decl.shape {
        Shape::Tuple(params) => texts.extend(params.iter().map(|p| &p.text)),
        Shape::Enum(variants) => {
            for variant in variants {
                match &variant.fields {
                    VariantFields::Unit => {}
                    VariantFields::Tuple(params) | VariantFields::Named(params) => {
                        texts.extend(params.iter().map(|p| &p.text));
                    }
                }
            }
        }
        Shape::Alias(text) => texts.push(text),
        Shape::Trait { supertraits, .. } => texts.extend(supertraits.iter().map(|b| &b.text)),
        Shape::Named | Shape::Unit => {}
    }
    texts
        .into_iter()
        .flat_map(Text::refs)
        .filter_map(|r| {
            let reference = &krate.refs[r];
            reference.target.filter(|&t| t != d && !reference.comment)
        })
        .collect()
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
    /// sequence.
    out: Vec<BTreeSet<DeclId>>,
    /// The declarations that name something or that something names.
    connected: Vec<DeclId>,
    unconnected: Vec<DeclId>,
    /// Test-support declarations, which leave the sequence. The names they
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
    for body in &krate.body_names {
        let Some(t) = krate.refs[body.name].target else {
            continue;
        };
        let owned = match body.owner {
            BodyOwner::Module => false,
            BodyOwner::Impl(i) => {
                let header = &krate.impls[i];
                header
                    .self_text
                    .refs()
                    .chain(header.trait_.iter().flat_map(|base| base.text.refs()))
                    .any(|r| krate.refs[r].target == Some(t))
            }
            BodyOwner::Trait(d) => d == t,
        };
        named[t] |= !owned;
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
/// declaration, so it never owns the type or delays its place. Only the
/// items' own text counts, not their comments.
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
    texts.extend(sig.where_clause.iter().flat_map(|clause| clause.0.iter()));
    texts
}
