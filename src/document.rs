//! Builds the outline document that both renderers print.

use std::collections::BTreeMap;

use crate::layout::{Block, Span};
use crate::layout_items::docs_block;
use crate::model::{Crate, DeclId, ModId, path_order};
use crate::order::Order;
use crate::source_links::LinkBase;
use crate::spellings::Entry;

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
    /// ` (source: ...)`, with one link or more.
    Source(Vec<Link>),
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
    pub(crate) links: Option<&'a LinkBase>,
    pub(crate) blocks: &'a [Block],
    pub(crate) texts: Vec<String>,
    pub(crate) decl_ids: Vec<String>,
    pub(crate) mod_ids: Vec<String>,
    pub(crate) nodes: Vec<Node>,
    /// Each mark line: the heading it sits under, its text, and its id.
    pub(crate) marks: Vec<(String, String, String)>,
    pub(crate) heading: String,
}

/// Builds the document from the ordered declarations and their blocks.
pub(crate) fn build(
    krate: &Crate,
    others: &[usize],
    order: &Order,
    blocks: &[Block],
    spellings: &[Entry],
    surface: &[crate::surface::Entry],
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
    if !surface.is_empty() {
        contents.push(Item {
            inlines: vec![anchor("Public surface", "surface")],
            children: Vec::new(),
        });
        b.heading(2, "surface", "Public surface");
        let items = b.surface_items(surface);
        b.nodes.push(Node::List {
            class: "surface",
            items,
        });
    }
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
    if !spellings.is_empty() {
        contents.push(Item {
            inlines: vec![anchor("Spellings", "spellings")],
            children: Vec::new(),
        });
        b.heading(2, "spellings", "Spellings");
        let items = b.spelling_items(spellings);
        b.nodes.push(Node::List {
            class: "spellings",
            items,
        });
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

pub(crate) fn anchor(text: &str, id: &str) -> Inline {
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
    pub(crate) fn heading(&mut self, level: u8, id: &str, text: &str) {
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
}
