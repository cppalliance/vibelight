//! Builds the outline document that both renderers print.

use std::collections::BTreeMap;

use crate::layout::{Block, Span};
use crate::layout_items::docs_block;
use crate::model::{Crate, DeclId, ModId, ValueRef};
use crate::order::Order;
use crate::settings::{KillList, Notation, Options, OrderBy};
use crate::source_links::LinkBase;
use crate::spellings::Entry;

/// A link to a source file and line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Link {
    pub(crate) text: String,
    pub(crate) target: String,
}

/// One link of a `Source:` line: the definition or an impl block, with the
/// names of the members an inherent impl adds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceLink {
    pub(crate) link: Link,
    /// Sorted; empty for the definition, a trait impl, and a module.
    pub(crate) members: Vec<String>,
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
    /// A kill list link: it points at the mark, under its heading.
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
    Source(Vec<SourceLink>),
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
    /// A horizontal rule that opens a group of declarations.
    Rule,
}

/// The whole outline of one target.
#[derive(Debug)]
pub(crate) struct Document {
    pub(crate) title: String,
    pub(crate) id: String,
    pub(crate) crate_docs: Option<Block>,
    /// The Legend section, printed after the crate docs; empty for a format
    /// without one.
    pub(crate) legend: Vec<Item>,
    pub(crate) contents: Vec<Item>,
    /// The macro counts.
    pub(crate) kill_summary: String,
    /// One entry per macro; empty when the kill list shows only its counts.
    pub(crate) kill: Vec<Item>,
    pub(crate) sections: Vec<Node>,
    pub(crate) refs: Vec<crate::model::TypeRef>,
    /// The id each declaration's heading has.
    pub(crate) decl_ids: Vec<String>,
    /// The id of each module-level function, constant, and static, which
    /// only HTML prints.
    pub(crate) value_ids: BTreeMap<ValueRef, String>,
}

pub(crate) struct Builder<'a> {
    pub(crate) krate: &'a Crate,
    pub(crate) links: Option<&'a LinkBase>,
    /// The blocks the document prints for the declarations.
    pub(crate) printed: &'a [Block],
    pub(crate) notation: Notation,
    pub(crate) texts: Vec<String>,
    pub(crate) decl_ids: Vec<String>,
    pub(crate) mod_ids: Vec<String>,
    pub(crate) nodes: Vec<Node>,
    /// Each mark line: the heading it sits under, its text, and its id.
    pub(crate) marks: Vec<(String, String, String)>,
    pub(crate) heading: String,
}

/// What a target's documents are built from.
#[derive(Clone, Copy)]
pub(crate) struct Inputs<'a> {
    pub(crate) krate: &'a Crate,
    /// The impls left for the Other impls section.
    pub(crate) others: &'a [usize],
    pub(crate) order: &'a Order,
    pub(crate) blocks: &'a [Block],
    /// The blocks to print, in the format's notation.
    pub(crate) printed: &'a [Block],
    pub(crate) spellings: &'a [Entry],
    pub(crate) surface: &'a [crate::surface::Entry],
    pub(crate) options: Options,
}

/// Builds the document from the ordered declarations and their blocks.
pub(crate) fn build(inputs: &Inputs<'_>, links: Option<&LinkBase>) -> Document {
    let Inputs {
        krate,
        others,
        order,
        blocks,
        printed,
        spellings,
        surface,
        options,
    } = *inputs;
    let texts = crate::order::block_texts(krate, blocks);
    let decl_ids = decl_ids(krate, &texts);
    let mod_ids = mod_ids(krate);
    let mut b = Builder {
        krate,
        links,
        printed,
        notation: options.notation,
        texts,
        decl_ids,
        mod_ids,
        nodes: Vec::new(),
        marks: Vec::new(),
        heading: String::new(),
    };
    let mut contents = Vec::new();
    if options.legend {
        contents.push(Item {
            inlines: vec![anchor("Legend", "legend")],
            children: Vec::new(),
        });
    }
    contents.extend(b.surface(surface));
    let modules = b.sorted_modules();
    match options.order {
        OrderBy::Groups => {
            contents.extend(b.grouped(order));
            contents.extend(b.other_impls_section(others));
            contents.push(b.modules_section(&modules));
        }
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
    let (kill_summary, entries) = b.kill_list();
    let kill = match options.kill_list {
        KillList::Full => entries,
    };
    let legend = if options.legend {
        match options.notation {
            Notation::Cpp => crate::legend::html_items(),
        }
    } else {
        Vec::new()
    };
    Document {
        title: krate.title.clone(),
        id: format!("crate.{}", krate.name),
        crate_docs: docs_block(&krate.modules[0].docs, &[], options.notation),
        legend,
        contents,
        kill_summary,
        kill,
        sections: b.nodes,
        refs: krate.refs.clone(),
        decl_ids: b.decl_ids,
        value_ids: crate::document_values::value_ids(krate),
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

/// Numbers the ids that repeat: the first of a group in `order` keeps its
/// id, and the rest get `-2`, `-3`, and so on.
pub(crate) fn suffixed(
    base: &[String],
    order: impl Fn(usize, usize) -> std::cmp::Ordering,
) -> Vec<String> {
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
