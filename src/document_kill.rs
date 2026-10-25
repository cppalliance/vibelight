//! Builds the kill list: every macro in the target, each linked to its mark.

use crate::document::{Builder, Inline, Item};
use crate::layout::Span;
use crate::model::{Holder, Loc, MacroKind, name_order, path_order};

impl Builder<'_> {
    /// Builds the kill list summary and entries, sorted by kind, then mark
    /// text, then the path of the heading that holds each macro.
    pub(crate) fn kill_list(&self) -> (String, Vec<Item>) {
        let krate = self.krate;
        let mut entries: Vec<(MacroKind, String, Vec<String>, String, String, Loc)> = krate
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
                (m.kind, m.text.clone(), path, heading, id, m.loc)
            })
            .collect();
        entries.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| name_order(&a.1, &b.1))
                .then_with(|| path_order(&a.2, &b.2))
                .then_with(|| {
                    self.krate.files[a.5.file]
                        .rel
                        .cmp(&self.krate.files[b.5.file].rel)
                })
                .then(a.5.line.cmp(&b.5.line))
        });
        let mut counts = [0usize; 4];
        for entry in &entries {
            counts[entry.0 as usize] += 1;
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
            .map(|(_, text, _, heading, id, loc)| {
                let mark = self.find_mark(&id, &text, &mut used);
                let mut inlines = vec![
                    Inline::Bold("MACRO".to_owned()),
                    Inline::Text(" ".to_owned()),
                    Inline::Code(vec![Span::Code(text)]),
                    Inline::Text(", in ".to_owned()),
                    Inline::Kill {
                        text: heading,
                        heading: id,
                        mark,
                    },
                ];
                if let Some(link) = self.link(loc, true) {
                    inlines.push(Inline::Source(link));
                }
                Item {
                    inlines,
                    children: Vec::new(),
                }
            })
            .collect();
        (summary, items)
    }

    /// Finds the first unused mark under `heading` with this text, or one
    /// of the same kind when an attribute list was merged.
    fn find_mark(&self, heading: &str, text: &str, used: &mut [bool]) -> Option<String> {
        let kind = text
            .split_once(':')
            .map(|(k, _)| k)
            .filter(|k| *k == "attribute" || *k == "derive");
        let exact = self
            .marks
            .iter()
            .enumerate()
            .position(|(i, m)| !used[i] && m.0 == heading && m.1 == text);
        let found = exact.or_else(|| {
            let kind = kind?;
            self.marks
                .iter()
                .enumerate()
                .position(|(i, m)| !used[i] && m.0 == heading && m.1.starts_with(kind))
        });
        found.map(|i| {
            used[i] = true;
            self.marks[i].2.clone()
        })
    }
}
