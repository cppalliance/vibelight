//! Builds the HTML's body module by module, and the sidebar's module tree.
//! Each module's section has its heading, its parent path dim before its
//! name, its `Source:` line, its docs, and a line of its anatomy, then its
//! types in the declarations' order, then its block of items. The tree
//! holds every module under its parent, with its types, functions,
//! constants, and statics, the public surface in bold.

use std::collections::BTreeSet;

use crate::anatomy::{self, Anatomy};
use crate::document::{Builder, Item, Node, crumb, strong_anchor};
use crate::document_sections::plain_links;
use crate::layout_items::{docs_block, module_block};
use crate::model::{Crate, DeclId, ItemKind, Loc, ModId, ValueRef, path_order};
use crate::order::Order;
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
                Kind::Macro(_) | Kind::Unlinked(_) => {}
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

    /// Adds a module's section: its heading, `Source:` line, docs, and
    /// anatomy, then its types, then its block of items.
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
        for &d in types {
            self.decl(d);
        }
        if let Some(block) = module_block(krate, m) {
            self.code(block);
        }
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
