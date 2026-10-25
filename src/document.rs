//! Builds the outline document that both renderers print.

use std::collections::BTreeMap;

use crate::layout::{Block, Span};
use crate::layout_items::{docs_block, module_block, other_impls};
use crate::model::{Crate, DeclId, Loc, ModId, path_order};
use crate::order::{Order, path_cmp};
use crate::source_links::LinkBase;

/// A link to a source file and line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) text: String,
    pub(crate) target: String,
}

/// One piece of inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Inline {
    Text(String),
    Bold(String),
    /// Inline code, styled for HTML.
    Code(Vec<Span>),
    /// A link to a heading.
    Anchor {
        text: String,
        id: String,
    },
    /// A kill list link: markdown points at the heading, HTML at the mark.
    Kill {
        text: String,
        heading: String,
        mark: Option<String>,
    },
    /// ` (source: ...)`.
    Source(Link),
    /// A hard line break inside a list item.
    Break,
}

/// One list item and its nested items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) inlines: Vec<Inline>,
    pub(crate) children: Vec<Item>,
}

/// One block-level piece of the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    Heading {
        level: u8,
        id: String,
        text: String,
    },
    Code(Block),
    /// A `Source:` line: the definition, then any impl blocks.
    Source(Vec<Link>),
    /// A bold label line outside a block, such as `MACRO derive: Clone`.
    Mark {
        label: &'static str,
        text: String,
        source: Option<Link>,
        id: Option<String>,
    },
    List {
        class: &'static str,
        items: Vec<Item>,
    },
}

/// The whole outline of one target.
#[derive(Debug)]
pub(crate) struct Document {
    pub(crate) title: String,
    pub(crate) id: String,
    pub(crate) crate_docs: Option<Block>,
    pub(crate) contents: Vec<Item>,
    pub(crate) kill_summary: String,
    pub(crate) kill: Vec<Item>,
    pub(crate) sections: Vec<Node>,
    pub(crate) refs: Vec<crate::model::TypeRef>,
    /// The id each declaration's heading has.
    pub(crate) decl_ids: Vec<String>,
}

pub(crate) struct Builder<'a> {
    pub(crate) krate: &'a Crate,
    links: Option<&'a LinkBase>,
    blocks: &'a [Block],
    texts: Vec<String>,
    pub(crate) decl_ids: Vec<String>,
    pub(crate) mod_ids: Vec<String>,
    nodes: Vec<Node>,
    /// Each mark line: the heading it sits under, its text, and its id.
    pub(crate) marks: Vec<(String, String, String)>,
    heading: String,
}

/// Builds the document from the ordered declarations and their blocks.
pub(crate) fn build(
    krate: &Crate,
    others: &[usize],
    order: &Order,
    blocks: &[Block],
    links: Option<&LinkBase>,
) -> Document {
    let texts = crate::order::block_texts(krate, blocks);
    let decl_ids = decl_ids(krate, &texts);
    let mod_ids = mod_ids(krate);
    let mut b = Builder {
        krate,
        links,
        blocks,
        texts,
        decl_ids,
        mod_ids,
        nodes: Vec::new(),
        marks: Vec::new(),
        heading: String::new(),
    };
    let mut contents = Vec::new();
    for (index, level) in order.levels.iter().enumerate() {
        let id = format!("level.{}", index + 1);
        let text = if index == 0 {
            "Level 1: top-level types".to_owned()
        } else {
            format!("Level {}", index + 1)
        };
        contents.push(b.section_item(&text, &id, level));
        b.heading(2, &id, &text);
        for &decl in level {
            b.decl(decl);
        }
    }
    if !order.unconnected.is_empty() {
        contents.push(b.section_item("Unconnected types", "unconnected", &order.unconnected));
        b.heading(2, "unconnected", "Unconnected types");
        for &decl in &order.unconnected {
            b.decl(decl);
        }
    }
    if !others.is_empty() {
        contents.push(Item {
            inlines: vec![anchor("Other impls", "impls")],
            children: Vec::new(),
        });
        b.heading(2, "impls", "Other impls");
        b.other_impls(others);
    }
    let mut modules: Vec<ModId> = (0..krate.modules.len()).collect();
    modules.sort_by(|&x, &y| {
        path_order(&krate.modules[x].path, &krate.modules[y].path)
            .then(b.mod_ids[x].cmp(&b.mod_ids[y]))
    });
    contents.push(Item {
        inlines: vec![anchor("Modules", "modules")],
        children: modules
            .iter()
            .map(|&m| Item {
                inlines: vec![anchor(&b.mod_heading(m), &b.mod_ids[m])],
                children: Vec::new(),
            })
            .collect(),
    });
    b.heading(2, "modules", "Modules");
    for &m in &modules {
        b.module(m);
    }
    let (kill_summary, kill) = b.kill_list();
    Document {
        title: krate.title.clone(),
        id: format!("crate.{}", krate.name),
        crate_docs: docs_block(&krate.modules[0].docs, &[]),
        contents,
        kill_summary,
        kill,
        sections: b.nodes,
        refs: krate.refs.clone(),
        decl_ids: b.decl_ids,
    }
}

fn anchor(text: &str, id: &str) -> Inline {
    Inline::Anchor {
        text: text.to_owned(),
        id: id.to_owned(),
    }
}

/// Gives each declaration its id; ids that would repeat get `-2`, `-3`,
/// and so on, ordered by the declarations' rendered text.
fn decl_ids(krate: &Crate, texts: &[String]) -> Vec<String> {
    let base: Vec<String> = (0..krate.decls.len())
        .map(|d| {
            let mut parts = vec![krate.name.clone()];
            parts.extend(
                krate
                    .decl_path(d)
                    .iter()
                    .map(|s| crate::typerefs::strip_raw(s)),
            );
            format!("type.{}", parts.join("."))
        })
        .collect();
    suffixed(&base, |a, b| texts[a].cmp(&texts[b]).then(a.cmp(&b)))
}

fn mod_ids(krate: &Crate) -> Vec<String> {
    let base: Vec<String> = krate
        .modules
        .iter()
        .map(|m| {
            let mut parts = vec![krate.name.clone()];
            parts.extend(m.path.iter().map(|s| crate::typerefs::strip_raw(s)));
            format!("mod.{}", parts.join("."))
        })
        .collect();
    suffixed(&base, |a, b| a.cmp(&b))
}

fn suffixed(base: &[String], order: impl Fn(usize, usize) -> std::cmp::Ordering) -> Vec<String> {
    let mut groups: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, id) in base.iter().enumerate() {
        groups.entry(id).or_default().push(i);
    }
    let mut out = base.to_vec();
    for members in groups.values_mut() {
        members.sort_by(|&a, &b| order(a, b));
        for (rank, &i) in members.iter().enumerate().skip(1) {
            out[i] = format!("{}-{}", base[i], rank + 1);
        }
    }
    out
}

impl Builder<'_> {
    fn heading(&mut self, level: u8, id: &str, text: &str) {
        id.clone_into(&mut self.heading);
        self.nodes.push(Node::Heading {
            level,
            id: id.to_owned(),
            text: text.to_owned(),
        });
    }

    pub(crate) fn decl_heading(&self, d: DeclId) -> String {
        self.krate.decl_path(d).join("::")
    }

    pub(crate) fn mod_heading(&self, m: ModId) -> String {
        let path = &self.krate.modules[m].path;
        if path.is_empty() {
            self.krate.title.clone()
        } else {
            path.join("::")
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

    fn decl(&mut self, d: DeclId) {
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

    fn module(&mut self, m: ModId) {
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

    fn other_impls(&mut self, ids: &[usize]) {
        let (lines, blocks) = other_impls(self.krate, ids);
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
                        inlines.push(Inline::Source(link));
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
