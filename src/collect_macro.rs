//! Collects macro items: module-level definitions and invocations, the
//! end marks an invocation gives the declarations it names, and the
//! attribute macros of a node.

use syn::{Attribute, ItemMacro};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::comments::Placed;
use crate::macros;
use crate::model::{Holder, Item as ModItem, ItemKind, Loc, MacroKind, MacroUse, Meta, Vis};
use crate::typerefs::{range_of, strip_raw};

impl Collector<'_> {
    pub(crate) fn item_macro(&mut self, here: &Here, m: &ItemMacro, placed: Placed) {
        if self.test_only(here, &m.attrs, m) {
            return;
        }
        self.discard(
            here,
            macros::delimiter_range(&m.mac.delimiter, &self.files[here.file]),
        );
        let notes = self.node_notes(
            here,
            &m.attrs,
            placed.leading,
            range_of(m, &self.files[here.file]),
        );
        let name = macros::path_text(&m.mac.path);
        let (text, sort, kind) = match &m.ident {
            Some(ident) if m.mac.path.is_ident("macro_rules") => {
                let ident = ident.to_string();
                if macros::is_exported(&m.attrs) {
                    self.exported_macros.push((ident.clone(), here.module));
                }
                (
                    macros::definition_text(&ident),
                    ident,
                    MacroKind::Definition,
                )
            }
            _ => (format!("{name}!"), name, MacroKind::Invocation),
        };
        let loc = loc(here, m.mac.path.segments[0].ident.span());
        if kind == MacroKind::Invocation {
            self.invocations
                .push((here.module, text.clone(), macros::idents(&m.mac.tokens)));
        }
        self.macros.push(MacroUse {
            kind,
            text: text.clone(),
            holder: Holder::Module(here.module),
            loc,
        });
        let (raw_doc, lints) = self.attr_facts(here, &m.attrs);
        let meta = Meta {
            notes,
            trailing: self.trailing(here, &placed.trailing),
            cfgs: attrs::cfgs(&m.attrs, &self.files[here.file]),
            attrs: attrs::api(&m.attrs, &self.files[here.file]),
            marks: Vec::new(),
            vis: String::new(),
            raw_doc,
            lints,
        };
        self.modules[here.module].items.push(ModItem {
            name: sort,
            vis: Vis::Public,
            kind: ItemKind::Mark(text),
            meta,
            loc,
        });
    }

    /// Gives each declaration named by a module-level invocation in its
    /// module a mark at the end of its block.
    pub(crate) fn finish_invocations(&mut self) {
        for (module, text, idents) in std::mem::take(&mut self.invocations) {
            for decl in self.decls.iter_mut().filter(|d| d.module == module) {
                if idents.contains(&strip_raw(&decl.name)) && !decl.end_marks.contains(&text) {
                    decl.end_marks.push(text.clone());
                }
            }
        }
    }

    /// Records attribute macros on a node in the kill list and returns its
    /// mark, `attribute: a, b`, if it has any.
    pub(crate) fn attribute_mark(
        &mut self,
        attrs: &[Attribute],
        helpers: bool,
        holder: Holder,
        loc: Loc,
    ) -> Option<String> {
        let names = macros::attribute_macros(attrs, helpers);
        self.attribute_names_mark(&names, holder, loc)
    }

    /// Records the attribute macros `names` in the kill list and returns
    /// their mark, `attribute: a, b`, if there are any.
    pub(crate) fn attribute_names_mark(
        &mut self,
        names: &[String],
        holder: Holder,
        loc: Loc,
    ) -> Option<String> {
        if names.is_empty() {
            return None;
        }
        let text = macros::mark_text(macros::ATTRIBUTE, names);
        self.macros.push(MacroUse {
            kind: MacroKind::Attribute,
            text: text.clone(),
            holder,
            loc,
        });
        Some(text)
    }
}
