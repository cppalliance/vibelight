//! The uses observed for each item of a run: the references that resolve
//! to it, the body paths that reach it or name it as their prefix, and the
//! qualified paths of the scanned sources that name it and no other item
//! their source can reach. A qualified path is lexical evidence; the rest
//! is resolved.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{AnalysisRun, library_of};
use crate::analysis_items::{ItemId, ItemRef, is_value};
use crate::census::{Context, QualifiedPathHit, SourceId};
use crate::model::{DeclKind, ItemKind, Member, MemberKind, ModId, RefId, Shape, ValueRef};
use crate::typerefs::strip_raw;

/// Where an observed use comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum UseKind {
    /// A name in a declaration: a signature, a field, a base, a bound, an
    /// alias, or the header of an impl that belongs to no declaration.
    Signature,
    /// A type name that a function body writes.
    Body,
    /// A link in a doc or a comment.
    Doc,
    Reexport,
    Derive,
    /// A path that a `serde` attribute names, which the derived code calls.
    Attr,
    /// The item a body path reaches: the edge at this index.
    Path(usize),
    /// The declaration that qualifies a body path, such as `Table` in
    /// `Table::new`: the edge at this index.
    Qualifier(usize),
    /// A qualified path that names the item and no other item its source
    /// can reach: the census path at this index.
    Qualified(usize),
}

/// One observed use of an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Use {
    pub(crate) item: ItemId,
    /// The census source the use is written in.
    pub(crate) source: SourceId,
    /// The module of the target's tree that holds it, when known.
    pub(crate) module: Option<ModId>,
    pub(crate) context: Context,
    pub(crate) kind: UseKind,
}

/// The places that an item's observed uses are in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Observed {
    pub(crate) packages: BTreeSet<usize>,
    pub(crate) targets: BTreeSet<usize>,
    /// Each module with its target.
    pub(crate) modules: BTreeSet<(usize, ModId)>,
    pub(crate) sources: BTreeSet<SourceId>,
    pub(crate) contexts: BTreeSet<Context>,
}

/// The items that a qualified path may name, by its last two segments.
struct Qualified {
    /// By qualifier, then by name: a module-level item under its module's
    /// name, or its crate's at the root, a module under its parent's, and a
    /// variant, a trait's item, or an impl member under its declaration's.
    pairs: BTreeMap<String, BTreeMap<String, Vec<ItemId>>>,
    /// The module-level items by name, for `crate::`, `self::`, and
    /// `super::`.
    names: BTreeMap<String, Vec<ItemId>>,
    /// The targets each target's code can name: itself and its analysis
    /// dependencies.
    reach: Vec<BTreeSet<usize>>,
    /// The targets that each package's tests, benches, and examples can
    /// name: its library and what the library can name.
    package_reach: Vec<BTreeSet<usize>>,
}

/// Tells whether a path can name a member: a function, constant, or type.
fn named(member: &Member) -> bool {
    matches!(
        member.kind,
        MemberKind::Fn(_) | MemberKind::Const { .. } | MemberKind::Type(_)
    )
}

impl Qualified {
    fn new(run: &AnalysisRun<'_>) -> Qualified {
        let mut index = Qualified {
            pairs: BTreeMap::new(),
            names: BTreeMap::new(),
            reach: Vec::new(),
            package_reach: Vec::new(),
        };
        for t in 0..run.targets.len() {
            index.target(run, t);
            let mut reach: BTreeSet<usize> = run.analysis_deps[t].values().copied().collect();
            reach.insert(t);
            index.reach.push(reach);
        }
        for p in 0..run.packages.len() {
            let reach = library_of(run.targets, p)
                .map_or_else(BTreeSet::new, |lib| index.reach[lib].clone());
            index.package_reach.push(reach);
        }
        index
    }

    /// Indexes the items of target `t` under their qualifiers.
    fn target(&mut self, run: &AnalysisRun<'_>, t: usize) {
        let krate = &run.krates[t];
        let at = |item| ItemId { target: t, item };
        let root = &run.targets[t].crate_name;
        let qualifier = |m: ModId| {
            krate.modules[m]
                .path
                .last()
                .map_or_else(|| root.clone(), |n| strip_raw(n))
        };
        let by_path: BTreeMap<&[String], ModId> = krate
            .modules
            .iter()
            .enumerate()
            .map(|(m, module)| (module.path.as_slice(), m))
            .collect();
        for (m, module) in krate.modules.iter().enumerate() {
            if let Some((name, parent)) = module.path.split_last() {
                if let Some(&p) = by_path.get(parent) {
                    self.add(qualifier(p), name, at(ItemRef::Module(m)), true);
                }
            }
            for (k, item) in module.items.iter().enumerate() {
                if is_value(&item.kind) {
                    let value = ItemRef::Value(ValueRef { module: m, item: k });
                    self.add(qualifier(m), &item.name, at(value), true);
                }
            }
        }
        for (d, decl) in krate.decls.iter().enumerate() {
            self.add(
                qualifier(decl.module),
                &decl.name,
                at(ItemRef::Decl(d)),
                true,
            );
            let owner = strip_raw(&decl.name);
            if let Shape::Enum(variants) = &decl.shape {
                for (k, variant) in variants.iter().enumerate() {
                    self.add(
                        owner.clone(),
                        &variant.name,
                        at(ItemRef::Variant(d, k)),
                        false,
                    );
                }
            }
            if decl.kind == DeclKind::Trait {
                for (k, member) in decl.members.iter().enumerate().filter(|(_, m)| named(m)) {
                    self.add(owner.clone(), &member.name, at(ItemRef::Own(d, k)), false);
                }
            }
        }
        for (i, item) in krate.impls.iter().enumerate() {
            let Some(d) = run.attached[t][i] else {
                continue;
            };
            let owner = strip_raw(&krate.decls[d].name);
            for (k, member) in item.members.iter().enumerate().filter(|(_, m)| named(m)) {
                self.add(
                    owner.clone(),
                    &member.name,
                    at(ItemRef::Inherent(i, k)),
                    false,
                );
            }
            for (k, member) in item
                .trait_members
                .iter()
                .enumerate()
                .filter(|(_, m)| named(m))
            {
                self.add(
                    owner.clone(),
                    &member.name,
                    at(ItemRef::TraitImpl(i, k)),
                    false,
                );
            }
        }
    }

    fn add(&mut self, qualifier: String, name: &str, item: ItemId, top: bool) {
        let name = strip_raw(name);
        if top {
            self.names.entry(name.clone()).or_default().push(item);
        }
        let by_name = self.pairs.entry(qualifier).or_default();
        by_name.entry(name).or_default().push(item);
    }

    /// Returns the one item that a qualified path names among the items its
    /// source can reach. A build script reaches none, `Self::` names no
    /// single item, and `crate::`, `self::`, and `super::` name a
    /// module-level item of the source's own target, which a doc test or a
    /// package's own file does not have; a path that starts with `::` never
    /// names one that way.
    fn unique(&self, run: &AnalysisRun<'_>, hit: &QualifiedPathHit) -> Option<ItemId> {
        let [.., qualifier, name] = hit.segments.as_slice() else {
            return None;
        };
        let source = &run.census.sources[hit.source];
        let found: Vec<ItemId> = match qualifier.as_str() {
            "Self" => return None,
            "crate" | "self" | "super" if hit.global => return None,
            "crate" | "self" | "super" => {
                let t = source
                    .target
                    .filter(|_| source.context != Context::Doctest)?;
                let items = self.names.get(name)?.iter().copied();
                items.filter(|item| item.target == t).collect()
            }
            _ => {
                let reach = match (source.target, source.context) {
                    (_, Context::Build) => return None,
                    (Some(t), _) => &self.reach[t],
                    (None, _) => &self.package_reach[source.package],
                };
                let items = self.pairs.get(qualifier)?.get(name)?.iter().copied();
                items.filter(|item| reach.contains(&item.target)).collect()
            }
        };
        match found.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }
}

impl AnalysisRun<'_> {
    /// Gathers every observed use, sorted by item, then by where it is.
    pub(crate) fn observe(&self) -> Vec<Use> {
        let mut out = Vec::new();
        for (t, krate) in self.krates.iter().enumerate() {
            for (r, kind) in self.origins(t).into_iter().enumerate() {
                let (Some(kind), Some(item)) = (kind, self.item_of_ref(t, r)) else {
                    continue;
                };
                let module = krate.refs[r].module;
                out.push(Use {
                    item,
                    source: self.census.tree_source(t, krate.modules[module].file),
                    module: Some(module),
                    context: Context::Production,
                    kind,
                });
            }
        }
        for (e, edge) in self.edges.iter().enumerate() {
            let body = &self.krates[edge.target].bodies[edge.body];
            let source = self.census.tree_source(edge.target, body.loc.file);
            let module = self.module_at(edge.target, body.loc.file, body.start);
            let found = [
                (edge.item, UseKind::Path(e)),
                (edge.prefix, UseKind::Qualifier(e)),
            ];
            for (item, kind) in found
                .into_iter()
                .filter_map(|(item, kind)| Some((item?, kind)))
            {
                out.push(Use {
                    item,
                    source,
                    module,
                    context: Context::Production,
                    kind,
                });
            }
        }
        let index = Qualified::new(self);
        for (h, hit) in self.census.paths.iter().enumerate() {
            if let Some(item) = index.unique(self, hit) {
                out.push(Use {
                    item,
                    source: hit.source,
                    module: hit.module,
                    context: hit.context,
                    kind: UseKind::Qualified(h),
                });
            }
        }
        out.sort();
        out
    }

    /// Returns, for each reference of target `t`, the kind of use it is, or
    /// `None` for a reference that is no use of its own: a body path's,
    /// which its edge stands for, and the self type of an impl that belongs
    /// to a declaration.
    fn origins(&self, t: usize) -> Vec<Option<UseKind>> {
        let krate = &self.krates[t];
        let mut origins: Vec<Option<UseKind>> = krate
            .refs
            .iter()
            .map(|r| {
                Some(if r.comment {
                    UseKind::Doc
                } else {
                    UseKind::Signature
                })
            })
            .collect();
        let mut mark = |r: RefId, kind: Option<UseKind>| {
            if let Some(slot) = origins.get_mut(r) {
                *slot = kind;
            }
        };
        for facts in krate.bodies.iter().map(|b| &b.facts) {
            let calls = facts.calls.iter().map(|c| (c.full_ref, c.prefix_ref));
            let paths = facts.constructs.iter().chain(&facts.patterns);
            for (full, prefix) in calls.chain(paths.map(|p| (p.full_ref, p.prefix_ref))) {
                mark(full, None);
                if let Some(prefix) = prefix {
                    mark(prefix, None);
                }
            }
        }
        for name in &krate.body_names {
            mark(name.name, Some(UseKind::Body));
        }
        for path in &krate.attr_paths {
            mark(path.path_ref, Some(UseKind::Attr));
        }
        for &r in krate.decls.iter().flat_map(|d| &d.derive_refs) {
            mark(r, Some(UseKind::Derive));
        }
        for item in krate.modules.iter().flat_map(|m| &m.items) {
            if let ItemKind::Reexport { text, .. } = &item.kind {
                for r in text.refs() {
                    mark(r, Some(UseKind::Reexport));
                }
            }
        }
        for (i, item) in krate.impls.iter().enumerate() {
            for r in (item.trait_members.iter())
                .filter_map(|m| m.kind.refs())
                .flatten()
            {
                mark(r, Some(UseKind::Signature));
            }
            if self.attached[t][i].is_some() {
                for r in item.self_text.refs() {
                    mark(r, None);
                }
            }
        }
        origins
    }

    /// Returns the observed uses of an item, sorted by where they are.
    pub(crate) fn uses_of(&self, item: ItemId) -> &[Use] {
        let start = self.uses.partition_point(|u| u.item < item);
        let end = self.uses.partition_point(|u| u.item <= item);
        &self.uses[start..end]
    }

    /// Returns where an item's observed uses are: in which packages,
    /// targets, modules, sources, and contexts.
    pub(crate) fn observed(&self, item: ItemId) -> Observed {
        let mut seen = Observed::default();
        for found in self.uses_of(item) {
            let source = &self.census.sources[found.source];
            seen.packages.insert(source.package);
            if let Some(t) = source.target {
                seen.targets.insert(t);
                if let Some(m) = found.module {
                    seen.modules.insert((t, m));
                }
            }
            seen.sources.insert(found.source);
            seen.contexts.insert(found.context);
        }
        seen
    }
}
