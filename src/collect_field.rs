//! Collects the fields of structs, unions, and enum variants.

use std::ops::Range;

use syn::visit::Visit;
use syn::{Field, FieldsNamed, FieldsUnnamed};

use crate::attrs;
use crate::collect::{Collector, Here, loc};
use crate::comments::{Attached, attach};
use crate::model::{DeclId, Holder, Member, MemberKind, Meta, Note, Param, Text};
use crate::typerefs::range_of;

impl Collector<'_> {
    /// Collects named fields, returning the members and the comments with
    /// nothing after them.
    pub(crate) fn named_fields(
        &mut self,
        here: &Here,
        named: &FieldsNamed,
        helpers: bool,
        id: DeclId,
    ) -> (Vec<Member>, Vec<Note>) {
        let files = self.files;
        let file = &files[here.file];
        let scope =
            file.end(named.brace_token.span.open())..file.start(named.brace_token.span.close());
        let fields: Vec<&Field> = named.named.iter().collect();
        let children: Vec<Range<usize>> = fields.iter().map(|f| range_of(*f, file)).collect();
        let Attached {
            leading,
            trailing,
            dangling,
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut members = Vec::new();
        for ((field, lead), trail) in fields.into_iter().zip(leading).zip(trailing) {
            if attrs::is_test_only(&field.attrs) {
                continue;
            }
            let Some(ident) = &field.ident else {
                continue;
            };
            let ty = self
                .slicer(here)
                .node(&field.ty, |f| f.visit_type(&field.ty));
            let loc = loc(here, ident.span());
            let notes = self.node_notes(here, &field.attrs, lead, range_of(field, file));
            let mark = self.attribute_mark(&field.attrs, helpers, Holder::Decl(id), loc);
            members.push(Member {
                name: ident.to_string(),
                vis: attrs::vis(&field.vis),
                kind: MemberKind::Field(ty),
                meta: Meta {
                    notes,
                    trailing: self.trailing(here, &trail),
                    cfgs: attrs::cfgs(&field.attrs, file),
                    attrs: attrs::api(&field.attrs, file),
                    marks: mark.into_iter().collect(),
                    vis: attrs::spelled(&field.vis, file),
                },
                loc,
                self_fields: Vec::new(),
            });
        }
        let dangling = self.notes(here, &[], dangling);
        (members, dangling)
    }

    /// Collects tuple fields as parameters, comments above each.
    pub(crate) fn tuple_fields(&mut self, here: &Here, unnamed: &FieldsUnnamed) -> Vec<Param> {
        let files = self.files;
        let file = &files[here.file];
        let scope =
            file.end(unnamed.paren_token.span.open())..file.start(unnamed.paren_token.span.close());
        let fields: Vec<&Field> = unnamed.unnamed.iter().collect();
        let children: Vec<Range<usize>> = fields.iter().map(|f| range_of(*f, file)).collect();
        let Attached {
            leading, trailing, ..
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut params = Vec::new();
        for ((field, lead), trail) in fields.into_iter().zip(leading).zip(trailing) {
            if attrs::is_test_only(&field.attrs) {
                continue;
            }
            let text = self
                .slicer(here)
                .node(&field.ty, |f| f.visit_type(&field.ty));
            let mut comments = lead;
            comments.extend(trail);
            let notes = self.node_notes(here, &field.attrs, comments, range_of(field, file));
            params.push(Param {
                text,
                meta: Meta {
                    notes,
                    cfgs: attrs::cfgs(&field.attrs, file),
                    vis: attrs::spelled(&field.vis, file),
                    ..Meta::default()
                },
            });
        }
        params
    }

    /// Collects a struct-like variant's fields as `name: Type` entries.
    pub(crate) fn variant_named(&mut self, here: &Here, named: &FieldsNamed) -> Vec<Param> {
        let files = self.files;
        let file = &files[here.file];
        let scope =
            file.end(named.brace_token.span.open())..file.start(named.brace_token.span.close());
        let fields: Vec<&Field> = named.named.iter().collect();
        let children: Vec<Range<usize>> = fields.iter().map(|f| range_of(*f, file)).collect();
        let Attached {
            leading, trailing, ..
        } = attach(&mut self.pools[here.file], file, scope, &children);
        let mut params = Vec::new();
        for ((field, lead), trail) in fields.into_iter().zip(leading).zip(trailing) {
            if attrs::is_test_only(&field.attrs) {
                continue;
            }
            let Some(ident) = &field.ident else {
                continue;
            };
            let mut text = Text::plain(format!("{ident}: "));
            text.append(
                &self
                    .slicer(here)
                    .node(&field.ty, |f| f.visit_type(&field.ty)),
            );
            let mut comments = lead;
            comments.extend(trail);
            let notes = self.node_notes(here, &field.attrs, comments, range_of(field, file));
            params.push(Param {
                text,
                meta: Meta {
                    notes,
                    cfgs: attrs::cfgs(&field.attrs, file),
                    ..Meta::default()
                },
            });
        }
        params
    }
}
