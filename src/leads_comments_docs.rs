//! The papercuts of a run's raw doc text: each doc of a node that holds a
//! papercut word or phrase outside its code spans and its boilerplate
//! passages.

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef, is_value};
use crate::leads::{Lead, Mode};
use crate::leads_comments::{PAPERCUT_PHRASES, hits, quoted, shown};
use crate::leads_comments_words::{prose_words, without_code};
use crate::leads_docs_text::{boilerplate, without};
use crate::leads_words::split_param;
use crate::model::{Crate, Loc, Param, Shape, ValueRef, VariantFields};
use crate::typerefs::strip_raw;

/// One raw doc text of the run and the node it documents.
struct Doc<'a> {
    item: ItemId,
    /// What the node's subject path adds to the item's: the position of a
    /// tuple field, or a variant's name and its field's name or position.
    part: Vec<String>,
    /// Where a field is written: its declaration's or its variant's line.
    /// `None` for an item, which the item locates.
    loc: Option<Loc>,
    text: &'a str,
}

/// Returns every raw doc text of target `t`: of its modules but the root,
/// which names no subject, its module values, its declarations with their
/// own members, variants, and fields, and the members of its impls.
fn docs<'a>(t: usize, krate: &'a Crate) -> Vec<Doc<'a>> {
    let mut out = Vec::new();
    let at = |item| ItemId { target: t, item };
    let mut add = |item: ItemRef, part: Vec<String>, loc: Option<Loc>, doc: &'a Option<String>| {
        if let Some(text) = doc {
            out.push(Doc {
                item: at(item),
                part,
                loc,
                text,
            });
        }
    };
    for (m, module) in krate.modules.iter().enumerate() {
        if !module.path.is_empty() {
            add(ItemRef::Module(m), Vec::new(), None, &module.raw_doc);
        }
        for (k, item) in module.items.iter().enumerate() {
            if is_value(&item.kind) {
                let value = ItemRef::Value(ValueRef { module: m, item: k });
                add(value, Vec::new(), None, &item.meta.raw_doc);
            }
        }
    }
    for (d, decl) in krate.decls.iter().enumerate() {
        add(ItemRef::Decl(d), Vec::new(), None, &decl.meta.raw_doc);
        for (k, member) in decl.members.iter().enumerate() {
            add(ItemRef::Own(d, k), Vec::new(), None, &member.meta.raw_doc);
        }
        let fields = |params: &'a [Param], named: bool| {
            (params.iter().enumerate()).map(move |(k, param)| {
                let name = (split_param(&param.text).filter(|_| named))
                    .map_or_else(|| k.to_string(), |(name, _)| strip_raw(&name));
                (name, &param.meta.raw_doc)
            })
        };
        match &decl.shape {
            Shape::Tuple(params) => {
                for (name, doc) in fields(params, false) {
                    add(ItemRef::Decl(d), vec![name], Some(decl.loc), doc);
                }
            }
            Shape::Enum(variants) => {
                for (v, variant) in variants.iter().enumerate() {
                    add(
                        ItemRef::Variant(d, v),
                        Vec::new(),
                        None,
                        &variant.meta.raw_doc,
                    );
                    let (params, named) = match &variant.fields {
                        VariantFields::Named(params) => (params, true),
                        VariantFields::Tuple(params) => (params, false),
                        VariantFields::Unit => continue,
                    };
                    for (name, doc) in fields(params, named) {
                        let part = vec![variant.name.clone(), name];
                        add(ItemRef::Decl(d), part, Some(variant.loc), doc);
                    }
                }
            }
            Shape::Named | Shape::Trait { .. } | Shape::Unit | Shape::Alias(_) => {}
        }
    }
    for (i, imp) in krate.impls.iter().enumerate() {
        for (k, member) in imp.members.iter().enumerate() {
            add(
                ItemRef::Inherent(i, k),
                Vec::new(),
                None,
                &member.meta.raw_doc,
            );
        }
        for (k, member) in imp.trait_members.iter().enumerate() {
            add(
                ItemRef::TraitImpl(i, k),
                Vec::new(),
                None,
                &member.meta.raw_doc,
            );
        }
    }
    out
}

/// `papercut` in docs: a node's doc that holds a papercut word or phrase
/// outside its code spans, shown by the first line that holds one.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        for doc in docs(t, krate) {
            let text = without(doc.text, &boilerplate(doc.text, &run.boilerplate));
            let mut found: Vec<&str> = Vec::new();
            let mut first = None;
            for line in text.lines() {
                for phrase in hits(&prose_words(&without_code(line)), &PAPERCUT_PHRASES) {
                    if !found.contains(&phrase) {
                        found.push(phrase);
                        first.get_or_insert(line);
                    }
                }
            }
            let Some(line) = first else {
                continue;
            };
            let evidence = format!("{} in the doc line \"{}\"", quoted(&found), shown(line));
            let lead = Lead::new("papercut", Mode::DirectSyntax, evidence);
            out.push(match doc.loc {
                Some(loc) => lead.part(run, doc.item, &doc.part).at(t, loc),
                None => lead.item(run, doc.item),
            });
        }
    }
}
