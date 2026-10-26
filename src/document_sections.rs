//! Builds the document's sections: declarations, modules, and other impls.

use crate::document::{Builder, Inline, Item, Link, Node, anchor};
use crate::layout::Block;
use crate::layout_items::{self, docs_block, module_block};
use crate::model::{DeclId, Loc, ModId};
use crate::order::path_cmp;

impl Builder<'_> {
    pub(crate) fn section_item(&self, text: &str, id: &str, decls: &[DeclId]) -> Item {
        Item {
            inlines: vec![anchor(text, id)],
            children: decls
                .iter()
                .map(|&d| Item {
                    inlines: vec![anchor(&self.decl_heading(d), &self.decl_ids[d])],
                    children: Vec::new(),
                })
                .collect(),
        }
    }

    pub(crate) fn link(&self, loc: Loc, with_line: bool) -> Option<Link> {
        let base = self.links?;
        let rel = &self.krate.files[loc.file].rel;
        let line = with_line.then_some(loc.line);
        Some(Link {
            text: base.text(rel, line),
            target: base.target(rel, line),
        })
    }

    fn mark(&mut self, label: &'static str, text: &str, loc: Option<Loc>) {
        let source = loc.and_then(|loc| self.link(loc, true));
        let id = (label == "MACRO").then(|| format!("macro.{}", self.marks.len() + 1));
        if let Some(id) = &id {
            self.marks
                .push((self.heading.clone(), text.to_owned(), id.clone()));
        }
        self.nodes.push(Node::Mark {
            label,
            text: text.to_owned(),
            source,
            id,
        });
    }

    fn code(&mut self, block: Block) {
        let repeats = block.repeats.clone();
        self.nodes.push(Node::Code(Block {
            lines: block.lines,
            repeats: Vec::new(),
        }));
        for repeat in repeats {
            self.mark(repeat.label, &repeat.text, repeat.loc);
        }
    }

    pub(crate) fn decl(&mut self, d: DeclId) {
        let decl = &self.krate.decls[d];
        let id = self.decl_ids[d].clone();
        self.heading(3, &id, &self.decl_heading(d));
        if self.links.is_some() {
            let mut impls = decl.impl_locs.clone();
            impls.sort_by(|a, b| {
                self.krate.files[a.file]
                    .rel
                    .cmp(&self.krate.files[b.file].rel)
                    .then(a.line.cmp(&b.line))
            });
            impls.dedup();
            let mut links: Vec<Link> = self.link(decl.loc, true).into_iter().collect();
            links.extend(impls.into_iter().filter_map(|loc| self.link(loc, true)));
            self.nodes.push(Node::Source(links));
        }
        if !decl.derives.is_empty() {
            self.mark(
                "MACRO",
                &format!("derive: {}", decl.derives.join(", ")),
                None,
            );
        }
        if !decl.attributes.is_empty() {
            self.mark(
                "MACRO",
                &format!("attribute: {}", decl.attributes.join(", ")),
                None,
            );
        }
        self.code(self.blocks[d].clone());
    }

    pub(crate) fn module(&mut self, m: ModId) {
        let module = &self.krate.modules[m];
        let id = self.mod_ids[m].clone();
        self.heading(3, &id, &self.mod_heading(m));
        if self.links.is_some() {
            let link = match module.inline {
                Some(loc) => self.link(loc, true),
                None => self.link(
                    Loc {
                        file: module.file,
                        line: 0,
                    },
                    false,
                ),
            };
            self.nodes.push(Node::Source(link.into_iter().collect()));
        }
        let docs = if m == 0 {
            docs_block(&[], &module.cfgs)
        } else {
            docs_block(&module.docs, &module.cfgs)
        };
        if let Some(docs) = docs {
            self.code(docs);
        }
        let mut decls: Vec<DeclId> = (0..self.krate.decls.len())
            .filter(|&d| self.krate.decls[d].module == m)
            .collect();
        decls.sort_by(|&a, &b| path_cmp(self.krate, &self.texts, a, b));
        if !decls.is_empty() {
            let items = decls
                .iter()
                .map(|&d| Item {
                    inlines: vec![anchor(&self.decl_heading(d), &self.decl_ids[d])],
                    children: Vec::new(),
                })
                .collect();
            self.nodes.push(Node::List { class: "", items });
        }
        if let Some(block) = module_block(self.krate, m) {
            self.code(block);
        }
    }

    pub(crate) fn other_impls(&mut self, ids: &[usize]) {
        let (lines, blocks) = layout_items::other_impls(self.krate, ids);
        if !lines.is_empty() {
            let items = lines
                .into_iter()
                .map(|entry| {
                    let mut inlines = Vec::new();
                    for note in &entry.notes {
                        inlines.push(Inline::Code(note.spans.clone()));
                        inlines.push(Inline::Break);
                    }
                    inlines.push(Inline::Code(entry.spans));
                    if let Some(link) = self.link(entry.loc, true) {
                        inlines.push(Inline::Source(vec![link]));
                    }
                    for mark in entry.marks {
                        inlines.push(Inline::Break);
                        inlines.push(Inline::Bold("MACRO".to_owned()));
                        inlines.push(Inline::Text(format!(" {mark}")));
                    }
                    Item {
                        inlines,
                        children: Vec::new(),
                    }
                })
                .collect();
            self.nodes.push(Node::List {
                class: "impls",
                items,
            });
        }
        for (block, loc) in blocks {
            if self.links.is_some() {
                let link = self.link(loc, true);
                self.nodes.push(Node::Source(link.into_iter().collect()));
            }
            self.code(block);
        }
    }
}
