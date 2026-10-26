//! Builds the Public surface section: every path a user of the crate can
//! name, each linking to the heading of what it names.

use crate::document::{Builder, Inline, Item, anchor};
use crate::surface::{Entry, Kind};

impl Builder<'_> {
    /// Returns one list item per surface path. A function's path ends in
    /// `()` and a macro's in `!`. Functions, constants, statics, and macros
    /// have no headings, so they link to the heading of the module that
    /// declares them. A re-export of another crate's item, or one the walk
    /// cannot place, shows the path as written after it and has no link.
    /// A path that two declarations share, as under different `cfg`s, is
    /// listed once, linking to the heading whose id sorts first.
    pub(crate) fn surface_items(&self, surface: &[Entry]) -> Vec<Item> {
        let mut lines: Vec<(String, Option<&str>)> = Vec::new();
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
                Some(last) if last.0 == text => last.1 = last.1.min(id),
                _ => lines.push((text, id)),
            }
        }
        lines
            .into_iter()
            .map(|(text, id)| Item {
                inlines: vec![match id {
                    Some(id) => anchor(&text, id),
                    None => Inline::Text(text),
                }],
                children: Vec::new(),
            })
            .collect()
    }
}
