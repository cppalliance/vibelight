//! Builds the document's pieces: source links, marks, code blocks,
//! declaration sections, and the Other impls section.

use crate::document::{Builder, Inline, Item, Link, MarkLine, Node, SourceLink, anchor, crumb};
use crate::layout::Block;
use crate::layout_items;
use crate::macros;
use crate::model::{DeclId, Loc};

impl Builder<'_> {
    /// Returns the link to a source line, in a real file: a line an
    /// expansion wrote links to its invocation.
    pub(crate) fn link(&self, loc: Loc, with_line: bool) -> Option<Link> {
        let base = self.links?;
        let loc = self.krate.real_loc(loc);
        let rel = &self.krate.files[loc.file].rel;
        let line = with_line.then_some(loc.line);
        Some(Link {
            text: base.text(rel, line),
            target: base.target(rel, line),
        })
    }

    /// Returns the macro, such as `events!`, whose invocation wrote a
    /// location, or `None` for a location in a real file.
    fn via(&self, loc: Loc) -> Option<String> {
        self.krate.via(loc.file).map(str::to_owned)
    }

    fn mark(&mut self, label: &'static str, text: &str, loc: Option<Loc>) {
        let source = loc.and_then(|loc| self.link(loc, true));
        let id = (label == "MACRO").then(|| format!("macro.{}", self.marks.len() + 1));
        if let Some(id) = &id {
            self.marks.push(MarkLine {
                heading_id: self.heading_id.clone(),
                text: text.to_owned(),
                id: id.clone(),
            });
        }
        self.nodes.push(Node::Mark {
            label,
            text: text.to_owned(),
            source,
            id,
        });
    }

    pub(crate) fn code(&mut self, block: Block) {
        let repeats = block.repeats.clone();
        self.nodes.push(Node::Code(Block {
            lines: block.lines,
            repeats: Vec::new(),
        }));
        for repeat in repeats {
            self.mark(repeat.label, &repeat.text, repeat.loc);
        }
    }

    /// Adds a declaration's section: its heading, its module's path dim
    /// before its name, its `Source:` line, its marks, and its block.
    pub(crate) fn decl(&mut self, d: DeclId) {
        let decl = &self.krate.decls[d];
        let id = self.decl_ids[d].clone();
        let crumb = crumb(&self.krate.modules[decl.module].path);
        self.heading_crumb(3, &id, crumb, &decl.name);
        if self.links.is_some() {
            let krate = self.krate;
            let mut impls: Vec<(Loc, Vec<String>, Option<String>)> = (decl.impl_locs.iter())
                .map(|i| (krate.real_loc(i.loc), i.members.clone(), self.via(i.loc)))
                .collect();
            impls.sort_by(|a, b| {
                (krate.files[a.0.file].rel.cmp(&krate.files[b.0.file].rel))
                    .then(a.0.line.cmp(&b.0.line))
            });
            impls.dedup_by(|later, kept| {
                let same = later.0 == kept.0;
                if same {
                    kept.1.append(&mut later.1);
                    kept.1.sort();
                    kept.1.dedup();
                    kept.2 = kept.2.take().or(later.2.take());
                }
                same
            });
            let mut links = plain_links(self.link(decl.loc, true), self.via(decl.loc));
            links.extend(impls.into_iter().filter_map(|(loc, members, via)| {
                self.link(loc, true)
                    .map(|link| SourceLink { link, members, via })
            }));
            self.nodes.push(Node::Source(links));
        }
        if !decl.derives.is_empty() {
            self.mark(
                "MACRO",
                &macros::mark_text(macros::DERIVE, &decl.derives),
                None,
            );
        }
        if !decl.attributes.is_empty() {
            self.mark(
                "MACRO",
                &macros::mark_text(macros::ATTRIBUTE, &decl.attributes),
                None,
            );
        }
        self.code(self.blocks[d].clone());
    }

    /// Adds the Other impls section and returns its contents entry, when
    /// an impl has no declared type to attach to.
    pub(crate) fn other_impls_section(&mut self, others: &[usize]) -> Option<Item> {
        if others.is_empty() {
            return None;
        }
        self.heading(2, "impls", "Other impls");
        self.other_impls(others);
        Some(Item {
            inlines: vec![anchor("Other impls", "impls")],
            children: Vec::new(),
        })
    }

    fn other_impls(&mut self, ids: &[usize]) {
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
                let (link, via) = (self.link(loc, true), self.via(loc));
                self.nodes.push(Node::Source(plain_links(link, via)));
            }
            self.code(block);
        }
    }
}

/// Returns a `Source:` line's links that list no members, with the macro
/// that wrote the node, if one did.
pub(crate) fn plain_links(link: Option<Link>, via: Option<String>) -> Vec<SourceLink> {
    let link = link.map(|link| SourceLink {
        link,
        members: Vec::new(),
        via,
    });
    link.into_iter().collect()
}
