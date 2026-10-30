//! The detectors of signatures, shapes, and impls: what types' fields,
//! variants, impls, and functions show about their state, their twins, and
//! their parameters. This file holds the group's registry and the
//! detectors that read one type's fields or variants; its siblings hold the
//! rest.

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, Field, FnKind, impls_of_trait, is_wire};
use crate::leads_types::Ty;
use crate::leads_words::words;
use crate::model::{Shape, VariantFields, Vis};
use crate::model_body::Root;
use crate::{leads_shapes_impls, leads_shapes_params, leads_shapes_traits, leads_shapes_twins};

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "option-as-flag",
    "contained-shape",
    "derived-field",
    "drop-option",
    "fallible-accessor",
    "direct-field-clusters",
    "option-shaped-enum",
    "same-shape",
    "generic-twin",
    "mirror-enum",
    "field-and-accessor",
    "redundant-constructor",
    "inverse-pair",
    "spelling-table",
    "same-signature",
    "cross-crate-duplicate",
    "direct-impl-field-split",
    "param-clump",
    "tuple-return",
    "out-param-cursor",
    "same-type-params",
    "underscore-param",
    "error-family",
    "uniform-payload",
    "single-variant-enum",
    "one-observed-trait-impl",
    "thin-wrapper-one-observed-user",
    "manual-serde",
];

/// The ids whose leads rest on what the run does not observe.
pub(crate) const ABSENT: &[&str] = &["one-observed-trait-impl", "thin-wrapper-one-observed-user"];

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let catalog = Catalog::new(run);
    option_as_flag(run, &catalog, out);
    derived_field(run, &catalog, out);
    drop_option(run, &catalog, out);
    fallible_accessor(run, &catalog, out);
    field_and_accessor(run, &catalog, out);
    enum_shapes(run, &catalog, out);
    leads_shapes_twins::find(run, &catalog, out);
    leads_shapes_impls::find(run, &catalog, out);
    leads_shapes_traits::find(run, &catalog, out);
    leads_shapes_params::find(run, &catalog, out);
}

/// Tells whether a type is `Option<()>` or `Option<bool>`.
fn is_flag(ty: &Ty) -> bool {
    ty.option()
        .is_some_and(|inner| inner.is_unit() || inner.is_bool())
}

/// `option-as-flag`: an `Option<()>` or `Option<bool>` field, parameter,
/// or return type. A wire type's field is left out, since an optional
/// boolean on the wire is the common way to say a flag was not sent.
fn option_as_flag(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    const ID: &str = "option-as-flag";
    for field in catalog
        .fields
        .iter()
        .filter(|f| is_flag(&f.ty) && !is_wire(run, f.decl))
    {
        let evidence = format!("`{}: {}`", field.name, field.ty.shown());
        out.push(field.add(run, Lead::new(ID, Mode::DirectSyntax, evidence)));
    }
    for func in catalog.funcs.iter().filter(|f| f.is_own()) {
        let mut seen: Vec<String> = (func.args.iter())
            .filter(|arg| is_flag(&arg.ty))
            .map(|arg| {
                format!(
                    "`{}: {}`",
                    arg.name.as_deref().unwrap_or("_"),
                    arg.ty.shown()
                )
            })
            .collect();
        if let Some(ret) = func.ret.as_ref().filter(|ret| is_flag(ret)) {
            seen.push(format!("returns `{}`", ret.shown()));
        }
        if !seen.is_empty() {
            let lead = Lead::new(ID, Mode::DirectSyntax, seen.join(", "));
            out.push(lead.item(run, func.item));
        }
    }
}

/// Returns a lowercase name and its singular.
fn stems(name: &str) -> [String; 2] {
    let lower = name.to_lowercase();
    let single = lower.strip_suffix('s').unwrap_or(&lower).to_owned();
    [lower, single]
}

/// `derived-field`: a span or range field beside a line, column, or offset
/// field, or a collection field beside a count, length, or `is_` field of
/// the same stem, in one struct, union, or variant.
fn derived_field(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut start = 0;
    while let Some(first) = catalog.fields.get(start) {
        let owner = (first.decl, first.variant);
        let len = (catalog.fields[start..].iter())
            .take_while(|f| (f.decl, f.variant) == owner)
            .count();
        let fields: Vec<&Field<'_>> = (catalog.fields[start..start + len].iter())
            .filter(|f| f.named)
            .collect();
        start += len;
        let has =
            |f: &Field<'_>, set: &[&str]| words(&f.name).iter().any(|w| set.contains(&w.as_str()));
        let mut found: Vec<(Vec<&Field<'_>>, String)> = Vec::new();
        let spans: Vec<&Field<'_>> = fields
            .iter()
            .copied()
            .filter(|f| has(f, &["span", "range"]))
            .collect();
        let places: Vec<&Field<'_>> = (fields.iter().copied())
            .filter(|f| has(f, &["line", "column", "offset"]))
            .collect();
        if !spans.is_empty() && !places.is_empty() {
            let names = |list: &[&Field<'_>]| {
                list.iter()
                    .map(|f| format!("`{}`", f.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let evidence = format!("{} beside {}", names(&spans), names(&places));
            found.push((spans.into_iter().chain(places).collect(), evidence));
        }
        for &held in fields.iter().filter(|f| f.ty.deref().is_collection()) {
            let held_stems = stems(&held.name);
            let shares = |other: &str| stems(other).iter().any(|s| held_stems.contains(s));
            for &other in fields.iter().filter(|f| f.name != held.name) {
                let name = other.name.as_str();
                let counted = (name
                    .strip_suffix("_len")
                    .or_else(|| name.strip_suffix("_count")))
                .is_some_and(shares);
                let flagged = name
                    .strip_prefix("is_")
                    .is_some_and(|rest| words(rest).iter().any(|w| shares(w)));
                if counted || flagged {
                    let evidence = format!("`{}` beside `{}`", held.name, other.name);
                    found.push((vec![held, other], evidence));
                }
            }
        }
        for (fields, evidence) in found {
            let mut lead = Lead::new("derived-field", Mode::DirectSyntax, evidence);
            for field in fields {
                lead = field.add(run, lead);
            }
            out.push(lead);
        }
    }
}

/// `drop-option`: the `Option` fields of a type with a `Drop` impl.
fn drop_option(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        for d in 0..krate.decls.len() {
            let Some(drop) = impls_of_trait(run, t, d, "std::ops::Drop").next() else {
                continue;
            };
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            let options: Vec<&Field<'_>> = (catalog.fields_of(at, None))
                .filter(|f| f.ty.option().is_some())
                .collect();
            if options.is_empty() {
                continue;
            }
            let shown: Vec<String> = (options.iter())
                .map(|f| format!("`{}: {}`", f.name, f.ty.shown()))
                .collect();
            let evidence = format!("a Drop impl beside {}", shown.join(", "));
            let mut lead = Lead::new("drop-option", Mode::DirectSyntax, evidence);
            lead = lead.at(t, krate.impls[drop].loc);
            for field in options {
                lead = field.add(run, lead);
            }
            out.push(lead);
        }
    }
}

/// Tells whether a member of declaration `decl` calls `take` on its field
/// `field` through `self`.
fn taken(run: &AnalysisRun<'_>, decl: ItemId, field: &str) -> bool {
    let ItemRef::Decl(d) = decl.item else {
        return false;
    };
    let t = decl.target;
    run.members(t, d).any(|(item, _)| {
        let body = run.body_of(ItemId { target: t, item });
        body.is_some_and(|b| {
            run.krates[t].bodies[b].facts.chains.iter().any(|chain| {
                matches!(&chain.root, Root::Field(name) if name == field)
                    && chain.methods.first().is_some_and(|m| m == "take")
            })
        })
    })
}

/// `fallible-accessor`: a method of no parameter besides `self` that
/// returns an `Option` or a `Result` and reaches only one field through
/// `self`, an `Option`. An `Option` accessor counts only when its type has
/// a `Drop` impl or a member takes the field, since an optional field with
/// an optional getter is common.
fn fallible_accessor(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for func in catalog.funcs.iter().filter(|f| f.args.is_empty()) {
        let (FnKind::Inherent, Some(d), [field], Some(ret)) =
            (func.kind, func.decl, func.self_fields, &func.ret)
        else {
            continue;
        };
        let result = ret.is_result();
        if !result && ret.option().is_none() {
            continue;
        }
        let t = func.item.target;
        let at = ItemId {
            target: t,
            item: ItemRef::Decl(d),
        };
        let held = catalog
            .fields_of(at, None)
            .find(|f| f.named && f.name == *field);
        let Some(held) = held.filter(|f| f.ty.option().is_some()) else {
            continue;
        };
        let why = if result {
            "it returns a Result"
        } else if impls_of_trait(run, t, d, "std::ops::Drop").next().is_some() {
            "its type has a Drop impl"
        } else if taken(run, at, field) {
            "a method takes the field"
        } else {
            continue;
        };
        let evidence = format!(
            "`{}` reaches only `{}: {}`, and {why}",
            func.name,
            held.name,
            held.ty.shown()
        );
        let lead = Lead::new("fallible-accessor", Mode::DirectSyntax, evidence);
        out.push(held.add(run, lead.item(run, func.item)));
    }
}

/// `field-and-accessor`: a field that is not private beside a method that
/// reaches only that field and is named after it, or `get_` and its name,
/// when the field is at least as visible as the method. A narrower field
/// behind a wider getter is the common read-only view.
fn field_and_accessor(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for func in &catalog.funcs {
        let (FnKind::Inherent, Some(d), [read]) = (func.kind, func.decl, func.self_fields) else {
            continue;
        };
        let named = func.name == *read || func.name.strip_prefix("get_") == Some(read.as_str());
        if !named || !func.args.is_empty() || !func.has_receiver() || func.ret.is_none() {
            continue;
        }
        let at = ItemId {
            target: func.item.target,
            item: ItemRef::Decl(d),
        };
        let field = (catalog.fields_of(at, None))
            .find(|f| f.named && f.name == *read && f.vis != Vis::Private && f.vis <= func.vis);
        if let Some(field) = field {
            let evidence = format!("`{} {}` and `fn {}`", field.meta.vis, field.name, func.name);
            let lead = Lead::new("field-and-accessor", Mode::DirectSyntax, evidence);
            out.push(field.add(run, lead.item(run, func.item)));
        }
    }
}

/// `single-variant-enum`, `option-shaped-enum`, and `uniform-payload`: an
/// enum of one variant; of one unit variant and one variant of one value;
/// and of two or more variants that all carry the same payload.
fn enum_shapes(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let Shape::Enum(variants) = &decl.shape else {
                continue;
            };
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            let payload = |v: usize| -> Vec<(String, String)> {
                (catalog.fields_of(at, Some(v)))
                    .map(|f| {
                        (
                            if f.named {
                                f.name.clone()
                            } else {
                                String::new()
                            },
                            f.ty.key(),
                        )
                    })
                    .collect()
            };
            let shown = |v: usize| -> String {
                let fields = catalog.fields_of(at, Some(v)).map(|f| f.ty.shown());
                fields.collect::<Vec<_>>().join(", ")
            };
            let lead = |id, evidence| Lead::new(id, Mode::DirectSyntax, evidence).item(run, at);
            if let [only] = variants.as_slice() {
                out.push(lead(
                    "single-variant-enum",
                    format!("one variant, `{}`", only.name),
                ));
                continue;
            }
            let one = |fields: &VariantFields| match fields {
                VariantFields::Tuple(params) | VariantFields::Named(params) => params.len() == 1,
                VariantFields::Unit => false,
            };
            if let [a, b] = variants.as_slice() {
                let pair = match (&a.fields, &b.fields) {
                    (VariantFields::Unit, fields) if one(fields) => Some((a, 1)),
                    (fields, VariantFields::Unit) if one(fields) => Some((b, 0)),
                    _ => None,
                };
                if let Some((unit, value)) = pair {
                    let evidence = format!(
                        "`{}` beside `{}({})`",
                        unit.name,
                        variants[value].name,
                        shown(value)
                    );
                    out.push(lead("option-shaped-enum", evidence));
                }
            }
            let first = payload(0);
            let carried = !first.is_empty() && (1..variants.len()).all(|v| payload(v) == first);
            if carried {
                let evidence = format!("every variant carries `({})`", shown(0));
                out.push(lead("uniform-payload", evidence));
            }
        }
    }
}
