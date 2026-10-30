//! The census detector of placement: an internal item that nothing ties to
//! its own module, whose every observed user sits in one other module. No
//! compiler warns about an item away from its users.

use std::collections::BTreeSet;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_uses::{Use, UseKind};
use crate::leads::{Lead, Mode};
use crate::leads_census_scope::{Candidate, Kind, consumers, elsewhere, in_macro, module_name};
use crate::leads_items::impl_home;
use crate::leads_words::words;
use crate::model::{ItemKind, MemberKind, ModId, Shape, Text, ValueRef, VariantFields};
use crate::model_body::BodyId;

/// The fewest other declarations and module values an item's module holds
/// for `defined-away-from-observed-users`: a module that holds only the
/// item is its own file, which is a common layout.
const NEIGHBORS_FROM: usize = 1;

/// The fewest consuming uses for `defined-away-from-observed-users`.
const USES_FROM: usize = 2;

/// The fewest letters of a word that ties an item's name to its module's,
/// such as `harden` in `hardening`.
const STEM_FROM: usize = 4;

/// Tells whether a name and a module's name share a word stem: one word
/// of [`STEM_FROM`] or more letters that starts the other.
fn namesake(name: &str, module: &str) -> bool {
    let theirs = words(module);
    words(name).iter().any(|word| {
        theirs.iter().any(|other| {
            let (short, long) = if word.len() <= other.len() {
                (word, other)
            } else {
                (other, word)
            };
            short.len() >= STEM_FROM && long.starts_with(short.as_str())
        })
    })
}

/// Returns the module an item is in: a module's is itself, a part's is its
/// declaration's, and an impl member's is its type's, or the impl's for an
/// impl of no declaration of the run.
fn home(run: &AnalysisRun<'_>, item: ItemId) -> ModId {
    let krate = &run.krates[item.target];
    match item.item {
        ItemRef::Module(m) => m,
        ItemRef::Decl(d) | ItemRef::Own(d, _) | ItemRef::Variant(d, _) => krate.decls[d].module,
        ItemRef::Value(at) => at.module,
        ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _) => impl_home(run, item.target, i),
    }
}

/// Tells whether an item is part of a candidate: the candidate itself, or a
/// field, variant, trait item, or impl member of its declaration.
fn part_of(run: &AnalysisRun<'_>, c: &Candidate<'_>, item: ItemId) -> bool {
    if item.target != c.item.target {
        return false;
    }
    match (item.item, c.item.item) {
        (ItemRef::Own(x, _) | ItemRef::Variant(x, _), ItemRef::Decl(d)) => x == d,
        (ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _), ItemRef::Decl(d)) => {
            run.attached[item.target][i] == Some(d)
        }
        (item, own) => item == own,
    }
}

/// Tells whether an item is another item of a candidate's module.
fn neighbor(run: &AnalysisRun<'_>, c: &Candidate<'_>, item: ItemId) -> bool {
    item.target == c.item.target && !part_of(run, c, item) && home(run, item) == c.module
}

/// Returns the texts a candidate's declaration writes and the bodies it
/// owns: a function's signature, generics and bounds included, and block, a
/// constant's or static's type and initializer, and a declaration's fields,
/// variants, alias target, and members, its impls' members included. `None`
/// for any other item.
fn written<'a>(run: &AnalysisRun<'a>, c: &Candidate<'_>) -> Option<(Vec<&'a Text>, Vec<BodyId>)> {
    let t = c.item.target;
    let krate = &run.krates[t];
    let mut texts: Vec<&'a Text> = Vec::new();
    let mut bodies: Vec<BodyId> = run.body_of(c.item).into_iter().collect();
    match c.item.item {
        ItemRef::Value(at) => match &krate.modules[at.module].items[at.item].kind {
            ItemKind::Fn(sig) => texts.extend(sig.texts()),
            ItemKind::Const { ty, .. } | ItemKind::Static { ty, .. } => texts.push(ty),
            _ => return None,
        },
        ItemRef::Decl(d) => {
            for (part, member) in run.members(t, d) {
                match &member.kind {
                    MemberKind::Fn(sig) => texts.extend(sig.texts()),
                    MemberKind::Field(ty) | MemberKind::Const { ty, .. } | MemberKind::Type(ty) => {
                        texts.push(ty);
                    }
                    MemberKind::Mark(_) | MemberKind::Unsupported(_) => {}
                }
                bodies.extend(run.body_of(ItemId {
                    target: t,
                    item: part,
                }));
            }
            match &krate.decls[d].shape {
                Shape::Tuple(params) => texts.extend(params.iter().map(|p| &p.text)),
                Shape::Enum(variants) => {
                    for variant in variants {
                        if let VariantFields::Tuple(params) | VariantFields::Named(params) =
                            &variant.fields
                        {
                            texts.extend(params.iter().map(|p| &p.text));
                        }
                    }
                }
                Shape::Alias(target) => texts.push(target),
                Shape::Named | Shape::Unit | Shape::Trait { .. } => {}
            }
        }
        _ => return None,
    }
    Some((texts, bodies))
}

/// Tells whether a candidate is tied to its own module: its name shares a
/// word stem with the module's, or a type its texts name, an item its
/// bodies reach, or a name its bodies call as a method or write in macro
/// arguments is another item of that module, or the candidate is no
/// function, constant, static, or declaration.
fn tied(run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
    let t = c.item.target;
    let krate = &run.krates[t];
    let own = krate.modules[c.module].path.last();
    if own.is_some_and(|module| namesake(&c.name, module)) {
        return true;
    }
    let Some((texts, bodies)) = written(run, c) else {
        return true;
    };
    let typed = |r| {
        run.item_of_ref(t, r)
            .is_some_and(|item| neighbor(run, c, item))
    };
    if texts.iter().flat_map(|text| text.refs()).any(typed) {
        return true;
    }
    let reached = |item: Option<ItemId>| item.is_some_and(|item| neighbor(run, c, item));
    let edges = bodies.iter().flat_map(|&b| run.edges_from(t, b));
    if edges
        .into_iter()
        .any(|e| reached(e.item) || reached(e.prefix))
    {
        return true;
    }
    let mut names: BTreeSet<&str> = BTreeSet::new();
    for (d, decl) in krate.decls.iter().enumerate() {
        if decl.module == c.module && c.decl != Some(d) {
            names.insert(&decl.name);
            names.extend(run.members(t, d).map(|(_, member)| member.name.as_str()));
        }
    }
    let values = krate.modules[c.module].items.iter();
    names.extend(
        values
            .filter(|item| item.name != c.name)
            .map(|item| item.name.as_str()),
    );
    bodies.iter().any(|&b| {
        let facts = &krate.bodies[b].facts;
        (facts.methods.iter().chain(&facts.macro_idents)).any(|name| names.contains(name.as_str()))
    })
}

/// Tells whether a use is in a body that is part of the candidate, such as
/// a method of its type.
fn own_body(run: &AnalysisRun<'_>, c: &Candidate<'_>, found: &Use) -> bool {
    let (UseKind::Path(e) | UseKind::Qualifier(e)) = found.kind else {
        return false;
    };
    let edge = &run.edges[e];
    let owner = run.body_items[edge.target][edge.body];
    owner.is_some_and(|item| {
        let item = ItemId {
            target: edge.target,
            item,
        };
        part_of(run, c, item)
    })
}

/// Returns the declarations and module values of module `m` of target `t`.
fn held(run: &AnalysisRun<'_>, t: usize, m: ModId) -> Vec<ItemId> {
    let krate = &run.krates[t];
    let at = |item| ItemId { target: t, item };
    let decls = (krate.decls.iter().enumerate())
        .filter(|(_, decl)| decl.module == m)
        .map(|(d, _)| at(ItemRef::Decl(d)));
    let values = (krate.modules[m].items.iter().enumerate())
        .filter(|(_, item)| crate::analysis_items::is_value(&item.kind))
        .map(|(k, _)| at(ItemRef::Value(ValueRef { module: m, item: k })));
    decls.chain(values).collect()
}

/// `defined-away-from-observed-users`: a declaration or module value off
/// every surface, in a module that holds other items, that nothing ties to
/// that module, whose two or more consuming uses outside its own bodies,
/// re-exports left out, are all in one other module of its target that is
/// not around its own, and that is the one item the user module takes from
/// its module. A module's entry point that reaches the rest of its module,
/// a helper module that gives one user several items, and a child's item
/// that its parent uses are the common split of one concern. Every hit of
/// its name is in its own file or that module's file, outside macros and
/// test code, so no bare token hides a user.
pub(crate) fn defined_away(run: &AnalysisRun<'_>, c: &Candidate<'_>, out: &mut Vec<Lead>) {
    let t = c.item.target;
    let listed = matches!(c.kind, Kind::Decl(_) | Kind::Function | Kind::Value);
    let neighbors: Vec<ItemId> = (held(run, t, c.module).into_iter())
        .filter(|&item| item != c.item)
        .collect();
    if !listed || run.on_surface(c.item) || neighbors.len() < NEIGHBORS_FROM {
        return;
    }
    let seen: Vec<&Use> = (consumers(run, c.item).into_iter())
        .filter(|u| u.kind != UseKind::Reexport && !own_body(run, c, u))
        .collect();
    let places: BTreeSet<(Option<usize>, Option<ModId>)> = (seen.iter())
        .map(|u| (run.census.sources[u.source].target, u.module))
        .collect();
    let [(Some(user_target), Some(user))] = places.into_iter().collect::<Vec<_>>()[..] else {
        return;
    };
    let krate = &run.krates[t];
    let (own, theirs) = (&krate.modules[c.module].path, &krate.modules[user].path);
    if user_target != t || own.starts_with(theirs) || seen.len() < USES_FROM || tied(run, c) {
        return;
    }
    let shared = neighbors.into_iter().any(|item| {
        (consumers(run, item).iter()).any(|u| u.kind != UseKind::Reexport && u.module == Some(user))
    });
    if shared {
        return;
    }
    let Some(loc) = run.loc(c.item) else {
        return;
    };
    let files = [
        run.census.tree_source(t, loc.file),
        run.census.tree_source(t, krate.modules[user].file),
    ];
    let stray = (run.census.token_hits(&c.name).iter()).any(|hit| !files.contains(&hit.source));
    if stray || elsewhere(&run.census.count(&c.name)) || in_macro(run, &c.name) {
        return;
    }
    let lexical = seen.iter().all(|u| matches!(u.kind, UseKind::Qualified(_)));
    let mode = if lexical {
        Mode::Lexical
    } else {
        Mode::Resolved
    };
    let evidence = format!(
        "{} observed uses, all in `{}`; nothing ties it to `{}`",
        seen.len(),
        module_name(run, t, user),
        module_name(run, t, c.module)
    );
    let lead = Lead::new("defined-away-from-observed-users", mode, evidence);
    out.push(lead.item(run, c.item).limits(&["macros"]));
}
