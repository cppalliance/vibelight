//! The facts lines under each declaration's heading, when the facts are on:
//! what the declaration holds, and what names it.

use crate::document::{Builder, Inline, Node, anchor};
use crate::model::{Decl, DeclId, MemberKind, Shape, Text, TypeRef, VariantFields};

impl Builder<'_> {
    /// Adds declaration `d`'s `Counts:` and `Used by:` lines, when the
    /// facts are on.
    pub(crate) fn facts(&mut self, d: DeclId) {
        let Some(order) = self.facts else {
            return;
        };
        let counts = counts(&self.krate.decls[d], &self.krate.refs);
        self.nodes.push(Node::Para {
            class: "facts",
            inlines: vec![Inline::Text(format!("Counts: {counts}"))],
        });
        let namers = &order.namers[d];
        let mut links: Vec<(String, String)> = namers
            .decls
            .iter()
            .map(|&n| (self.decl_heading(n), self.decl_ids[n].clone()))
            .collect();
        links.extend(
            namers
                .modules
                .iter()
                .map(|&m| (self.mod_heading(m), self.mod_ids[m].clone())),
        );
        let mut inlines = vec![Inline::Text("Used by: ".to_owned())];
        if links.is_empty() {
            inlines.push(Inline::Text("nothing".to_owned()));
        }
        for (index, (text, id)) in links.iter().enumerate() {
            if index > 0 {
                inlines.push(Inline::Text(", ".to_owned()));
            }
            inlines.push(anchor(text, id));
        }
        self.nodes.push(Node::Para {
            class: "facts",
            inlines,
        });
    }
}

/// Returns what a declaration holds, such as `2 variants, 3 functions`:
/// its variants, its fields with how many are an `Option`, its associated
/// types and constants, its special members, and its functions.
fn counts(decl: &Decl, refs: &[TypeRef]) -> String {
    let mut fields: Vec<&Text> = decl
        .members
        .iter()
        .filter_map(|m| match &m.kind {
            MemberKind::Field(ty) => Some(ty),
            _ => None,
        })
        .collect();
    if let Shape::Tuple(params) = &decl.shape {
        fields.extend(params.iter().map(|p| &p.text));
    }
    let options = fields.iter().filter(|ty| is_option(ty, refs)).count();
    let variants = match &decl.shape {
        Shape::Enum(variants) => variants.len(),
        _ => 0,
    };
    let associated = decl
        .members
        .iter()
        .filter(|m| matches!(m.kind, MemberKind::Type(_) | MemberKind::Const { .. }))
        .count();
    let functions = decl
        .members
        .iter()
        .filter(|m| matches!(m.kind, MemberKind::Fn(_)))
        .count();
    let mut parts = Vec::new();
    if variants > 0 {
        parts.push(count(variants, "variant", "variants"));
    }
    if !fields.is_empty() {
        let part = count(fields.len(), "field", "fields");
        parts.push(if options > 0 {
            format!("{part} ({options} Option)")
        } else {
            part
        });
    }
    if associated > 0 {
        parts.push(count(associated, "associated item", "associated items"));
    }
    if !decl.special.is_empty() {
        parts.push(count(
            decl.special.len(),
            "special member",
            "special members",
        ));
    }
    if functions > 0 {
        parts.push(count(functions, "function", "functions"));
    }
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Tells whether a type is an `Option`, however its path is written.
fn is_option(ty: &Text, refs: &[TypeRef]) -> bool {
    let text = ty.visible(refs);
    let head = text.split('<').next().unwrap_or_default();
    head.rsplit("::").next() == Some("Option")
}

/// Returns one variant's name and field types, as a shape compares them.
pub(crate) fn variant_text(name: &str, fields: &VariantFields, refs: &[TypeRef]) -> String {
    let types = |params: &[crate::model::Param]| -> Vec<String> {
        params.iter().map(|p| p.text.visible(refs)).collect()
    };
    match fields {
        VariantFields::Unit => name.to_owned(),
        VariantFields::Tuple(params) => format!("{name}({})", types(params).join(", ")),
        VariantFields::Named(params) => format!("{name} {{ {} }}", types(params).join(", ")),
    }
}
