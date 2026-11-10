//! Builds the outline document that the HTML renderer prints, and its
//! pieces: source links, marks, code blocks, declaration sections, and the
//! Other impls section.

pub(crate) mod body;
mod kill;
mod modules;
mod spellings;
mod surface;
mod values;

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::html::fn_ids;
use crate::html::layout::items::docs_block;
use crate::html::layout::{self, Block, Span};
use crate::html::order::Order;
use crate::macros;
use crate::model::{Crate, CrossRef, DeclId, Loc, ModId, ValueRef};
use crate::module_graph::ModEdge;
use crate::source_links::LinkBase;
use crate::spellings::Entry;

/// A link to a source file and line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) text: String,
    pub(crate) target: String,
}

/// One link of a `Source:` line: the definition or an impl block, with the
/// names of the members an inherent impl adds, and the macro whose
/// invocation wrote it, such as `events!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceLink {
    pub(crate) link: Link,
    /// Sorted; empty for the definition, a trait impl, and a module.
    pub(crate) members: Vec<String>,
    pub(crate) via: Option<String>,
}

/// One piece of inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Inline {
    Text(String),
    Bold(String),
    /// Dim text, such as the count beside a module's link in the sidebar.
    Dim(String),
    /// Inline code, styled for HTML.
    Code(Vec<Span>),
    /// A link to a heading, bold for a node on the public surface.
    Anchor {
        text: String,
        id: String,
        strong: bool,
    },
    /// A link to an item of another crate of the run, which the renderer
    /// points at that crate's page, or leaves as plain text when the run
    /// holds no page for it.
    Cross {
        text: String,
        to: CrossRef,
    },
    /// A kill list link: it points at the mark, under its heading.
    Kill {
        text: String,
        heading_id: String,
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
    /// A heading: dim text that leads to it, such as a module's parent path
    /// `engine::`, then its own text.
    Heading {
        level: u8,
        id: String,
        crumb: String,
        text: String,
    },
    Code(Block),
    /// A `Source:` line: the definition, then any impl blocks.
    Source(Vec<SourceLink>),
    /// A line of plain text in a styled paragraph, such as a module's
    /// anatomy.
    Note {
        class: &'static str,
        text: String,
    },
    /// A line of inline content in a styled paragraph, such as a module's
    /// coupling, which links the modules it names.
    Line {
        class: &'static str,
        inlines: Vec<Inline>,
    },
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
    /// The sidebar's first list: the page's sections, with the module tree.
    pub(crate) contents: Vec<Item>,
    /// The sidebar's second list: each module that declares a symbol, with
    /// its symbols below it.
    pub(crate) symbols: Vec<Item>,
    /// The macro counts.
    pub(crate) kill_summary: String,
    /// One entry per macro.
    pub(crate) kill: Vec<Item>,
    pub(crate) sections: Vec<Node>,
    pub(crate) refs: Vec<crate::model::TypeRef>,
    /// The id each declaration's heading has.
    pub(crate) decl_ids: Vec<String>,
    /// The id of each module-level function, constant, and static.
    pub(crate) value_ids: BTreeMap<ValueRef, String>,
}

/// One mark line the builder has added.
pub(crate) struct MarkLine {
    /// The id of the heading it sits under.
    pub(crate) heading_id: String,
    pub(crate) text: String,
    pub(crate) id: String,
}

pub(crate) struct Builder<'a> {
    pub(crate) krate: &'a Crate,
    pub(crate) links: Option<&'a LinkBase>,
    /// The blocks the document prints for the declarations.
    pub(crate) blocks: &'a [Block],
    pub(crate) decl_ids: Vec<String>,
    pub(crate) mod_ids: Vec<String>,
    /// The id of each module-level function, constant, and static.
    pub(crate) value_ids: BTreeMap<ValueRef, String>,
    /// For each module, the other modules whose items it uses.
    pub(crate) graph: &'a [BTreeMap<ModId, ModEdge>],
    /// For each module, the other modules that use its items.
    pub(crate) users: Vec<BTreeMap<ModId, ModEdge>>,
    pub(crate) nodes: Vec<Node>,
    /// Each mark line, in the order it was added.
    pub(crate) marks: Vec<MarkLine>,
    pub(crate) heading_id: String,
}

/// What a target's documents are built from.
#[derive(Clone, Copy)]
pub(crate) struct Inputs<'a> {
    pub(crate) krate: &'a Crate,
    /// The impls left for the Other impls section.
    pub(crate) others: &'a [usize],
    pub(crate) order: &'a Order,
    pub(crate) blocks: &'a [Block],
    pub(crate) spellings: &'a [Entry],
    pub(crate) surface: &'a [crate::surface::Entry],
    /// The target's module graph: for each module, the other modules whose
    /// items it uses.
    pub(crate) graph: &'a [BTreeMap<ModId, ModEdge>],
}

/// Builds the document from the ordered declarations and their blocks.
pub(crate) fn build(inputs: &Inputs<'_>, links: Option<&LinkBase>) -> Document {
    let Inputs {
        krate,
        others,
        order,
        blocks,
        spellings,
        surface,
        graph,
    } = *inputs;
    let texts = crate::html::order::block_texts(krate, blocks);
    let decl_ids = decl_ids(krate, &texts);
    let mod_ids = mod_ids(krate);
    let mut b = Builder {
        krate,
        links,
        blocks,
        decl_ids,
        mod_ids,
        value_ids: crate::html::document::values::value_ids(krate),
        graph,
        users: crate::module_graph::used_by(graph),
        nodes: Vec::new(),
        marks: Vec::new(),
        heading_id: String::new(),
    };
    let mut contents: Vec<Item> = b.surface(surface).into_iter().collect();
    let (tree, symbols) = b.modules_body(order, surface);
    contents.push(tree);
    contents.extend(b.other_impls_section(others));
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
        symbols,
        kill_summary,
        kill,
        sections: b.nodes,
        refs: krate.refs.clone(),
        decl_ids: b.decl_ids,
        value_ids: b.value_ids,
    }
}

/// Returns the crumb that leads to a node of module `path`: each segment
/// followed by `::`, or nothing for the root.
pub(crate) fn crumb(path: &[String]) -> String {
    let mut out = path.join("::");
    if !out.is_empty() {
        out.push_str("::");
    }
    out
}

pub(crate) fn anchor(text: &str, id: &str) -> Inline {
    strong_anchor(text, id, false)
}

/// Returns a link to a heading, bold when `strong`.
pub(crate) fn strong_anchor(text: &str, id: &str, strong: bool) -> Inline {
    Inline::Anchor {
        text: text.to_owned(),
        id: id.to_owned(),
        strong,
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

/// Numbers the ids that repeat: the first of a group in `order` keeps its
/// id, and the rest get `-2`, `-3`, and so on.
pub(crate) fn suffixed(base: &[String], order: impl Fn(usize, usize) -> Ordering) -> Vec<String> {
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
        self.heading_crumb(level, id, String::new(), text);
    }

    /// Adds a heading whose dim `crumb` leads to its text.
    pub(crate) fn heading_crumb(&mut self, level: u8, id: &str, crumb: String, text: &str) {
        id.clone_into(&mut self.heading_id);
        self.nodes.push(Node::Heading {
            level,
            id: id.to_owned(),
            crumb,
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
        let block = &self.blocks[d];
        self.code(Block {
            lines: fn_ids::anchored(&block.lines, &fn_ids::fn_prefix(&id)),
            repeats: block.repeats.clone(),
        });
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
        let (lines, blocks) = layout::items::other_impls(self.krate, ids);
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
