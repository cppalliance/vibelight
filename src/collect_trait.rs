//! Collects the items of traits, and the members of trait impls, which only
//! analysis reads.

use std::ops::Range;

use syn::visit::Visit;
use syn::{Attribute, Expr, ImplItem, TraitItem};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::collect_body::self_fields;
use crate::collect_body_code::Code;
use crate::collect_impl::Parts;
use crate::comments::{Attached, attach};
use crate::macros;
use crate::model::{
    BodyOwner, DeclId, Holder, Loc, MacroKind, MacroUse, Member, MemberKind, Meta, Note, Text, Vis,
};
use crate::model_body::{BodyKind, Owner};
use crate::source::{Comment, SourceFile, scan};
use crate::typerefs::range_of;

impl Collector<'_> {
    /// Collects a trait's items; every one is public.
    #[expect(
        clippy::too_many_lines,
        reason = "one match covers every trait item kind and the unsupported fallback"
    )]
    pub(crate) fn trait_items(
        &mut self,
        here: &Here,
        items: &[TraitItem],
        scope: Range<usize>,
        id: DeclId,
    ) -> (Vec<Member>, Vec<Note>) {
        let files = self.files;
        let file = &files[here.file];
        let children: Vec<Range<usize>> = items.iter().map(|i| range_of(i, file)).collect();
        let Attached {
            leading,
            trailing,
            dangling,
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut members = Vec::new();
        for ((item, lead), trail) in items.iter().zip(leading).zip(trailing) {
            if self.test_only(here, trait_item_attrs(item), item) {
                continue;
            }
            let range = range_of(item, file);
            let member = match item {
                TraitItem::Const(c) => {
                    let here = here.with(&c.generics);
                    let ty = self.slicer(&here).node(&c.ty, |f| f.visit_type(&c.ty));
                    let loc = loc(&here, c.ident.span());
                    let (value, marks) = match &c.default {
                        Some((_, expr)) => {
                            let owner = Owner::Trait(id, c.ident.to_string());
                            let holder = Holder::Decl(id);
                            self.value(&here, expr, holder, loc, BodyKind::AssocConst, owner)
                        }
                        None => (None, Vec::new()),
                    };
                    let kind = MemberKind::Const { ty, value };
                    let parts = Parts {
                        attrs: &c.attrs,
                        ident: &c.ident,
                        vis: None,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                TraitItem::Fn(f) => {
                    let here = here.with(&f.sig.generics);
                    if let Some(block) = &f.default {
                        let owner = Owner::Trait(id, f.sig.ident.to_string());
                        let code = Code::Block(block);
                        self.body(&here, BodyKind::TraitDefault, owner, code, Some(&f.sig));
                        self.body_names(&here, block, BodyOwner::Trait(id));
                    }
                    let sig = self.sig(&here, &f.sig);
                    let loc = loc(&here, f.sig.ident.span());
                    let marks = self
                        .attribute_mark(&f.attrs, false, Holder::Decl(id), loc)
                        .into_iter()
                        .collect();
                    let kind = MemberKind::Fn(sig);
                    let parts = Parts {
                        attrs: &f.attrs,
                        ident: &f.sig.ident,
                        vis: None,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                TraitItem::Type(t) => {
                    let here = here.with(&t.generics);
                    let text = self.trait_type(&here, t);
                    let loc = loc(&here, t.ident.span());
                    let kind = MemberKind::Type(text);
                    let parts = Parts {
                        attrs: &t.attrs,
                        ident: &t.ident,
                        vis: None,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks: Vec::new(),
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                TraitItem::Macro(m) => {
                    self.discard(here, macros::delimiter_range(&m.mac.delimiter, file));
                    let text = format!("{}!", macros::path_text(&m.mac.path));
                    let loc = loc(here, m.mac.path.segments[0].ident.span());
                    self.macros.push(MacroUse {
                        kind: MacroKind::Invocation,
                        text: text.clone(),
                        holder: Holder::Decl(id),
                        loc,
                    });
                    Some(Member {
                        name: text.clone(),
                        vis: Vis::Public,
                        kind: MemberKind::Mark(text),
                        meta: Meta::default(),
                        loc,
                        self_fields: Vec::new(),
                    })
                }
                TraitItem::Verbatim(tokens) => {
                    let loc = loc(here, macros::tokens_span(tokens));
                    self.warn(loc, "unsupported verbatim trait item".to_owned());
                    Some(Member {
                        name: String::new(),
                        vis: Vis::Public,
                        kind: MemberKind::Unsupported("verbatim trait item".to_owned()),
                        meta: Meta::default(),
                        loc,
                        self_fields: Vec::new(),
                    })
                }
                _ => {
                    let loc = Loc {
                        file: here.file,
                        line: file.line_of(range_of(item, file).start),
                    };
                    self.warn(loc, "unsupported trait item".to_owned());
                    Some(Member {
                        name: String::new(),
                        vis: Vis::Public,
                        kind: MemberKind::Unsupported("unknown trait item".to_owned()),
                        meta: Meta::default(),
                        loc,
                        self_fields: Vec::new(),
                    })
                }
            };
            members.extend(member);
        }
        let dangling = self.notes(here, &[], dangling);
        (members, dangling)
    }

    /// Returns an associated type's text after `using`: its name, generics,
    /// bounds, default, and `where` clause.
    fn trait_type(&mut self, here: &Here, t: &syn::TraitItemType) -> Text {
        let files = self.files;
        let file = &files[here.file];
        let mut text = Text::plain(t.ident.to_string());
        text.append(&self.generics_text(here, &t.generics));
        if let (Some(first), Some(last)) = (t.bounds.first(), t.bounds.last()) {
            let range = range_of(first, file).start..range_of(last, file).end;
            let (bounds, _) = self.slicer(here).range(range, |f| {
                for bound in &t.bounds {
                    f.visit_type_param_bound(bound);
                }
            });
            text.push_str(": ");
            text.append(&bounds);
        }
        if let Some((_, ty)) = &t.default {
            text.push_str(" = ");
            text.append(&self.slicer(here).node(ty, |f| f.visit_type(ty)));
        }
        if let Some(clause) = self.where_text(here, t.generics.where_clause.as_ref()) {
            text.push_str(" ");
            text.append(&clause.joined());
        }
        text
    }

    /// Builds one item of a trait impl for analysis only: a function's
    /// signature, a constant's type and value, or a type. It keeps plain
    /// comments and the raw doc text, but no doc note, mark, or kill list
    /// entry, and every reference it writes becomes a comment reference, so
    /// no outline counts or shows it. A function's or constant's code
    /// records a body. Any other item gives `None`.
    pub(crate) fn trait_member(
        &mut self,
        here: &Here,
        id: usize,
        item: &ImplItem,
        lead: Vec<Comment>,
        trail: &[Comment],
        impl_cfgs: &[String],
    ) -> Option<Member> {
        let files = self.files;
        let file = &files[here.file];
        let first = self.refs.len();
        let (attrs, ident, vis, kind, loc, self_fields) = match item {
            ImplItem::Fn(f) => {
                let here = here.with(&f.sig.generics);
                let owner = Owner::Impl(id, f.sig.ident.to_string());
                let code = Code::Block(&f.block);
                self.body(&here, BodyKind::TraitImplMethod, owner, code, Some(&f.sig));
                let kind = MemberKind::Fn(self.sig(&here, &f.sig));
                let loc = loc(&here, f.sig.ident.span());
                (
                    &f.attrs,
                    &f.sig.ident,
                    &f.vis,
                    kind,
                    loc,
                    self_fields(&f.block),
                )
            }
            ImplItem::Const(c) => {
                let here = here.with(&c.generics);
                let ty = self.slicer(&here).node(&c.ty, |f| f.visit_type(&c.ty));
                let value = written_value(file, &c.expr);
                let owner = Owner::Impl(id, c.ident.to_string());
                self.body(
                    &here,
                    BodyKind::AssocConst,
                    owner,
                    Code::Expr(&c.expr),
                    None,
                );
                let kind = MemberKind::Const { ty, value };
                (
                    &c.attrs,
                    &c.ident,
                    &c.vis,
                    kind,
                    loc(&here, c.ident.span()),
                    Vec::new(),
                )
            }
            ImplItem::Type(t) => {
                let here = here.with(&t.generics);
                let mut text = Text::plain(t.ident.to_string());
                text.append(&self.generics_text(&here, &t.generics));
                text.push_str(" = ");
                text.append(&self.slicer(&here).node(&t.ty, |f| f.visit_type(&t.ty)));
                let kind = MemberKind::Type(text);
                (
                    &t.attrs,
                    &t.ident,
                    &t.vis,
                    kind,
                    loc(&here, t.ident.span()),
                    Vec::new(),
                )
            }
            _ => return None,
        };
        let mut comments = lead;
        comments.extend(self.pools[here.file].take(range_of(item, file)));
        let notes = self.notes(here, &[], comments);
        for reference in &mut self.refs[first..] {
            reference.comment = true;
        }
        let (raw_doc, lints) = self.attr_facts(here, attrs);
        let mut cfgs = attrs::cfgs(attrs, file);
        cfgs.extend(impl_cfgs.iter().cloned());
        attrs::sort_marks(&mut cfgs);
        Some(Member {
            name: ident.to_string(),
            vis: attrs::vis(vis),
            kind,
            meta: Meta {
                notes,
                trailing: self.trailing(here, trail),
                cfgs,
                attrs: attrs::api(attrs, file),
                marks: Vec::new(),
                vis: attrs::spelled(vis, file),
                raw_doc,
                lints,
            },
            loc,
            self_fields,
        })
    }
}

/// Returns a trait item's attributes; a verbatim or unknown item has none.
fn trait_item_attrs(item: &TraitItem) -> &[Attribute] {
    match item {
        TraitItem::Const(c) => &c.attrs,
        TraitItem::Fn(f) => &f.attrs,
        TraitItem::Type(t) => &t.attrs,
        TraitItem::Macro(m) => &m.attrs,
        _ => &[],
    }
}

/// Returns a constant's value as written, or `None` when it invokes a macro
/// or holds a comment, the way `value` decides, without recording a mark.
fn written_value(file: &SourceFile, expr: &Expr) -> Option<String> {
    let range = range_of(expr, file);
    let plain = macros::expr_macros(expr).is_empty() && scan(&file.text, range.clone()).is_empty();
    plain.then(|| file.clean(range))
}
