//! Collects impl blocks and the members of impls and traits.

use std::ops::Range;

use syn::visit::Visit;
use syn::{Attribute, ImplItem, ItemImpl, TraitItem, Type};

use crate::attrs;
use crate::collect::{Collector, Here, delimiter_range, is_public, loc, sort_marks, tokens_span};
use crate::collect_bases::single_arg;
use crate::comments::{Attached, attach};
use crate::macros;
use crate::model::{
    Base, DeclId, Holder, Impl, Loc, MacroKind, MacroUse, Member, MemberKind, Meta, Note, Text,
};
use crate::source::Comment;
use crate::typerefs::range_of;
/// The parts of one impl or trait member.
struct Parts<'p> {
    attrs: &'p [Attribute],
    ident: &'p syn::Ident,
    public: bool,
    kind: MemberKind,
    lead: Vec<Comment>,
    trail: &'p [Comment],
    loc: Loc,
    marks: Vec<String>,
    range: Range<usize>,
}

impl Collector<'_> {
    pub(crate) fn item_impl(
        &mut self,
        here: &Here,
        i: &ItemImpl,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&i.attrs) {
            return;
        }
        let here = here.with(&i.generics);
        let id = self.impls.len();
        let files = self.files;
        let file = &files[here.file];
        let self_range = range_of(&*i.self_ty, file);
        let (self_text, ids) = self
            .slicer(&here)
            .range(self_range.clone(), |f| f.visit_type(&i.self_ty));
        let self_ref = match &*i.self_ty {
            Type::Path(path) if path.qself.is_none() => ids
                .iter()
                .find(|(r, _)| r.start == self_range.start)
                .map(|(_, id)| *id),
            _ => None,
        };
        let trait_ = i.trait_.as_ref().map(|(path, _)| {
            let mut text = Text::default();
            if i.unsafety.is_some() {
                text.push_str("unsafe ");
            }
            if i.modifiers.polarity.is_some() {
                text.push_str("!");
            }
            let path_range = range_of(path, file);
            let start = path_range.start;
            let (slice, ids) = self.slicer(&here).range(path_range, |f| f.bare_path(path));
            text.append(&slice);
            let name = path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            Base {
                text,
                name,
                path_ref: ids
                    .iter()
                    .find(|(r, _)| r.start == start)
                    .map(|(_, id)| *id),
                arg: single_arg(path).map(|ty| self.argument(&here, ty)),
                generic: !i.generics.params.is_empty() || i.generics.where_clause.is_some(),
                long: false,
            }
        });
        let loc = loc(&here, i.impl_token.span);
        let scope = file.end(i.brace_token.span.open())..file.start(i.brace_token.span.close());
        let cfgs = attrs::cfgs(&i.attrs, file);
        let (members, marks, dangling) =
            self.impl_items(&here, &i.items, scope.clone(), trait_.is_some(), &cfgs, id);
        self.discard(&here, scope);
        let range = range_of(i, file);
        let head_end = file.start(i.brace_token.span.open());
        let mut notes = self.node_notes(&here, &i.attrs, lead, range.start..head_end);
        if let Some(text) = self.trailing(&here, trail) {
            notes.push(Note::Plain(vec![text]));
        }
        let attributes = macros::attribute_macros(&i.attrs, false);
        if !attributes.is_empty() {
            self.macros.push(MacroUse {
                kind: MacroKind::Attribute,
                text: format!("attribute: {}", attributes.join(", ")),
                holder: Holder::Impl(id),
                loc,
            });
        }
        self.impls.push(Impl {
            loc,
            self_text,
            self_ref,
            trait_,
            members,
            marks,
            attributes,
            notes,
            dangling,
            attached: None,
        });
    }

    /// Collects an impl's members; a trait impl's members are not shown, but
    /// its macro invocations are.
    #[expect(
        clippy::too_many_lines,
        reason = "one match covers every impl item kind and the unsupported fallback"
    )]
    fn impl_items(
        &mut self,
        here: &Here,
        items: &[ImplItem],
        scope: Range<usize>,
        trait_impl: bool,
        impl_cfgs: &[String],
        id: usize,
    ) -> (Vec<Member>, Vec<String>, Vec<Note>) {
        let files = self.files;
        let file = &files[here.file];
        let children: Vec<Range<usize>> = items.iter().map(|i| range_of(i, file)).collect();
        let Attached {
            leading,
            trailing,
            dangling,
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut members = Vec::new();
        let mut marks = Vec::new();
        for ((item, lead), trail) in items.iter().zip(leading).zip(trailing) {
            let range = range_of(item, file);
            let member = match item {
                ImplItem::Const(c) if !attrs::is_test_only(&c.attrs) && !trait_impl => {
                    let here = here.with(&c.generics);
                    let ty = self.slicer(&here).node(&c.ty, |f| f.visit_type(&c.ty));
                    let loc = loc(&here, c.ident.span());
                    let (value, marks) = self.value(&here, &c.expr, Holder::Impl(id), loc);
                    let kind = MemberKind::Const { ty, value };
                    let public = is_public(&c.vis);
                    let parts = Parts {
                        attrs: &c.attrs,
                        ident: &c.ident,
                        public,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                ImplItem::Fn(f) if !attrs::is_test_only(&f.attrs) && !trait_impl => {
                    self.discard(here, range_of(&f.block, file));
                    let here = here.with(&f.sig.generics);
                    let sig = self.sig(&here, &f.sig);
                    let loc = loc(&here, f.sig.ident.span());
                    let marks = self
                        .attribute_mark(&f.attrs, false, Holder::Impl(id), loc)
                        .into_iter()
                        .collect();
                    let kind = MemberKind::Fn(sig);
                    let public = is_public(&f.vis);
                    let parts = Parts {
                        attrs: &f.attrs,
                        ident: &f.sig.ident,
                        public,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                ImplItem::Type(t) if !attrs::is_test_only(&t.attrs) && !trait_impl => {
                    let here = here.with(&t.generics);
                    let mut text = Text::plain(t.ident.to_string());
                    text.append(&self.generics_text(&here, &t.generics));
                    text.push_str(" = ");
                    text.append(&self.slicer(&here).node(&t.ty, |f| f.visit_type(&t.ty)));
                    let loc = loc(&here, t.ident.span());
                    let kind = MemberKind::Type(text);
                    let public = is_public(&t.vis);
                    let parts = Parts {
                        attrs: &t.attrs,
                        ident: &t.ident,
                        public,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks: Vec::new(),
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                ImplItem::Macro(m) if !attrs::is_test_only(&m.attrs) => {
                    self.discard(here, delimiter_range(&m.mac.delimiter, file));
                    let text = format!("{}!", macros::path_text(&m.mac.path));
                    let loc = loc(here, m.mac.path.segments[0].ident.span());
                    self.macros.push(MacroUse {
                        kind: MacroKind::Invocation,
                        text: text.clone(),
                        holder: Holder::Impl(id),
                        loc,
                    });
                    marks.push(text);
                    None
                }
                ImplItem::Verbatim(tokens) if !trait_impl => {
                    let loc = loc(here, tokens_span(tokens));
                    self.warn(loc, "unsupported verbatim impl item".to_owned());
                    Some(Member {
                        name: String::new(),
                        public: true,
                        kind: MemberKind::Unsupported("verbatim impl item".to_owned()),
                        meta: Meta::default(),
                        loc,
                    })
                }
                ImplItem::Const(_)
                | ImplItem::Fn(_)
                | ImplItem::Type(_)
                | ImplItem::Macro(_)
                | ImplItem::Verbatim(_) => None,
                _ => {
                    let loc = Loc {
                        file: here.file,
                        line: file.line_of(range_of(item, file).start),
                    };
                    self.warn(loc, "unsupported impl item".to_owned());
                    Some(Member {
                        name: String::new(),
                        public: true,
                        kind: MemberKind::Unsupported("unknown impl item".to_owned()),
                        meta: Meta::default(),
                        loc,
                    })
                }
            };
            if let Some(mut member) = member {
                member.meta.cfgs.extend(impl_cfgs.iter().cloned());
                sort_marks(&mut member.meta.cfgs);
                members.push(member);
            }
        }
        let dangling = if trait_impl {
            Vec::new()
        } else {
            self.notes(here, &[], dangling)
        };
        (members, marks, dangling)
    }

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
            let range = range_of(item, file);
            let member = match item {
                TraitItem::Const(c) if !attrs::is_test_only(&c.attrs) => {
                    let here = here.with(&c.generics);
                    let ty = self.slicer(&here).node(&c.ty, |f| f.visit_type(&c.ty));
                    let loc = loc(&here, c.ident.span());
                    let (value, marks) = match &c.default {
                        Some((_, expr)) => self.value(&here, expr, Holder::Decl(id), loc),
                        None => (None, Vec::new()),
                    };
                    let kind = MemberKind::Const { ty, value };
                    let parts = Parts {
                        attrs: &c.attrs,
                        ident: &c.ident,
                        public: true,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                TraitItem::Fn(f) if !attrs::is_test_only(&f.attrs) => {
                    if let Some(block) = &f.default {
                        self.discard(here, range_of(block, file));
                    }
                    let here = here.with(&f.sig.generics);
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
                        public: true,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                TraitItem::Type(t) if !attrs::is_test_only(&t.attrs) => {
                    let here = here.with(&t.generics);
                    let text = self.trait_type(&here, t);
                    let loc = loc(&here, t.ident.span());
                    let kind = MemberKind::Type(text);
                    let parts = Parts {
                        attrs: &t.attrs,
                        ident: &t.ident,
                        public: true,
                        kind,
                        lead,
                        trail: &trail,
                        loc,
                        marks: Vec::new(),
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                TraitItem::Macro(m) if !attrs::is_test_only(&m.attrs) => {
                    self.discard(here, delimiter_range(&m.mac.delimiter, file));
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
                        public: true,
                        kind: MemberKind::Mark(text),
                        meta: Meta::default(),
                        loc,
                    })
                }
                TraitItem::Verbatim(tokens) => {
                    let loc = loc(here, tokens_span(tokens));
                    self.warn(loc, "unsupported verbatim trait item".to_owned());
                    Some(Member {
                        name: String::new(),
                        public: true,
                        kind: MemberKind::Unsupported("verbatim trait item".to_owned()),
                        meta: Meta::default(),
                        loc,
                    })
                }
                TraitItem::Const(_)
                | TraitItem::Fn(_)
                | TraitItem::Type(_)
                | TraitItem::Macro(_) => None,
                _ => {
                    let loc = Loc {
                        file: here.file,
                        line: file.line_of(range_of(item, file).start),
                    };
                    self.warn(loc, "unsupported trait item".to_owned());
                    Some(Member {
                        name: String::new(),
                        public: true,
                        kind: MemberKind::Unsupported("unknown trait item".to_owned()),
                        meta: Meta::default(),
                        loc,
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
            text.append(&clause);
        }
        text
    }

    /// Builds one member; its notes take every comment still unused inside
    /// its range.
    fn member(&mut self, here: &Here, parts: Parts<'_>) -> Member {
        let notes = self.node_notes(here, parts.attrs, parts.lead, parts.range);
        Member {
            name: parts.ident.to_string(),
            public: parts.public,
            kind: parts.kind,
            meta: Meta {
                notes,
                trailing: self.trailing(here, parts.trail),
                cfgs: attrs::cfgs(parts.attrs, &self.files[here.file]),
                marks: parts.marks,
            },
            loc: parts.loc,
        }
    }
}
