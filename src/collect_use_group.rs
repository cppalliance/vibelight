//! Expands `use` trees into the names they bind, and gives each re-exported
//! name the comments written next to it inside the braces.

use std::ops::Range;

use syn::spanned::Spanned;
use syn::{ItemUse, UseTree};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::comments::{Placed, place_comments};
use crate::model::{Item, ItemKind, Meta, Note, Text, name_order};
use crate::source::SourceFile;
use crate::typerefs::strip_raw;

/// One name a `use` tree binds.
pub(crate) struct Binding {
    /// The bound name, or `None` for a glob.
    pub(crate) name: Option<String>,
    /// The source path as written, with `r#` kept, ending in the item's own
    /// name; a glob's module path.
    pub(crate) written: Vec<String>,
    pub(crate) rename: Option<String>,
    /// Where the name, its rename, or the glob's `*` is written.
    range: Range<usize>,
}

impl Binding {
    /// Returns the source path with `r#` stripped from each segment.
    pub(crate) fn path(&self) -> Vec<String> {
        self.written.iter().map(|s| strip_raw(s)).collect()
    }
}

/// One re-exported name and its own comments, before sorting.
struct Named {
    sort: String,
    text: Text,
    notes: Vec<Note>,
    trailing: Option<String>,
}

impl Collector<'_> {
    /// Pushes one module item per name a public `use` binds. A comment
    /// inside the braces belongs to the name after it. When the `use` has
    /// comments of its own and binds more than one name, its names form a
    /// group, which prints together under those comments.
    pub(crate) fn reexports(
        &mut self,
        here: &Here,
        u: &ItemUse,
        placed: Placed,
        bindings: &[Binding],
    ) {
        let files = self.files;
        let file = &files[here.file];
        let range = crate::typerefs::range_of(u, file);
        let global = u.leading_colon.is_some();
        let children: Vec<Range<usize>> = bindings.iter().map(|b| b.range.clone()).collect();
        let (per_name, dangling) =
            place_comments(&mut self.pools[here.file], file, range.clone(), &children);
        let mut notes = self.notes(here, &u.attrs, placed.leading);
        notes.extend(self.notes(here, &[], dangling));
        let mut named = Vec::new();
        for (binding, around) in bindings.iter().zip(per_name) {
            named.push(Named {
                sort: sort_name(binding),
                text: self.reexport_text(here, binding, global),
                notes: self.notes(here, &[], around.leading),
                trailing: self.trailing(here, &around.trailing),
            });
        }
        named.sort_by(|a, b| {
            name_order(&a.sort, &b.sort)
                .then_with(|| a.text.visible(&self.refs).cmp(&b.text.visible(&self.refs)))
        });
        let group = (!notes.is_empty() && named.len() > 1).then_some(range.start);
        let cfgs = attrs::cfgs(&u.attrs, file);
        let api = attrs::api(&u.attrs, file);
        let trailing = self.trailing(here, &placed.trailing);
        let loc = loc(here, u.use_token.span);
        let vis = attrs::vis(&u.vis);
        let (raw_doc, lints) = self.attr_facts(here, &u.attrs);
        for (index, entry) in named.into_iter().enumerate() {
            let mut meta = Meta {
                notes: entry.notes,
                trailing: entry.trailing,
                cfgs: cfgs.clone(),
                attrs: api.clone(),
                marks: Vec::new(),
                vis: attrs::spelled(&u.vis, file),
                raw_doc: raw_doc.clone(),
                lints: lints.clone(),
            };
            if index == 0 {
                meta.notes.splice(0..0, notes.iter().cloned());
                meta.trailing = match (trailing.clone(), meta.trailing) {
                    (Some(outer), Some(inner)) => Some(format!("{inner} {outer}")),
                    (outer, inner) => inner.or(outer),
                };
            }
            self.modules[here.module].items.push(Item {
                name: entry.sort,
                vis,
                kind: ItemKind::Reexport {
                    text: entry.text,
                    group,
                },
                meta,
                loc,
            });
        }
    }
}

/// The name a re-export sorts by: its rename, its own name, or `*`.
fn sort_name(binding: &Binding) -> String {
    match (&binding.name, &binding.rename) {
        (None, _) => "*".to_owned(),
        (Some(_), Some(rename)) => rename.clone(),
        (Some(_), None) => binding.written.last().cloned().unwrap_or_default(),
    }
}

/// Expands a `use` tree into the names it binds, in written order.
pub(crate) fn expand(
    tree: &UseTree,
    file: &SourceFile,
    written: &mut Vec<String>,
    out: &mut Vec<Binding>,
) {
    match tree {
        UseTree::Path(p) => {
            written.push(p.ident.to_string());
            expand(&p.tree, file, written, out);
            written.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.to_string();
            let range = file.start(n.ident.span())..file.end(n.ident.span());
            if name == "self" {
                let bound = written.last().cloned().unwrap_or_default();
                out.push(Binding {
                    name: Some(strip_raw(&bound)),
                    written: written.clone(),
                    rename: None,
                    range,
                });
            } else {
                let mut shown = written.clone();
                shown.push(name.clone());
                out.push(Binding {
                    name: Some(strip_raw(&name)),
                    written: shown,
                    rename: None,
                    range,
                });
            }
        }
        UseTree::Rename(r) => {
            let name = r.ident.to_string();
            let mut shown = written.clone();
            if name != "self" {
                shown.push(name);
            }
            let rename = r.rename.to_string();
            out.push(Binding {
                name: Some(strip_raw(&rename)),
                written: shown,
                rename: Some(rename),
                range: file.start(r.ident.span())..file.end(r.rename.span()),
            });
        }
        UseTree::Glob(g) => out.push(Binding {
            name: None,
            written: written.clone(),
            rename: None,
            range: file.start(g.span())..file.end(g.span()),
        }),
        UseTree::Group(g) => {
            for item in &g.items {
                expand(item, file, written, out);
            }
        }
    }
}
