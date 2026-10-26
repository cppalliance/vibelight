//! Collects structs, enums, unions, traits, and aliases.

use std::ops::Range;

use syn::visit::Visit;
use syn::{
    Attribute, Fields, ItemEnum, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion,
};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::comments::{Attached, attach};
use crate::macros;
use crate::model::{
    Decl, DeclKind, Holder, Loc, MacroKind, MacroUse, Member, Meta, Note, RefId, Shape, Text,
    Variant, VariantFields, Vis,
};
use crate::source::Comment;
use crate::typerefs::range_of;

/// The parts every declaration shares.
struct Head {
    kind: DeclKind,
    name: String,
    vis: Vis,
    loc: Loc,
    generics: Text,
    where_clause: Option<Text>,
    meta: Meta,
    derives: Vec<String>,
    derive_refs: Vec<RefId>,
    attributes: Vec<String>,
}

impl Collector<'_> {
    pub(crate) fn item_struct(
        &mut self,
        here: &Here,
        s: &ItemStruct,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&s.attrs) {
            return;
        }
        let here = here.with(&s.generics);
        let id = self.decls.len();
        let helpers = !macros::derives(&s.attrs).is_empty();
        let mut members = Vec::new();
        let mut dangling = Vec::new();
        let shape = match &s.fields {
            Fields::Named(named) => {
                (members, dangling) = self.named_fields(&here, named, helpers, id);
                Shape::Named
            }
            Fields::Unnamed(unnamed) => Shape::Tuple(self.tuple_fields(&here, unnamed)),
            Fields::Unit => Shape::Unit,
        };
        let range = range_of(s, &self.files[here.file]);
        let loc = loc(&here, s.ident.span());
        let head = self.head(
            &here,
            DeclKind::Struct,
            &s.ident,
            &s.vis,
            &s.attrs,
            &s.generics,
            lead,
            trail,
            range,
            loc,
        );
        self.push_decl(&here, head, shape, members, dangling);
    }

    pub(crate) fn item_union(
        &mut self,
        here: &Here,
        u: &ItemUnion,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&u.attrs) {
            return;
        }
        let here = here.with(&u.generics);
        let id = self.decls.len();
        let helpers = !macros::derives(&u.attrs).is_empty();
        let (members, dangling) = self.named_fields(&here, &u.fields, helpers, id);
        let range = range_of(u, &self.files[here.file]);
        let loc = loc(&here, u.ident.span());
        let head = self.head(
            &here,
            DeclKind::Union,
            &u.ident,
            &u.vis,
            &u.attrs,
            &u.generics,
            lead,
            trail,
            range,
            loc,
        );
        self.push_decl(&here, head, Shape::Named, members, dangling);
    }

    pub(crate) fn item_enum(
        &mut self,
        here: &Here,
        e: &ItemEnum,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&e.attrs) {
            return;
        }
        let here = here.with(&e.generics);
        let id = self.decls.len();
        let helpers = !macros::derives(&e.attrs).is_empty();
        let files = self.files;
        let file = &files[here.file];
        let scope = file.end(e.brace_token.span.open())..file.start(e.brace_token.span.close());
        let children: Vec<Range<usize>> = e.variants.iter().map(|v| range_of(v, file)).collect();
        let Attached {
            leading,
            trailing,
            dangling,
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut variants = Vec::new();
        for ((variant, lead_v), trail_v) in e.variants.iter().zip(leading).zip(trailing) {
            if attrs::is_test_only(&variant.attrs) {
                continue;
            }
            let fields = match &variant.fields {
                Fields::Unit => VariantFields::Unit,
                Fields::Unnamed(unnamed) => VariantFields::Tuple(self.tuple_fields(&here, unnamed)),
                Fields::Named(named) => VariantFields::Named(self.variant_named(&here, named)),
            };
            let discriminant = variant.discriminant.as_ref().map(|(_, expr)| {
                let range = range_of(expr, file);
                self.discard(&here, range.clone());
                file.clean(range)
            });
            let loc = loc(&here, variant.ident.span());
            let notes = self.node_notes(&here, &variant.attrs, lead_v, range_of(variant, file));
            let mark = self.attribute_mark(&variant.attrs, helpers, Holder::Decl(id), loc);
            variants.push(Variant {
                name: variant.ident.to_string(),
                fields,
                discriminant,
                meta: Meta {
                    notes,
                    trailing: self.trailing(&here, &trail_v),
                    cfgs: attrs::cfgs(&variant.attrs, file),
                    attrs: attrs::api(&variant.attrs, file),
                    marks: mark.into_iter().collect(),
                },
                loc,
            });
        }
        let dangling = self.notes(&here, &[], dangling);
        let range = range_of(e, file);
        let loc = loc(&here, e.ident.span());
        let head = self.head(
            &here,
            DeclKind::Enum,
            &e.ident,
            &e.vis,
            &e.attrs,
            &e.generics,
            lead,
            trail,
            range,
            loc,
        );
        self.push_decl(&here, head, Shape::Enum(variants), Vec::new(), dangling);
    }

    pub(crate) fn item_trait(
        &mut self,
        here: &Here,
        t: &ItemTrait,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&t.attrs) {
            return;
        }
        let here = here.with(&t.generics);
        let id = self.decls.len();
        let files = self.files;
        let file = &files[here.file];
        let mut prefix = String::new();
        if t.unsafety.is_some() {
            prefix.push_str("unsafe ");
        }
        if t.modifiers.auto_token.is_some() {
            prefix.push_str("auto ");
        }
        let supertraits = t
            .supertraits
            .iter()
            .map(|bound| self.bound_base(&here, bound))
            .collect();
        let scope = file.end(t.brace_token.span.open())..file.start(t.brace_token.span.close());
        let (members, dangling) = self.trait_items(&here, &t.items, scope, id);
        let range = range_of(t, file);
        let loc = loc(&here, t.ident.span());
        let head = self.head(
            &here,
            DeclKind::Trait,
            &t.ident,
            &t.vis,
            &t.attrs,
            &t.generics,
            lead,
            trail,
            range,
            loc,
        );
        self.push_decl(
            &here,
            head,
            Shape::Trait {
                prefix,
                supertraits,
            },
            members,
            dangling,
        );
    }

    pub(crate) fn item_trait_alias(
        &mut self,
        here: &Here,
        t: &ItemTraitAlias,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&t.attrs) {
            return;
        }
        let here = here.with(&t.generics);
        let file = &self.files[here.file];
        let bounds = match (t.bounds.first(), t.bounds.last()) {
            (Some(first), Some(last)) => {
                let range = range_of(first, file).start..range_of(last, file).end;
                self.slicer(&here)
                    .range(range, |f| {
                        for bound in &t.bounds {
                            f.visit_type_param_bound(bound);
                        }
                    })
                    .0
            }
            _ => Text::default(),
        };
        let range = range_of(t, &self.files[here.file]);
        let loc = loc(&here, t.ident.span());
        let head = self.head(
            &here,
            DeclKind::TraitAlias,
            &t.ident,
            &t.vis,
            &t.attrs,
            &t.generics,
            lead,
            trail,
            range,
            loc,
        );
        self.push_decl(&here, head, Shape::Alias(bounds), Vec::new(), Vec::new());
    }

    pub(crate) fn item_type(
        &mut self,
        here: &Here,
        t: &ItemType,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        if attrs::is_test_only(&t.attrs) {
            return;
        }
        let here = here.with(&t.generics);
        let target = self.slicer(&here).node(&*t.ty, |f| f.visit_type(&t.ty));
        let range = range_of(t, &self.files[here.file]);
        let loc = loc(&here, t.ident.span());
        let head = self.head(
            &here,
            DeclKind::Alias,
            &t.ident,
            &t.vis,
            &t.attrs,
            &t.generics,
            lead,
            trail,
            range,
            loc,
        );
        self.push_decl(&here, head, Shape::Alias(target), Vec::new(), Vec::new());
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "each argument is one part of the head"
    )]
    fn head(
        &mut self,
        here: &Here,
        kind: DeclKind,
        ident: &syn::Ident,
        vis: &syn::Visibility,
        attrs: &[Attribute],
        generics: &syn::Generics,
        lead: Vec<Comment>,
        trail: &[Comment],
        range: Range<usize>,
        loc: Loc,
    ) -> Head {
        let generics_text = self.generics_text(here, generics);
        let where_clause = self.where_text(here, generics.where_clause.as_ref());
        let notes = self.node_notes(here, attrs, lead, range);
        let derives = macros::derives(attrs);
        let written = macros::derive_paths(attrs);
        let derive_refs = derives
            .iter()
            .map(|name| {
                let span = written.iter().find(|(n, _)| n == name).map(|(_, s)| *s);
                self.derive_ref(here, name, span)
            })
            .collect();
        let attributes = macros::attribute_macros(attrs, !derives.is_empty());
        let id = self.decls.len();
        if !derives.is_empty() {
            self.macros.push(MacroUse {
                kind: MacroKind::Derive,
                text: format!("derive: {}", derives.join(", ")),
                holder: Holder::Decl(id),
                loc,
            });
        }
        if !attributes.is_empty() {
            self.macros.push(MacroUse {
                kind: MacroKind::Attribute,
                text: format!("attribute: {}", attributes.join(", ")),
                holder: Holder::Decl(id),
                loc,
            });
        }
        Head {
            kind,
            name: ident.to_string(),
            vis: attrs::vis(vis),
            loc,
            generics: generics_text,
            where_clause,
            meta: Meta {
                notes,
                trailing: self.trailing(here, trail),
                cfgs: attrs::cfgs(attrs, &self.files[here.file]),
                attrs: attrs::api(attrs, &self.files[here.file]),
                marks: Vec::new(),
            },
            derives,
            derive_refs,
            attributes,
        }
    }

    fn push_decl(
        &mut self,
        here: &Here,
        head: Head,
        shape: Shape,
        members: Vec<Member>,
        dangling: Vec<Note>,
    ) {
        self.decls.push(Decl {
            kind: head.kind,
            name: head.name,
            module: here.module,
            loc: head.loc,
            generics: head.generics,
            where_clause: head.where_clause,
            shape,
            members,
            meta: head.meta,
            derives: head.derives,
            derive_refs: head.derive_refs,
            special: Vec::new(),
            attributes: head.attributes,
            end_marks: Vec::new(),
            dangling,
            bases: Vec::new(),
            impl_notes: Vec::new(),
            impl_locs: Vec::new(),
            vis: head.vis,
            test_support: false,
        });
    }
}
