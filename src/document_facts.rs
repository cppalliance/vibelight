//! The facts lines under each declaration's heading, when the facts are on:
//! what the declaration holds, and what names it.

use std::collections::{BTreeMap, BTreeSet};

use crate::document::{Builder, Inline, Node, anchor};
use crate::model::{Decl, DeclId, MemberKind, Shape, Text, TypeRef, VariantFields, name_order};

impl Builder<'_> {
    /// Adds declaration `d`'s `Counts:`, `Used by:`, and, when its
    /// functions form groups, `Clusters:` lines, when the facts are on.
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
        if let Some(text) = clusters(&self.krate.decls[d]) {
            self.nodes.push(Node::Para {
                class: "facts",
                inlines: vec![Inline::Text(format!("Clusters: {text}"))],
            });
        }
    }
}

/// Returns the text of a type's `Clusters:` line, or `None` when its
/// functions form fewer than two groups. A field that more than half the
/// functions reach through `self` is a hub. Two functions are in one group
/// when a chain of shared fields other than hubs joins them; a function
/// that reaches only hubs, or no field, is in no group. Each group is named
/// after the field most of its functions reach, ties going to the first by
/// name, and the groups sort by that name.
fn clusters(decl: &Decl) -> Option<String> {
    let fields: BTreeSet<&str> = decl
        .members
        .iter()
        .filter(|m| matches!(m.kind, MemberKind::Field(_)))
        .map(|m| m.name.as_str())
        .collect();
    let functions: Vec<(&str, Vec<&str>)> = decl
        .members
        .iter()
        .filter(|m| matches!(m.kind, MemberKind::Fn(_)))
        .map(|m| {
            let reached = m.self_fields.iter().map(String::as_str);
            (
                m.name.as_str(),
                reached.filter(|f| fields.contains(f)).collect(),
            )
        })
        .collect();
    let mut reach: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, reached) in &functions {
        for field in reached {
            *reach.entry(field).or_default() += 1;
        }
    }
    let hubs: Vec<&str> = reach
        .iter()
        .filter(|&(_, &n)| n * 2 > functions.len())
        .map(|(&field, _)| field)
        .collect();
    let mut root: Vec<usize> = (0..functions.len()).collect();
    let mut first: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, (_, reached)) in functions.iter().enumerate() {
        for &field in reached.iter().filter(|f| !hubs.contains(f)) {
            let j = *first.entry(field).or_insert(i);
            let (a, b) = (top(&root, i), top(&root, j));
            root[a.max(b)] = a.min(b);
        }
    }
    let mut groups: BTreeMap<usize, (Vec<&str>, BTreeMap<&str, usize>)> = BTreeMap::new();
    for (i, (name, reached)) in functions.iter().enumerate() {
        let kept: Vec<&str> = reached
            .iter()
            .copied()
            .filter(|f| !hubs.contains(f))
            .collect();
        if kept.is_empty() {
            continue;
        }
        let (names, counts) = groups.entry(top(&root, i)).or_default();
        names.push(name);
        for field in kept {
            *counts.entry(field).or_default() += 1;
        }
    }
    if groups.len() < 2 {
        return None;
    }
    let mut parts: Vec<(&str, String)> = groups
        .into_values()
        .map(|(mut names, counts)| {
            let lead = counts
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| name_order(b.0, a.0)))
                .map_or("", |(&field, _)| field);
            names.sort_by(|a, b| name_order(a, b));
            names.dedup();
            (lead, format!("{lead} ({})", names.join(", ")))
        })
        .collect();
    parts.sort_by(|a, b| name_order(a.0, b.0));
    let mut text: Vec<String> = parts.into_iter().map(|(_, part)| part).collect();
    if !hubs.is_empty() {
        text.push(format!("hubs: {}", hubs.join(", ")));
    }
    Some(text.join("; "))
}

/// Follows `root` links from `i` to its group's first function.
fn top(root: &[usize], mut i: usize) -> usize {
    while root[i] != i {
        i = root[i];
    }
    i
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
