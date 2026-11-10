//! Builds the Public surface section: every path a user of the crate can
//! name, each linking to the heading of what it names.

use crate::html::document::{Builder, Inline, Item, Node, anchor};
use crate::model::{CrossRef, ValueRef};
use crate::surface::{Entry, Kind};

/// Where a surface line links.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Target<'a> {
    /// The heading of this page with the id.
    Page(&'a str),
    /// An item of another crate of the run.
    Cross(CrossRef),
}

/// One surface line before it is made into inline content.
struct Line<'a> {
    /// The path, ending in `()` for a function and `!` for a macro.
    text: String,
    /// The path a re-export of another crate's item is written with.
    written: Option<&'a str>,
    target: Option<Target<'a>>,
    test_support: bool,
}

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
    /// `()` and a macro's in `!`. Functions, constants, and statics link to
    /// their ids, and macros, which have none, to the heading of the module
    /// that declares them. A re-export of an item of another crate of the
    /// run shows the path as written after it, and links to the item on that
    /// crate's page. A re-export of an item of a crate outside the run, or
    /// one the walk cannot place, shows the path as written and has no link.
    /// A path that two declarations share, as under different `cfg`s, is
    /// listed once, linking to the heading whose id sorts first.
    fn surface_items(&self, surface: &[Entry]) -> Vec<Item> {
        let mut lines: Vec<Line<'_>> = Vec::new();
        for entry in surface {
            let mut text = entry.path.join("::");
            let value = |at: &ValueRef| {
                Target::Page((self.value_ids.get(at)).unwrap_or(&self.mod_ids[at.module]))
            };
            let mut written = None;
            let target = match &entry.kind {
                Kind::Module(m) => Some(Target::Page(&self.mod_ids[*m])),
                Kind::Value(at) => Some(value(at)),
                Kind::Decl(d) => Some(Target::Page(&self.decl_ids[*d])),
                Kind::Function(at) => {
                    text.push_str("()");
                    Some(value(at))
                }
                Kind::Macro(m) => {
                    text.push('!');
                    Some(Target::Page(&self.mod_ids[*m]))
                }
                Kind::Cross(path, to) => {
                    written = Some(path.as_str());
                    Some(Target::Cross(*to))
                }
                Kind::Unlinked(path) => {
                    written = Some(path.as_str());
                    None
                }
            };
            match lines.last_mut() {
                Some(last) if last.text == text && last.written == written => {
                    last.target = last.target.min(target);
                    last.test_support &= entry.test_support;
                }
                _ => lines.push(Line {
                    text,
                    written,
                    target,
                    test_support: entry.test_support,
                }),
            }
        }
        lines
            .into_iter()
            .map(|line| {
                let Line {
                    text,
                    written,
                    target,
                    test_support,
                } = line;
                let mut inlines = vec![match target {
                    Some(Target::Page(id)) => anchor(&text, id),
                    Some(Target::Cross(to)) => Inline::Cross { text, to },
                    None => Inline::Text(text),
                }];
                if let Some(path) = written {
                    inlines.push(Inline::Text(format!(" ({path})")));
                }
                if test_support {
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
