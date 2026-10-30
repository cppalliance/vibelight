//! The detectors of a type's impls and conversions: methods that fall into
//! groups by the fields they reach, an impl file whose methods reach only
//! fields another file's reach, constructors that take the same types,
//! conversions both ways, and a unit enum spelled in more than one place.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, FnKind, Func, impls_of_trait, is_seam, trait_of};
use crate::leads_types::{Tok, Ty, item_name, leaf};
use crate::model::{FileId, MemberKind, ModId, Shape, VariantFields};
use crate::model_body::Single;

/// The fewest methods that reach fields for `direct-field-clusters`.
const CLUSTERS_FROM: usize = 4;

/// The fewest methods in each group of `direct-field-clusters`.
const CLUSTER_METHODS: usize = 2;

/// The fewest named fields a type has for `direct-impl-field-split`.
const SPLIT_FIELDS: usize = 3;

/// The inherent methods of each declaration.
type Methods<'c, 'a> = BTreeMap<ItemId, Vec<&'c Func<'a>>>;

pub(crate) fn find(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut methods: Methods<'_, '_> = BTreeMap::new();
    for func in &catalog.funcs {
        if let (FnKind::Inherent, Some(d)) = (func.kind, func.decl) {
            let at = ItemId {
                target: func.item.target,
                item: ItemRef::Decl(d),
            };
            methods.entry(at).or_default().push(func);
        }
    }
    direct_field_clusters(run, catalog, &methods, out);
    direct_impl_field_split(run, catalog, &methods, out);
    redundant_constructor(run, &methods, out);
    inverse_pair(run, catalog, out);
    spelling_table(run, catalog, out);
}

/// Joins names with commas.
fn listed<'s>(names: impl IntoIterator<Item = &'s str>) -> String {
    names.into_iter().collect::<Vec<_>>().join(", ")
}

/// Returns the named fields of a declaration.
fn field_names<'c>(catalog: &'c Catalog<'_>, at: ItemId) -> BTreeSet<&'c str> {
    (catalog.fields_of(at, None))
        .filter(|f| f.named)
        .map(|f| f.name.as_str())
        .collect()
}

/// `direct-field-clusters`: the inherent methods of a type that reach its
/// fields through `self`, at least [`CLUSTERS_FROM`] of them, fall into two
/// or more groups of at least [`CLUSTER_METHODS`] methods once each hub, a
/// field more than half of them reach, is left out. Aliases are not
/// followed.
fn direct_field_clusters(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    methods: &Methods<'_, '_>,
    out: &mut Vec<Lead>,
) {
    for (&at, funcs) in methods {
        let fields = field_names(catalog, at);
        let reaches = |func: &Func<'_>, field: &str| func.self_fields.iter().any(|s| s == field);
        let reaching: Vec<&Func<'_>> = (funcs.iter().copied())
            .filter(|f| fields.iter().any(|field| reaches(f, field)))
            .collect();
        if reaching.len() < CLUSTERS_FROM || fields.len() < 2 {
            continue;
        }
        let hubs: BTreeSet<&str> = (fields.iter().copied())
            .filter(|field| {
                reaching.iter().filter(|f| reaches(f, field)).count() * 2 > reaching.len()
            })
            .collect();
        let mut groups: Vec<(BTreeSet<&str>, Vec<&str>)> = Vec::new();
        for func in &reaching {
            let own: BTreeSet<&str> = (func.self_fields.iter().map(String::as_str))
                .filter(|f| fields.contains(f) && !hubs.contains(f))
                .collect();
            if own.is_empty() {
                continue;
            }
            let mut group = (own, vec![func.name.as_str()]);
            groups.retain(|other| {
                if other.0.is_disjoint(&group.0) {
                    return true;
                }
                group.0.extend(other.0.iter().copied());
                group.1.extend(other.1.iter().copied());
                false
            });
            groups.push(group);
        }
        groups.retain(|group| group.1.len() >= CLUSTER_METHODS);
        if groups.len() < 2 {
            continue;
        }
        for group in &mut groups {
            group.1.sort_unstable();
        }
        groups.sort();
        let shown: Vec<String> = (groups.iter())
            .map(|(fields, by)| {
                format!(
                    "{} by {}",
                    listed(fields.iter().copied()),
                    listed(by.iter().copied())
                )
            })
            .collect();
        let mut evidence = shown.join("; ");
        if !hubs.is_empty() {
            evidence = format!("{evidence}; hubs {}", listed(hubs.iter().copied()));
        }
        let lead = Lead::new("direct-field-clusters", Mode::DirectSyntax, evidence);
        out.push(lead.item(run, at));
    }
}

/// `direct-impl-field-split`: the inherent methods of a type of at least
/// [`SPLIT_FIELDS`] named fields that sit in one file reach, through
/// `self`, only some fields, all of which the methods in another file
/// reach.
fn direct_impl_field_split(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    methods: &Methods<'_, '_>,
    out: &mut Vec<Lead>,
) {
    for (&at, funcs) in methods {
        let fields = field_names(catalog, at);
        if fields.len() < SPLIT_FIELDS {
            continue;
        }
        let mut files: BTreeMap<FileId, (BTreeSet<&str>, Vec<ItemId>)> = BTreeMap::new();
        for func in funcs {
            let reached: Vec<&str> = (func.self_fields.iter().map(String::as_str))
                .filter(|f| fields.contains(f))
                .collect();
            if let (Some(loc), false) = (run.loc(func.item), reached.is_empty()) {
                let file = files.entry(loc.file).or_default();
                file.0.extend(reached);
                file.1.push(func.item);
            }
        }
        let rel = |file: FileId| run.krates[at.target].files[file].rel.as_str();
        for (&file, (reached, items)) in &files {
            if reached.len() == fields.len() {
                continue;
            }
            let other =
                (files.iter()).find(|(o, (wider, _))| **o != file && reached.is_subset(wider));
            let Some((&other, _)) = other else {
                continue;
            };
            let names: Vec<&str> = reached.iter().copied().collect();
            let evidence = format!(
                "the methods in {} reach only fields the methods in {} reach: {}",
                rel(file),
                rel(other),
                names.join(", ")
            );
            let mut lead = Lead::new("direct-impl-field-split", Mode::DirectSyntax, evidence);
            lead = lead.item(run, at);
            for &item in items {
                lead = lead.item(run, item);
            }
            out.push(lead);
        }
    }
}

/// Tells whether a return type is `Self`, or a `Result` or an `Option` of
/// it.
fn constructs(ret: &Ty) -> bool {
    let is_self = |ty: &Ty| ty.key() == "Self";
    is_self(ret)
        || ret.generic().is_some_and(|(head, args)| {
            matches!(leaf(head).as_str(), "Result" | "Option") && args.first().is_some_and(is_self)
        })
}

/// Tells whether an item's body is one call to a path whose last segment
/// is `name`.
fn forwards(run: &AnalysisRun<'_>, item: ItemId, name: &str) -> bool {
    run.body_of(item).is_some_and(|b| {
        matches!(&run.krates[item.target].bodies[b].facts.single,
            Some(Single::Call(path)) if path.last().is_some_and(|last| last == name))
    })
}

/// `redundant-constructor`: two or more inherent functions of a struct or
/// union that return `Self` and take the same parameter types, or `new()`
/// beside a derived or written `Default` when neither calls the other. An
/// enum's constructors that take one type build different variants, and
/// are left out.
fn redundant_constructor(run: &AnalysisRun<'_>, methods: &Methods<'_, '_>, out: &mut Vec<Lead>) {
    const ID: &str = "redundant-constructor";
    for (&at, funcs) in methods {
        let ctors: Vec<&Func<'_>> = (funcs.iter().copied())
            .filter(|f| !f.has_receiver() && f.ret.as_ref().is_some_and(constructs))
            .collect();
        let enumerated = match at.item {
            ItemRef::Decl(d) => matches!(run.krates[at.target].decls[d].shape, Shape::Enum(_)),
            _ => true,
        };
        let mut by_args: BTreeMap<Vec<String>, Vec<&Func<'_>>> = BTreeMap::new();
        for &ctor in ctors.iter().filter(|_| !enumerated) {
            let types = ctor.args.iter().map(|arg| arg.ty.key()).collect();
            by_args.entry(types).or_default().push(ctor);
        }
        for same in by_args.values().filter(|same| same.len() > 1) {
            let names: Vec<&str> = same.iter().map(|f| f.name.as_str()).collect();
            let types: Vec<String> = same[0].args.iter().map(|arg| arg.ty.shown()).collect();
            let evidence = format!("{} each take `({})`", names.join(", "), types.join(", "));
            let mut lead = Lead::new(ID, Mode::DirectSyntax, evidence);
            for func in same {
                lead = lead.item(run, func.item);
            }
            out.push(lead);
        }
        let new = ctors.iter().find(|f| f.name == "new" && f.args.is_empty());
        let (Some(new), ItemRef::Decl(d)) = (new, at.item) else {
            continue;
        };
        let (t, krate) = (at.target, &run.krates[at.target]);
        let derived = (krate.decls[d].written_derives.iter()).any(|name| leaf(name) == "Default");
        let written = impls_of_trait(run, t, d, "std::default::Default").next();
        let default = written.and_then(|i| {
            let members = krate.impls[i].trait_members.iter().enumerate();
            let mut fns =
                members.filter(|(_, m)| m.name == "default" && matches!(m.kind, MemberKind::Fn(_)));
            fns.next().map(|(k, _)| ItemId {
                target: t,
                item: ItemRef::TraitImpl(i, k),
            })
        });
        let calls = forwards(run, new.item, "default")
            || default.is_some_and(|item| forwards(run, item, "new"));
        if (derived || written.is_some()) && !calls {
            let how = if derived { "a derived" } else { "a written" };
            let evidence = format!("`new()` beside {how} Default, and neither calls the other");
            out.push(
                Lead::new(ID, Mode::DirectSyntax, evidence)
                    .item(run, new.item)
                    .item(run, at),
            );
        }
    }
}

/// Returns the one name a type is, as compared, with `Self` as `own`;
/// `None` for anything else or a generic parameter.
fn named(ty: &Ty, own: Option<&String>) -> Option<String> {
    match ty.deref().0.as_slice() {
        [Tok::Name(name)] if name == "Self" => own.cloned(),
        [Tok::Name(name)] if !name.starts_with('$') => Some(name.clone()),
        _ => None,
    }
}

/// `inverse-pair`: `From` impls both ways between two types of the run,
/// and functions of one module, free or associated without `self`, that
/// turn one type into another and back. A constructor and a getter of one
/// type are the common newtype pair and are left out, and so is a pair of
/// access seams, which open a narrower pair to wider callers.
fn inverse_pair(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    const ID: &str = "inverse-pair";
    let mut from: BTreeMap<(String, String), (ItemId, usize)> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (i, imp) in krate.impls.iter().enumerate() {
            let arg = imp.trait_.as_ref().and_then(|base| base.arg.as_ref());
            let (Some(d), Some(arg)) = (run.attached[t][i], arg) else {
                continue;
            };
            if trait_of(run, t, i).as_deref() != Some("std::convert::From") {
                continue;
            }
            let mut source = Ty::of(run, t, arg, None);
            if source.0.last() == Some(&Tok::Code("&".to_owned())) {
                source.0.pop();
            }
            let to = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            if let Some(source) = named(&source, None) {
                from.insert((source, item_name(run, to)), (to, i));
            }
        }
    }
    for ((a, b), &(to, i)) in &from {
        let Some(&(back, j)) = from.get(&(b.clone(), a.clone())).filter(|_| a < b) else {
            continue;
        };
        let evidence = format!(
            "`From<{}> for {}` and `From<{}> for {}`",
            leaf(a),
            leaf(b),
            leaf(b),
            leaf(a)
        );
        let lead = Lead::new(ID, Mode::DirectSyntax, evidence)
            .item(run, to)
            .item(run, back);
        let (here, there) = (
            &run.krates[to.target].impls[i],
            &run.krates[back.target].impls[j],
        );
        out.push(lead.at(to.target, here.loc).at(back.target, there.loc));
    }
    let mut ways: BTreeMap<(usize, ModId, String, String), Vec<&Func<'_>>> = BTreeMap::new();
    for func in catalog
        .funcs
        .iter()
        .filter(|f| f.is_own() && !f.has_receiver())
    {
        let t = func.item.target;
        let own = (func.decl).map(|d| {
            item_name(
                run,
                ItemId {
                    target: t,
                    item: ItemRef::Decl(d),
                },
            )
        });
        let input = match func.args.as_slice() {
            [arg] => named(&arg.ty, own.as_ref()),
            _ => None,
        };
        let output = func.ret.as_ref().and_then(|ret| named(ret, own.as_ref()));
        let (Some(x), Some(y)) = (input, output) else {
            continue;
        };
        if x != y && (x.starts_with('#') || y.starts_with('#')) {
            ways.entry((t, func.module, x, y)).or_default().push(func);
        }
    }
    for ((t, m, x, y), there) in &ways {
        let back = ways.get(&(*t, *m, y.clone(), x.clone()));
        let Some(back) = back.filter(|_| x < y) else {
            continue;
        };
        if (there.iter().chain(back)).all(|func| is_seam(run, catalog, func)) {
            continue;
        }
        let names = |funcs: &[&Func<'_>]| {
            funcs
                .iter()
                .map(|f| format!("`{}`", f.name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let evidence = format!(
            "`{}` to `{}` by {}, and back by {}",
            leaf(x),
            leaf(y),
            names(there),
            names(back)
        );
        let mut lead = Lead::new(ID, Mode::DirectSyntax, evidence);
        for func in there.iter().chain(back) {
            lead = lead.item(run, func.item);
        }
        out.push(lead);
    }
}

/// `spelling-table`: an enum of two or more unit variants whose spellings
/// are written in two or more places: `serde` renames, a `Display` impl or
/// derive, and each function of the run that takes it, or a method of it
/// that takes `self`, and returns a `&'static str`.
fn spelling_table(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut tables: BTreeMap<String, Vec<&Func<'_>>> = BTreeMap::new();
    for func in catalog.funcs.iter().filter(|f| f.is_own()) {
        let spells =
            (func.ret.as_ref()).is_some_and(|ret| ret.key() == "&'static std::primitive::str");
        let t = func.item.target;
        let spelled = match (spells, func.has_receiver(), func.args.as_slice(), func.decl) {
            (true, true, [], Some(d)) => Some(item_name(
                run,
                ItemId {
                    target: t,
                    item: ItemRef::Decl(d),
                },
            )),
            (true, false, [arg], _) => named(&arg.ty, None).filter(|name| name.starts_with('#')),
            _ => None,
        };
        if let Some(spelled) = spelled {
            tables.entry(spelled).or_default().push(func);
        }
    }
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let Shape::Enum(variants) = &decl.shape else {
                continue;
            };
            let unit = variants
                .iter()
                .all(|v| matches!(v.fields, VariantFields::Unit));
            if variants.len() < 2 || !unit {
                continue;
            }
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            let renamed = |attrs: &[String]| {
                attrs
                    .iter()
                    .any(|a| a.starts_with("serde") && a.contains("rename"))
            };
            let serde =
                renamed(&decl.meta.attrs) || variants.iter().any(|v| renamed(&v.meta.attrs));
            let derived = (decl.written_derives.iter()).any(|n| {
                matches!(
                    leaf(n).as_str(),
                    "AsRefStr" | "Display" | "EnumString" | "IntoStaticStr"
                )
            });
            let display = derived
                || impls_of_trait(run, t, d, "std::fmt::Display")
                    .next()
                    .is_some();
            let fns = tables
                .get(&item_name(run, at))
                .map_or(&[][..], Vec::as_slice);
            let mut places = Vec::new();
            if serde {
                places.push("serde renames".to_owned());
            }
            if display {
                places.push("Display".to_owned());
            }
            places.extend(fns.iter().map(|f| format!("`{}`", f.name)));
            if places.len() < 2 {
                continue;
            }
            let evidence = format!(
                "{} unit variants spelled by {}",
                variants.len(),
                places.join(", ")
            );
            let mut lead = Lead::new("spelling-table", Mode::DirectSyntax, evidence).item(run, at);
            for func in fns {
                lead = lead.item(run, func.item);
            }
            out.push(lead);
        }
    }
}
