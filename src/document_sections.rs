//! Builds the document's sections: declarations, modules, and other impls.

use std::collections::BTreeMap;

use crate::document::{Builder, Inline, Item, Link, MarkLine, Node, SourceLink, anchor};
use crate::groups::{Group, groups};
use crate::layout::Block;
use crate::layout_items::{self, docs_block, module_block};
use crate::macros;
use crate::model::{Crate, DeclId, Loc, ModId};
use crate::order::{Order, path_cmp};

/// Returns the path of the module that holds a declaration.
fn module_path(krate: &Crate, d: DeclId) -> Vec<String> {
    krate.modules[krate.decls[d].module].path.clone()
}

/// Returns each group's caption: its module and its lead type joined by a
/// middle dot, such as the module `execute::scheduler` and the type
/// `Scheduler`, so a sidebar of groups reads as a list of top-level objects.
/// A caption that still repeats takes ` (2)`, ` (3)`, and so on. `leads`
/// gives each group's lead type.
fn captions(krate: &Crate, groups: &[Group], leads: &[DeclId]) -> Vec<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    groups
        .iter()
        .zip(leads)
        .map(|(group, &lead)| {
            let lead = &krate.decls[lead].name;
            let caption = format!("{} \u{b7} {lead}", group.caption(&krate.title));
            let times = seen.entry(caption.clone()).or_insert(0);
            *times += 1;
            if *times == 1 {
                caption
            } else {
                format!("{caption} ({times})")
            }
        })
        .collect()
}

impl Builder<'_> {
    /// Adds the groups of the dependency order, each opened by a rule and a
    /// heading named for its module, then Test support and Unconnected
    /// types, which open the same way, and returns their contents entries.
    pub(crate) fn grouped(&mut self, order: &Order) -> Vec<Item> {
        let mut contents = Vec::new();
        let krate = self.krate;
        let modules = groups(&order.sequence, |d| module_path(krate, d));
        let leads: Vec<DeclId> = modules
            .iter()
            .map(|group| group.lead(|d| module_path(krate, d)))
            .collect();
        let captions = captions(krate, &modules, &leads);
        for (index, ((group, text), &lead)) in modules.iter().zip(&captions).zip(&leads).enumerate()
        {
            let id = format!("group.{}", index + 1);
            contents.push(self.group_item(text, &id, group, lead));
            self.section_start(&id, text);
            for &decl in &group.decls {
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
            self.section_start(id, text);
            for &decl in decls {
                self.decl(decl);
            }
        }
        contents
    }

    /// Adds the rule and the heading that open a section of types.
    fn section_start(&mut self, id: &str, text: &str) {
        self.nodes.push(Node::Rule);
        self.heading(2, id, text);
    }
    /// Returns a group's entry in the sidebar: its caption, then its types but
    /// the lead, which the caption already names. Each child is named from
    /// the group's own module on, so it does not repeat the parent's name.
    fn group_item(&self, text: &str, id: &str, group: &Group, lead: DeclId) -> Item {
        let named = |d: DeclId| {
            let path = self.krate.decl_path(d);
            match path.strip_prefix(group.key.as_slice()) {
                Some(rest) if !group.key.is_empty() => rest.join("::"),
                _ => path.join("::"),
            }
        };
        Item {
            inlines: vec![anchor(text, id)],
            children: group
                .decls
                .iter()
                .filter(|&&d| d != lead)
                .map(|&d| Item {
                    inlines: vec![anchor(&named(d), &self.decl_ids[d])],
                    children: Vec::new(),
                })
                .collect(),
        }
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

    /// Adds the Modules section, one H3 per module, and returns its
    /// contents entry.
    pub(crate) fn modules_section(&mut self, modules: &[ModId]) -> Item {
        self.heading(2, "modules", "Modules");
        for &m in modules {
            self.module(m);
        }
        Item {
            inlines: vec![anchor("Modules", "modules")],
            children: modules
                .iter()
                .map(|&m| Item {
                    inlines: vec![anchor(&self.mod_heading(m), &self.mod_ids[m])],
                    children: Vec::new(),
                })
                .collect(),
        }
    }

    fn module(&mut self, m: ModId) {
        self.module_head(m, 3);
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

    /// Adds a module's heading at `level`, its `Source:` line, and its docs.
    fn module_head(&mut self, m: ModId, level: u8) {
        let module = &self.krate.modules[m];
        let id = self.mod_ids[m].clone();
        self.heading(level, &id, &self.mod_heading(m));
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
