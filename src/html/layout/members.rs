//! Lays out the entries between a type's braces: members and variants.

use crate::access::Access;
use crate::html::layout::groups::{Entry, reaches_line, spaced};
use crate::html::layout::marker::Marker;
use crate::html::layout::{
    Line, Span, above, attributes_end, code, const_line, fn_lines, key, note_lines, plain,
    vis_spans, width,
};
use crate::html::wrap::MAX_WIDTH;
use crate::model::{Crate, Member, MemberKind, Param, TypeRef, Variant, VariantFields};

/// Returns a member's entry, with a `reaches` line above it when it reaches
/// less than its visibility gives and its type's note, `shown`, says
/// otherwise.
pub(crate) fn member_entry(krate: &Crate, member: &Member, shown: Option<Access>) -> Entry {
    let refs = &krate.refs;
    let attrs_above = matches!(member.kind, MemberKind::Type(_) | MemberKind::Const { .. });
    let (mut lines, repeats) = above(&member.meta, 4, member.loc, attrs_above);
    lines.extend(reaches_line(&member.meta, 4, shown));
    let name = Span::Name(member.name.clone(), member.loc);
    match &member.kind {
        MemberKind::Type(text) => {
            let mut spans = vec![key("using"), plain(" ")];
            let mut rest = code(text);
            if let Some(Span::Code(first)) = rest.first_mut() {
                let split = first
                    .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '#'))
                    .unwrap_or(first.len());
                let (ident, after) = first.split_at(split);
                let after = after.to_owned();
                spans.push(Span::Name(ident.to_owned(), member.loc));
                *first = after;
            }
            spans.extend(rest);
            lines.push(Line {
                indent: 4,
                spans,
                trailing: member.meta.trailing.clone(),
            });
        }
        MemberKind::Const { ty, value } => {
            let mut spans = vec![
                key("static"),
                plain(" "),
                key("constexpr"),
                plain(" "),
                name,
                plain(": "),
            ];
            spans.extend(code(ty));
            lines.push(const_line(
                spans,
                value.as_deref(),
                4,
                member.meta.trailing.clone(),
                refs,
            ));
        }
        MemberKind::Field(ty) => {
            let mut spans = vec![name, plain(": ")];
            spans.extend(code(ty));
            spans.push(plain(","));
            spans.extend(attributes_end(&member.meta.attrs));
            lines.push(Line {
                indent: 4,
                spans,
                trailing: member.meta.trailing.clone(),
            });
        }
        MemberKind::Fn(sig) => {
            let trailing = member.meta.trailing.clone();
            lines.extend(fn_lines(
                krate,
                sig,
                Span::Fn(sig.name.clone(), member.loc, None),
                trailing,
                4,
                member.kind.is_static_fn(),
                &member.meta.attrs,
            ));
        }
        MemberKind::Mark(_) | MemberKind::Unsupported(_) => {}
    }
    Entry {
        name: member.name.clone(),
        lines,
        repeats,
    }
}

pub(crate) fn variant_entry(variant: &Variant, refs: &[TypeRef]) -> Entry {
    let (mut lines, repeats) = above(&variant.meta, 4, variant.loc, true);
    let name = Span::Name(variant.name.clone(), variant.loc);
    let discriminant = variant
        .discriminant
        .as_ref()
        .map(|d| plain(&format!(" = {d}")));
    let trailing = variant.meta.trailing.clone();
    match &variant.fields {
        VariantFields::Unit => {
            let mut spans = vec![name];
            spans.extend(discriminant);
            spans.push(plain(","));
            lines.push(Line {
                indent: 4,
                spans,
                trailing,
            });
        }
        VariantFields::Tuple(fields) if fields.iter().any(|f| !f.meta.notes.is_empty()) => {
            lines.push(Line {
                indent: 4,
                spans: vec![name, plain("(")],
                trailing: None,
            });
            let mut fields = param_lines(fields, 8, false);
            let mut last = fields.pop().unwrap_or_default();
            last.spans.push(plain(")"));
            last.spans.extend(discriminant);
            last.spans.push(plain(","));
            last.trailing = trailing;
            lines.extend(fields);
            lines.push(last);
        }
        VariantFields::Tuple(fields) => {
            let mut spans = vec![name, plain("(")];
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    spans.push(plain(", "));
                }
                spans.extend(code(&field.text));
            }
            spans.push(plain(")"));
            spans.extend(discriminant);
            spans.push(plain(","));
            lines.push(Line {
                indent: 4,
                spans,
                trailing,
            });
        }
        VariantFields::Named(fields) => {
            let mut one = vec![name.clone(), plain(" { ")];
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    one.push(plain(", "));
                }
                one.extend(code(&field.text));
            }
            one.push(plain(" }"));
            one.extend(discriminant.clone());
            one.push(plain(","));
            let commented = fields.iter().any(|f| !f.meta.notes.is_empty());
            if !commented && width(4, &one, refs) <= MAX_WIDTH {
                lines.push(Line {
                    indent: 4,
                    spans: one,
                    trailing,
                });
            } else {
                lines.extend(expanded_variant(name, fields, discriminant, trailing));
            }
        }
    }
    Entry {
        name: variant.name.clone(),
        lines,
        repeats,
    }
}

/// Returns a struct-like variant with one field per line, each field's
/// comments above it.
fn expanded_variant(
    name: Span,
    fields: &[Param],
    discriminant: Option<Span>,
    trailing: Option<String>,
) -> Vec<Line> {
    let mut lines = vec![
        Line {
            indent: 4,
            spans: vec![name],
            trailing: None,
        },
        Line {
            indent: 4,
            spans: vec![plain("{")],
            trailing: None,
        },
    ];
    lines.extend(spaced(fields.iter().map(|field| {
        let mut field_lines = note_lines(&field.meta.notes, 8, Marker::DOC);
        let mut spans = code(&field.text);
        spans.push(plain(","));
        field_lines.push(Line {
            indent: 8,
            spans,
            trailing: None,
        });
        field_lines
    })));
    let mut close = vec![plain("}")];
    close.extend(discriminant);
    close.push(plain(","));
    lines.push(Line {
        indent: 4,
        spans: close,
        trailing,
    });
    lines
}

/// Returns one line per parameter, comments above each, every line but
/// the last ending in a comma; a tuple field starts with its visibility.
/// On a public surface, `surface`, a field that is not `pub` prints dim.
pub(crate) fn param_lines(fields: &[Param], indent: usize, surface: bool) -> Vec<Line> {
    spaced(fields.iter().enumerate().map(|(i, field)| {
        let mut lines = note_lines(&field.meta.notes, indent, Marker::DOC);
        let mut spans = field_spans(field, surface);
        if i + 1 < fields.len() {
            spans.push(plain(","));
        }
        lines.push(Line {
            indent,
            spans,
            trailing: None,
        });
        lines
    }))
}

/// Returns a tuple field: its visibility and its type. On a public
/// surface, `surface`, a field that is not `pub` is a dim run, since a
/// user of the surface cannot name it.
pub(crate) fn field_spans(field: &Param, surface: bool) -> Vec<Span> {
    let mut spans = vis_spans(&field.meta.vis);
    spans.extend(code(&field.text));
    if surface && field.meta.vis != "pub" {
        spans.insert(0, Span::DimOpen);
        spans.push(Span::DimClose);
    }
    spans
}

/// Returns the private fields of a type on a public surface as one entry:
/// each field's name and type on a line of its own, dim, with no comment,
/// `cfg` line, or attribute, and no blank line between two fields. They
/// are there to show what a value holds, not to be read as API.
pub(crate) fn private_fields(fields: &[&Member]) -> Entry {
    let lines = (fields.iter())
        .filter_map(|member| {
            let MemberKind::Field(ty) = &member.kind else {
                return None;
            };
            let mut spans = vec![
                Span::DimOpen,
                Span::Name(member.name.clone(), member.loc),
                plain(": "),
            ];
            spans.extend(code(ty));
            spans.extend([plain(","), Span::DimClose]);
            Some(Line {
                indent: 4,
                spans,
                trailing: None,
            })
        })
        .collect();
    Entry {
        name: String::new(),
        lines,
        repeats: Vec::new(),
    }
}
