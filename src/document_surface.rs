//! Builds the Public surface section: every path a user of the crate can
//! name, each linking to the heading of what it names.

use crate::document::{Builder, Inline, Item, Node, anchor};
use crate::surface::{Entry, Kind};

impl Builder<'_> {
    /// Adds the Public surface section and returns its contents entry, when
    /// the surface has entries.
    pub(crate) fn surface(&mut self, surface: &[Entry]) -> Option<Item> {
        if surface.is_empty() {
            return None;
        }
        self.heading(2, "surface", "Public surface");
        let items = self.surface_items(surface);
        self.nodes.push(Node::List {
            class: "surface",
            items,
        });
        Some(Item {
            inlines: vec![anchor("Public surface", "surface")],
            children: Vec::new(),
        })
    }

    /// Returns one list item per surface path. A function's path ends in
    /// `()` and a macro's in `!`. Functions, constants, statics, and macros
    /// have no headings, so they link to the heading of the module that
    /// declares them. A re-export of another crate's item, or one the walk
    /// cannot place, shows the path as written after it and has no link.
    /// A path that two declarations share, as under different `cfg`s, is
    /// listed once, linking to the heading whose id sorts first.
    fn surface_items(&self, surface: &[Entry]) -> Vec<Item> {
        let mut lines: Vec<(String, Option<&str>, bool)> = Vec::new();
        for entry in surface {
            let mut text = entry.path.join("::");
            let id = match &entry.kind {
                Kind::Module(m) | Kind::Value(m) => Some(&self.mod_ids[*m]),
                Kind::Decl(d) => Some(&self.decl_ids[*d]),
                Kind::Function(m) => {
                    text.push_str("()");
                    Some(&self.mod_ids[*m])
                }
                Kind::Macro(m) => {
                    text.push('!');
                    Some(&self.mod_ids[*m])
                }
                Kind::Unlinked(written) => {
                    text = format!("{text} ({written})");
                    None
                }
            };
            let id = id.map(String::as_str);
            match lines.last_mut() {
                Some(last) if last.0 == text => {
                    last.1 = last.1.min(id);
                    last.2 &= entry.test_support;
                }
                _ => lines.push((text, id, entry.test_support)),
            }
        }
        lines
            .into_iter()
            .map(|(text, id, support)| {
                let mut inlines = vec![match id {
                    Some(id) => anchor(&text, id),
                    None => Inline::Text(text),
                }];
                if support {
                    inlines.push(Inline::Text(" (test support)".to_owned()));
                }
                Item {
                    inlines,
                    children: Vec::new(),
                }
            })
            .collect()
    }
}
