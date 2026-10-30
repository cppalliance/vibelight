//! The census detectors of paths: a `pub use` that no observed path
//! imports through, a private function with one observed caller or a chain
//! of such functions, a fallible function that tests reach beside its one
//! caller, and a type alias with one observed use.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::leads::{Lead, Mode, Subject};
use crate::leads_census_scope::{
    Candidate, Kind, Scope, consumers, elsewhere, hits_in, in_macro, limits, module_name,
};
use crate::leads_types::Ty;
use crate::model::{DeclKind, ItemKind, MemberKind, ModId, Text, Vis};
use crate::model_body::{BodyId, Role};
use crate::root::TargetKind;
use crate::surface::module_at;
use crate::test_support::is_test_support;
use crate::typerefs::strip_raw;

/// The fewest functions of a chain of single callers that one
/// `one-observed-caller` lead names whatever their sizes.
const CHAIN_FROM: usize = 3;

/// The canonical token count from which a function with one caller, off a
/// long chain, is a named step rather than an inline candidate.
const TINY_BODY: usize = 20;

/// Finds the group's leads on paths.
pub(crate) fn find(
    run: &AnalysisRun<'_>,
    scope: &Scope,
    candidates: &[Candidate<'_>],
    out: &mut Vec<Lead>,
) {
    no_observed_reexport_use(run, scope, out);
    one_observed_caller(run, scope, candidates, out);
    guarded_error_path(run, scope, candidates, out);
    for c in candidates {
        let open = !c.support && !scope.unseen(run, c);
        if open && c.kind == Kind::Decl(DeclKind::Alias) {
            one_observed_alias_use(run, c, out);
        }
    }
}

/// Returns the modules that a glob import of the run's production code
/// imports from, with their targets: a `use` of `path::*`, or a `pub use`
/// of one.
fn globbed(run: &AnalysisRun<'_>) -> BTreeSet<(usize, ModId)> {
    let mut out = BTreeSet::new();
    for (u, krate) in run.krates.iter().enumerate() {
        for (from, module) in krate.modules.iter().enumerate() {
            let mut globs: Vec<(Vec<String>, bool)> = (module.uses.iter())
                .filter(|import| import.name.is_none())
                .map(|import| (import.path.clone(), import.global))
                .collect();
            for item in &module.items {
                if let (ItemKind::Reexport { text, .. }, "*") = (&item.kind, item.name.as_str()) {
                    let written = text.visible(&krate.refs);
                    let global = written.starts_with("::");
                    let segments = written.trim_start_matches("::").split("::");
                    let path = segments.filter(|s| *s != "*").map(strip_raw).collect();
                    globs.push((path, global));
                }
            }
            for (path, global) in globs {
                if let Some(m) = module_at(krate, from, &path, global) {
                    out.insert((u, m));
                    continue;
                }
                let Some((first, rest)) = path.split_first() else {
                    continue;
                };
                if let Some(&t) = run.deps[u].get(first) {
                    if let Some(m) = module_at(&run.krates[t], 0, rest, false) {
                        out.insert((t, m));
                    }
                }
            }
        }
    }
    out
}

/// Tells whether a re-export of target `t` names a declaration that a
/// signature of the run names, which the re-export may only make nameable
/// to the users of that signature.
fn nameable(run: &AnalysisRun<'_>, t: usize, text: &Text) -> bool {
    let reached = text.refs().next().and_then(|r| run.item_of_ref(t, r));
    reached.is_some_and(|item| {
        matches!(item.item, ItemRef::Decl(_))
            && (run.uses_of(item).iter()).any(|u| u.kind == UseKind::Signature)
    })
}

/// `no-observed-reexport-use`: a `pub use` of one name in a binary, or in
/// a library whose package does not allow publishing, when no qualified
/// path of the run writes that name right after the re-exporting module's
/// name, the name another target writes for the crate at the root, or
/// `crate`, `self`, or `super`, and no glob import of production code
/// reads the module. A name that test code writes is left out, since its
/// glob imports go unread, and so is a declaration that a signature
/// names, since the re-export may keep it nameable.
fn no_observed_reexport_use(run: &AnalysisRun<'_>, scope: &Scope, out: &mut Vec<Lead>) {
    let mut pairs: BTreeSet<(&str, &str)> = BTreeSet::new();
    for path in &run.census.paths {
        pairs.extend(
            path.segments
                .windows(2)
                .map(|w| (w[0].as_str(), w[1].as_str())),
        );
    }
    let globbed = globbed(run);
    for (t, krate) in run.krates.iter().enumerate() {
        let info = &run.targets[t];
        let lib = info.kind == TargetKind::Lib;
        if scope.blind_surface(t) || (lib && run.packages[info.package].facts.publish_allowed) {
            continue;
        }
        let mut roots: Vec<&str> = vec![info.crate_name.as_str()];
        for deps in &run.deps {
            roots.extend(
                deps.iter()
                    .filter(|&(_, &d)| d == t)
                    .map(|(n, _)| n.as_str()),
            );
        }
        for (m, module) in krate.modules.iter().enumerate() {
            if module.test_support || globbed.contains(&(t, m)) {
                continue;
            }
            let own = module.path.last().map(|n| strip_raw(n));
            let mut through: Vec<&str> = vec!["crate", "self", "super"];
            match &own {
                Some(name) => through.push(name.as_str()),
                None => through.extend(&roots),
            }
            for item in &module.items {
                let ItemKind::Reexport { text, .. } = &item.kind else {
                    continue;
                };
                let name = strip_raw(&item.name);
                let shown = item.vis == Vis::Public && name != "*";
                if !shown || is_test_support(&item.meta.cfgs) {
                    continue;
                }
                let imported = through.iter().any(|q| pairs.contains(&(*q, name.as_str())));
                if imported || elsewhere(&run.census.count(&name)) || nameable(run, t, text) {
                    continue;
                }
                let evidence = format!(
                    "`pub use {}`, but no observed path imports `{name}` through `{}`",
                    text.visible(&krate.refs),
                    module_name(run, t, m)
                );
                let mut lead = Lead::new("no-observed-reexport-use", Mode::Lexical, evidence);
                let mut path = module.path.clone();
                path.push(name);
                lead.subjects.push(Subject {
                    target: t,
                    module: m,
                    path,
                });
                let mut base = vec!["macros"];
                if lib {
                    base.push("external-consumers");
                }
                out.push(lead.at(t, item.loc).limits(&base));
            }
        }
    }
}

/// `one-observed-caller`: a private function of a module or an inherent
/// impl, other than `main`, whose every edge is a call from one body other
/// than its own, when the hits of its name beyond the run's declarations
/// are exactly those calls, so that no call in macro arguments, test code,
/// or method syntax hides. Functions whose one caller is such a function
/// form a chain: one lead names a chain of [`CHAIN_FROM`] or more, and a
/// shorter chain yields a lead for each function of fewer than
/// [`TINY_BODY`] canonical tokens. A larger helper with one caller is the
/// common way to name a step, so it alone is no lead.
fn one_observed_caller(
    run: &AnalysisRun<'_>,
    scope: &Scope,
    candidates: &[Candidate<'_>],
    out: &mut Vec<Lead>,
) {
    let mut sole: BTreeMap<ItemId, ItemId> = BTreeMap::new();
    for c in candidates {
        let t = c.item.target;
        let function = match c.kind {
            Kind::Function => true,
            Kind::Associated => {
                matches!(
                    run.member(t, c.item.item).map(|m| &m.kind),
                    Some(MemberKind::Fn(_))
                )
            }
            _ => false,
        };
        let open = function && c.vis == Vis::Private && c.name != "main" && !c.support;
        if !open || scope.unseen(run, c) {
            continue;
        }
        let kinds: Vec<(PathKind, usize, BodyId)> = (run.edges_to(c.item))
            .map(|edge| (edge.kind, edge.target, edge.body))
            .collect();
        let direct = kinds
            .iter()
            .all(|(kind, ..)| *kind == PathKind::Call(Role::Call));
        let bodies: BTreeSet<(usize, BodyId)> = kinds.iter().map(|&(_, t, b)| (t, b)).collect();
        let [(ct, cb)] = bodies.into_iter().collect::<Vec<_>>()[..] else {
            continue;
        };
        let Some(caller) = run.body_items[ct][cb] else {
            continue;
        };
        let caller = ItemId {
            target: ct,
            item: caller,
        };
        let counted = run.census.count(&c.name).uses() == kinds.len();
        if direct && counted && caller != c.item {
            sole.insert(c.item, caller);
        }
    }
    let mut chains: BTreeMap<ItemId, Vec<ItemId>> = BTreeMap::new();
    for &callee in sole.keys() {
        let mut top = callee;
        let mut seen = BTreeSet::from([callee]);
        while let Some(&caller) = sole.get(&top) {
            if !sole.contains_key(&caller) || !seen.insert(caller) {
                break;
            }
            top = caller;
        }
        chains.entry(top).or_default().push(callee);
    }
    let tokens = |item: ItemId| {
        let body = run.body_of(item);
        body.map_or(0, |b| {
            run.krates[item.target].bodies[b].facts.fingerprint.tokens
        })
    };
    for chain in chains.into_values() {
        if chain.len() >= CHAIN_FROM {
            out.push(caller_lead(run, &sole, &chain, tokens));
            continue;
        }
        for &callee in chain.iter().filter(|&&callee| tokens(callee) < TINY_BODY) {
            out.push(caller_lead(run, &sole, &[callee], tokens));
        }
    }
}

/// Builds a `one-observed-caller` lead on functions with one observed
/// caller each, named with their sizes and callers.
fn caller_lead(
    run: &AnalysisRun<'_>,
    sole: &BTreeMap<ItemId, ItemId>,
    callees: &[ItemId],
    tokens: impl Fn(ItemId) -> usize,
) -> Lead {
    let shown = |item: ItemId| run.path(item).join("::");
    let found: Vec<String> = (callees.iter())
        .map(|&callee| {
            let caller = shown(sole[&callee]);
            format!(
                "`{}` of {} tokens from `{caller}`",
                shown(callee),
                tokens(callee)
            )
        })
        .collect();
    let evidence = format!("one observed caller each: {}", found.join(", "));
    let mut lead = Lead::new("one-observed-caller", Mode::Resolved, evidence);
    for &callee in callees {
        lead = lead.item(run, callee);
    }
    lead.limits(&["generated-code", "macros"])
}

/// Tells whether an item is a function of a module or an impl that returns
/// a `Result`.
fn returns_result(run: &AnalysisRun<'_>, item: ItemId) -> bool {
    let t = item.target;
    let sig = match item.item {
        ItemRef::Value(at) => match &run.krates[t].modules[at.module].items[at.item].kind {
            ItemKind::Fn(sig) => Some(sig),
            _ => None,
        },
        _ => match run.member(t, item.item).map(|member| &member.kind) {
            Some(MemberKind::Fn(sig)) => Some(sig),
            _ => None,
        },
    };
    let ret = sig.and_then(|sig| sig.ret.as_ref());
    ret.is_some_and(|ret| Ty::of(run, t, ret, None).is_result())
}

/// `guarded-error-path`: a function that is not `pub`, of a module or an
/// inherent impl, outside test support, that returns a `Result` and builds
/// an `Err`, whose every observed edge is a direct call from one function
/// of production code other than its own, and whose name, which the run
/// declares once and no macro argument writes, production code writes
/// only at those calls while test code writes it too. The caller returns
/// no `Result`, so it handles the error rather than passing it on, and is
/// not the stem of the function's name, as `start` is of `start_with`, a
/// seam that lets tests pass their own parts. The tests reach the error
/// path that the one caller may already rule out.
fn guarded_error_path(
    run: &AnalysisRun<'_>,
    scope: &Scope,
    candidates: &[Candidate<'_>],
    out: &mut Vec<Lead>,
) {
    let support: BTreeSet<ItemId> = (candidates.iter().filter(|c| c.support))
        .map(|c| c.item)
        .collect();
    for c in candidates {
        let t = c.item.target;
        let function = match c.kind {
            Kind::Function => true,
            Kind::Method | Kind::Associated => matches!(
                run.member(t, c.item.item).map(|m| &m.kind),
                Some(MemberKind::Fn(_))
            ),
            _ => false,
        };
        let open = function && c.vis != Vis::Public && !c.support;
        let unique = run.census.declared(&c.name) == 1 && !in_macro(run, &c.name);
        if !open || !unique || scope.unseen(run, c) || !returns_result(run, c.item) {
            continue;
        }
        let builds =
            (run.body_of(c.item)).is_some_and(|b| run.krates[t].bodies[b].facts.counts.errs > 0);
        if !builds {
            continue;
        }
        let edges: Vec<(PathKind, usize, BodyId)> = (run.edges_to(c.item))
            .map(|edge| (edge.kind, edge.target, edge.body))
            .collect();
        let bodies: BTreeSet<(usize, BodyId)> = edges.iter().map(|&(_, t, b)| (t, b)).collect();
        let [(ct, cb)] = bodies.into_iter().collect::<Vec<_>>()[..] else {
            continue;
        };
        let Some(caller) = run.body_items[ct][cb].map(|item| ItemId { target: ct, item }) else {
            continue;
        };
        let direct = (edges.iter()).all(|(kind, ..)| *kind == PathKind::Call(Role::Call));
        let count = run.census.count(&c.name);
        let produced = hits_in(&count, Context::Production).saturating_sub(count.declared);
        let tested = hits_in(&count, Context::Test);
        let sole = !support.contains(&caller) && produced == edges.len();
        let stem = run
            .path(caller)
            .last()
            .map(|name| format!("{}_", strip_raw(name)));
        let seam = stem.is_some_and(|stem| c.name.starts_with(&stem));
        if !direct || caller == c.item || !sole || tested == 0 || seam {
            continue;
        }
        if returns_result(run, caller) {
            continue;
        }
        let evidence = format!(
            "`{}` returns a `Result` and builds `Err`; its one observed production caller is `{}`, and test code writes `{}` {tested} times, so the caller may already rule the error out",
            c.name,
            run.path(caller).join("::"),
            c.name
        );
        let base = limits(run, c.item, &["generated-code", "macros"]);
        let lead = Lead::new("guarded-error-path", Mode::Lexical, evidence);
        out.push(lead.item(run, c.item).limits(&base));
    }
}

/// `one-observed-alias-use`: a type alias whose name the scanned sources
/// write once beyond the run's declarations, where a consuming use other
/// than a re-export resolves to it.
fn one_observed_alias_use(run: &AnalysisRun<'_>, c: &Candidate<'_>, out: &mut Vec<Lead>) {
    let uses: Vec<_> = (consumers(run, c.item).into_iter())
        .filter(|u| u.kind != UseKind::Reexport)
        .collect();
    let Some(first) = uses.first() else {
        return;
    };
    if run.census.count(&c.name).uses() != 1 {
        return;
    }
    let source = &run.census.sources[first.source];
    let place = match (source.target, first.module) {
        (Some(t), Some(m)) => format!("`{}`", module_name(run, t, m)),
        _ => source.rel.clone(),
    };
    let evidence = format!(
        "one hit of `{}` beyond its declaration, a use in {place}",
        c.name
    );
    let base = limits(run, c.item, &["macros"]);
    let lead = Lead::new("one-observed-alias-use", Mode::Resolved, evidence);
    out.push(lead.item(run, c.item).limits(&base));
}
