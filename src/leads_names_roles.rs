//! The detectors of role names: one field or parameter name that holds
//! several types in one module, one type held under several names that
//! share a word in one module, a role named in one synonym and typed in
//! another, and a string that holds a kind or a state. A declaration's
//! fields and its members' parameters count in the declaration's module,
//! wherever the impls are. Each type compares at its core, so a pointer, a
//! lock, or an `Option` around a type is the same role.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemRef;
use crate::leads::{Lead, Mode};
use crate::leads_comments_words::KEYWORDS;
use crate::leads_items::{Catalog, FnKind, is_wire};
use crate::leads_types::{Ty, leaf};
use crate::leads_words::words;
use crate::model::{Meta, ModId};

/// The fewest names that share a word for `same-type-many-role-names`.
const ROLES_FROM: usize = 3;

/// The names that hold whatever a function is about, so that their types
/// differ by design.
const GENERIC_NAMES: [&str; 10] = [
    "arg", "args", "data", "input", "item", "items", "other", "output", "result", "value",
];

/// The words too common to show that names share a role.
const STOP_WORDS: [&str; 9] = [
    "and", "for", "from", "get", "has", "into", "new", "the", "with",
];

/// The parameter and field names that a `String` should not hold.
const STRINGLY: [&str; 6] = ["kind", "mode", "role", "state", "status", "type"];

/// The synonym sets of version 1: the words of each set name one role.
const SYNONYMS_V1: [&[&str]; 4] = [
    &["label", "name", "title"],
    &["body", "content", "source", "text"],
    &["class", "kind", "type"],
    &["id", "index", "key"],
];

/// One named field or parameter.
struct Site {
    target: usize,
    module: ModId,
    name: String,
    /// The type it holds, as [`Ty::core`] gives it.
    core: Ty,
    /// The field, or the function, by its index in the catalog.
    of: Of,
}

#[derive(Clone, Copy)]
enum Of {
    Field(usize),
    Func(usize),
}

pub(crate) fn find(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let sites = sites(run, catalog);
    same_name_many_types(run, catalog, &sites, out);
    same_type_many_role_names(run, catalog, &sites, out);
    synonym_mix(run, catalog, out);
    stringly_typed(run, catalog, out);
}

/// Tells whether a role's core type is one the detectors compare: no
/// generic parameter, `Self`, unit, or `impl` or `dyn` trait.
fn compared(core: &Ty) -> bool {
    !(core.is_generic() || core.is_unit() || core.is_opaque())
}

/// Returns every named field, and every named parameter of a function
/// whose signature no trait impl sets, whose core type is compared.
fn sites(run: &AnalysisRun<'_>, catalog: &Catalog<'_>) -> Vec<Site> {
    let mut out = Vec::new();
    for (f, field) in catalog.fields.iter().enumerate() {
        let ItemRef::Decl(d) = field.decl.item else {
            continue;
        };
        let core = field.ty.core();
        if field.named && compared(&core) {
            out.push(Site {
                target: field.decl.target,
                module: run.krates[field.decl.target].decls[d].module,
                name: field.name.clone(),
                core,
                of: Of::Field(f),
            });
        }
    }
    for (f, func) in catalog.funcs.iter().enumerate() {
        if func.kind == FnKind::TraitImpl {
            continue;
        }
        for arg in &func.args {
            let core = arg.ty.core();
            if let (Some(name), true) = (&arg.name, compared(&core)) {
                out.push(Site {
                    target: func.item.target,
                    module: func.module,
                    name: name.clone(),
                    core,
                    of: Of::Func(f),
                });
            }
        }
    }
    out
}

/// Adds a role to a lead as a subject: its field, or its function.
fn add(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, lead: Lead, of: Of) -> Lead {
    match of {
        Of::Field(f) => catalog.fields[f].add(run, lead),
        Of::Func(f) => lead.item(run, catalog.funcs[f].item),
    }
}

/// `same-name-many-types`: one field or parameter name, compared without
/// case and underscores, that holds two or more core types in one module,
/// one of them all `std`'s, such as a `PathBuf` beside a `String` or a
/// `u32` beside a `usize`. Two types of the run under one name are the
/// common naming of one role by its kind, a name of [`GENERIC_NAMES`] holds
/// anything, and a name that holds only errors chains them; all three are
/// left out.
fn same_name_many_types(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    sites: &[Site],
    out: &mut Vec<Lead>,
) {
    let mut groups: BTreeMap<(usize, ModId, String), BTreeMap<&Ty, Vec<&Site>>> = BTreeMap::new();
    for site in sites
        .iter()
        .filter(|s| !GENERIC_NAMES.contains(&s.name.as_str()))
    {
        let name = site.name.to_lowercase().replace('_', "");
        let types = groups.entry((site.target, site.module, name)).or_default();
        types.entry(&site.core).or_default().push(site);
    }
    let error = |ty: &&Ty| (ty.names().next()).is_some_and(|name| leaf(name).ends_with("Error"));
    for types in groups.values() {
        let std = types.keys().any(|ty| ty.is_std());
        if types.len() < 2 || !std || types.keys().all(error) {
            continue;
        }
        let Some(first) = types.values().flatten().next() else {
            continue;
        };
        let shown: Vec<String> = (types.iter())
            .map(|(ty, sites)| format!("`{}` in {}", ty.shown(), sites.len()))
            .collect();
        let evidence = format!("`{}` holds {}", first.name, shown.join(", "));
        let mut lead = Lead::new("same-name-many-types", Mode::DirectSyntax, evidence);
        for sites in types.values() {
            lead = add(run, catalog, lead, sites[0].of);
        }
        out.push(lead);
    }
}

/// `same-type-many-role-names`: one core type that [`ROLES_FROM`] or more
/// field or parameter names hold in one module, where those names share a
/// word of three or more letters that the type's own names do not hold;
/// the shared word is required for every type, primitives included.
fn same_type_many_role_names(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    sites: &[Site],
    out: &mut Vec<Lead>,
) {
    let mut groups: BTreeMap<(usize, ModId, &Ty), BTreeMap<&str, &Site>> = BTreeMap::new();
    for site in sites {
        let names = groups
            .entry((site.target, site.module, &site.core))
            .or_default();
        names.entry(site.name.as_str()).or_insert(site);
    }
    for ((_, _, ty), names) in groups {
        if names.len() < ROLES_FROM {
            continue;
        }
        let typed: BTreeSet<String> = ty.names().flat_map(|name| words(&leaf(name))).collect();
        let mut stems: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for &name in names.keys() {
            let own: BTreeSet<String> = words(name).into_iter().collect();
            for word in own {
                let plain = !STOP_WORDS.contains(&word.as_str()) && !typed.contains(&word);
                if word.len() >= 3 && plain {
                    stems.entry(word).or_default().push(name);
                }
            }
        }
        let best = (stems.iter())
            .filter(|(_, named)| named.len() >= ROLES_FROM)
            .max_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| b.0.cmp(a.0)));
        let Some((stem, named)) = best else {
            continue;
        };
        let shown: Vec<String> = named.iter().map(|name| format!("`{name}`")).collect();
        let evidence = format!(
            "`{}` as {}, which share `{stem}`",
            ty.shown(),
            shown.join(", ")
        );
        let mut lead = Lead::new("same-type-many-role-names", Mode::DirectSyntax, evidence);
        for name in named {
            lead = add(run, catalog, lead, names[name].of);
        }
        out.push(lead);
    }
}

/// `stringly-typed`: a `String` or `str` field or parameter named kind,
/// type, mode, status, role, or state. A wire type's field is left out,
/// since a protocol may define the strings it carries.
fn stringly_typed(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let stringly = |name: &str, ty: &Ty| {
        STRINGLY.contains(&name.trim_end_matches('_')) && ty.base().is_string()
    };
    let fields = catalog
        .fields
        .iter()
        .filter(|f| f.named && !is_wire(run, f.decl));
    for field in fields.filter(|f| stringly(&f.name, &f.ty)) {
        let evidence = format!("`{}: {}`", field.name, field.ty.shown());
        out.push(field.add(
            run,
            Lead::new("stringly-typed", Mode::DirectSyntax, evidence),
        ));
    }
    for func in catalog.funcs.iter().filter(|f| f.kind != FnKind::TraitImpl) {
        let found: Vec<String> = (func.args.iter())
            .filter_map(|arg| Some((arg.name.as_deref()?, &arg.ty)))
            .filter(|(name, ty)| stringly(name, ty))
            .map(|(name, ty)| format!("`{name}: {}`", ty.shown()))
            .collect();
        if !found.is_empty() {
            let lead = Lead::new("stringly-typed", Mode::DirectSyntax, found.join(", "));
            out.push(lead.item(run, func.item));
        }
    }
}

/// Returns the words of the names a type writes that are not `std`'s.
fn type_words(ty: &Ty) -> Vec<String> {
    (ty.base().names())
        .filter(|name| !name.starts_with("std::") && !name.starts_with('$') && *name != "Self")
        .flat_map(|name| words(&leaf(name)))
        .collect()
}

/// Returns the values that a node's `serde` attributes rename it to.
fn renames(meta: &Meta) -> Vec<String> {
    let mut out = Vec::new();
    for attr in meta.attrs.iter().filter(|attr| attr.starts_with("serde")) {
        for (at, _) in attr.match_indices("rename = \"") {
            let rest = &attr[at + "rename = \"".len()..];
            out.extend(rest.split('"').next().map(str::to_owned));
        }
    }
    out
}

/// Returns the synonyms of one [`SYNONYMS_V1`] set that a role mixes: a
/// word of its name, and another word of the set in its type or rename
/// that its name does not use. A keyword, such as `type`, is no identifier,
/// so a name that stands in for it, such as `kind`, mixes nothing with it.
fn mixed(name: &str, other: &[String]) -> Option<String> {
    let own = words(name);
    SYNONYMS_V1.iter().find_map(|set| {
        let pick = |list: &[String]| -> BTreeSet<String> {
            list.iter()
                .filter(|w| set.contains(&w.as_str()) && !KEYWORDS.contains(&w.as_str()))
                .cloned()
                .collect()
        };
        let (named, elsewhere) = (pick(&own), pick(other));
        let mixes = !named.is_empty() && !elsewhere.is_empty() && named.is_disjoint(&elsewhere);
        mixes.then(|| {
            named
                .union(&elsewhere)
                .cloned()
                .collect::<Vec<_>>()
                .join(" and ")
        })
    })
}

/// `synonym-mix`: fields, parameters, and functions whose names use a word
/// of one [`SYNONYMS_V1`] set while the names in their types or return
/// types, or their `serde` renames, use another word of that set, such as
/// `name: ArgLabel`. One lead per pair of words in a target names every
/// role that mixes them. A name that its type's own name holds, such as
/// `text: BodyText`, mixes nothing.
fn synonym_mix(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut mixes: BTreeMap<(usize, String), Vec<(String, Of)>> = BTreeMap::new();
    for (f, field) in catalog.fields.iter().enumerate().filter(|(_, f)| f.named) {
        let mut other = type_words(&field.ty);
        let renamed = renames(field.meta);
        other.extend(renamed.iter().flat_map(|name| words(name)));
        if let Some(found) = mixed(&field.name, &other) {
            let mut site = format!("{}: {}", field.name, field.ty.shown());
            if let Some(name) = renamed.first() {
                site = format!("{site}, renamed {name}");
            }
            let roles = mixes.entry((field.decl.target, found)).or_default();
            roles.push((site, Of::Field(f)));
        }
    }
    let funcs =
        (catalog.funcs.iter().enumerate()).filter(|(_, f)| f.is_own() || f.kind == FnKind::Trait);
    for (f, func) in funcs {
        let mut sites: Vec<(String, &str, Vec<String>)> = Vec::new();
        if let Some(ret) = &func.ret {
            let site = format!("fn {} -> {}", func.name, ret.shown());
            sites.push((site, &func.name, type_words(ret)));
        }
        for arg in &func.args {
            if let Some(name) = &arg.name {
                sites.push((
                    format!("{name}: {}", arg.ty.shown()),
                    name,
                    type_words(&arg.ty),
                ));
            }
        }
        for (site, name, other) in sites {
            if let Some(found) = mixed(name, &other) {
                let roles = mixes.entry((func.item.target, found)).or_default();
                roles.push((site, Of::Func(f)));
            }
        }
    }
    for ((_, found), roles) in mixes {
        let mut shown: Vec<String> = Vec::new();
        for (site, _) in &roles {
            let site = format!("`{site}`");
            if !shown.contains(&site) {
                shown.push(site);
            }
        }
        let evidence = format!("{found}, synonyms of set v1, in {}", shown.join(", "));
        let mut lead = Lead::new("synonym-mix", Mode::DirectSyntax, evidence);
        for &(_, of) in &roles {
            lead = add(run, catalog, lead, of);
        }
        out.push(lead);
    }
}
