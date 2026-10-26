//! Builds the Public surface section: every path a user of the crate can
//! name, each linking to the heading of what it names.

use crate::document::{Builder, Item, anchor};
use crate::surface::{Entry, Kind};

impl Builder<'_> {
    /// Returns one list item per surface path. A function's path ends in
    /// `()`. Functions, constants, and statics have no headings, so they
    /// link to the heading of the module that declares them. A path that
    /// two declarations share, as under different `cfg`s, is listed once,
    /// linking to the heading whose id sorts first.
    pub(crate) fn surface_items(&self, surface: &[Entry]) -> Vec<Item> {
        let mut lines: Vec<(String, &str)> = Vec::new();
        for entry in surface {
            let mut text = entry.path.join("::");
            let id = match entry.kind {
                Kind::Module(m) | Kind::Value(m) => &self.mod_ids[m],
                Kind::Decl(d) => &self.decl_ids[d],
                Kind::Function(m) => {
                    text.push_str("()");
                    &self.mod_ids[m]
                }
            };
            match lines.last_mut() {
                Some(last) if last.0 == text => last.1 = last.1.min(id.as_str()),
                _ => lines.push((text, id)),
            }
        }
        lines
            .into_iter()
            .map(|(text, id)| Item {
                inlines: vec![anchor(&text, id)],
                children: Vec::new(),
            })
            .collect()
    }
}
