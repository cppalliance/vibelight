//! Builds the HTML's body module by module, and the sidebar's module tree.
//! Each module's section has its heading, its parent path dim before its
//! name, its `Source:` line, its docs, a line of its anatomy, and a line of
//! its coupling when other modules use it or it uses them, then its types
//! in the declarations' order, then its block of items. The tree holds
//! every module under its parent, with its types, functions, constants,
//! and statics, the public surface in bold.

use std::collections::BTreeSet;

use crate::anatomy::{self, Anatomy};
use crate::html::document::{
    Builder, Inline, Item, Node, anchor, crumb, plain_links, strong_anchor,
};
use crate::html::layout::items::{docs_block, module_block};
use crate::html::order::Order;
use crate::model::{Crate, DeclId, ItemKind, Loc, ModId, ValueRef, path_order};
use crate::surface::{Entry, Kind};

/// The nodes a library's public surface names, which the tree shows bold.
#[derive(Default)]
struct Surface {
    modules: BTreeSet<ModId>,
    decls: BTreeSet<DeclId>,
    values: BTreeSet<ValueRef>,
}

impl Surface {
    fn of(entries: &[Entry]) -> Surface {
        let mut out = Surface::default();
        for entry in entries {
            match entry.kind {
                Kind::Module(m) => {
                    out.modules.insert(m);
                }
                Kind::Decl(d) => {
                    out.decls.insert(d);
                }
                Kind::Value(at) | Kind::Function(at) => {
                    out.values.insert(at);
                }
                Kind::Macro(_) | Kind::Cross(..) | Kind::Unlinked(_) => {}
            }
        }
        out
    }
}

impl Builder<'_> {
    /// Adds one section per module, in path order, each with its types in
    /// `order`, and returns the sidebar's module tree.
    pub(crate) fn modules_body(&mut self, order: &Order, surface: &[Entry]) -> Item {
        let krate = self.krate;
        let all: Vec<DeclId> = (order.sequence.iter())
            .chain(&order.unconnected)
            .chain(&order.test_support)
            .copied()
            .collect();
        for m in self.sorted_modules() {
            let types: Vec<DeclId> = (all.iter().copied())
                .filter(|&d| krate.decls[d].module == m)
                .collect();
            self.module_section(m, &types);
        }
        self.tree(0, &Surface::of(surface))
    }

    /// Adds a module's section: its heading, `Source:` line, docs, anatomy,
    /// and coupling, then its types, then its block of items.
    fn module_section(&mut self, m: ModId, types: &[DeclId]) {
        let krate = self.krate;
        let module = &krate.modules[m];
        let (crumb, name) = match module.path.split_last() {
            Some((name, up)) => (crumb(up), name.clone()),
            None => (String::new(), krate.title.clone()),
        };
        let id = self.mod_ids[m].clone();
        self.heading_crumb(2, &id, crumb, &name);
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
            self.nodes.push(Node::Source(plain_links(link, None)));
        }
        let notes = if m == 0 { &[][..] } else { &module.docs[..] };
        if let Some(docs) = docs_block(notes, &module.cfgs) {
            self.code(docs);
        }
        self.nodes.push(Node::Note {
            class: "anatomy",
            text: anatomy_line(krate, m),
        });
        if let Some(inlines) = self.coupling_line(m) {
            self.nodes.push(Node::Line {
                class: "coupling",
                inlines,
            });
        }
        for &d in types {
            self.decl(d);
        }
        if let Some(block) = module_block(krate, m) {
            self.code(block);
        }
    }

    /// Returns module `m`'s coupling line, or none when it uses no other
    /// module and no other module uses it: the modules that use its items
    /// and the modules whose items it uses, in path order, each linked to
    /// its section with how many items, then its instability, such as
    /// `Used by 1 module: vm (3). Uses 2 modules: error (4), messages (2).
    /// Instability 0.67.`
    fn coupling_line(&self, m: ModId) -> Option<Vec<Inline>> {
        let (used_by, uses) = (&self.users[m], &self.graph[m]);
        if used_by.is_empty() && uses.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        for (label, edges) in [("Used by", used_by), ("Uses", uses)] {
            if edges.is_empty() {
                continue;
            }
            let n = edges.len();
            let noun = if n == 1 { "module" } else { "modules" };
            out.push(Inline::Text(format!("{label} {n} {noun}: ")));
            let mut others: Vec<(ModId, usize)> = (edges.iter())
                .map(|(&other, edge)| (other, edge.items))
                .collect();
            others.sort_by(|a, b| self.module_order(a.0, b.0));
            for (i, (other, items)) in others.into_iter().enumerate() {
                if i > 0 {
                    out.push(Inline::Text(", ".to_owned()));
                }
                out.push(anchor(&self.mod_heading(other), &self.mod_ids[other]));
                out.push(Inline::Text(format!(" ({items})")));
            }
            out.push(Inline::Text(". ".to_owned()));
        }
        let instability = instability(used_by.len(), uses.len());
        out.push(Inline::Text(format!("Instability {instability}.")));
        Some(out)
    }

    /// Returns module `m`'s entry in the sidebar's tree: its child modules,
    /// types, functions, constants, and statics, in path order. Extern
    /// items and macros have no entry.
    fn tree(&self, m: ModId, surface: &Surface) -> Item {
        let krate = self.krate;
        let module = &krate.modules[m];
        let mut entries: Vec<(Vec<String>, &str, Item)> = Vec::new();
        for (c, child) in krate.modules.iter().enumerate() {
            if child.parent == Some(m) {
                let item = self.tree(c, surface);
                entries.push((child.path.clone(), &self.mod_ids[c], item));
            }
        }
        for (d, decl) in krate.decls.iter().enumerate() {
            if decl.module == m {
                let item = leaf(&decl.name, &self.decl_ids[d], surface.decls.contains(&d));
                entries.push((krate.decl_path(d), &self.decl_ids[d], item));
            }
        }
        for (k, item) in module.items.iter().enumerate() {
            let at = ValueRef { module: m, item: k };
            let Some(id) = self.value_ids.get(&at) else {
                continue;
            };
            let text = match item.kind {
                ItemKind::Fn(_) => format!("{}()", item.name),
                _ => item.name.clone(),
            };
            let mut path = module.path.clone();
            path.push(item.name.clone());
            entries.push((path, id, leaf(&text, id, surface.values.contains(&at))));
        }
        entries.sort_by(|a, b| path_order(&a.0, &b.0).then_with(|| a.1.cmp(b.1)));
        let name = (module.path.last()).map_or(krate.title.as_str(), String::as_str);
        let strong = surface.modules.contains(&m);
        Item {
            inlines: vec![strong_anchor(name, &self.mod_ids[m], strong)],
            children: entries.into_iter().map(|(_, _, item)| item).collect(),
        }
    }
}

/// Returns a tree entry with no children.
fn leaf(text: &str, id: &str, strong: bool) -> Item {
    Item {
        inlines: vec![strong_anchor(text, id, strong)],
        children: Vec::new(),
    }
}

/// Returns a module's anatomy line: what it is, how far it reaches when
/// that is less than written, what it declares, and the same items by
/// written visibility, such as `pub mod, reaches pub(crate): 1 type, 2
/// functions; pub 2, pub(self) 1`.
fn anatomy_line(krate: &Crate, m: ModId) -> String {
    let module = &krate.modules[m];
    let stamp = module.access.as_ref();
    let vis = crate::access::spelled(krate, module.vis);
    let mut head = if module.parent.is_none() {
        "crate root".to_owned()
    } else if vis == "pub(self)" {
        "mod".to_owned()
    } else {
        format!("{vis} mod")
    };
    if let Some(reach) = stamp.and_then(|s| s.reaches.as_ref()) {
        head = format!("{head}, reaches {reach}");
    }
    let Anatomy { counts, mix } = anatomy::of(krate, m);
    let names = [
        ("type", "types"),
        ("function", "functions"),
        ("value", "values"),
        ("re-export", "re-exports"),
    ];
    let counted: Vec<String> = (names.iter().zip(counts))
        .filter(|&(_, n)| n > 0)
        .map(|(&(one, many), n)| format!("{n} {}", if n == 1 { one } else { many }))
        .collect();
    if counted.is_empty() {
        return head;
    }
    let mix: Vec<String> = mix.iter().map(|(vis, n)| format!("{vis} {n}")).collect();
    format!("{head}: {}; {}", counted.join(", "), mix.join(", "))
}

/// Returns the instability of a module or an outline that `fan_in` others
/// use and that uses `fan_out` others, `fan_out / (fan_in + fan_out)`,
/// with two decimals.
#[expect(
    clippy::cast_precision_loss,
    reason = "a count of modules or outlines is far below 2^53, so f64 holds it exactly"
)]
pub(crate) fn instability(fan_in: usize, fan_out: usize) -> String {
    format!("{:.2}", fan_out as f64 / (fan_in + fan_out) as f64)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use crate::html::document::{Inputs, build};
    use crate::html::order::Order;
    use crate::module_graph::ModEdge;

    /// Renders the page of a crate named `t` made of one `lib.rs`, whose
    /// module graph has an edge for each `(from, to, items)`: module `from`
    /// uses `items` items of module `to`, both by path. Returns each
    /// coupling line with the id of the section it is in.
    fn coupling(lib: &str, edges: &[(&str, &str, usize)]) -> Vec<(String, String)> {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), lib).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        let at = |path: &str| {
            (krate.modules.iter())
                .position(|m| m.path.join("::") == path)
                .unwrap()
        };
        let mut graph = vec![BTreeMap::new(); krate.modules.len()];
        for &(from, to, items) in edges {
            let edge = ModEdge {
                items,
                resolved: true,
            };
            graph[at(from)].insert(at(to), edge);
        }
        let inputs = Inputs {
            krate: &krate,
            others: &[],
            order: &Order::default(),
            blocks: &[],
            spellings: &[],
            surface: &[],
            graph: &graph,
        };
        let doc = build(&inputs, None);
        let page = crate::html::render_html::fragment(&doc, None, &[], crate::stamp::PLACEHOLDER);
        let mut section = "";
        let mut out = Vec::new();
        for line in page.lines() {
            if let Some(rest) = line.strip_prefix("<h2 id=\"") {
                section = rest.split('"').next().unwrap();
            } else if let Some(text) = line.strip_prefix("<p class=\"coupling\">") {
                let text = text.strip_suffix("</p>").unwrap();
                out.push((section.to_owned(), text.to_owned()));
            }
        }
        out
    }

    #[test]
    fn a_coupling_line_counts_and_links_the_modules_in_path_order() {
        let lib = "pub mod d {}\npub mod b {\n    pub mod c {}\n}\npub mod a {}\npub mod e {}\n";
        let edges = [
            ("a", "d", 1),
            ("a", "b::c", 5),
            ("b", "a", 2),
            ("b::c", "d", 3),
        ];
        let to = |path: &str| {
            let id = path.replace("::", ".");
            format!("<a href=\"#mod.t.{id}\">{path}</a>")
        };
        let lines = [
            (
                "mod.t.a",
                format!(
                    "Used by 1 module: {} (2). Uses 2 modules: {} (5), {} (1). Instability 0.67.",
                    to("b"),
                    to("b::c"),
                    to("d")
                ),
            ),
            (
                "mod.t.b",
                format!("Uses 1 module: {} (2). Instability 1.00.", to("a")),
            ),
            (
                "mod.t.b.c",
                format!(
                    "Used by 1 module: {} (5). Uses 1 module: {} (3). Instability 0.50.",
                    to("a"),
                    to("d")
                ),
            ),
            (
                "mod.t.d",
                format!(
                    "Used by 2 modules: {} (1), {} (3). Instability 0.00.",
                    to("a"),
                    to("b::c")
                ),
            ),
        ];
        let lines = lines.map(|(id, text)| (id.to_owned(), text));
        assert_eq!(
            coupling(lib, &edges),
            lines,
            "no line for the crate root or e"
        );
    }
}
