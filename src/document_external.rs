//! The External items section, when the facts are on: each item of another
//! crate that the signatures name, grouped by crate, with its uses, its
//! users, and, for a crate of the run, its first doc line.

use std::collections::BTreeMap;

use crate::document::{Builder, Inline, Item, Node, anchor};
use crate::layout::Span;
use crate::model::{Crate, CrossRef, DeclId, ModId, Note, Reach, RefId, path_order};
use crate::order::{item_texts, path_cmp};
use crate::prelude::is_standard;

/// Returns each declaration's first doc line, for every crate of a run;
/// the External items section shows it for an item another crate names.
pub(crate) fn first_docs(krates: &[Crate]) -> Vec<Vec<Option<String>>> {
    krates
        .iter()
        .map(|krate| {
            krate
                .decls
                .iter()
                .map(|decl| {
                    decl.meta.notes.iter().find_map(|note| match note {
                        Note::Doc(lines) => lines
                            .iter()
                            .map(|line| line.visible(&krate.refs).trim().to_owned())
                            .find(|line| !line.is_empty()),
                        Note::Plain(_) => None,
                    })
                })
                .collect()
        })
        .collect()
}

/// One external item: how often it is named, and where.
#[derive(Default)]
struct Uses {
    count: usize,
    decls: Vec<DeclId>,
    modules: Vec<ModId>,
    doc: Option<String>,
}

impl Builder<'_> {
    /// Adds the External items section and returns its contents entry,
    /// when the facts are on and a signature names another crate's item.
    pub(crate) fn external(&mut self, docs: &[Vec<Option<String>>]) -> Option<Item> {
        self.facts?;
        let krate = self.krate;
        let mut items: BTreeMap<Vec<String>, Uses> = BTreeMap::new();
        let mut note = |id: RefId, user: Result<DeclId, ModId>| {
            let r = &krate.refs[id];
            let Some(path) = r
                .canonical
                .as_ref()
                .filter(|_| r.target.is_none() && !r.comment)
            else {
                return;
            };
            let first = path.first().map_or("", String::as_str);
            if path.len() < 2 || is_standard(path) || ["crate", "self", "super"].contains(&first) {
                return;
            }
            let uses = items.entry(path.clone()).or_default();
            uses.count += 1;
            match user {
                Ok(d) if !uses.decls.contains(&d) => uses.decls.push(d),
                Err(m) if !uses.modules.contains(&m) => uses.modules.push(m),
                _ => {}
            }
            if let Some(CrossRef {
                krate,
                reach: Reach::Decl(decl),
            }) = r.foreign
            {
                uses.doc = docs.get(krate).and_then(|d| d.get(decl)).cloned().flatten();
            }
        };
        let mut seen = std::collections::BTreeSet::new();
        for (d, block) in self.blocks.iter().enumerate() {
            for span in block.lines.iter().flat_map(|line| &line.spans) {
                if let Span::Ref(id) = span {
                    if seen.insert(*id) {
                        note(*id, Ok(d));
                    }
                }
            }
        }
        for (m, module) in krate.modules.iter().enumerate() {
            for text in module.items.iter().flat_map(|item| item_texts(&item.kind)) {
                for id in text.refs() {
                    if seen.insert(id) {
                        note(id, Err(m));
                    }
                }
            }
        }
        if items.is_empty() {
            return None;
        }
        let mut sorted: Vec<(Vec<String>, Uses)> = items.into_iter().collect();
        sorted.sort_by(|a, b| path_order(&a.0, &b.0));
        let mut list: Vec<Item> = Vec::new();
        let mut crate_name = String::new();
        for (path, uses) in sorted {
            if path[0] != crate_name {
                crate_name.clone_from(&path[0]);
                list.push(Item {
                    inlines: vec![Inline::Code(vec![Span::Code(crate_name.clone())])],
                    children: Vec::new(),
                });
            }
            let item = self.external_item(&path, uses);
            if let Some(group) = list.last_mut() {
                group.children.push(item);
            }
        }
        self.heading(2, "external", "External items");
        self.nodes.push(Node::List {
            class: "external",
            items: list,
        });
        Some(Item {
            inlines: vec![anchor("External items", "external")],
            children: Vec::new(),
        })
    }

    /// Returns one external item's entry: its path, its uses and users, and
    /// its doc line as a nested entry.
    fn external_item(&self, path: &[String], mut uses: Uses) -> Item {
        uses.decls
            .sort_by(|&a, &b| path_cmp(self.krate, &self.texts, a, b));
        let modules = &self.krate.modules;
        uses.modules
            .sort_by(|&a, &b| path_order(&modules[a].path, &modules[b].path).then(a.cmp(&b)));
        let times = if uses.count == 1 { "use" } else { "uses" };
        let mut inlines = vec![
            Inline::Code(vec![Span::Code(path.join("::"))]),
            Inline::Text(format!(", {} {times} by ", uses.count)),
        ];
        let links = uses
            .decls
            .iter()
            .map(|&d| (self.decl_heading(d), self.decl_ids[d].clone()))
            .chain(
                uses.modules
                    .iter()
                    .map(|&m| (self.mod_heading(m), self.mod_ids[m].clone())),
            );
        for (index, (text, id)) in links.enumerate() {
            if index > 0 {
                inlines.push(Inline::Text(", ".to_owned()));
            }
            inlines.push(anchor(&text, &id));
        }
        Item {
            inlines,
            children: uses
                .doc
                .into_iter()
                .map(|doc| Item {
                    inlines: vec![Inline::Text(doc)],
                    children: Vec::new(),
                })
                .collect(),
        }
    }
}
