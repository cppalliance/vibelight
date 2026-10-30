//! Builds the kill list: every macro in the target, each linked to its mark.

use crate::document::{Builder, Inline, Item};
use crate::layout::Span;
use crate::macros;
use crate::model::{Holder, Loc, MacroKind, name_order, path_order};

/// One macro of the kill list, with the heading that holds it.
struct KillEntry {
    kind: MacroKind,
    text: String,
    path: Vec<String>,
    /// The text of the holding heading.
    heading: String,
    /// The id of the holding heading.
    id: String,
    loc: Loc,
}

impl Builder<'_> {
    /// Builds the kill list summary and entries, sorted by kind, then mark
    /// text, then the path of the heading that holds each macro.
    pub(crate) fn kill_list(&self) -> (String, Vec<Item>) {
        let krate = self.krate;
        let mut entries: Vec<KillEntry> = krate
            .macros
            .iter()
            .map(|m| {
                let (path, heading, id) = match m.holder {
                    Holder::Decl(d) => (
                        krate.decl_path(d),
                        self.decl_heading(d),
                        self.decl_ids[d].clone(),
                    ),
                    Holder::Module(id) => (
                        krate.modules[id].path.clone(),
                        self.mod_heading(id),
                        self.mod_ids[id].clone(),
                    ),
                    Holder::Impl(_) => (
                        vec!["Other impls".to_owned()],
                        "Other impls".to_owned(),
                        "impls".to_owned(),
                    ),
                };
                KillEntry {
                    kind: m.kind,
                    text: m.text.clone(),
                    path,
                    heading,
                    id,
                    loc: m.loc,
                }
            })
            .collect();
        entries.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| name_order(&a.text, &b.text))
                .then_with(|| path_order(&a.path, &b.path))
                .then_with(|| {
                    self.krate.files[a.loc.file]
                        .rel
                        .cmp(&self.krate.files[b.loc.file].rel)
                })
                .then(a.loc.line.cmp(&b.loc.line))
        });
        let mut counts = [0usize; 4];
        for entry in &entries {
            counts[entry.kind as usize] += 1;
        }
        let names = [
            ("definition", "definitions"),
            ("invocation", "invocations"),
            ("derive list", "derive lists"),
            ("attribute list", "attribute lists"),
        ];
        let parts: Vec<String> = counts
            .iter()
            .zip(names)
            .filter(|(count, _)| **count > 0)
            .map(|(count, (one, many))| format!("{count} {}", if *count == 1 { one } else { many }))
            .collect();
        let summary = if parts.is_empty() {
            "none".to_owned()
        } else {
            parts.join(", ")
        };
        let mut used = vec![false; self.marks.len()];
        let items = entries
            .into_iter()
            .map(
                |KillEntry {
                     text,
                     heading,
                     id,
                     loc,
                     ..
                 }| {
                    let mark = self.find_mark(&id, &text, &mut used);
                    let mut inlines = vec![
                        Inline::Bold("MACRO".to_owned()),
                        Inline::Text(" ".to_owned()),
                        Inline::Code(vec![Span::Code(text)]),
                        Inline::Text(", in ".to_owned()),
                        Inline::Kill {
                            text: heading,
                            heading_id: id,
                            mark,
                        },
                    ];
                    if let Some(link) = self.link(loc, true) {
                        inlines.push(Inline::Source(vec![link]));
                    }
                    Item {
                        inlines,
                        children: Vec::new(),
                    }
                },
            )
            .collect();
        (summary, items)
    }

    /// Finds the first unused mark under the heading with `heading_id` and
    /// this text, or one of the same kind when an attribute list was merged.
    fn find_mark(&self, heading_id: &str, text: &str, used: &mut [bool]) -> Option<String> {
        let kind = macros::mark_parts(text)
            .map(|(kind, _)| kind)
            .filter(|kind| matches!(*kind, macros::ATTRIBUTE | macros::DERIVE));
        let exact = self
            .marks
            .iter()
            .enumerate()
            .position(|(i, m)| !used[i] && m.heading_id == heading_id && m.text == text);
        let found = exact.or_else(|| {
            let kind = kind?;
            self.marks.iter().enumerate().position(|(i, m)| {
                !used[i]
                    && m.heading_id == heading_id
                    && macros::mark_parts(&m.text).is_some_and(|(k, _)| k == kind)
            })
        });
        found.map(|i| {
            used[i] = true;
            self.marks[i].id.clone()
        })
    }
}
