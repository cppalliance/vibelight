//! The Same shapes section, when the facts are on: declarations of one
//! kind whose variants or fields have the same names and types, which are
//! candidates for merging.

use std::collections::BTreeMap;

use crate::document::{Builder, Inline, Item, Node, anchor};
use crate::document_facts::variant_text;
use crate::model::{Decl, DeclId, DeclKind, MemberKind, Shape, TypeRef, name_order};
use crate::order::path_cmp;

impl Builder<'_> {
    /// Adds the Same shapes section and returns its contents entry, when
    /// the facts are on and two declarations share a shape.
    pub(crate) fn shapes(&mut self) -> Option<Item> {
        self.facts?;
        let mut groups: BTreeMap<String, (String, Vec<DeclId>)> = BTreeMap::new();
        for (d, decl) in self.krate.decls.iter().enumerate() {
            if let Some((key, label)) = shape(decl, &self.krate.refs) {
                groups.entry(key).or_insert((label, Vec::new())).1.push(d);
            }
        }
        let mut lists: Vec<(String, Vec<DeclId>)> = groups
            .into_values()
            .filter(|(_, decls)| decls.len() > 1)
            .collect();
        if lists.is_empty() {
            return None;
        }
        let cmp = |a: DeclId, b: DeclId| path_cmp(self.krate, &self.texts, a, b);
        for (_, decls) in &mut lists {
            decls.sort_by(|&a, &b| cmp(a, b));
        }
        lists.sort_by(|a, b| cmp(a.1[0], b.1[0]));
        let items = lists
            .iter()
            .map(|(label, decls)| {
                let mut inlines = vec![Inline::Text(format!("{label}: "))];
                for (index, &d) in decls.iter().enumerate() {
                    if index > 0 {
                        inlines.push(Inline::Text(", ".to_owned()));
                    }
                    inlines.push(anchor(&self.decl_heading(d), &self.decl_ids[d]));
                }
                Item {
                    inlines,
                    children: Vec::new(),
                }
            })
            .collect();
        self.heading(2, "shapes", "Same shapes");
        self.nodes.push(Node::List {
            class: "shapes",
            items,
        });
        Some(Item {
            inlines: vec![anchor("Same shapes", "shapes")],
            children: Vec::new(),
        })
    }
}

/// Returns the key that declarations of one shape share, and the label of
/// their group, such as `enums with the same 3 variants`. An enum compares
/// its variants in declared order, and a struct or union its fields by
/// name; docs never count. A declaration with fewer than two variants or
/// fields has no shape.
fn shape(decl: &Decl, refs: &[TypeRef]) -> Option<(String, String)> {
    let (kind, noun, entries): (&str, &str, Vec<String>) = match (decl.kind, &decl.shape) {
        (DeclKind::Enum, Shape::Enum(variants)) => (
            "enums",
            "variants",
            variants
                .iter()
                .map(|v| variant_text(&v.name, &v.fields, refs))
                .collect(),
        ),
        (DeclKind::Struct | DeclKind::Union, Shape::Named) => {
            let mut fields: Vec<(&str, String)> = decl
                .members
                .iter()
                .filter_map(|m| match &m.kind {
                    MemberKind::Field(ty) => Some((m.name.as_str(), ty.visible(refs))),
                    _ => None,
                })
                .collect();
            fields.sort_by(|a, b| name_order(a.0, b.0).then_with(|| a.1.cmp(&b.1)));
            let kind = if decl.kind == DeclKind::Union {
                "unions"
            } else {
                "structs"
            };
            let entries = fields
                .into_iter()
                .map(|(name, ty)| format!("{name}: {ty}"))
                .collect();
            (kind, "fields", entries)
        }
        (DeclKind::Struct, Shape::Tuple(params)) => (
            "tuple structs",
            "fields",
            params.iter().map(|p| p.text.visible(refs)).collect(),
        ),
        _ => return None,
    };
    (entries.len() > 1).then(|| {
        let label = format!("{kind} with the same {} {noun}", entries.len());
        (format!("{kind}\n{}", entries.join("\n")), label)
    })
}
