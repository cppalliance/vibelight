//! The paths of a run's bodies and what they reach. Analysis resolves them
//! with its own dependency map, which adds each binary's own library, and
//! changes no crate. A path reaches a free function, constant, static, or
//! type through its own resolution; `Type::name` reaches a variant of an
//! enum or the one inherent member of that name. `Self` and `Self::name`
//! resolve inside an impl of a declaration of the run, where `Self::name`
//! reaches a variant from any impl but a member only from an inherent one.
//! Trait methods and defaults, generics, locals, and names outside the run
//! stay unresolved.

use std::collections::BTreeMap;

use crate::analysis::{AnalysisRun, TargetInfo, library_of};
use crate::analysis_items::{ItemId, ItemRef};
use crate::model::{Crate, CrossRef, DeclId, MemberKind, Reach, RefId, Shape};
use crate::model_body::{Body, BodyId, Owner, Role};
use crate::resolve_run;
use crate::root::TargetKind;
use crate::typerefs::strip_raw;

/// What kind of body path an edge is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PathKind {
    /// A path in expression position, with how the body uses it.
    Call(Role),
    /// The path of a struct literal.
    Construct,
    /// A path in a pattern.
    Pattern,
}

/// How an edge's path reached its item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Via {
    /// The path itself resolves, in its own crate or in another crate of
    /// the run.
    Path,
    /// The prefix names a declaration with one inherent member of the last
    /// segment's name.
    Member,
    /// `Self`, or `Self::name` inside an inherent impl.
    SelfType,
    /// The prefix names an enum, and the last segment one of its variants.
    Variant,
    /// Nothing the rules allow: a trait method or default, a generic, a
    /// local, or a name outside the run.
    Unresolved,
}

/// One path of one body and what it reaches.
#[derive(Clone, Debug)]
pub(crate) struct Edge {
    /// The target of the body.
    pub(crate) target: usize,
    pub(crate) body: BodyId,
    pub(crate) kind: PathKind,
    /// The index of the path in the body's calls, constructs, or patterns.
    pub(crate) fact: usize,
    /// The item the path reaches.
    pub(crate) item: Option<ItemId>,
    /// The declaration the path's prefix names, such as `Table` in
    /// `Table::new`, or the self type of `Self::name`.
    pub(crate) prefix: Option<ItemId>,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "only the tests read which rule resolved a path")
    )]
    pub(crate) via: Via,
}

impl Edge {
    /// Returns the edge of path `fact` of a body, with what it reached.
    fn new(target: usize, body: BodyId, kind: PathKind, fact: usize, reached: Reached) -> Self {
        Self {
            target,
            body,
            kind,
            fact,
            item: reached.item,
            prefix: reached.prefix,
            via: reached.via,
        }
    }
}

/// What one path reached: the item, the declaration its prefix names, and
/// the rule that resolved it.
#[derive(Clone, Copy, Debug)]
struct Reached {
    item: Option<ItemId>,
    prefix: Option<ItemId>,
    via: Via,
}

impl Reached {
    /// Returns a path that reached `item` by the rule `via`.
    fn at(item: ItemId, prefix: Option<ItemId>, via: Via) -> Self {
        Self {
            item: Some(item),
            prefix,
            via,
        }
    }

    /// Returns a path that reached no item, so that `Via::Unresolved` and
    /// a missing item agree by construction.
    fn open(prefix: Option<ItemId>) -> Self {
        Self {
            item: None,
            prefix,
            via: Via::Unresolved,
        }
    }
}

/// Returns each target's dependency map for analysis: the map name
/// resolution used, and for a binary also its package's library, under the
/// library's crate name unless the map already holds that name.
pub(crate) fn augmented(targets: &[TargetInfo]) -> Vec<BTreeMap<String, usize>> {
    targets
        .iter()
        .map(|info| {
            let mut deps = info.deps.clone();
            let lib = library_of(targets, info.package);
            if let (TargetKind::Bin, Some(lib)) = (info.kind, lib) {
                deps.entry(targets[lib].crate_name.clone()).or_insert(lib);
            }
            deps
        })
        .collect()
}

/// Returns, for each crate, the item in another crate that each reference
/// reaches through `deps`, kept only for a reference that reaches nothing
/// in its own crate or through `link`.
pub(crate) fn cross(
    krates: &[Crate],
    deps: &[BTreeMap<String, usize>],
) -> Vec<BTreeMap<RefId, CrossRef>> {
    resolve_run::cross_refs(krates, deps)
        .into_iter()
        .zip(krates)
        .map(|(found, krate)| {
            found
                .into_iter()
                .zip(&krate.refs)
                .enumerate()
                .filter(|(_, (_, r))| {
                    r.decl().is_none() && r.value().is_none() && r.foreign.is_none()
                })
                .filter_map(|(r, (cross, _))| Some((r, cross?)))
                .collect()
        })
        .collect()
}

impl AnalysisRun<'_> {
    /// Resolves every path of every body into an edge, in target and body
    /// order, and in each body its calls, struct literals, and patterns.
    pub(crate) fn link_bodies(&self) -> Vec<Edge> {
        let mut edges = Vec::new();
        for (t, krate) in self.krates.iter().enumerate() {
            for (b, body) in krate.bodies.iter().enumerate() {
                let facts = &body.facts;
                for (fact, call) in facts.calls.iter().enumerate() {
                    let reached = self.resolve(t, body, &call.path, call.full_ref, call.prefix_ref);
                    let kind = PathKind::Call(call.role);
                    edges.push(Edge::new(t, b, kind, fact, reached));
                }
                let lists = [
                    (PathKind::Construct, &facts.constructs),
                    (PathKind::Pattern, &facts.patterns),
                ];
                for (kind, list) in lists {
                    for (fact, path) in list.iter().enumerate() {
                        let reached =
                            self.resolve(t, body, &path.path, path.full_ref, path.prefix_ref);
                        edges.push(Edge::new(t, b, kind, fact, reached));
                    }
                }
            }
        }
        edges
    }

    /// Returns what one path of a body of target `t` reaches, the
    /// declaration its prefix names, and how.
    fn resolve(
        &self,
        t: usize,
        body: &Body,
        path: &[String],
        full: RefId,
        prefix: Option<RefId>,
    ) -> Reached {
        let at = |target, item| ItemId { target, item };
        if path.first().is_some_and(|first| first == "Self") {
            let Owner::Impl(i, _) = &body.owner else {
                return Reached::open(None);
            };
            let Some(d) = self.attached[t][*i] else {
                return Reached::open(None);
            };
            let own = at(t, ItemRef::Decl(d));
            let inherent = self.krates[t].impls[*i].trait_.is_none();
            return match path {
                [_] => Reached::at(own, None, Via::SelfType),
                [_, name] => match self.variant(t, d, name) {
                    Some(variant) => Reached::at(at(t, variant), Some(own), Via::Variant),
                    None if inherent => match self.inherent(t, d, name) {
                        Some(member) => Reached::at(at(t, member), Some(own), Via::SelfType),
                        None => Reached::open(Some(own)),
                    },
                    None => Reached::open(Some(own)),
                },
                _ => Reached::open(None),
            };
        }
        let owner = prefix.and_then(|r| self.decl_of_ref(t, r));
        let prefix = owner.map(|(c, d)| at(c, ItemRef::Decl(d)));
        if let Some(item) = self.item_of_ref(t, full) {
            return Reached::at(item, prefix, Via::Path);
        }
        let (Some((c, d)), Some(name)) = (owner, path.last()) else {
            return Reached::open(prefix);
        };
        if let Some(variant) = self.variant(c, d, name) {
            return Reached::at(at(c, variant), prefix, Via::Variant);
        }
        match self.inherent(c, d, name) {
            Some(member) => Reached::at(at(c, member), prefix, Via::Member),
            None => Reached::open(prefix),
        }
    }

    /// Returns the inherent function or constant named `name` of
    /// declaration `d` of target `t`. Members that share a name differ in
    /// their `cfg`s, and the first by `cfg` lines wins, as for values; two
    /// with the same `cfg`s are no single member.
    fn inherent(&self, t: usize, d: DeclId, name: &str) -> Option<ItemRef> {
        let krate = &self.krates[t];
        let mut found: Vec<(String, ItemRef)> = Vec::new();
        for &i in &self.impls_of[t][d] {
            if krate.impls[i].trait_.is_some() {
                continue;
            }
            for (k, member) in krate.impls[i].members.iter().enumerate() {
                let callable = matches!(member.kind, MemberKind::Fn(_) | MemberKind::Const { .. });
                if callable && strip_raw(&member.name) == name {
                    found.push((member.meta.cfgs.join(" "), ItemRef::Inherent(i, k)));
                }
            }
        }
        found.sort();
        let distinct = found.windows(2).all(|w| w[0].0 != w[1].0);
        found.first().filter(|_| distinct).map(|(_, item)| *item)
    }

    /// Returns the variant named `name` of declaration `d` of target `t`,
    /// when the declaration is an enum.
    fn variant(&self, t: usize, d: DeclId, name: &str) -> Option<ItemRef> {
        let Shape::Enum(variants) = &self.krates[t].decls[d].shape else {
            return None;
        };
        variants
            .iter()
            .position(|v| strip_raw(&v.name) == name)
            .map(|k| ItemRef::Variant(d, k))
    }

    /// Returns the item a reference of target `t` reaches: a declaration or
    /// value of its own crate, or of another crate through `link` or the
    /// analysis map.
    pub(crate) fn item_of_ref(&self, t: usize, r: RefId) -> Option<ItemId> {
        let reference = self.krates.get(t)?.refs.get(r)?;
        if let Some(d) = reference.decl() {
            return Some(ItemId::decl(t, d));
        }
        if let Some(at) = reference.value() {
            return Some(ItemId {
                target: t,
                item: ItemRef::Value(at),
            });
        }
        let cross = reference
            .foreign
            .or_else(|| self.cross[t].get(&r).copied())?;
        let item = match cross.reach {
            Reach::Decl(d) => ItemRef::Decl(d),
            Reach::Value(at) => ItemRef::Value(at),
        };
        Some(ItemId {
            target: cross.krate,
            item,
        })
    }

    /// Returns the declaration a reference of target `t` reaches, with the
    /// target that declares it.
    pub(crate) fn decl_of_ref(&self, t: usize, r: RefId) -> Option<(usize, DeclId)> {
        match self.item_of_ref(t, r)? {
            ItemId {
                target,
                item: ItemRef::Decl(d),
            } => Some((target, d)),
            _ => None,
        }
    }

    /// Returns the edges out of a body of target `t`: its calls, struct
    /// literals, and patterns.
    pub(crate) fn edges_from(&self, t: usize, body: BodyId) -> &[Edge] {
        let start = self
            .edges
            .partition_point(|e| (e.target, e.body) < (t, body));
        let end = self
            .edges
            .partition_point(|e| (e.target, e.body) <= (t, body));
        &self.edges[start..end]
    }

    /// Returns the edges into an item: every body path that reaches it.
    pub(crate) fn edges_to(&self, item: ItemId) -> impl Iterator<Item = &Edge> {
        self.edges_in
            .get(&item)
            .into_iter()
            .flatten()
            .map(|&e| &self.edges[e])
    }
}
