//! Builds the document's sections: declarations, modules, and other impls.

use crate::document::{Builder, Inline, Item, Link, Node, SourceLink, anchor};
use crate::layout::Block;
use crate::layout_items::{self, docs_block, module_block};
use crate::model::{DeclId, Loc, ModId};
use crate::order::{Order, path_cmp};

impl Builder<'_> {
    /// Adds the level sections, then Test support and Unconnected types,
    /// and returns their contents entries.
    pub(crate) fn ordered(&mut self, order: &Order) -> Vec<Item> {
        let mut contents = Vec::new();
        for (index, level) in order.levels.iter().enumerate() {
            let id = format!("level.{}", index + 1);
            let text = if index == 0 {
                "Level 1: top-level types".to_owned()
            } else {
                format!("Level {}", index + 1)
            };
            contents.push(self.section_item(&text, &id, level));
            self.heading(2, &id, &text);
            for &decl in level {
                self.decl(decl);
            }
        }
        let rest = [
            ("Test support", "test-support", &order.test_support),
            ("Unconnected types", "unconnected", &order.unconnected),
        ];
        for (text, id, decls) in rest {
            if decls.is_empty() {
                continue;
            }
            contents.push(self.section_item(text, id, decls));
            self.heading(2, id, text);
            for &decl in decls {
                self.decl(decl);
            }
        }
        contents
    }

    fn section_item(&self, text: &str, id: &str, decls: &[DeclId]) -> Item {
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
                self.krate.files[a.loc.file]
                    .rel
                    .cmp(&self.krate.files[b.loc.file].rel)
                    .then(a.loc.line.cmp(&b.loc.line))
            });
            impls.dedup_by(|a, b| a.loc == b.loc);
            let mut links = plain_links(self.link(decl.loc, true));
            links.extend(impls.into_iter().filter_map(|i| {
                self.link(i.loc, true).map(|link| SourceLink {
                    link,
                    members: i.members,
                })
            }));
            self.nodes.push(Node::Source(links));
        }
        self.facts(d);
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
            self.nodes.push(Node::Source(plain_links(link)));
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
                self.nodes.push(Node::Source(plain_links(link)));
            }
            self.code(block);
        }
    }
}

/// Returns a `Source:` line's links that list no members.
fn plain_links(link: Option<Link>) -> Vec<SourceLink> {
    link.into_iter()
        .map(|link| SourceLink {
            link,
            members: Vec::new(),
        })
        .collect()
}
