//! The items of a run's analysis view: an id for every item that analysis
//! indexes, the declaration each impl belongs to by attach's rule, the item
//! each body belongs to, each declaration's logical members, the modules of
//! each file, doc summaries, and the items on each library's surface.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::analysis::AnalysisRun;
use crate::census::NameCount;
use crate::census_docs::summary;
use crate::model::{
    Crate, Decl, DeclId, DeclKind, FileId, Item, ItemKind, Loc, Member, MemberKind, Meta, ModId,
    Module, Shape, ValueRef, Variant, Vis,
};
use crate::model_body::{Body, BodyId, BodyKind, Owner};
use crate::surface;
use crate::typerefs::strip_raw;

/// For each file of a crate, the modules it holds with their bytes, by
/// ascending start.
pub(crate) type Scopes = Vec<Vec<(Range<usize>, ModId)>>;

/// An item of one crate that analysis indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ItemRef {
    Module(ModId),
    /// A struct, enum, union, trait, trait alias, or type alias.
    Decl(DeclId),
    /// A module's function, constant, static, or extern item.
    Value(ValueRef),
    /// A declaration's own member, by its index in the members: a field of
    /// a struct or union, or a trait's item.
    Own(DeclId, usize),
    /// An enum's variant, by its index in the variants.
    Variant(DeclId, usize),
    /// A member of an inherent impl, by its index in the impl's members.
    Inherent(usize, usize),
    /// A member of a trait impl, by its index in the impl's trait members.
    TraitImpl(usize, usize),
}

/// An item of the run: its target, and the item in that target's crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ItemId {
    pub(crate) target: usize,
    pub(crate) item: ItemRef,
}

/// The node behind an item; a module carries the name of its crate, which
/// names the root.
enum Node<'a> {
    Module(&'a Module, &'a str),
    Decl(&'a Decl),
    Value(&'a Item),
    Variant(&'a Variant),
    Member(&'a Member),
}

/// Returns, for each impl of a crate, the declaration it belongs to: the
/// struct, enum, or union that its self type resolves to, which is the rule
/// attach moves impls by.
pub(crate) fn attach(krate: &Crate) -> Vec<Option<DeclId>> {
    krate
        .impls
        .iter()
        .map(|i| {
            i.self_ref.and_then(|r| krate.refs[r].target).filter(|&d| {
                matches!(
                    krate.decls[d].kind,
                    DeclKind::Struct | DeclKind::Enum | DeclKind::Union
                )
            })
        })
        .collect()
}

/// Returns, for each declaration, the impls that belong to it, ascending.
pub(crate) fn impls_by_decl(krate: &Crate, attached: &[Option<DeclId>]) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new(); krate.decls.len()];
    for (i, decl) in attached.iter().enumerate() {
        if let Some(d) = decl {
            out[*d].push(i);
        }
    }
    out
}

/// Returns, for each body of a crate, the item it belongs to; `None` for an
/// anonymous `const _`. Members that share a name under different `cfg`s
/// are told apart by place: a body belongs to the last member of its name
/// and kind that starts on or before the body's line.
pub(crate) fn body_items(krate: &Crate) -> Vec<Option<ItemRef>> {
    krate
        .bodies
        .iter()
        .map(|body| body_item(krate, body))
        .collect()
}

fn body_item(krate: &Crate, body: &Body) -> Option<ItemRef> {
    let consts = body.kind == BodyKind::AssocConst;
    let last = |members: &[Member], name: &str| {
        members
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| {
                let kind = if consts {
                    matches!(m.kind, MemberKind::Const { .. })
                } else {
                    matches!(m.kind, MemberKind::Fn(_))
                };
                kind && m.name == name && m.loc.line <= body.loc.line
            })
            .map(|(k, _)| k)
    };
    match &body.owner {
        Owner::Item(at) => Some(ItemRef::Value(*at)),
        Owner::Anon(_) => None,
        Owner::Impl(i, name) => {
            let item = krate.impls.get(*i)?;
            if item.trait_.is_some() {
                last(&item.trait_members, name).map(|k| ItemRef::TraitImpl(*i, k))
            } else {
                last(&item.members, name).map(|k| ItemRef::Inherent(*i, k))
            }
        }
        Owner::Trait(d, name) => {
            last(&krate.decls.get(*d)?.members, name).map(|k| ItemRef::Own(*d, k))
        }
        Owner::Variant(d, name) => match &krate.decls.get(*d)?.shape {
            Shape::Enum(variants) => variants
                .iter()
                .enumerate()
                .rev()
                .find(|(_, v)| v.name == *name && v.loc.line <= body.loc.line)
                .map(|(k, _)| ItemRef::Variant(*d, k)),
            _ => None,
        },
    }
}

/// Returns the body of each item that has one; the first body wins.
pub(crate) fn bodies_by_item(items: &[Option<ItemRef>]) -> BTreeMap<ItemRef, BodyId> {
    let mut out = BTreeMap::new();
    for (b, item) in items.iter().enumerate() {
        if let Some(item) = item {
            out.entry(*item).or_insert(b);
        }
    }
    out
}

/// Returns, for each file of a crate, the modules it holds with their
/// bytes; an outer module comes before the modules inside it.
pub(crate) fn scopes(krate: &Crate) -> Scopes {
    let mut out: Scopes = vec![Vec::new(); krate.files.len()];
    for (m, module) in krate.modules.iter().enumerate() {
        if !module.scope.is_empty() {
            out[module.file].push((module.scope.clone(), m));
        }
    }
    for list in &mut out {
        list.sort_by_key(|(range, m)| (range.start, Reverse(range.end), *m));
    }
    out
}

/// Returns the innermost of a file's modules whose bytes hold `offset`.
pub(crate) fn innermost(modules: &[(Range<usize>, ModId)], offset: usize) -> Option<ModId> {
    modules
        .iter()
        .rev()
        .find(|(range, _)| range.contains(&offset))
        .map(|(_, m)| *m)
}

/// Tells whether a module item is a value: a function, constant, static,
/// or extern item.
pub(crate) fn is_value(kind: &ItemKind) -> bool {
    matches!(
        kind,
        ItemKind::Fn(_)
            | ItemKind::Const { .. }
            | ItemKind::Static { .. }
            | ItemKind::Foreign { .. }
    )
}

/// Returns the summary of each declaration and module value whose raw doc
/// text has one.
pub(crate) fn summaries(krate: &Crate) -> BTreeMap<ItemRef, String> {
    let decls = krate
        .decls
        .iter()
        .enumerate()
        .map(|(d, decl)| (ItemRef::Decl(d), &decl.meta.raw_doc));
    let values = krate.modules.iter().enumerate().flat_map(|(m, module)| {
        module
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| is_value(&item.kind))
            .map(move |(k, item)| {
                let at = ValueRef { module: m, item: k };
                (ItemRef::Value(at), &item.meta.raw_doc)
            })
    });
    decls
        .chain(values)
        .filter_map(|(item, doc)| Some((item, summary(doc.as_deref()?)?)))
        .collect()
}

/// Returns the items that a library's surface entries name: the root
/// module, modules, declarations, and module values. A value is found by
/// the entry's name in the entry's module, or through a public re-export
/// of that name.
pub(crate) fn surface_items(krate: &Crate, entries: &[surface::Entry]) -> BTreeSet<ItemRef> {
    let mut reexported: BTreeMap<String, Vec<ValueRef>> = BTreeMap::new();
    for item in krate.modules.iter().flat_map(|m| &m.items) {
        if let ItemKind::Reexport { text, .. } = &item.kind {
            if let Some(at) = text.refs().next().and_then(|r| krate.refs[r].value) {
                reexported
                    .entry(strip_raw(&item.name))
                    .or_default()
                    .push(at);
            }
        }
    }
    let mut out = BTreeSet::from([ItemRef::Module(0)]);
    for entry in entries {
        match &entry.kind {
            surface::Kind::Module(m) => {
                out.insert(ItemRef::Module(*m));
            }
            surface::Kind::Decl(d) => {
                out.insert(ItemRef::Decl(*d));
            }
            surface::Kind::Function(m) | surface::Kind::Value(m) => {
                let Some(name) = entry.path.last().map(|n| strip_raw(n)) else {
                    continue;
                };
                let own = krate.modules[*m]
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| is_value(&item.kind) && strip_raw(&item.name) == name)
                    .map(|(k, _)| ValueRef {
                        module: *m,
                        item: k,
                    });
                let through = reexported.get(&name).into_iter().flatten().copied();
                let through = through.filter(|at| at.module == *m);
                out.extend(own.chain(through).map(ItemRef::Value));
            }
            surface::Kind::Macro(_) | surface::Kind::Unlinked(_) => {}
        }
    }
    out
}

impl<'a> AnalysisRun<'a> {
    /// Returns the logical members of declaration `d` of target `t`: its
    /// own fields or trait items, then the members of each impl that
    /// belongs to it, in impl order, inherent and trait impls alike.
    pub(crate) fn members(
        &self,
        t: usize,
        d: DeclId,
    ) -> impl Iterator<Item = (ItemRef, &'a Member)> {
        let krate: &'a Crate = &self.krates[t];
        let own = krate.decls[d]
            .members
            .iter()
            .enumerate()
            .map(move |(k, m)| (ItemRef::Own(d, k), m));
        let impls = self.impls_of[t][d].iter().flat_map(move |&i| {
            let item = &krate.impls[i];
            let inherent = item.members.iter().enumerate();
            let traits = item.trait_members.iter().enumerate();
            let inherent = inherent.map(move |(k, m)| (ItemRef::Inherent(i, k), m));
            inherent.chain(traits.map(move |(k, m)| (ItemRef::TraitImpl(i, k), m)))
        });
        own.chain(impls)
    }

    /// Returns the member an item of target `t` is: a declaration's own
    /// member or an impl's.
    pub(crate) fn member(&self, t: usize, item: ItemRef) -> Option<&'a Member> {
        let krate: &'a Crate = self.krates.get(t)?;
        match item {
            ItemRef::Own(d, k) => krate.decls.get(d)?.members.get(k),
            ItemRef::Inherent(i, k) => krate.impls.get(i)?.members.get(k),
            ItemRef::TraitImpl(i, k) => krate.impls.get(i)?.trait_members.get(k),
            ItemRef::Module(_) | ItemRef::Decl(_) | ItemRef::Value(_) | ItemRef::Variant(..) => {
                None
            }
        }
    }

    /// Returns the body of an item: a function's block, or a constant's,
    /// static's, or discriminant's initializer.
    pub(crate) fn body_of(&self, item: ItemId) -> Option<BodyId> {
        self.item_bodies.get(item.target)?.get(&item.item).copied()
    }

    fn node(&self, item: ItemId) -> Option<Node<'a>> {
        let krate: &'a Crate = self.krates.get(item.target)?;
        Some(match item.item {
            ItemRef::Module(m) => {
                let root = self.targets.get(item.target)?.crate_name.as_str();
                Node::Module(krate.modules.get(m)?, root)
            }
            ItemRef::Decl(d) => Node::Decl(krate.decls.get(d)?),
            ItemRef::Value(at) => {
                let value = krate.modules.get(at.module)?.items.get(at.item)?;
                if !is_value(&value.kind) {
                    return None;
                }
                Node::Value(value)
            }
            ItemRef::Variant(d, k) => match &krate.decls.get(d)?.shape {
                Shape::Enum(variants) => Node::Variant(variants.get(k)?),
                _ => return None,
            },
            ItemRef::Own(..) | ItemRef::Inherent(..) | ItemRef::TraitImpl(..) => {
                Node::Member(self.member(item.target, item.item)?)
            }
        })
    }

    /// Returns an item's name as written, `r#` kept; the root module's is
    /// its crate's name. `None` when the item does not exist.
    pub(crate) fn name(&self, item: ItemId) -> Option<&'a str> {
        Some(match self.node(item)? {
            Node::Module(module, root) => module.path.last().map_or(root, String::as_str),
            Node::Decl(decl) => &decl.name,
            Node::Value(value) => &value.name,
            Node::Variant(variant) => &variant.name,
            Node::Member(member) => &member.name,
        })
    }

    /// Returns an item's path below its crate's root, such as
    /// `["parse", "Args", "new"]`. A member of an impl that belongs to no
    /// declaration is named after the impl's self type as Rust writes it.
    pub(crate) fn path(&self, item: ItemId) -> Vec<String> {
        let krate = &self.krates[item.target];
        let mut path = match item.item {
            ItemRef::Module(m) => return krate.modules[m].path.clone(),
            ItemRef::Decl(d) => return krate.decl_path(d),
            ItemRef::Value(at) => krate.modules[at.module].path.clone(),
            ItemRef::Own(d, _) | ItemRef::Variant(d, _) => krate.decl_path(d),
            ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _) => {
                match self.attached[item.target][i] {
                    Some(d) => krate.decl_path(d),
                    None => vec![krate.impls[i].self_text.rust_visible(&krate.refs)],
                }
            }
        };
        path.extend(self.name(item).map(str::to_owned));
        path
    }

    /// Returns where an item is written; a module that is a file of its
    /// own is at its first line.
    pub(crate) fn loc(&self, item: ItemId) -> Option<Loc> {
        Some(match self.node(item)? {
            Node::Module(module, _) => module.inline.unwrap_or(Loc {
                file: module.file,
                line: 1,
            }),
            Node::Decl(decl) => decl.loc,
            Node::Value(value) => value.loc,
            Node::Variant(variant) => variant.loc,
            Node::Member(member) => member.loc,
        })
    }

    /// Returns an item's visibility as written; a variant is public.
    pub(crate) fn vis(&self, item: ItemId) -> Option<Vis> {
        Some(match self.node(item)? {
            Node::Module(module, _) => module.vis,
            Node::Decl(decl) => decl.vis,
            Node::Value(value) => value.vis,
            Node::Variant(_) => Vis::Public,
            Node::Member(member) => member.vis,
        })
    }

    /// Returns an item's comments, attributes, raw doc, and lints; a
    /// module keeps its own.
    pub(crate) fn meta(&self, item: ItemId) -> Option<&'a Meta> {
        match self.node(item)? {
            Node::Module(..) => None,
            Node::Decl(decl) => Some(&decl.meta),
            Node::Value(value) => Some(&value.meta),
            Node::Variant(variant) => Some(&variant.meta),
            Node::Member(member) => Some(&member.meta),
        }
    }

    /// Returns the summary of a declaration's or module value's raw doc.
    pub(crate) fn summary(&self, item: ItemId) -> Option<&str> {
        self.summaries
            .get(item.target)?
            .get(&item.item)
            .map(String::as_str)
    }

    /// Tells whether a library's users can name an item: one its surface
    /// names, a variant or a trait's item of such a declaration, a public
    /// field or inherent member of one, or a member of a trait impl that
    /// belongs to one. Nothing of a target that is not a library is.
    pub(crate) fn on_surface(&self, item: ItemId) -> bool {
        let Some(set) = self.surface_items.get(item.target) else {
            return false;
        };
        let krate = &self.krates[item.target];
        let shown = |d: Option<DeclId>| d.is_some_and(|d| set.contains(&ItemRef::Decl(d)));
        let public = |member: Option<&Member>| member.is_some_and(|m| m.vis == Vis::Public);
        match item.item {
            ItemRef::Module(_) | ItemRef::Decl(_) | ItemRef::Value(_) => set.contains(&item.item),
            ItemRef::Variant(d, _) => shown(Some(d)),
            ItemRef::Own(d, k) => {
                let trait_item = krate.decls[d].kind == DeclKind::Trait;
                shown(Some(d)) && (trait_item || public(krate.decls[d].members.get(k)))
            }
            ItemRef::Inherent(i, k) => {
                shown(self.attached[item.target][i]) && public(krate.impls[i].members.get(k))
            }
            ItemRef::TraitImpl(i, _) => shown(self.attached[item.target][i]),
        }
    }

    /// Returns the innermost module of target `t` that holds a byte offset
    /// of one of its files.
    pub(crate) fn module_at(&self, t: usize, file: FileId, offset: usize) -> Option<ModId> {
        innermost(self.scopes.get(t)?.get(file)?, offset)
    }

    /// Counts the hits of an item's name, written without `r#`, by
    /// package, target, and context, beside the run's declarations of it.
    pub(crate) fn name_count(&self, item: ItemId) -> Option<NameCount> {
        Some(self.census.count(&strip_raw(self.name(item)?)))
    }
}
