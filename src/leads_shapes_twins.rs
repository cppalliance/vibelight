//! The detectors of twins: declarations of one shape, a shape whose every
//! field is in another, two structs that differ in one type, enums with the
//! same variant names, an uncommon item that several targets declare alike,
//! and one signature under several names.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, Field, FnKind, Func, escapes, trait_of};
use crate::leads_types::{Tok, Ty, item_name, leaf};
use crate::model::{Decl, DeclId, DeclKind, ModId, Shape, name_order};
use crate::typerefs::strip_raw;

/// The fewest fields a shape has for `contained-shape`.
const CONTAINED_FROM: usize = 3;

/// The fewest inherent methods each of two generic twins has, and the
/// fewest of them whose signatures match.
const TWIN_METHODS: usize = 3;

/// The fewest variants an enum has for `mirror-enum`.
const MIRROR_FROM: usize = 2;

/// The most items of the run with one name for `cross-crate-duplicate` to
/// call the name uncommon.
const UNCOMMON: usize = 3;

pub(crate) fn find(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    same_shape(run, catalog, out);
    contained_shape(run, catalog, out);
    generic_twin(run, catalog, out);
    mirror_enum(run, catalog, out);
    cross_crate_duplicate(run, catalog, out);
    same_signature(run, catalog, out);
}

/// Returns the shape that declarations of one kind share: the kind's label
/// and one entry per variant in order, or per field sorted by name, each
/// type as `render` writes it; `None` for fewer than two entries. Docs never
/// count.
fn shape(
    catalog: &Catalog<'_>,
    at: ItemId,
    decl: &Decl,
    render: fn(&Ty) -> String,
) -> Option<(&'static str, Vec<String>)> {
    let entry = |f: &Field<'_>| {
        if f.named {
            format!("{}: {}", f.name, render(&f.ty))
        } else {
            render(&f.ty)
        }
    };
    let (kind, entries) = match (decl.kind, &decl.shape) {
        (DeclKind::Enum, Shape::Enum(variants)) => {
            let entries = variants.iter().enumerate().map(|(v, variant)| {
                let fields: Vec<&Field<'_>> = catalog.fields_of(at, Some(v)).collect();
                let list: Vec<String> = fields.iter().copied().map(entry).collect();
                match fields.first() {
                    None => variant.name.clone(),
                    Some(f) if f.named => format!("{} {{ {} }}", variant.name, list.join(", ")),
                    Some(_) => format!("{}({})", variant.name, list.join(", ")),
                }
            });
            ("enums", entries.collect())
        }
        (DeclKind::Struct | DeclKind::Union, Shape::Named) => {
            let mut fields: Vec<String> = catalog.fields_of(at, None).map(entry).collect();
            fields.sort_by(|a, b| name_order(a, b));
            let kind = if decl.kind == DeclKind::Union {
                "unions"
            } else {
                "structs"
            };
            (kind, fields)
        }
        (DeclKind::Struct, Shape::Tuple(_)) => (
            "tuple structs",
            catalog.fields_of(at, None).map(entry).collect(),
        ),
        _ => return None,
    };
    (entries.len() > 1).then_some((kind, entries))
}

/// `same-shape`: declarations of one kind in one target whose variants, in
/// order, or whose fields have the same names and types.
fn same_shape(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut groups: BTreeMap<(usize, &str, Vec<String>), Vec<ItemId>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let at = decl_item(t, d);
            if let Some((kind, entries)) = shape(catalog, at, decl, Ty::key) {
                groups.entry((t, kind, entries)).or_default().push(at);
            }
        }
    }
    for ((_, kind, entries), decls) in groups {
        if decls.len() < 2 {
            continue;
        }
        let noun = if kind == "enums" {
            "variants"
        } else {
            "fields"
        };
        let evidence = format!("{kind} with the same {} {noun}", entries.len());
        let mut lead = Lead::new("same-shape", Mode::DirectSyntax, evidence);
        for at in decls {
            lead = lead.item(run, at);
        }
        out.push(lead);
    }
}

fn decl_item(t: usize, d: DeclId) -> ItemId {
    ItemId {
        target: t,
        item: ItemRef::Decl(d),
    }
}

/// Returns a shape's owner: its declaration, or the variant of it.
fn owner(decl: ItemId, variant: Option<usize>) -> ItemId {
    match (variant, decl.item) {
        (Some(v), ItemRef::Decl(d)) => ItemId {
            target: decl.target,
            item: ItemRef::Variant(d, v),
        },
        _ => decl,
    }
}

/// Returns an owner's name: a declaration's, or `Enum::Variant`.
fn owner_name(run: &AnalysisRun<'_>, item: ItemId) -> String {
    let path = run.path(item);
    let keep = if matches!(item.item, ItemRef::Variant(..)) {
        2
    } else {
        1
    };
    path[path.len().saturating_sub(keep)..].join("::")
}

/// A struct, union, or variant whose fields all have names, as
/// `contained-shape` compares it: each field's name and type.
struct Held {
    decl: ItemId,
    variant: Option<usize>,
    fields: BTreeSet<(String, String)>,
}

/// `contained-shape`: a struct, union, or named variant payload of at
/// least [`CONTAINED_FROM`] fields whose every field, by name and type, is
/// a field of another one in its target, one lead with every one that
/// holds it. Two structs with the same fields are `same-shape`'s.
fn contained_shape(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut shapes: Vec<Held> = Vec::new();
    let mut start = 0;
    while let Some(first) = catalog.fields.get(start) {
        let key = (first.decl, first.variant);
        let group: Vec<&Field<'_>> = (catalog.fields[start..].iter())
            .take_while(|f| (f.decl, f.variant) == key)
            .collect();
        start += group.len();
        if group.iter().all(|f| f.named) {
            shapes.push(Held {
                decl: key.0,
                variant: key.1,
                fields: group.iter().map(|f| (f.name.clone(), f.ty.key())).collect(),
            });
        }
    }
    let mut index: BTreeMap<&(String, String), Vec<usize>> = BTreeMap::new();
    for (s, shape) in shapes.iter().enumerate() {
        for field in &shape.fields {
            index.entry(field).or_default().push(s);
        }
    }
    let kind = |decl: ItemId| match decl.item {
        ItemRef::Decl(d) => Some(run.krates[decl.target].decls[d].kind),
        _ => None,
    };
    for (s, shape) in shapes.iter().enumerate() {
        let fields = &shape.fields;
        let Some(first) = fields
            .iter()
            .next()
            .filter(|_| fields.len() >= CONTAINED_FROM)
        else {
            continue;
        };
        let mut bigs = Vec::new();
        for &o in &index[first] {
            let other = &shapes[o];
            if o == s || other.decl.target != shape.decl.target || !fields.is_subset(&other.fields)
            {
                continue;
            }
            let equal = fields.len() == other.fields.len();
            let structs = shape.variant.is_none() && other.variant.is_none();
            if equal && (o < s || (structs && kind(shape.decl) == kind(other.decl))) {
                continue;
            }
            bigs.push(owner(other.decl, other.variant));
        }
        if bigs.is_empty() {
            continue;
        }
        let small = owner(shape.decl, shape.variant);
        let names: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
        let holders: Vec<String> = (bigs.iter())
            .map(|&big| format!("`{}`", owner_name(run, big)))
            .collect();
        let verb = if holders.len() == 1 { "holds" } else { "hold" };
        let (held, names) = (owner_name(run, small), names.join(", "));
        let evidence = format!(
            "{} {verb} every field of `{held}`: {names}",
            holders.join(", ")
        );
        let lead = Lead::new("contained-shape", Mode::DirectSyntax, evidence).item(run, small);
        out.push(bigs.into_iter().fold(lead, |lead, big| lead.item(run, big)));
    }
}

/// One struct as `generic-twin` compares it: its fields' types, its
/// inherent methods' signatures, and the traits it implements, with each
/// name that starts with its own, such as its iterator's, as `Self` and the
/// rest of the name.
struct Twin {
    at: ItemId,
    fields: Vec<Ty>,
    methods: BTreeMap<String, Ty>,
    traits: BTreeSet<String>,
}

/// Returns a type with each name of the run that starts with `own` and a
/// capital after it written as `Self` and the rest of the name.
fn selfish(ty: &Ty, own: &str) -> Ty {
    let tok = |tok: &Tok| match tok {
        Tok::Name(name) if name.starts_with('#') => match leaf(name).strip_prefix(own) {
            Some(rest) if rest.starts_with(char::is_uppercase) => Tok::Name(format!("Self{rest}")),
            _ => tok.clone(),
        },
        _ => tok.clone(),
    };
    Ty(ty.0.iter().map(tok).collect())
}

/// Returns the structs of target `t` with at least [`TWIN_METHODS`]
/// inherent methods.
fn twins(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, t: usize) -> Vec<Twin> {
    let mut methods: BTreeMap<DeclId, Vec<&Func<'_>>> = BTreeMap::new();
    for func in &catalog.funcs {
        if let (FnKind::Inherent, Some(d), true) = (func.kind, func.decl, func.item.target == t) {
            methods.entry(d).or_default().push(func);
        }
    }
    let krate = &run.krates[t];
    let mut out = Vec::new();
    for (d, funcs) in methods {
        let decl = &krate.decls[d];
        if decl.kind != DeclKind::Struct || funcs.len() < TWIN_METHODS {
            continue;
        }
        let at = decl_item(t, d);
        let name = strip_raw(&decl.name);
        let fields: Vec<Ty> = catalog
            .fields_of(at, None)
            .map(|f| selfish(&f.ty, &name))
            .collect();
        if fields.is_empty() {
            continue;
        }
        let methods = (funcs.iter())
            .map(|f| (f.name.clone(), selfish(&f.signature(run), &name)))
            .collect();
        let traits = run.impls_of[t][d]
            .iter()
            .filter_map(|&i| trait_of(run, t, i));
        let traits = traits.map(|name| leaf(&name)).collect();
        out.push(Twin {
            at,
            fields,
            methods,
            traits,
        });
    }
    out
}

/// `generic-twin`: two structs of one target that are the same after one
/// name in a generic field type of the first is replaced by another name:
/// their fields, but for at most one, and at least [`TWIN_METHODS`]
/// methods of one name and signature, one of which writes the replaced
/// name or a name that starts with its struct's.
fn generic_twin(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for t in 0..run.krates.len() {
        let twins = twins(run, catalog, t);
        let mut holes: BTreeMap<Vec<Tok>, Vec<(usize, &str)>> = BTreeMap::new();
        for (w, twin) in twins.iter().enumerate() {
            for ty in twin.fields.iter().filter(|ty| ty.generic().is_some()) {
                for (p, tok) in ty.0.iter().enumerate().skip(1) {
                    let Tok::Name(name) = tok else {
                        continue;
                    };
                    if Ty::name(name).primitive().is_none() {
                        let mut skeleton = ty.0.clone();
                        skeleton[p] = Tok::Code("_".to_owned());
                        holes.entry(skeleton).or_default().push((w, name.as_str()));
                    }
                }
            }
        }
        let mut seen = BTreeSet::new();
        for (skeleton, list) in &holes {
            for (i, &(a, x)) in list.iter().enumerate() {
                for &(b, y) in &list[i + 1..] {
                    if a == b || x == y || seen.contains(&(a, b)) {
                        continue;
                    }
                    if let Some(evidence) = twin_evidence(&twins[a], &twins[b], (x, y), skeleton) {
                        seen.insert((a, b));
                        let lead = Lead::new("generic-twin", Mode::DirectSyntax, evidence);
                        out.push(lead.item(run, twins[a].at).item(run, twins[b].at));
                    }
                }
            }
        }
    }
}

/// Returns what makes `a` and `b` generic twins once name `from` in `a` is
/// `to`, or `None` when they are not.
fn twin_evidence(a: &Twin, b: &Twin, (from, to): (&str, &str), skeleton: &[Tok]) -> Option<String> {
    let swap = |ty: &Ty| {
        let tok = |tok: &Tok| match tok {
            Tok::Name(name) if name == from => Tok::Name(to.to_owned()),
            _ => tok.clone(),
        };
        Ty(ty.0.iter().map(tok).collect())
    };
    let mut unmatched = b.fields.clone();
    let mut extra = 0;
    for field in a.fields.iter().map(swap) {
        match unmatched.iter().position(|other| *other == field) {
            Some(found) => {
                unmatched.remove(found);
            }
            None => extra += 1,
        }
    }
    if extra + unmatched.len() > 1 {
        return None;
    }
    let matched: Vec<&str> = (a.methods.iter())
        .filter(|(name, sig)| b.methods.get(*name) == Some(&swap(sig)))
        .map(|(name, _)| name.as_str())
        .collect();
    let holed = matched.iter().any(|name| {
        (b.methods[*name].names()).any(|n| n == to || (n.starts_with("Self") && n != "Self"))
    });
    if matched.len() < TWIN_METHODS || !holed {
        return None;
    }
    let shared = Ty(skeleton.to_vec()).shown();
    let mut evidence = format!("`{shared}`; methods {}", matched.join(", "));
    let traits: Vec<&str> = a
        .traits
        .intersection(&b.traits)
        .map(String::as_str)
        .collect();
    if !traits.is_empty() {
        evidence = format!("{evidence}; impls {}", traits.join(", "));
    }
    Some(evidence)
}

/// `mirror-enum`: enums anywhere in the run with the same variant names,
/// unless they are one target's enums of one shape, which `same-shape`
/// names.
fn mirror_enum(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut groups: BTreeMap<Vec<String>, Vec<ItemId>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let Shape::Enum(variants) = &decl.shape else {
                continue;
            };
            let mut names: Vec<String> = variants.iter().map(|v| strip_raw(&v.name)).collect();
            names.sort();
            if names.len() >= MIRROR_FROM {
                groups.entry(names).or_default().push(decl_item(t, d));
            }
        }
    }
    for (names, enums) in groups {
        if enums.len() < 2 {
            continue;
        }
        let shapes: BTreeSet<Option<Vec<String>>> = (enums.iter())
            .map(|&at| {
                let ItemRef::Decl(d) = at.item else {
                    return None;
                };
                let decl = &run.krates[at.target].decls[d];
                shape(catalog, at, decl, Ty::key).map(|(_, entries)| entries)
            })
            .collect();
        let one_target = enums.iter().all(|e| e.target == enums[0].target);
        if one_target && shapes.len() == 1 && !shapes.contains(&None) {
            continue;
        }
        let evidence = format!("variants {}", names.join(", "));
        let mut lead = Lead::new("mirror-enum", Mode::DirectSyntax, evidence);
        for at in enums {
            lead = lead.item(run, at);
        }
        out.push(lead);
    }
}

/// `cross-crate-duplicate`: a name that at most [`UNCOMMON`] items of the
/// run have, declared in two or more targets with the same fields or
/// variants, or as free functions with the same parameter and return
/// types, each type compared by its last name.
fn cross_crate_duplicate(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut groups: BTreeMap<(String, &str, String), Vec<ItemId>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let at = decl_item(t, d);
            if let Some((kind, entries)) = shape(catalog, at, decl, Ty::shown) {
                let key = (strip_raw(&decl.name), kind, entries.join("; "));
                groups.entry(key).or_default().push(at);
            }
        }
    }
    for func in &catalog.funcs {
        if func.kind == FnKind::Free && !func.args.is_empty() {
            let key = (func.name.clone(), "functions", func.shown());
            groups.entry(key).or_default().push(func.item);
        }
    }
    for ((name, kind, _), items) in groups {
        let targets: BTreeSet<usize> = items.iter().map(|item| item.target).collect();
        if targets.len() < 2 || run.census.declared(&name) > UNCOMMON {
            continue;
        }
        let evidence = format!("`{name}` in {} targets, {kind} of one shape", targets.len());
        let mut lead = Lead::new("cross-crate-duplicate", Mode::DirectSyntax, evidence);
        for item in items {
            lead = lead.item(run, item);
        }
        out.push(lead);
    }
}

/// `same-signature`: functions of a target, under two or more names in
/// two or more modules, with one signature of at least two parameters
/// besides the receiver that names something other than a primitive.
/// `Self` stands for the function's own type, and a function that a body
/// passes as a value, such as a handler, has the signature its callback
/// sets and is left out.
fn same_signature(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut groups: BTreeMap<(usize, Ty), Vec<&Func<'_>>> = BTreeMap::new();
    let funcs = (catalog.funcs.iter()).filter(|f| f.is_own() && f.args.len() >= 2);
    for func in funcs.filter(|f| !escapes(run, f.item)) {
        let mut sig = func.signature(run);
        if let Some(d) = func.decl {
            let own = Tok::Name(item_name(run, decl_item(func.item.target, d)));
            for tok in &mut sig.0 {
                if matches!(tok, Tok::Name(name) if name == "Self") {
                    *tok = own.clone();
                }
            }
        }
        if sig.names().any(|name| Ty::name(name).primitive().is_none()) {
            groups
                .entry((func.item.target, sig))
                .or_default()
                .push(func);
        }
    }
    for funcs in groups.into_values() {
        let names: BTreeSet<&str> = funcs.iter().map(|f| f.name.as_str()).collect();
        let modules: BTreeSet<ModId> = funcs.iter().map(|f| f.module).collect();
        if names.len() < 2 || modules.len() < 2 {
            continue;
        }
        let shown: Vec<&str> = names.into_iter().collect();
        let evidence = format!("`{}` under {}", funcs[0].shown(), shown.join(", "));
        let mut lead = Lead::new("same-signature", Mode::DirectSyntax, evidence);
        for func in funcs {
            lead = lead.item(run, func.item);
        }
        out.push(lead);
    }
}
