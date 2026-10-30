//! The catalog records of modules and what they hold: each module's own
//! record, its re-exports, constants, statics, free functions, and extern
//! items, every macro definition and invocation, and each construct the
//! outline does not know. The function and value records of members are
//! built here too.

use crate::analysis_items::ItemRef;
use crate::leads_words::split_param;
use crate::macros;
use crate::model::{
    self, Holder, ItemKind, MacroKind, MemberKind, Meta, ModId, Sig, ValueRef, Vis, name_order,
};
use crate::render_slop::{Doc, Outline, Rank};
use crate::render_slop_catalog::{
    ExternRecord, FnRecord, MacroRecord, ModRecord, ParamDoc, UnsupportedRecord, UseRecord,
    ValueRecord, VisCounts,
};
use crate::render_slop_decl::impl_owner;
use crate::render_slop_facts::{comment, flag, module_path, some};
use crate::render_slop_schema::Record;
use crate::render_slop_text::{
    const_head, extern_item, literal, reexport, rust, sig as sig_text, static_head,
};
use crate::test_support::is_test_support;

/// Where a member or a module item sits, and what its record needs.
pub(crate) struct Place<'m> {
    pub(crate) item: ItemRef,
    pub(crate) path: String,
    pub(crate) meta: &'m Meta,
    pub(crate) loc: model::Loc,
    pub(crate) module: ModId,
    /// The header of the trait impl that holds it.
    pub(crate) impl_header: Option<String>,
    pub(crate) test_support: bool,
}

impl Place<'_> {
    /// Returns `Some(true)` when the node is test support: its owner is, or
    /// its own `cfg`s make it so, as for a declaration, and `None` otherwise.
    pub(crate) fn support(&self) -> Option<bool> {
        flag(self.test_support || is_test_support(&self.meta.cfgs))
    }
}

/// Returns a path below the crate root: a module's, then a name.
pub(crate) fn under(path: &[String], name: &str) -> String {
    let mut path = path.to_vec();
    path.push(name.to_owned());
    path.join("::")
}

/// Adds every module's records, its items' records, and every macro's.
pub(crate) fn modules(outline: &mut Outline<'_>) {
    let krate = outline.krate();
    for (m, module) in krate.modules.iter().enumerate() {
        module_record(outline, m);
        for (k, item) in module.items.iter().enumerate() {
            let at = ItemRef::Value(ValueRef { module: m, item: k });
            let place = Place {
                item: at,
                path: under(&module.path, &item.name),
                meta: &item.meta,
                loc: item.loc,
                module: m,
                impl_header: None,
                test_support: module.test_support,
            };
            item_record(outline, &place, &item.kind);
        }
    }
    macros(outline);
}

/// Adds a module's own record, with its doc after it.
fn module_record(outline: &mut Outline<'_>, m: ModId) {
    let krate = outline.krate();
    let module = &krate.modules[m];
    let loc = module.inline.unwrap_or(model::Loc {
        file: module.file,
        line: 1,
    });
    let types = krate.decls.iter().filter(|d| d.module == m).count();
    let mut functions = VisCounts::default();
    for item in module
        .items
        .iter()
        .filter(|i| matches!(i.kind, ItemKind::Fn(_)))
    {
        let slot = match item.vis {
            Vis::Public => &mut functions.public,
            Vis::Internal => &mut functions.internal,
            Vis::Private => &mut functions.private,
        };
        *slot.get_or_insert(0) += 1;
    }
    let any = functions.public.is_some() || functions.internal.is_some();
    let path = module_path(krate, m);
    let record = ModRecord {
        path: path.clone(),
        loc: outline.own(loc),
        vis: (m != 0).then(|| {
            match module.vis {
                Vis::Public => "public",
                Vis::Internal => "internal",
                Vis::Private => "private",
            }
            .to_owned()
        }),
        cfgs: some(module.cfgs.clone()),
        lints: some(module.lints.clone()),
        test_support: flag(module.test_support),
        types: (types > 0).then_some(types),
        functions: (any || functions.private.is_some()).then_some(functions),
        comment: comment(&module.docs, None),
        dangling: comment(&module.dangling, None),
    };
    let doc = Doc {
        path: &path,
        text: module.raw_doc.as_deref(),
    };
    outline.push_ranked(Rank::Own, m, Record::Mod(record), doc);
}

/// Adds a module item's record: a re-export, a constant, a static, a free
/// function, an extern item, or an unsupported construct. A macro's record
/// comes from the crate's macro list.
fn item_record(outline: &mut Outline<'_>, place: &Place<'_>, kind: &ItemKind) {
    let krate = outline.krate();
    let meta = place.meta;
    match kind {
        ItemKind::Reexport { text, .. } => {
            let rust = reexport(text, meta, &krate.refs);
            let record = UseRecord {
                path: place.path.clone(),
                rust: rust.clone(),
                loc: outline.own(place.loc),
                cfgs: some(meta.cfgs.clone()),
                test_support: place.support(),
                comment: comment(&meta.notes, meta.trailing.as_ref()),
            };
            let doc = Doc {
                path: &place.path,
                text: None,
            };
            let rank = Rank::Reexport(format!("{rust} {}", meta.cfgs.join(" ")));
            outline.push_ranked(rank, place.module, Record::Use(record), doc);
        }
        ItemKind::Mark(_) => {}
        ItemKind::Const { ty, value } => {
            let name = place.path.rsplit("::").next().unwrap_or_default();
            let head = const_head(name, ty, meta, &krate.refs);
            push_value(outline, place, false, head, value.as_deref());
        }
        ItemKind::Static { mutable, ty, value } => {
            let name = place.path.rsplit("::").next().unwrap_or_default();
            let head = static_head(name, *mutable, ty, meta, &krate.refs);
            push_value(outline, place, true, head, value.as_deref());
        }
        ItemKind::Fn(sig) => push_fn(outline, place, sig),
        ItemKind::Foreign { abi, item } => {
            let name = place.path.rsplit("::").next().unwrap_or_default();
            let record = ExternRecord {
                path: place.path.clone(),
                rust: extern_item(abi, name, item, meta, &krate.refs),
                loc: outline.own(place.loc),
                cfgs: some(meta.cfgs.clone()),
                attrs: some(meta.attrs.clone()),
                lints: some(meta.lints.clone()),
                test_support: place.support(),
                comment: comment(&meta.notes, meta.trailing.as_ref()),
            };
            let doc = Doc {
                path: &place.path,
                text: meta.raw_doc.as_deref(),
            };
            outline.push(place.module, place.loc, Record::Extern(record), doc);
        }
        ItemKind::Unsupported(what) => {
            let path = module_path(krate, place.module);
            push_unsupported(outline, place.module, path, what, place.loc);
        }
    }
}

/// Adds the record of a construct the outline does not know.
pub(crate) fn push_unsupported(
    outline: &mut Outline<'_>,
    module: ModId,
    path: String,
    what: &str,
    loc: model::Loc,
) {
    let record = UnsupportedRecord {
        path,
        text: what.to_owned(),
        loc: outline.own(loc),
    };
    let doc = Doc {
        path: "",
        text: None,
    };
    outline.push(module, loc, Record::Unsupported(record), doc);
}

/// Returns the attribute macros among a node's marks, sorted.
fn attributes(meta: &Meta) -> Option<Vec<String>> {
    let mut names: Vec<String> = (meta.marks.iter())
        .filter_map(|mark| macros::mark_parts(mark))
        .filter(|(kind, _)| *kind == macros::ATTRIBUTE)
        .flat_map(|(_, names)| names)
        .map(str::to_owned)
        .collect();
    names.sort_by(|a, b| name_order(a, b));
    names.dedup();
    some(names)
}

/// Returns the doc and comments above each parameter that has some.
fn param_docs(sig: &Sig, refs: &[model::TypeRef]) -> Option<Vec<ParamDoc>> {
    let docs = sig.params.iter().filter_map(|param| {
        let lines: Vec<String> = (param.meta.raw_doc.iter().cloned())
            .chain(comment(&param.meta.notes, None))
            .collect();
        if lines.is_empty() {
            return None;
        }
        let name = split_param(&param.text).map_or_else(|| rust(&param.text, refs), |(p, _)| p);
        Some(ParamDoc {
            param: name,
            doc: lines.join("\n"),
        })
    });
    some(docs.collect())
}

/// Adds a function's record: free, inherent, a trait impl's, or a trait's
/// own.
pub(crate) fn push_fn(outline: &mut Outline<'_>, place: &Place<'_>, sig: &Sig) {
    let krate = outline.krate();
    let meta = place.meta;
    let facts = &outline.facts;
    let (crates, tests) = facts.observed(place.item);
    let (body, ops, comments) = facts.body(place.item);
    let record = FnRecord {
        path: place.path.clone(),
        sig: sig_text(sig, &meta.vis, &krate.refs),
        loc: outline.own(place.loc),
        impl_header: place.impl_header.clone(),
        cfgs: some(meta.cfgs.clone()),
        attrs: some(meta.attrs.clone()),
        attributes: attributes(meta),
        lints: some(meta.lints.clone()),
        test_support: place.support(),
        param_docs: param_docs(sig, &krate.refs),
        direct_callers: facts.direct_callers(place.item),
        reexported_by: facts.reexported_by(place.item),
        crates,
        tests,
        body,
        ops,
        comments,
        comment: comment(&meta.notes, meta.trailing.as_ref()),
    };
    let doc = Doc {
        path: &place.path,
        text: meta.raw_doc.as_deref(),
    };
    outline.push(place.module, place.loc, Record::Fn(record), doc);
}

/// Adds a constant's or a static's record, its head written already.
pub(crate) fn push_value(
    outline: &mut Outline<'_>,
    place: &Place<'_>,
    is_static: bool,
    head: String,
    value: Option<&str>,
) {
    let meta = place.meta;
    let facts = &outline.facts;
    let (crates, tests) = facts.observed(place.item);
    let (body, _, comments) = facts.body(place.item);
    let record = ValueRecord {
        path: place.path.clone(),
        rust: head,
        value: value.and_then(literal),
        loc: outline.own(place.loc),
        impl_header: place.impl_header.clone(),
        cfgs: some(meta.cfgs.clone()),
        attrs: some(meta.attrs.clone()),
        lints: some(meta.lints.clone()),
        test_support: place.support(),
        reexported_by: facts.reexported_by(place.item),
        crates,
        tests,
        body,
        comments,
        comment: comment(&meta.notes, meta.trailing.as_ref()),
    };
    let record = if is_static {
        Record::Static(record)
    } else {
        Record::Const(record)
    };
    let doc = Doc {
        path: &place.path,
        text: meta.raw_doc.as_deref(),
    };
    outline.push(place.module, place.loc, record, doc);
}

/// Adds a record for each macro definition and invocation of the crate:
/// a definition under its own name, an invocation under what holds it.
fn macros(outline: &mut Outline<'_>) {
    let krate = outline.krate();
    for mark in &krate.macros {
        if !matches!(mark.kind, MacroKind::Definition | MacroKind::Invocation) {
            continue;
        }
        let (module, mut path, test_support, meta) = match mark.holder {
            Holder::Module(m) => {
                let module = &krate.modules[m];
                let meta = (module.items.iter())
                    .find(|item| {
                        item.loc == mark.loc
                            && matches!(&item.kind, ItemKind::Mark(t) if *t == mark.text)
                    })
                    .map(|item| &item.meta);
                (m, module.path.clone(), module.test_support, meta)
            }
            Holder::Decl(d) => {
                let decl = &krate.decls[d];
                let meta = (decl.members.iter())
                    .find(|m| {
                        m.loc == mark.loc
                            && matches!(&m.kind, MemberKind::Mark(t) if *t == mark.text)
                    })
                    .map(|member| &member.meta);
                (decl.module, krate.decl_path(d), decl.test_support, meta)
            }
            Holder::Impl(i) => {
                let (module, path, test_support) = impl_owner(outline, i);
                (module, path, test_support, None)
            }
        };
        if let Some(name) = macros::definition_name(&mark.text) {
            path.push(name.to_owned());
        }
        let shown = if path.is_empty() {
            "crate".to_owned()
        } else {
            path.join("::")
        };
        let empty = Meta::default();
        let meta = meta.unwrap_or(&empty);
        let record = MacroRecord {
            path: shown.clone(),
            rust: mark.text.clone(),
            loc: outline.own(mark.loc),
            cfgs: some(meta.cfgs.clone()),
            attrs: some(meta.attrs.clone()),
            test_support: flag(test_support || is_test_support(&meta.cfgs)),
            comment: comment(&meta.notes, meta.trailing.as_ref()),
        };
        let doc = Doc {
            path: &shown,
            text: meta.raw_doc.as_deref(),
        };
        outline.push(module, mark.loc, Record::Macro(record), doc);
    }
}
