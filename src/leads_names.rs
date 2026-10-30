//! The detectors of names, attributes, placement, and visibility: words
//! left from another design, lint suppressions, test knobs, std names,
//! collisions, role names and synonyms, wrapper families, getter styles,
//! and the placement of impls and items. This file holds the group's
//! registry and the detectors that read one name at a time; its siblings
//! hold the rest.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, FnKind, Func, impl_module, parent_module};
use crate::leads_types::Ty;
use crate::leads_words::words;
use crate::model::{DeclKind, ItemKind, Meta, ModId, Receiver, Shape, ValueRef};
use crate::model_body::Role;
use crate::prelude::prelude_path;
use crate::test_support::is_test_support;
use crate::typerefs::strip_raw;
use crate::{leads_names_modules, leads_names_roles};

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "compat-shim",
    "lint-suppression",
    "test-knob",
    "prelude-name",
    "name-collision",
    "same-name-many-types",
    "same-type-many-role-names",
    "synonym-mix",
    "wrapper-name-family",
    "getter-style-mix",
    "impl-spread",
    "single-item-module",
    "pub-off-surface",
    "hidden-non-exhaustive",
    "stringly-typed",
    "module-cycle",
];

/// The ids whose leads rest on what the run does not observe.
pub(crate) const ABSENT: &[&str] = &["pub-off-surface", "hidden-non-exhaustive"];

/// The whole words that name a compatibility shim.
const SHIM_WORDS: [&str; 5] = ["compat", "legacy", "old", "shim", "v2"];

/// The whole words that name a test knob.
const KNOB_WORDS: [&str; 5] = ["fake", "fixture", "mock", "override", "test"];

/// The `std` items outside the prelude whose names a crate item may
/// shadow, with their paths.
const STD_NAMES: [(&str, &str); 15] = [
    ("Arc", "std::sync::Arc"),
    ("BTreeMap", "std::collections::BTreeMap"),
    ("BTreeSet", "std::collections::BTreeSet"),
    ("Cell", "std::cell::Cell"),
    ("Cow", "std::borrow::Cow"),
    ("Duration", "std::time::Duration"),
    ("File", "std::fs::File"),
    ("HashMap", "std::collections::HashMap"),
    ("HashSet", "std::collections::HashSet"),
    ("Instant", "std::time::Instant"),
    ("Mutex", "std::sync::Mutex"),
    ("Path", "std::path::Path"),
    ("PathBuf", "std::path::PathBuf"),
    ("Rc", "std::rc::Rc"),
    ("RefCell", "std::cell::RefCell"),
];

/// The suffixes that name a wrapper's inner function.
const WRAPPER_SUFFIXES: [&str; 4] = ["_body", "_helper", "_impl", "_inner"];

/// The suffixes that name the private implementation behind a public
/// wrapper, such as `parse` over `parse_inner`.
const IMPL_SUFFIXES: [&str; 3] = ["_body", "_impl", "_inner"];

/// One named node of the run, as the name detectors read it.
struct Named<'a> {
    item: ItemId,
    name: &'a str,
    /// The module that declares it: a member's is its declaration's, and a
    /// module's is its parent's.
    module: ModId,
    lints: &'a [String],
    attrs: &'a [String],
    /// Whether it is test support.
    support: bool,
    /// Whether a trait sets its name: a member of a trait impl.
    fixed: bool,
}

/// Returns every named node of the run: modules but the roots,
/// declarations, module values, declarations' own members and variants,
/// and impl members.
fn named_nodes<'a>(run: &AnalysisRun<'a>) -> Vec<Named<'a>> {
    let mut out = Vec::new();
    for (t, krate) in run.krates.iter().enumerate() {
        let at = |item| ItemId { target: t, item };
        let node =
            |item, name: &'a str, module, meta: &'a Meta, (support, fixed): (bool, bool)| Named {
                item: at(item),
                name,
                module,
                lints: &meta.lints,
                attrs: &meta.attrs,
                support: support || is_test_support(&meta.cfgs),
                fixed,
            };
        for (m, module) in krate.modules.iter().enumerate() {
            if let Some(name) = module.path.last() {
                out.push(Named {
                    item: at(ItemRef::Module(m)),
                    name,
                    module: parent_module(krate, m).unwrap_or(m),
                    lints: &module.lints,
                    attrs: &[],
                    support: module.test_support,
                    fixed: false,
                });
            }
            for (k, item) in module.items.iter().enumerate() {
                let value = ItemRef::Value(ValueRef { module: m, item: k });
                if crate::analysis_items::is_value(&item.kind) {
                    let flags = (module.test_support, false);
                    out.push(node(value, &item.name, m, &item.meta, flags));
                }
            }
        }
        for (d, decl) in krate.decls.iter().enumerate() {
            let (m, flags) = (decl.module, (decl.test_support, false));
            out.push(node(ItemRef::Decl(d), &decl.name, m, &decl.meta, flags));
            for (k, member) in decl.members.iter().enumerate() {
                out.push(node(
                    ItemRef::Own(d, k),
                    &member.name,
                    m,
                    &member.meta,
                    flags,
                ));
            }
            if let Shape::Enum(variants) = &decl.shape {
                for (k, variant) in variants.iter().enumerate() {
                    let at = ItemRef::Variant(d, k);
                    out.push(node(at, &variant.name, m, &variant.meta, flags));
                }
            }
        }
        for (i, imp) in krate.impls.iter().enumerate() {
            let decl = run.attached[t][i];
            let support = decl.is_some_and(|d| krate.decls[d].test_support);
            let m = decl.map_or_else(|| impl_module(krate, i), |d| krate.decls[d].module);
            for (k, member) in imp.members.iter().enumerate() {
                let at = ItemRef::Inherent(i, k);
                out.push(node(at, &member.name, m, &member.meta, (support, false)));
            }
            for (k, member) in imp.trait_members.iter().enumerate() {
                let at = ItemRef::TraitImpl(i, k);
                out.push(node(at, &member.name, m, &member.meta, (support, true)));
            }
        }
    }
    out.retain(|node| !node.name.is_empty());
    out
}

/// Returns the words of a module's path, each also without a plural `s`,
/// which name what the module's items are about.
fn module_words(run: &AnalysisRun<'_>, t: usize, m: ModId) -> BTreeSet<String> {
    let segments = run.krates[t].modules[m].path.iter();
    let words = segments.flat_map(|segment| words(segment));
    let single = |word: String| [word.strip_suffix('s').unwrap_or(&word).to_owned(), word];
    words.flat_map(single).collect()
}

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let catalog = Catalog::new(run);
    let nodes = named_nodes(run);
    words_of_nodes(run, &catalog, &nodes, out);
    prelude_name(run, out);
    wrapper_name_family(run, &catalog, out);
    getter_style_mix(run, &catalog, out);
    leads_names_roles::find(run, &catalog, out);
    leads_names_modules::find(run, out);
}

/// Returns the first of a name's words that `set` holds.
fn word_in(name: &str, set: &[&str]) -> Option<String> {
    words(name)
        .into_iter()
        .find(|word| set.contains(&word.as_str()))
}

/// `compat-shim`, `lint-suppression`, and `test-knob`: a node marked
/// `deprecated` or named with a whole shim word, a node with an `allow`
/// lint attribute or an `expect` without a reason, and a node outside test
/// support named with a whole test word that no segment of its module's
/// path holds, since a module named for a word makes it the domain's. A
/// trait sets the names of its impls' members, so their names are not
/// read; a variant's named field counts as a node.
fn words_of_nodes(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    nodes: &[Named<'_>],
    out: &mut Vec<Lead>,
) {
    let lead = |id, evidence| Lead::new(id, Mode::DirectSyntax, evidence);
    for node in nodes {
        let deprecated = node
            .attrs
            .iter()
            .find(|attr| attr.starts_with("deprecated"));
        let shim = word_in(node.name, &SHIM_WORDS).filter(|_| !node.fixed);
        let evidence = match (deprecated, shim) {
            (Some(attr), _) => Some(format!("`#[{attr}]`")),
            (None, Some(word)) => Some(format!("the word `{word}` in `{}`", node.name)),
            (None, None) => None,
        };
        if let Some(evidence) = evidence {
            out.push(lead("compat-shim", evidence).item(run, node.item));
        }
        let lints = suppressions(node.lints);
        if !lints.is_empty() {
            out.push(lead("lint-suppression", lints.join(", ")).item(run, node.item));
        }
        let knob = word_in(node.name, &KNOB_WORDS).filter(|_| !node.support && !node.fixed);
        if let Some(word) =
            knob.filter(|w| !module_words(run, node.item.target, node.module).contains(w))
        {
            let evidence = format!("the word `{word}` in `{}`", node.name);
            out.push(lead("test-knob", evidence).item(run, node.item));
        }
    }
    for field in catalog
        .fields
        .iter()
        .filter(|f| f.variant.is_some() && f.named)
    {
        if let Some(word) = word_in(&field.name, &SHIM_WORDS) {
            let evidence = format!("the word `{word}` in `{}`", field.name);
            out.push(field.add(run, lead("compat-shim", evidence)));
        }
        let ItemRef::Decl(d) = field.decl.item else {
            continue;
        };
        let decl = &run.krates[field.decl.target].decls[d];
        let knob = word_in(&field.name, &KNOB_WORDS).filter(|_| !decl.test_support);
        if let Some(word) =
            knob.filter(|w| !module_words(run, field.decl.target, decl.module).contains(w))
        {
            let evidence = format!("the word `{word}` in `{}`", field.name);
            out.push(field.add(run, lead("test-knob", evidence)));
        }
    }
    for field in catalog
        .fields
        .iter()
        .filter(|f| f.variant.is_some() || !f.named)
    {
        let lints = suppressions(&field.meta.lints);
        if !lints.is_empty() {
            out.push(field.add(run, lead("lint-suppression", lints.join(", "))));
        }
    }
}

/// Returns the lint attributes that suppress a lint, as evidence shows
/// them: each `allow`, and each `expect` without a reason. An `expect`
/// with a reason states why, and the compiler warns once its lint no
/// longer fires.
fn suppressions(lints: &[String]) -> Vec<String> {
    let justified = |lint: &str| {
        lint.match_indices("reason").any(|(at, word)| {
            let rest = lint[at + word.len()..].trim_start();
            let quoted = rest.strip_prefix('=').map(str::trim_start);
            (quoted.and_then(|text| text.strip_prefix('"')))
                .is_some_and(|text| !text.starts_with('"'))
        })
    };
    (lints.iter())
        .filter(|lint| {
            lint.starts_with("allow(") || (lint.starts_with("expect(") && !justified(lint))
        })
        .map(|lint| format!("`#[{lint}]`"))
        .collect()
}

/// Returns the `std` path an item's name shadows: a prelude type or
/// trait, or one of [`STD_NAMES`].
fn std_path(name: &str) -> Option<&'static str> {
    let prelude = prelude_path(name).filter(|path| !path.starts_with("std::primitive::"));
    prelude.or_else(|| {
        STD_NAMES
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, path)| *path)
    })
}

/// `prelude-name`: a declaration named like a prelude or common `std`
/// type or trait, or a function named `drop`. A `Result` alias of a
/// `Result` is the common convention and is left out.
fn prelude_name(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let name = strip_raw(&decl.name);
            let Some(path) = std_path(&name) else {
                continue;
            };
            if let (DeclKind::Alias, Shape::Alias(target)) = (decl.kind, &decl.shape) {
                let target = Ty::of(run, t, target, None);
                if name == "Result" && target.is_result() {
                    continue;
                }
            }
            let evidence = format!("named like `{path}`");
            let at = ItemId {
                target: t,
                item: ItemRef::Decl(d),
            };
            out.push(Lead::new("prelude-name", Mode::DirectSyntax, evidence).item(run, at));
        }
        for (m, module) in krate.modules.iter().enumerate() {
            for (k, item) in module.items.iter().enumerate() {
                if matches!(item.kind, ItemKind::Fn(_)) && strip_raw(&item.name) == "drop" {
                    let at = ItemId {
                        target: t,
                        item: ItemRef::Value(ValueRef { module: m, item: k }),
                    };
                    let lead = Lead::new(
                        "prelude-name",
                        Mode::DirectSyntax,
                        "named like `std::mem::drop`",
                    );
                    out.push(lead.item(run, at));
                }
            }
        }
    }
}

/// `wrapper-name-family`: functions of one module, or methods of one type,
/// named `x` and `x_inner`, `x_impl`, `x_body`, `x_helper`, or `do_x`, one
/// lead per `x` with every sibling. A sibling of [`IMPL_SUFFIXES`] that is
/// less visible than `x` and that only the family is observed to call is
/// the private half of a public wrapper, a design, and is left out.
fn wrapper_name_family(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut scopes: BTreeMap<(usize, bool, usize), BTreeMap<&str, Vec<&Func<'_>>>> =
        BTreeMap::new();
    for func in catalog.funcs.iter().filter(|f| f.is_own()) {
        let scope = match (func.kind, func.decl) {
            (FnKind::Inherent, Some(d)) => (func.item.target, true, d),
            (FnKind::Inherent, None) => continue,
            _ => (func.item.target, false, func.module),
        };
        let names = scopes.entry(scope).or_default();
        names.entry(func.name.as_str()).or_default().push(func);
    }
    for names in scopes.values() {
        let mut families: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for &name in names.keys() {
            let inner = WRAPPER_SUFFIXES
                .iter()
                .find_map(|suffix| name.strip_suffix(suffix));
            let base = inner
                .or_else(|| name.strip_prefix("do_"))
                .filter(|base| !base.is_empty() && names.contains_key(base));
            if let Some(base) = base {
                families.entry(base).or_default().push(name);
            }
        }
        for (base, siblings) in families {
            let bases = &names[base];
            let family: BTreeSet<ItemId> = (bases.iter())
                .chain(siblings.iter().flat_map(|name| &names[name]))
                .map(|func| func.item)
                .collect();
            let widest = bases.iter().map(|func| func.vis).min();
            let private_half = |name: &str| {
                IMPL_SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
                    && (names[name].iter())
                        .all(|func| Some(func.vis) > widest && called_within(run, func, &family))
            };
            let open: Vec<&str> = siblings.into_iter().filter(|n| !private_half(n)).collect();
            if open.is_empty() {
                continue;
            }
            let shown: Vec<String> = open.iter().map(|name| format!("`{name}`")).collect();
            let evidence = format!("`{base}` and {}", shown.join(", "));
            let mut lead = Lead::new("wrapper-name-family", Mode::DirectSyntax, evidence);
            for func in bases
                .iter()
                .chain(open.iter().flat_map(|name| &names[name]))
            {
                lead = lead.item(run, func.item);
            }
            out.push(lead);
        }
    }
}

/// Tells whether a function has observed callers, every one a direct call
/// from a body of `family`.
fn called_within(run: &AnalysisRun<'_>, func: &Func<'_>, family: &BTreeSet<ItemId>) -> bool {
    let mut edges = run.edges_to(func.item).peekable();
    edges.peek().is_some()
        && edges.all(|edge| {
            let caller = run.body_items[edge.target][edge.body];
            edge.kind == PathKind::Call(Role::Call)
                && caller.is_some_and(|item| {
                    family.contains(&ItemId {
                        target: edge.target,
                        item,
                    })
                })
        })
}

/// `getter-style-mix`: one target has getters named `get_x` beside
/// getters named after the field they read: methods that take `&self`
/// alone and return a value.
fn getter_style_mix(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let mut styles: BTreeMap<usize, (Vec<&Func<'_>>, Vec<&Func<'_>>)> = BTreeMap::new();
    for func in &catalog.funcs {
        let getter = func.kind == FnKind::Inherent
            && func.sig.receiver == Receiver::Shared
            && func.args.is_empty()
            && func.ret.as_ref().is_some_and(|ret| !ret.is_unit());
        let Some(d) = func.decl.filter(|_| getter) else {
            continue;
        };
        let style = styles.entry(func.item.target).or_default();
        if func.name.len() > 4 && func.name.starts_with("get_") {
            style.0.push(func);
            continue;
        }
        let at = ItemId {
            target: func.item.target,
            item: ItemRef::Decl(d),
        };
        let field = catalog
            .fields_of(at, None)
            .any(|f| f.named && f.name == func.name);
        if field && func.self_fields.contains(&func.name) {
            style.1.push(func);
        }
    }
    for (prefixed, bare) in styles.into_values() {
        let (Some(first), Some(plain)) = (prefixed.first(), bare.first()) else {
            continue;
        };
        let evidence = format!(
            "{} `get_` getters, such as `{}`, beside {} named after their field, such as `{}`",
            prefixed.len(),
            first.name,
            bare.len(),
            plain.name
        );
        let mut lead = Lead::new("getter-style-mix", Mode::DirectSyntax, evidence);
        for func in &prefixed {
            lead = lead.item(run, func.item);
        }
        out.push(lead);
    }
}
