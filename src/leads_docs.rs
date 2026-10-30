//! The detectors of doc text: what docs say about state, history,
//! visibility, and errors, what they echo or repeat, the names they write
//! that nothing reaches, and the claims that the written declarations
//! contradict. This file holds the group's registry and the docs of the
//! run that every detector reads; its siblings hold the detectors.

use std::collections::BTreeMap;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::Lead;
use crate::leads_docs_text::{Prose, read};
use crate::leads_items::{Catalog, impl_module, impls_of_trait};
use crate::leads_types::{Ty, item_name, leaf};
use crate::model::{
    Crate, DeclId, DeclKind, Foreign, ItemKind, Loc, MemberKind, Meta, ModId, Note, RefId, Shape,
    Text, ValueRef, Vis,
};
use crate::typerefs::strip_raw;
use crate::{
    leads_docs_claims, leads_docs_lists, leads_docs_phrases, leads_docs_refs, leads_docs_repeat,
};

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "conditional-field",
    "partial-doc-list",
    "history-words",
    "visibility-excuse",
    "doc-echo",
    "repeated-doc-text",
    "doc-unresolved-link",
    "doc-unresolved-backtick",
    "doc-unknown-param",
    "doc-signature-mismatch",
    "doc-count-mismatch",
    "doc-const-mismatch",
    "container-invariant-claim",
    "impossible-error",
];

/// The ids whose leads rest on what the run does not observe.
pub(crate) const ABSENT: &[&str] = &["doc-unresolved-link", "doc-unresolved-backtick"];

/// The subject path segment that stands for a crate's root module.
const ROOT: &str = "crate";

/// What a doc documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Node {
    Module,
    Decl(DeclId),
    /// A field, as an index into the catalog's fields.
    Field(usize),
    Variant(DeclId, usize),
    /// A function, as an index into the catalog's functions.
    Func(usize),
    /// A constant, a static, an extern item other than a function's, or
    /// an associated constant or type.
    Value,
}

impl Node {
    /// Returns the kind of node as a number, which tells apart two nodes of
    /// one path, such as a field and its getter.
    pub(crate) fn rank(self) -> u8 {
        match self {
            Node::Module => 0,
            Node::Decl(_) => 1,
            Node::Field(_) => 2,
            Node::Variant(..) => 3,
            Node::Func(_) => 4,
            Node::Value => 5,
        }
    }
}

/// One doc text of the run, with what it documents.
pub(crate) struct Doc<'a> {
    pub(crate) item: ItemId,
    /// What the subject path adds to the item's: a field's variant and
    /// name, or [`ROOT`] for a crate's root module.
    pub(crate) part: Vec<String>,
    pub(crate) loc: Loc,
    pub(crate) node: Node,
    /// The module that holds the documented node.
    pub(crate) module: ModId,
    pub(crate) vis: Vis,
    /// The node's flattened notes, whose names resolve.
    pub(crate) notes: &'a [Note],
    pub(crate) prose: Prose<'a>,
}

impl Doc<'_> {
    /// Adds the documented node to a lead as a subject, located where it
    /// is written.
    pub(crate) fn add(&self, run: &AnalysisRun<'_>, lead: Lead) -> Lead {
        lead.part(run, self.item, &self.part)
            .at(self.item.target, self.loc)
    }

    /// Returns the subject path of the documented node.
    pub(crate) fn path(&self, run: &AnalysisRun<'_>) -> Vec<String> {
        let mut path = run.path(self.item);
        path.extend(self.part.iter().cloned());
        path
    }

    /// Returns the names that the doc's flattened notes resolve.
    pub(crate) fn refs(&self) -> impl Iterator<Item = RefId> + '_ {
        let lines = self.notes.iter().flat_map(|note| match note {
            Note::Doc(lines) => lines.as_slice(),
            Note::Plain(_) => &[],
        });
        lines.flat_map(Text::refs)
    }
}

/// Every doc of a run, and the catalog its detectors share.
pub(crate) struct Docs<'a> {
    pub(crate) catalog: Catalog<'a>,
    /// Every doc text of the run: each target's modules', values',
    /// declarations', variants', and associated items', then the catalog's
    /// fields' and functions'.
    pub(crate) all: Vec<Doc<'a>>,
    /// Each declaration of the run, by its name as [`Tok::Name`] holds it,
    /// such as `#3::parse::Args`.
    ///
    /// [`Tok::Name`]: crate::leads_types::Tok::Name
    decls: BTreeMap<String, (usize, DeclId)>,
}

impl<'a> Docs<'a> {
    pub(crate) fn new(run: &AnalysisRun<'a>) -> Docs<'a> {
        let catalog = Catalog::new(run);
        let mut docs = Vec::new();
        let mut decls = BTreeMap::new();
        let krates: &'a [Crate] = run.krates;
        for (t, krate) in krates.iter().enumerate() {
            nodes(run, t, krate, &mut docs);
            for d in 0..krate.decls.len() {
                let at = ItemId {
                    target: t,
                    item: ItemRef::Decl(d),
                };
                decls.insert(item_name(run, at), (t, d));
            }
        }
        for (f, field) in catalog.fields.iter().enumerate() {
            let module = match field.decl.item {
                ItemRef::Decl(d) => run.krates[field.decl.target].decls[d].module,
                _ => 0,
            };
            let place = (field.decl, field.part(run), field.loc);
            docs.extend(doc(place, Node::Field(f), module, field.vis, field.meta));
        }
        for (f, func) in catalog.funcs.iter().enumerate() {
            if let (Some(meta), Some(loc)) = (run.meta(func.item), run.loc(func.item)) {
                let place = (func.item, Vec::new(), loc);
                docs.extend(doc(place, Node::Func(f), func.module, func.vis, meta));
            }
        }
        Docs {
            catalog,
            all: docs,
            decls,
        }
    }

    /// Returns the declaration that a name of a compared type reaches.
    pub(crate) fn decl(&self, name: &str) -> Option<(usize, DeclId)> {
        self.decls.get(name).copied()
    }

    /// Tells whether a compared type may hold what one of `heads` names:
    /// it is opaque or generic, names a type alias of the run, or names a
    /// type whose last segment holds one of `heads`.
    pub(crate) fn may_hold(&self, run: &AnalysisRun<'_>, ty: &Ty, heads: &[&str]) -> bool {
        let alias = |name: &str| {
            self.decl(name)
                .is_some_and(|(t, d)| run.krates[t].decls[d].kind == DeclKind::Alias)
        };
        ty.is_opaque()
            || ty.is_generic()
            || ty.names().any(|name| {
                let last = leaf(name);
                alias(name) || heads.iter().any(|head| last.contains(head))
            })
    }
}

/// Builds the doc of a node, at its item, subject part, and location,
/// when it has doc text.
fn doc(
    (item, part, loc): (ItemId, Vec<String>, Loc),
    node: Node,
    module: ModId,
    vis: Vis,
    meta: &Meta,
) -> Option<Doc<'_>> {
    Some(Doc {
        item,
        part,
        loc,
        node,
        module,
        vis,
        notes: &meta.notes,
        prose: read(meta.raw_doc.as_deref()?),
    })
}

/// Adds the docs of a crate's modules, declarations, variants, and values
/// that the catalog does not hold.
fn nodes<'a>(run: &AnalysisRun<'a>, t: usize, krate: &'a Crate, docs: &mut Vec<Doc<'a>>) {
    let at = |item| ItemId { target: t, item };
    for (m, module) in krate.modules.iter().enumerate() {
        let item = at(ItemRef::Module(m));
        let part = if m == 0 {
            vec![ROOT.to_owned()]
        } else {
            Vec::new()
        };
        if let (Some(raw), Some(loc)) = (&module.raw_doc, run.loc(item)) {
            docs.push(Doc {
                item,
                part,
                loc,
                node: Node::Module,
                module: m,
                vis: module.vis,
                notes: &module.docs,
                prose: read(raw),
            });
        }
        for (k, value) in module.items.iter().enumerate() {
            if matches!(
                value.kind,
                ItemKind::Const { .. } | ItemKind::Static { .. } | ItemKind::Foreign { .. }
            ) {
                let item = at(ItemRef::Value(ValueRef { module: m, item: k }));
                let place = (item, Vec::new(), value.loc);
                docs.extend(doc(place, Node::Value, m, value.vis, &value.meta));
            }
        }
    }
    let valued = |kind: &MemberKind| matches!(kind, MemberKind::Const { .. } | MemberKind::Type(_));
    for (d, decl) in krate.decls.iter().enumerate() {
        let place = (at(ItemRef::Decl(d)), Vec::new(), decl.loc);
        docs.extend(doc(place, Node::Decl(d), decl.module, decl.vis, &decl.meta));
        if decl.kind == DeclKind::Trait {
            for (k, member) in decl.members.iter().enumerate() {
                if valued(&member.kind) {
                    let place = (at(ItemRef::Own(d, k)), Vec::new(), member.loc);
                    docs.extend(doc(place, Node::Value, decl.module, decl.vis, &member.meta));
                }
            }
        }
        if let Shape::Enum(variants) = &decl.shape {
            for (v, variant) in variants.iter().enumerate() {
                let place = (at(ItemRef::Variant(d, v)), Vec::new(), variant.loc);
                let node = Node::Variant(d, v);
                docs.extend(doc(place, node, decl.module, Vis::Public, &variant.meta));
            }
        }
    }
    for (i, imp) in krate.impls.iter().enumerate() {
        let module =
            (run.attached[t][i]).map_or_else(|| impl_module(krate, i), |d| krate.decls[d].module);
        let inherent = (imp.members.iter().enumerate()).map(|(k, m)| (ItemRef::Inherent(i, k), m));
        let traited =
            (imp.trait_members.iter().enumerate()).map(|(k, m)| (ItemRef::TraitImpl(i, k), m));
        for (item, member) in inherent.chain(traited) {
            if valued(&member.kind) {
                let place = (at(item), Vec::new(), member.loc);
                docs.extend(doc(place, Node::Value, module, member.vis, &member.meta));
            }
        }
    }
}

/// Returns the type of a value: a constant's, a static's, or an
/// associated constant's or type's.
pub(crate) fn value_type<'a>(run: &AnalysisRun<'a>, item: ItemId) -> Option<&'a Text> {
    let krate: &'a Crate = run.krates.get(item.target)?;
    match item.item {
        ItemRef::Value(at) => match &krate.modules.get(at.module)?.items.get(at.item)?.kind {
            ItemKind::Const { ty, .. }
            | ItemKind::Static { ty, .. }
            | ItemKind::Foreign {
                item: Foreign::Static { ty, .. },
                ..
            } => Some(ty),
            _ => None,
        },
        _ => match &run.member(item.target, item.item)?.kind {
            MemberKind::Const { ty, .. } | MemberKind::Type(ty) => Some(ty),
            _ => None,
        },
    }
}

/// Tells whether declaration `d` of target `t` is an error type: a struct,
/// enum, or union named with `Error`, or one with a `std::error::Error`
/// impl.
pub(crate) fn is_error(run: &AnalysisRun<'_>, t: usize, d: DeclId) -> bool {
    let decl = &run.krates[t].decls[d];
    let name = strip_raw(&decl.name);
    let typed = matches!(
        decl.kind,
        DeclKind::Struct | DeclKind::Enum | DeclKind::Union
    );
    let named = name.ends_with("Error") || name.ends_with("Errors");
    typed
        && (named
            || impls_of_trait(run, t, d, "std::error::Error")
                .next()
                .is_some())
}

/// Shows a field or a type as a reader sees it, such as `names:
/// Vec<String>`.
pub(crate) fn typed(name: &str, ty: &Ty) -> String {
    format!("`{name}: {}`", ty.shown())
}

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let docs = Docs::new(run);
    leads_docs_phrases::find(run, &docs, out);
    leads_docs_lists::find(run, &docs, out);
    leads_docs_repeat::find(run, &docs, out);
    leads_docs_refs::find(run, &docs, out);
    leads_docs_claims::find(run, &docs, out);
}
