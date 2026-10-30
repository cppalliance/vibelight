//! Collects impl blocks and the members of impls and traits.

use std::ops::Range;

use syn::visit::Visit;
use syn::{Attribute, ImplItem, ItemImpl, Type, Visibility};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::collect_bases::single_arg;
use crate::collect_body::self_fields;
use crate::collect_body_code::Code;
use crate::comments::{Placed, place_comments};
use crate::macros;
use crate::model::{
    Base, BodyOwner, Holder, Impl, Loc, MacroKind, MacroUse, Member, MemberKind, Meta, Note, Text,
    Vis,
};
use crate::model_body::{BodyKind, Owner};
use crate::typerefs::range_of;
/// The parts of one impl or trait member.
pub(crate) struct Parts<'p> {
    pub(crate) attrs: &'p [Attribute],
    pub(crate) ident: &'p syn::Ident,
    /// `None` for a trait item, which is public.
    pub(crate) vis: Option<&'p Visibility>,
    pub(crate) kind: MemberKind,
    pub(crate) placed: Placed,
    pub(crate) loc: Loc,
    pub(crate) marks: Vec<String>,
    pub(crate) range: Range<usize>,
}

/// What an impl's body holds.
struct ImplBody {
    members: Vec<Member>,
    trait_members: Vec<Member>,
    marks: Vec<String>,
    dangling: Vec<Note>,
}

impl Collector<'_> {
    pub(crate) fn item_impl(&mut self, here: &Here, i: &ItemImpl, placed: Placed) {
        if self.test_only(here, &i.attrs, i) {
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
        let body = self.impl_items(&here, &i.items, scope.clone(), trait_.is_some(), &cfgs, id);
        self.discard(&here, scope);
        let range = range_of(i, file);
        let head_end = file.start(i.brace_token.span.open());
        let first = [
            i.modifiers.defaultness.map(|t| t.span),
            i.unsafety.map(|t| t.span),
        ];
        let first = first
            .into_iter()
            .flatten()
            .next()
            .unwrap_or(i.impl_token.span);
        let header = file
            .clean(file.start(first)..head_end)
            .trim_end_matches(',')
            .to_owned();
        let mut notes = self.node_notes(&here, &i.attrs, placed.leading, range.start..head_end);
        if let Some(text) = self.trailing(&here, &placed.trailing) {
            notes.push(Note::Plain(vec![text]));
        }
        let attributes = macros::attribute_macros(&i.attrs, false);
        self.attribute_names_mark(&attributes, Holder::Impl(id), loc);
        self.impls.push(Impl {
            loc,
            header,
            self_text,
            self_ref,
            trait_,
            members: body.members,
            trait_members: body.trait_members,
            marks: body.marks,
            attributes,
            notes,
            dangling: body.dangling,
            attached: None,
        });
    }

    /// Collects an impl's members; a trait impl's members are not shown, but
    /// its macro invocations are, and analysis keeps its functions,
    /// constants, and types apart.
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
    ) -> ImplBody {
        let files = self.files;
        let file = &files[here.file];
        let children: Vec<Range<usize>> = items.iter().map(|i| range_of(i, file)).collect();
        let (placed, dangling) = place_comments(&mut self.pools[here.file], file, scope, &children);
        let mut members = Vec::new();
        let mut trait_members = Vec::new();
        let mut marks = Vec::new();
        for (item, placed) in items.iter().zip(placed) {
            if self.test_only(here, impl_item_attrs(item), item) {
                continue;
            }
            let range = range_of(item, file);
            if let ImplItem::Fn(f) = item {
                let here = here.with(&f.sig.generics);
                self.body_names(&here, &f.block, BodyOwner::Impl(id));
            }
            let placed = if trait_impl {
                let member = self.trait_member(here, id, item, placed, impl_cfgs);
                trait_members.extend(member);
                Placed::default()
            } else {
                placed
            };
            let member = match item {
                ImplItem::Const(c) if !trait_impl => {
                    let here = here.with(&c.generics);
                    let ty = self.slicer(&here).node(&c.ty, |f| f.visit_type(&c.ty));
                    let loc = loc(&here, c.ident.span());
                    let owner = Owner::Impl(id, c.ident.to_string());
                    let kind = BodyKind::AssocConst;
                    let holder = Holder::Impl(id);
                    let (value, marks) = self.value(&here, &c.expr, holder, loc, kind, owner);
                    let kind = MemberKind::Const { ty, value };
                    let parts = Parts {
                        attrs: &c.attrs,
                        ident: &c.ident,
                        vis: Some(&c.vis),
                        kind,
                        placed,
                        loc,
                        marks,
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                ImplItem::Fn(f) if !trait_impl => {
                    let here = here.with(&f.sig.generics);
                    let owner = Owner::Impl(id, f.sig.ident.to_string());
                    let code = Code::Block(&f.block);
                    self.body(&here, BodyKind::Method, owner, code, Some(&f.sig));
                    let sig = self.sig(&here, &f.sig);
                    let loc = loc(&here, f.sig.ident.span());
                    let marks = self
                        .attribute_mark(&f.attrs, false, Holder::Impl(id), loc)
                        .into_iter()
                        .collect();
                    let kind = MemberKind::Fn(sig);
                    let parts = Parts {
                        attrs: &f.attrs,
                        ident: &f.sig.ident,
                        vis: Some(&f.vis),
                        kind,
                        placed,
                        loc,
                        marks,
                        range,
                    };
                    let mut member = self.member(&here, parts);
                    member.self_fields = self_fields(&f.block);
                    Some(member)
                }
                ImplItem::Type(t) if !trait_impl => {
                    let here = here.with(&t.generics);
                    let mut text = Text::plain(t.ident.to_string());
                    text.append(&self.generics_text(&here, &t.generics));
                    text.push_str(" = ");
                    text.append(&self.slicer(&here).node(&t.ty, |f| f.visit_type(&t.ty)));
                    let loc = loc(&here, t.ident.span());
                    let kind = MemberKind::Type(text);
                    let parts = Parts {
                        attrs: &t.attrs,
                        ident: &t.ident,
                        vis: Some(&t.vis),
                        kind,
                        placed,
                        loc,
                        marks: Vec::new(),
                        range,
                    };
                    Some(self.member(&here, parts))
                }
                ImplItem::Macro(m) => {
                    self.discard(here, macros::delimiter_range(&m.mac.delimiter, file));
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
                    let loc = loc(here, macros::tokens_span(tokens));
                    self.warn(loc, "unsupported verbatim impl item".to_owned());
                    Some(Member {
                        name: String::new(),
                        vis: Vis::Public,
                        kind: MemberKind::Unsupported("verbatim impl item".to_owned()),
                        meta: Meta::default(),
                        loc,
                        self_fields: Vec::new(),
                    })
                }
                ImplItem::Const(_)
                | ImplItem::Fn(_)
                | ImplItem::Type(_)
                | ImplItem::Verbatim(_) => None,
                _ => {
                    let loc = Loc {
                        file: here.file,
                        line: file.line_of(range_of(item, file).start),
                    };
                    self.warn(loc, "unsupported impl item".to_owned());
                    Some(Member {
                        name: String::new(),
                        vis: Vis::Public,
                        kind: MemberKind::Unsupported("unknown impl item".to_owned()),
                        meta: Meta::default(),
                        loc,
                        self_fields: Vec::new(),
                    })
                }
            };
            if let Some(mut member) = member {
                member.meta.cfgs.extend(impl_cfgs.iter().cloned());
                attrs::sort_marks(&mut member.meta.cfgs);
                members.push(member);
            }
        }
        let dangling = if trait_impl {
            Vec::new()
        } else {
            self.notes(here, &[], dangling)
        };
        ImplBody {
            members,
            trait_members,
            marks,
            dangling,
        }
    }

    /// Builds one member; its notes take every comment still unused inside
    /// its range.
    pub(crate) fn member(&mut self, here: &Here, parts: Parts<'_>) -> Member {
        let notes = self.node_notes(here, parts.attrs, parts.placed.leading, parts.range);
        let (raw_doc, lints) = self.attr_facts(here, parts.attrs);
        let file = &self.files[here.file];
        Member {
            name: parts.ident.to_string(),
            vis: parts.vis.map_or(Vis::Public, attrs::vis),
            kind: parts.kind,
            meta: Meta {
                notes,
                trailing: self.trailing(here, &parts.placed.trailing),
                cfgs: attrs::cfgs(parts.attrs, file),
                attrs: attrs::api(parts.attrs, file),
                marks: parts.marks,
                vis: parts
                    .vis
                    .map(|v| attrs::spelled(v, file))
                    .unwrap_or_default(),
                raw_doc,
                lints,
            },
            loc: parts.loc,
            self_fields: Vec::new(),
        }
    }
}

/// Returns an impl item's attributes; a verbatim or unknown item has none.
fn impl_item_attrs(item: &ImplItem) -> &[Attribute] {
    match item {
        ImplItem::Const(c) => &c.attrs,
        ImplItem::Fn(f) => &f.attrs,
        ImplItem::Type(t) => &t.attrs,
        ImplItem::Macro(m) => &m.attrs,
        _ => &[],
    }
}
