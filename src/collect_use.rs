//! Collects `use` items, extern blocks, constants, and statics.

use std::ops::Range;

use syn::visit::Visit;
use syn::{
    Expr, ForeignItem, ItemConst, ItemForeignMod, ItemStatic, ItemUse, StaticMutability, Visibility,
};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::collect_use_group::{Binding, expand};
use crate::comments::{Attached, attach};
use crate::macros;
use crate::model::{
    Foreign, Holder, Item, ItemKind, Loc, MacroKind, MacroUse, Meta, Text, TypeRef, Use, Vis,
};
use crate::source::{Comment, scan};
use crate::typerefs::range_of;

impl Collector<'_> {
    pub(crate) fn item_use(
        &mut self,
        here: &Here,
        u: &ItemUse,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&u.attrs) {
            return;
        }
        let global = u.leading_colon.is_some();
        let files = self.files;
        let file = &files[here.file];
        let mut bindings = Vec::new();
        expand(
            &u.tree,
            file,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut bindings,
        );
        for binding in &bindings {
            self.modules[here.module].uses.push(Use {
                name: binding.name.clone(),
                path: binding.path.clone(),
                global,
            });
        }
        if matches!(u.vis, Visibility::Inherited) {
            self.discard(here, range_of(u, file));
            return;
        }
        self.reexports(here, u, lead, trail, &bindings);
    }

    /// Returns a re-export's text: `path::Item`, `Alias = path::Item`, or
    /// `path::*`, with the path as one reference.
    pub(crate) fn reexport_text(&mut self, here: &Here, binding: &Binding, global: bool) -> Text {
        let lead = if global { "::" } else { "" };
        let written = format!("{lead}{}", binding.written.join("::"));
        let mut text = Text::default();
        if binding.name.is_none() {
            text.push_str(&format!("{written}::*"));
            return text;
        }
        if let Some(rename) = &binding.rename {
            text.push_str(&format!("{rename} = "));
        }
        let id = self.refs.len();
        self.refs.push(TypeRef {
            text: written,
            path: binding.path.clone(),
            global,
            module: here.module,
            generics: Vec::new(),
            comment: true,
            target: None,
            canonical: None,
            written: None,
            foreign: None,
        });
        text.push_ref(id);
        text
    }

    pub(crate) fn item_const(
        &mut self,
        here: &Here,
        c: &ItemConst,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&c.attrs) || c.ident == "_" {
            return;
        }
        let here = here.with(&c.generics);
        let ty = self.slicer(&here).node(&*c.ty, |f| f.visit_type(&c.ty));
        let loc = loc(&here, c.ident.span());
        let (value, marks) = self.value(&here, &c.expr, Holder::Module(here.module), loc);
        let kind = ItemKind::Const { ty, value };
        self.push_value_item(
            &here,
            &c.attrs,
            &c.vis,
            c.ident.to_string(),
            kind,
            lead,
            trail,
            loc,
            marks,
            range_of(c, &self.files[here.file]),
        );
    }

    pub(crate) fn item_static(
        &mut self,
        here: &Here,
        s: &ItemStatic,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&s.attrs) {
            return;
        }
        let ty = self.slicer(here).node(&*s.ty, |f| f.visit_type(&s.ty));
        let loc = loc(here, s.ident.span());
        let (value, marks) = self.value(here, &s.expr, Holder::Module(here.module), loc);
        let mutable = matches!(s.mutability, StaticMutability::Mut(_));
        let kind = ItemKind::Static { mutable, ty, value };
        self.push_value_item(
            here,
            &s.attrs,
            &s.vis,
            s.ident.to_string(),
            kind,
            lead,
            trail,
            loc,
            marks,
            range_of(s, &self.files[here.file]),
        );
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "each argument is one part of the item"
    )]
    fn push_value_item(
        &mut self,
        here: &Here,
        attrs: &[syn::Attribute],
        vis: &Visibility,
        name: String,
        kind: ItemKind,
        lead: Vec<Comment>,
        trail: &[Comment],
        loc: Loc,
        marks: Vec<String>,
        range: Range<usize>,
    ) {
        let notes = self.node_notes(here, attrs, lead, range);
        let meta = Meta {
            notes,
            trailing: self.trailing(here, trail),
            cfgs: attrs::cfgs(attrs, &self.files[here.file]),
            attrs: attrs::api(attrs, &self.files[here.file]),
            marks,
        };
        self.modules[here.module].items.push(Item {
            name,
            vis: attrs::vis(vis),
            kind,
            meta,
            loc,
        });
    }

    /// Returns a constant's value as written, or `None` with one mark per
    /// macro the value calls. A value holding a comment is not shown either.
    pub(crate) fn value(
        &mut self,
        here: &Here,
        expr: &Expr,
        holder: Holder,
        loc: Loc,
    ) -> (Option<String>, Vec<String>) {
        let files = self.files;
        let file = &files[here.file];
        let range = range_of(expr, file);
        let calls = macros::expr_macros(expr);
        let has_comment = !scan(&file.text, range.clone()).is_empty();
        self.discard(here, range.clone());
        if !calls.is_empty() {
            for call in &calls {
                self.macros.push(MacroUse {
                    kind: MacroKind::Invocation,
                    text: call.clone(),
                    holder,
                    loc,
                });
            }
            return (None, calls);
        }
        if has_comment {
            return (None, Vec::new());
        }
        (Some(file.clean(range)), Vec::new())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one match covers every foreign item kind and the unsupported fallback"
    )]
    pub(crate) fn item_foreign(
        &mut self,
        here: &Here,
        block: &ItemForeignMod,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&block.attrs) {
            return;
        }
        let files = self.files;
        let file = &files[here.file];
        let abi = block.abi.name.as_ref().map_or_else(
            || "\"C\"".to_owned(),
            |name| format!("\"{}\"", name.value()),
        );
        let scope =
            file.end(block.brace_token.span.open())..file.start(block.brace_token.span.close());
        let children: Vec<Range<usize>> = block.items.iter().map(|i| range_of(i, file)).collect();
        let Attached {
            leading,
            trailing,
            dangling,
        } = attach(&mut self.pools[here.file], file, scope.clone(), &children);
        let head = range_of(block, file).start..scope.start;
        let mut block_notes = self.node_notes(here, &block.attrs, lead, head);
        if let Some(text) = self.trailing(here, trail) {
            block_notes.push(crate::model::Note::Plain(vec![text]));
        }
        let cfgs = attrs::cfgs(&block.attrs, file);
        for ((item, lead_i), trail_i) in block.items.iter().zip(leading).zip(trailing) {
            let range = range_of(item, file);
            let (attrs, vis, name, foreign, loc) = match item {
                ForeignItem::Fn(f) => {
                    let here = here.with(&f.sig.generics);
                    let sig = self.sig(&here, &f.sig);
                    (
                        &f.attrs,
                        &f.vis,
                        f.sig.ident.to_string(),
                        Foreign::Fn(sig),
                        loc(&here, f.sig.ident.span()),
                    )
                }
                ForeignItem::Static(s) => {
                    let ty = self.slicer(here).node(&*s.ty, |f| f.visit_type(&s.ty));
                    let mutable = matches!(s.mutability, StaticMutability::Mut(_));
                    (
                        &s.attrs,
                        &s.vis,
                        s.ident.to_string(),
                        Foreign::Static { mutable, ty },
                        loc(here, s.ident.span()),
                    )
                }
                ForeignItem::Type(t) => (
                    &t.attrs,
                    &t.vis,
                    t.ident.to_string(),
                    Foreign::Type,
                    loc(here, t.ident.span()),
                ),
                ForeignItem::Macro(m) => {
                    let text = format!("{}!", macros::path_text(&m.mac.path));
                    let loc = loc(here, m.mac.path.segments[0].ident.span());
                    self.discard(here, range);
                    self.macros.push(MacroUse {
                        kind: MacroKind::Invocation,
                        text: text.clone(),
                        holder: Holder::Module(here.module),
                        loc,
                    });
                    let sort = macros::path_text(&m.mac.path);
                    self.modules[here.module].items.push(Item {
                        name: sort,
                        vis: Vis::Public,
                        kind: ItemKind::Mark(text),
                        meta: Meta::default(),
                        loc,
                    });
                    continue;
                }
                ForeignItem::Verbatim(tokens) => {
                    let loc = loc(here, macros::tokens_span(tokens));
                    self.unsupported_item(here, "verbatim foreign item", loc, true);
                    continue;
                }
                _ => {
                    let loc = Loc {
                        file: here.file,
                        line: file.line_of(range.start),
                    };
                    self.unsupported_item(here, "unknown foreign item", loc, true);
                    continue;
                }
            };
            if attrs::is_test_only(attrs) {
                continue;
            }
            let mut notes = std::mem::take(&mut block_notes);
            notes.extend(self.node_notes(here, attrs, lead_i, range));
            let mut item_cfgs = attrs::cfgs(attrs, file);
            item_cfgs.extend(cfgs.iter().cloned());
            attrs::sort_marks(&mut item_cfgs);
            let meta = Meta {
                notes,
                trailing: self.trailing(here, &trail_i),
                cfgs: item_cfgs,
                attrs: attrs::api(attrs, file),
                marks: Vec::new(),
            };
            self.modules[here.module].items.push(Item {
                name,
                vis: attrs::vis(vis),
                kind: ItemKind::Foreign {
                    abi: abi.clone(),
                    item: foreign,
                },
                meta,
                loc,
            });
        }
        let dangling = self.notes(here, &[], dangling);
        self.modules[here.module].dangling.extend(dangling);
    }
}
