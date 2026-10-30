//! Checks the invariants of a run's analysis view before anything reads
//! it, so that a broken view stops the run as an internal error rather than
//! reaching a detector: the tables of every target, the items and their
//! bodies, the edges, the census, and the observed uses. Every query the
//! detectors use is checked against the tables it reads.

use std::collections::BTreeSet;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::{Edge, PathKind, Via};
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::model::{DeclKind, Reach};
use crate::model_body::{BodyKind, Owner};
use crate::root::TargetKind;
use crate::surface;
use crate::typerefs::strip_raw;

impl AnalysisRun<'_> {
    /// Checks the view's invariants.
    ///
    /// # Errors
    /// Returns a message that names the first invariant that fails.
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.check_targets()?;
        self.check_items()?;
        self.check_edges()?;
        self.check_census()?;
        self.check_uses()
    }

    /// Checks the per-target tables, each target's package and dependency
    /// map, and the files each package and target reads.
    fn check_targets(&self) -> Result<(), String> {
        let n = self.krates.len();
        let lens = [
            self.targets.len(),
            self.deps.len(),
            self.cross.len(),
            self.attached.len(),
            self.impls_of.len(),
            self.body_items.len(),
            self.item_bodies.len(),
            self.scopes.len(),
            self.summaries.len(),
            self.surfaces.len(),
            self.surface_items.len(),
            self.spellings.len(),
            self.census.tree.len(),
        ];
        if lens.iter().any(|&len| len != n) {
            return Err("a table of the targets does not have one entry per target".to_owned());
        }
        let mut stems = BTreeSet::new();
        for (t, info) in self.targets.iter().enumerate() {
            let listed = self
                .packages
                .get(info.package)
                .is_some_and(|p| p.targets.contains(&t));
            if !listed || info.crate_name.is_empty() || !stems.insert(info.stem.as_str()) {
                return Err(format!(
                    "target {} is not listed once in its package",
                    info.stem
                ));
            }
            for (name, &lib) in &self.deps[t] {
                let library = lib != t
                    && self
                        .targets
                        .get(lib)
                        .is_some_and(|l| l.kind == TargetKind::Lib);
                let own = info.deps.get(name) == Some(&lib);
                let sibling = info.kind == TargetKind::Bin
                    && self
                        .targets
                        .get(lib)
                        .is_some_and(|l| l.package == info.package && l.crate_name == *name);
                if !library || !(own || sibling) {
                    return Err(format!("target {} may not name {name}", info.stem));
                }
            }
            let kept = info.deps.keys().all(|name| self.deps[t].contains_key(name));
            let loaded = info.files.iter().all(|extra| {
                extra.context == Context::Test && self.krates[t].test_files.contains(&extra.rel)
            });
            if !kept || !loaded {
                return Err(format!(
                    "target {} lost a dependency or reads a stray file",
                    info.stem
                ));
            }
        }
        for (p, package) in self.packages.iter().enumerate() {
            let ordered = package.targets.windows(2).all(|w| w[0] < w[1]);
            let owned = package
                .targets
                .iter()
                .all(|&t| self.targets.get(t).is_some_and(|info| info.package == p));
            let libs = package
                .targets
                .iter()
                .filter(|&&t| self.targets[t].kind == TargetKind::Lib)
                .count();
            if !ordered || !owned || libs > 1 {
                return Err(format!("package {p} does not own its targets in order"));
            }
            let facts = &package.facts;
            let roots = (facts.tests.iter().chain(&facts.benches))
                .chain(&facts.examples)
                .chain(&facts.build);
            let outside = package.files.iter().all(|extra| {
                !matches!(extra.context, Context::Production | Context::Doctest)
                    && !extra.rel.contains('\\')
            });
            for root in roots {
                if !outside || !package.files.iter().any(|extra| extra.rel == *root) {
                    return Err(format!(
                        "package {p} does not read its root {root} as a root"
                    ));
                }
            }
        }
        Ok(())
    }

    /// Checks the logical attachment, each declaration's members, each
    /// body's item, and the module scopes, then the docs of each target.
    fn check_items(&self) -> Result<(), String> {
        for (t, krate) in self.krates.iter().enumerate() {
            let attached = &self.attached[t];
            let sized = attached.len() == krate.impls.len()
                && self.impls_of[t].len() == krate.decls.len()
                && self.body_items[t].len() == krate.bodies.len()
                && self.scopes[t].len() == krate.files.len();
            let placed: usize = self.impls_of[t].iter().map(Vec::len).sum();
            if !sized || placed != attached.iter().flatten().count() {
                return Err(format!(
                    "the item tables of {} do not fit its crate",
                    krate.name
                ));
            }
            for (d, impls) in self.impls_of[t].iter().enumerate() {
                let decl = &krate.decls[d];
                let typed = matches!(
                    decl.kind,
                    DeclKind::Struct | DeclKind::Enum | DeclKind::Union
                );
                let belong = impls.windows(2).all(|w| w[0] < w[1])
                    && impls.iter().all(|&i| attached.get(i) == Some(&Some(d)))
                    && (impls.is_empty() || typed);
                let expected = decl.members.len()
                    + impls
                        .iter()
                        .map(|&i| krate.impls[i].members.len() + krate.impls[i].trait_members.len())
                        .sum::<usize>();
                let mut members = 0;
                for (item, member) in self.members(t, d) {
                    members += 1;
                    if !self
                        .member(t, item)
                        .is_some_and(|m| std::ptr::eq(m, member))
                    {
                        return Err(format!("a member of {} is out of place", decl.name));
                    }
                }
                if !belong || members != expected {
                    return Err(format!("the impls of {} do not belong to it", decl.name));
                }
            }
            for (b, item) in self.body_items[t].iter().enumerate() {
                let body = &krate.bodies[b];
                let Some(item) = *item else {
                    if body.kind != BodyKind::AnonConst {
                        return Err(format!("body {b} of {} belongs to no item", krate.name));
                    }
                    continue;
                };
                let fits = matches!(
                    (item, body.kind),
                    (
                        ItemRef::Value(_),
                        BodyKind::Fn | BodyKind::Const | BodyKind::Static
                    ) | (
                        ItemRef::Inherent(..),
                        BodyKind::Method | BodyKind::AssocConst
                    ) | (
                        ItemRef::TraitImpl(..),
                        BodyKind::TraitImplMethod | BodyKind::AssocConst
                    ) | (
                        ItemRef::Own(..),
                        BodyKind::TraitDefault | BodyKind::AssocConst
                    ) | (ItemRef::Variant(..), BodyKind::Discriminant)
                );
                let id = ItemId { target: t, item };
                let owner = match &body.owner {
                    Owner::Impl(_, name) | Owner::Trait(_, name) | Owner::Variant(_, name) => {
                        Some(name)
                    }
                    Owner::Item(_) | Owner::Anon(_) => None,
                };
                let named = self
                    .name(id)
                    .is_some_and(|name| owner.is_none_or(|own| own == name));
                let held = self.loc(id).is_some_and(|loc| loc.file == body.file);
                if !fits || !named || !held || self.body_of(id) != Some(b) {
                    return Err(format!("body {b} of {} has the wrong item", krate.name));
                }
            }
            for (f, modules) in self.scopes[t].iter().enumerate() {
                for (range, m) in modules {
                    let fits = krate.modules.get(*m).is_some_and(|module| module.file == f)
                        && range.end <= krate.files[f].text.len()
                        && self.module_at(t, f, range.start) == Some(*m);
                    if !fits {
                        return Err(format!("module {m} of {} is out of its file", krate.name));
                    }
                }
            }
            self.check_docs(t)?;
        }
        Ok(())
    }

    /// Checks target `t`'s summaries, surface, and spellings.
    fn check_docs(&self, t: usize) -> Result<(), String> {
        let krate = &self.krates[t];
        for (&item, text) in &self.summaries[t] {
            let id = ItemId { target: t, item };
            let doc = self.meta(id).and_then(|meta| meta.raw_doc.as_deref());
            let fits = matches!(item, ItemRef::Decl(_) | ItemRef::Value(_))
                && self.summary(id) == Some(text.as_str())
                && doc.is_some_and(|doc| doc.contains(text.as_str()))
                && text.trim() == text
                && !text.is_empty()
                && !text.contains("\n\n");
            if !fits {
                return Err(format!(
                    "a summary of {} is not its doc's first paragraph",
                    krate.name
                ));
            }
        }
        if self.targets[t].kind == TargetKind::Lib {
            let decls = self.surfaces[t]
                .iter()
                .filter_map(|entry| match entry.kind {
                    surface::Kind::Decl(d) => Some(ItemId {
                        target: t,
                        item: ItemRef::Decl(d),
                    }),
                    _ => None,
                });
            let items = self.surface_items[t]
                .iter()
                .map(|&item| ItemId { target: t, item });
            for id in decls.chain(items) {
                if self.name(id).is_none() || !self.on_surface(id) {
                    return Err(format!("a surface item of {} is not on it", krate.name));
                }
            }
        } else if !self.surfaces[t].is_empty() || !self.surface_items[t].is_empty() {
            return Err(format!("{} has a surface but is no library", krate.name));
        }
        for entry in &self.spellings[t] {
            let sound = entry.spellings.len() > 1
                && entry
                    .spellings
                    .iter()
                    .all(|s| s.count > 0 && s.locs.iter().all(|loc| loc.file < krate.files.len()));
            if !sound {
                return Err(format!(
                    "the spelling of {} in {} is malformed",
                    entry.item, krate.name
                ));
            }
        }
        Ok(())
    }

    /// Checks that each body has one edge per path, in order, each fitting
    /// its fact, and that the cross references and the index of edges by
    /// item agree with the edges.
    fn check_edges(&self) -> Result<(), String> {
        let rank = |kind: PathKind| match kind {
            PathKind::Call(_) => 0,
            PathKind::Construct => 1,
            PathKind::Pattern => 2,
        };
        let key = |e: &Edge| (e.target, e.body, rank(e.kind), e.fact);
        if !self.edges.windows(2).all(|w| key(&w[0]) < key(&w[1])) {
            return Err("the edges are not in target, body, kind, and source order".to_owned());
        }
        for (t, krate) in self.krates.iter().enumerate() {
            for (b, body) in krate.bodies.iter().enumerate() {
                let facts = &body.facts;
                let edges = self.edges_from(t, b);
                if edges.len() != facts.calls.len() + facts.constructs.len() + facts.patterns.len()
                {
                    return Err(format!(
                        "body {b} of {} has not one edge per path",
                        krate.name
                    ));
                }
                for edge in edges {
                    let fits = match edge.kind {
                        PathKind::Call(role) => {
                            facts.calls.get(edge.fact).is_some_and(|c| c.role == role)
                        }
                        PathKind::Construct => edge.fact < facts.constructs.len(),
                        PathKind::Pattern => edge.fact < facts.patterns.len(),
                    };
                    let resolved = (edge.via == Via::Unresolved) == edge.item.is_none();
                    let exists = edge.item.is_none_or(|item| self.name(item).is_some());
                    let prefixed = edge.prefix.is_none_or(|p| {
                        matches!(p.item, ItemRef::Decl(_)) && self.name(p).is_some()
                    });
                    if !(fits && resolved && exists && prefixed) {
                        return Err(format!(
                            "an edge of body {b} of {} is malformed",
                            krate.name
                        ));
                    }
                }
            }
            for (&r, cross) in &self.cross[t] {
                let open = krate.refs.get(r).is_some_and(|r| {
                    r.target.is_none() && r.value.is_none() && r.foreign.is_none()
                });
                let reached = self
                    .item_of_ref(t, r)
                    .is_some_and(|item| self.name(item).is_some());
                let decl =
                    self.decl_of_ref(t, r).is_some() == matches!(cross.reach, Reach::Decl(_));
                if !open || !reached || !decl || cross.krate == t {
                    return Err(format!("reference {r} of {} crosses wrongly", krate.name));
                }
            }
        }
        let mut indexed = 0;
        for (&item, list) in &self.edges_in {
            let fits = list.windows(2).all(|w| w[0] < w[1])
                && list.iter().all(|&e| {
                    self.edges
                        .get(e)
                        .is_some_and(|edge| edge.item == Some(item))
                })
                && self.edges_to(item).count() == list.len();
            if !fits {
                return Err("the edges into an item are not indexed in order".to_owned());
            }
            indexed += list.len();
        }
        if indexed != self.edges.iter().filter(|e| e.item.is_some()).count() {
            return Err("an edge that reaches an item is not indexed".to_owned());
        }
        Ok(())
    }

    /// Checks that the uses are sorted, that each used item exists and is
    /// found the same way by every query, and that each use fits its kind.
    fn check_uses(&self) -> Result<(), String> {
        if !self.uses.windows(2).all(|w| w[0] <= w[1]) {
            return Err("the observed uses are not sorted".to_owned());
        }
        let count = |kind: fn(&UseKind) -> bool| self.uses.iter().filter(|u| kind(&u.kind)).count();
        let reached = self.edges.iter().filter(|e| e.item.is_some()).count();
        let prefixed = self.edges.iter().filter(|e| e.prefix.is_some()).count();
        if count(|k| matches!(k, UseKind::Path(_))) != reached
            || count(|k| matches!(k, UseKind::Qualifier(_))) != prefixed
        {
            return Err("an edge is not one use of what it reaches and names".to_owned());
        }
        let mut start = 0;
        while let Some(first) = self.uses.get(start) {
            let item = first.item;
            let uses = self.uses_of(item);
            let name = self.name(item).ok_or("a used item does not exist")?;
            let path = self.path(item);
            let found = self.loc(item).is_some()
                && self.vis(item).is_some()
                && (path.last().map(String::as_str) == Some(name)
                    || item.item == ItemRef::Module(0));
            if !found || uses.is_empty() {
                return Err(format!(
                    "{} is not found the same way by every query",
                    path.join("::")
                ));
            }
            let lexical = uses.iter().any(|u| matches!(u.kind, UseKind::Qualified(_)));
            let count = (lexical.then(|| self.name_count(item)))
                .flatten()
                .unwrap_or_default();
            let unraw = strip_raw(name);
            for u in uses {
                let source = self
                    .census
                    .sources
                    .get(u.source)
                    .ok_or("a use has no source")?;
                let fits = match u.kind {
                    UseKind::Path(e) => self
                        .edges
                        .get(e)
                        .is_some_and(|edge| edge.item == Some(item)),
                    UseKind::Qualifier(e) => self
                        .edges
                        .get(e)
                        .is_some_and(|edge| edge.prefix == Some(item)),
                    UseKind::Qualified(h) => self.census.paths.get(h).is_some_and(|hit| {
                        hit.source == u.source
                            && hit.context == u.context
                            && hit.module == u.module
                            && hit.segments.last() == Some(&unraw)
                            && count.total > 0
                            && count.uses() <= count.total
                    }),
                    UseKind::Signature
                    | UseKind::Body
                    | UseKind::Doc
                    | UseKind::Reexport
                    | UseKind::Derive
                    | UseKind::Attr => {
                        u.context == Context::Production
                            && u.module.is_some()
                            && source.file.is_some()
                    }
                };
                if !fits {
                    return Err(format!(
                        "a use of {} does not fit its kind",
                        path.join("::")
                    ));
                }
            }
            let observed = self.observed(item);
            let consistent = !observed.contexts.is_empty()
                && observed.sources.len() <= uses.len()
                && observed
                    .modules
                    .iter()
                    .all(|(t, _)| observed.targets.contains(t))
                && (observed.targets.iter())
                    .all(|&t| observed.packages.contains(&self.targets[t].package));
            if !consistent {
                return Err(format!(
                    "the observed places of {} disagree",
                    path.join("::")
                ));
            }
            start += uses.len();
        }
        Ok(())
    }
}
