//! The catalog records of declarations and impls: each struct, enum,
//! union, trait, and alias with its whole shape, each field and variant
//! that has comments of its own, each item of a trait, and each impl with
//! its members. An impl belongs to its type's module, as attach would move
//! it, and impls of one owner with one header share one record.

use std::collections::BTreeMap;

use crate::analysis_items::ItemRef;
use crate::leads_items::impl_module;
use crate::leads_words::split_param;
use crate::model::{
    self, DeclId, DeclKind, Member, MemberKind, Meta, ModId, Param, Shape, VariantFields,
    name_order,
};
use crate::render_slop::{Doc, Outline};
use crate::render_slop_catalog::{
    AliasRecord, FieldRecord, ImplRecord, TraitRecord, TypeRecord, VariantRecord,
};
use crate::render_slop_facts::{comment, flag, note_doc, some};
use crate::render_slop_items::{Place, push_fn, push_unsupported, push_value, under};
use crate::render_slop_schema::Record;
use crate::render_slop_text::{member_head, named_field, shape, tuple_field, variant};
use crate::test_support::is_test_support;

/// Returns the module an impl's records belong to, the path of its owner,
/// and whether the owner is test support: the declaration it belongs to,
/// or, for an impl that belongs to none, its self type as written.
pub(crate) fn impl_owner(outline: &Outline<'_>, i: usize) -> (ModId, Vec<String>, bool) {
    let krate = outline.krate;
    if let Some(d) = outline.run.attached[outline.t][i] {
        let decl = &krate.decls[d];
        return (decl.module, krate.decl_path(d), decl.test_support);
    }
    let m = impl_module(krate, i);
    let owner = krate.impls[i].self_text.rust_visible(&krate.refs);
    (m, vec![owner], krate.modules[m].test_support)
}

/// Tells whether a node has comments of its own: a doc or a plain one.
fn commented(meta: &Meta) -> bool {
    meta.raw_doc.is_some() || comment(&meta.notes, meta.trailing.as_ref()).is_some()
}

/// Adds every declaration's records: its own, its commented fields and
/// variants, its trait items, and its unsupported members.
pub(crate) fn decls(outline: &mut Outline<'_>) {
    let krate = outline.krate;
    for (d, decl) in krate.decls.iter().enumerate() {
        decl_record(outline, d);
        parts(outline, d);
        let path = krate.decl_path(d);
        for (k, member) in decl.members.iter().enumerate() {
            let place = Place {
                item: ItemRef::Own(d, k),
                path: under(&path, &member.name),
                meta: &member.meta,
                loc: member.loc,
                module: decl.module,
                impl_header: None,
                test_support: decl.test_support,
            };
            member_record(outline, &place, member, &path);
        }
    }
}

/// Adds a declaration's own record, with its doc after it.
fn decl_record(outline: &mut Outline<'_>, d: DeclId) {
    let krate = outline.krate;
    let decl = &krate.decls[d];
    let facts = &outline.facts;
    let path = krate.decl_path(d).join("::");
    let item = ItemRef::Decl(d);
    let (crates, tests) = facts.observed(item);
    let named_by = facts.named_by(d);
    let reexported_by = facts.reexported_by(item);
    let rust = shape(decl, &krate.refs);
    let loc = outline.own(decl.loc);
    let meta = &decl.meta;
    let comment_text = comment(&meta.notes, meta.trailing.as_ref());
    let dangling = comment(&decl.dangling, None);
    let record = match decl.kind {
        DeclKind::Struct | DeclKind::Enum | DeclKind::Union => {
            let record = TypeRecord {
                path: path.clone(),
                rust,
                loc,
                cfgs: some(meta.cfgs.clone()),
                attrs: some(meta.attrs.clone()),
                derives: some(decl.written_derives.clone()),
                attributes: some(decl.attributes.clone()),
                lints: some(meta.lints.clone()),
                test_support: flag(decl.test_support),
                named_by,
                reexported_by,
                crates,
                tests,
                comment: comment_text,
                dangling,
            };
            match decl.kind {
                DeclKind::Enum => Record::Enum(record),
                DeclKind::Union => Record::Union(record),
                _ => Record::Struct(record),
            }
        }
        DeclKind::Trait => Record::Trait(TraitRecord {
            path: path.clone(),
            rust,
            loc,
            cfgs: some(meta.cfgs.clone()),
            attrs: some(meta.attrs.clone()),
            attributes: some(decl.attributes.clone()),
            lints: some(meta.lints.clone()),
            test_support: flag(decl.test_support),
            named_by,
            reexported_by,
            crates,
            tests,
            comment: comment_text,
            dangling,
        }),
        DeclKind::TraitAlias | DeclKind::Alias => Record::Alias(AliasRecord {
            path: path.clone(),
            rust,
            loc,
            impl_header: None,
            cfgs: some(meta.cfgs.clone()),
            attrs: some(meta.attrs.clone()),
            lints: some(meta.lints.clone()),
            test_support: flag(decl.test_support),
            named_by,
            reexported_by,
            crates,
            tests,
            comment: comment_text,
        }),
    };
    let doc = Doc {
        path: &path,
        text: meta.raw_doc.as_deref(),
    };
    outline.push(decl.module, decl.loc, record, doc);
}

/// Adds a record for each field and variant of a declaration that has
/// comments of its own, or a variant whose discriminant holds comments.
fn parts(outline: &mut Outline<'_>, d: DeclId) {
    let krate = outline.krate;
    let decl = &krate.decls[d];
    let path = krate.decl_path(d);
    let refs = &krate.refs;
    let field = |outline: &mut Outline<'_>, path: String, rust: String, loc, meta: &Meta| {
        let record = FieldRecord {
            path: path.clone(),
            rust,
            loc: outline.own(loc),
            test_support: flag(decl.test_support || is_test_support(&meta.cfgs)),
            comment: comment(&meta.notes, meta.trailing.as_ref()),
        };
        let doc = Doc {
            path: &path,
            text: meta.raw_doc.as_deref(),
        };
        outline.push(decl.module, loc, Record::Field(record), doc);
    };
    let params = |outline: &mut Outline<'_>, owner: &[String], params: &[Param], loc, named| {
        for (k, param) in params
            .iter()
            .enumerate()
            .filter(|(_, p)| commented(&p.meta))
        {
            let name = split_param(&param.text)
                .filter(|_| named)
                .map_or_else(|| k.to_string(), |(pattern, _)| pattern);
            field(
                outline,
                under(owner, &name),
                tuple_field(param, refs),
                loc,
                &param.meta,
            );
        }
    };
    match &decl.shape {
        Shape::Named => {
            for member in decl.members.iter().filter(|m| commented(&m.meta)) {
                if let MemberKind::Field(ty) = &member.kind {
                    let rust = named_field(&member.meta, &member.name, ty, refs);
                    let at = under(&path, &member.name);
                    field(outline, at, rust, member.loc, &member.meta);
                }
            }
        }
        Shape::Tuple(fields) => params(outline, &path, fields, decl.loc, false),
        Shape::Enum(variants) => {
            for (v, item) in variants.iter().enumerate() {
                let comments = outline.facts.comments(ItemRef::Variant(d, v));
                let at = under(&path, &item.name);
                if commented(&item.meta) || comments.is_some() {
                    let record = VariantRecord {
                        path: at.clone(),
                        rust: variant(item, refs),
                        loc: outline.own(item.loc),
                        test_support: flag(decl.test_support || is_test_support(&item.meta.cfgs)),
                        comments,
                        comment: comment(&item.meta.notes, item.meta.trailing.as_ref()),
                    };
                    let doc = Doc {
                        path: &at,
                        text: item.meta.raw_doc.as_deref(),
                    };
                    outline.push(decl.module, item.loc, Record::Variant(record), doc);
                }
                let mut owner = path.clone();
                owner.push(item.name.clone());
                let (fields, named) = match &item.fields {
                    VariantFields::Unit => continue,
                    VariantFields::Tuple(fields) => (fields, false),
                    VariantFields::Named(fields) => (fields, true),
                };
                params(outline, &owner, fields, item.loc, named);
            }
        }
        Shape::Trait { .. } | Shape::Unit | Shape::Alias(_) => {}
    }
}

/// Adds a member's record: a function, an associated constant or type, or
/// an unsupported construct. A field prints in its type's shape, and a
/// macro mark has a macro record.
fn member_record(outline: &mut Outline<'_>, place: &Place<'_>, member: &Member, owner: &[String]) {
    let refs = &outline.krate.refs;
    match &member.kind {
        MemberKind::Fn(sig) => push_fn(outline, place, sig),
        MemberKind::Const { value, .. } => {
            let head = member_head(&member.kind, &member.name, &member.meta, refs);
            push_value(outline, place, false, head, value.as_deref());
        }
        MemberKind::Type(_) => {
            let meta = place.meta;
            let record = AliasRecord {
                path: place.path.clone(),
                rust: member_head(&member.kind, &member.name, meta, refs),
                loc: outline.own(place.loc),
                impl_header: place.impl_header.clone(),
                cfgs: some(meta.cfgs.clone()),
                attrs: some(meta.attrs.clone()),
                lints: some(meta.lints.clone()),
                test_support: place.support(),
                comment: comment(&meta.notes, meta.trailing.as_ref()),
                ..AliasRecord::default()
            };
            let doc = Doc {
                path: &place.path,
                text: meta.raw_doc.as_deref(),
            };
            outline.push(place.module, place.loc, Record::Alias(record), doc);
        }
        MemberKind::Unsupported(what) => {
            push_unsupported(outline, place.module, owner.join("::"), what, place.loc);
        }
        MemberKind::Field(_) | MemberKind::Mark(_) => {}
    }
}

/// Adds the impls' records: one for each owner and header, with every
/// impl block that has them, and then a record for each of their members.
pub(crate) fn impls(outline: &mut Outline<'_>) {
    let krate = outline.krate;
    let mut groups: BTreeMap<(Vec<String>, String), Vec<usize>> = BTreeMap::new();
    for (i, item) in krate.impls.iter().enumerate() {
        let (_, path, _) = impl_owner(outline, i);
        groups
            .entry((path, item.header.clone()))
            .or_default()
            .push(i);
    }
    for ((path, header), list) in groups {
        let (module, _, test_support) = impl_owner(outline, list[0]);
        let mut at: Vec<model::Loc> = list.iter().map(|&i| krate.impls[i].loc).collect();
        let rel = |loc: &model::Loc| krate.files[loc.file].rel.clone();
        at.sort_by(|a, b| rel(a).cmp(&rel(b)).then(a.line.cmp(&b.line)));
        at.dedup();
        let mut attributes = Vec::new();
        let mut macros = Vec::new();
        let mut comments = Vec::new();
        let mut dangling = Vec::new();
        let mut docs = Vec::new();
        for &i in &list {
            let item = &krate.impls[i];
            attributes.extend(item.attributes.iter().cloned());
            macros.extend(item.marks.iter().cloned());
            comments.extend(comment(&item.notes, None));
            dangling.extend(comment(&item.dangling, None));
            docs.extend(note_doc(&item.notes, krate));
        }
        for list in [&mut attributes, &mut macros] {
            list.sort_by(|a, b| name_order(a, b));
            list.dedup();
        }
        let shown = path.join("::");
        let record = ImplRecord {
            path: shown.clone(),
            rust: header,
            locs: at.iter().map(|&loc| outline.own(loc)).collect(),
            attributes: some(attributes),
            macros: some(macros),
            test_support: flag(test_support),
            comment: (!comments.is_empty()).then(|| comments.join("\n")),
            dangling: (!dangling.is_empty()).then(|| dangling.join("\n")),
        };
        let doc = docs.join("\n");
        let doc = Doc {
            path: &shown,
            text: (!doc.is_empty()).then_some(doc.as_str()),
        };
        outline.push(module, at[0], Record::Impl(record), doc);
        for &i in &list {
            impl_members(outline, i, module, &path, test_support);
        }
    }
}

/// Adds the records of one impl's members: an inherent impl's, or a trait
/// impl's under its header.
fn impl_members(outline: &mut Outline<'_>, i: usize, module: ModId, owner: &[String], ts: bool) {
    let krate = outline.krate;
    let item = &krate.impls[i];
    let inherent = (item.members.iter().enumerate()).map(|(k, m)| (ItemRef::Inherent(i, k), m));
    let traits =
        (item.trait_members.iter().enumerate()).map(|(k, m)| (ItemRef::TraitImpl(i, k), m));
    for (at, member) in inherent.chain(traits) {
        let place = Place {
            item: at,
            path: under(owner, &member.name),
            meta: &member.meta,
            loc: member.loc,
            module,
            impl_header: matches!(at, ItemRef::TraitImpl(..)).then(|| item.header.clone()),
            test_support: ts,
        };
        member_record(outline, &place, member, owner);
    }
}
