//! The detectors of function bodies: bodies that only forward a call or a
//! field, bodies of one shape, `Result` functions without an error path of
//! their own, parameters that are only passed on, arguments that every
//! caller passes alike, literals that several bodies repeat, and docs far
//! longer than the code they describe. This file holds the group's
//! registry and the detectors that read calls; its siblings hold the rest.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::analysis_paths::PathKind;
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, FnKind, Func};
use crate::leads_types::Ty;
use crate::leads_words::words;
use crate::model::Vis;
use crate::model_body::{BodyFacts, BodyId, CallFact, Role, Single};
use crate::{leads_body_clone, leads_body_literals};

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "forwarder",
    "body-clone",
    "no-direct-error-path",
    "tramp-param",
    "constant-arg",
    "repeated-literal",
    "repeated-number",
    "doc-outweighs-body",
];

/// The ids whose leads rest on what the run does not observe.
pub(crate) const ABSENT: &[&str] = &["no-direct-error-path", "constant-arg"];

/// The fewest observed calls whose literal arguments `constant-arg`
/// compares.
const CONSTANT_ARG_CALLS: usize = 2;

/// The macros that build an error.
const ERROR_MACROS: [&str; 5] = ["anyhow", "bail", "ensure", "eyre", "format_err"];

/// The methods that build an error or carry one through.
const ERROR_METHODS: [&str; 7] = [
    "and_then",
    "context",
    "ok_or",
    "ok_or_else",
    "or_else",
    "transpose",
    "with_context",
];

/// The functions of a run by item.
type Funcs<'c, 'a> = BTreeMap<ItemId, &'c Func<'a>>;

/// A chain of `tramp-param`: the function a parameter goes to, the name it
/// takes there, and the function that one passes it on to.
type Chain = (ItemId, String, ItemId);

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let catalog = Catalog::new(run);
    let funcs: Funcs<'_, '_> = (catalog.funcs.iter()).map(|f| (f.item, f)).collect();
    let forwarders = forwarder(run, &catalog, &funcs, out);
    tramp_param(run, &catalog, &funcs, &forwarders, out);
    no_direct_error_path(run, &catalog, &funcs, out);
    constant_arg(run, &catalog, out);
    leads_body_clone::find(run, out);
    leads_body_literals::find(run, out);
}

/// Returns an item's body and its facts.
fn body<'a>(run: &AnalysisRun<'a>, item: ItemId) -> Option<(BodyId, &'a BodyFacts)> {
    let b = run.body_of(item)?;
    Some((b, &run.krates[item.target].bodies[b].facts))
}

/// Returns the function of the run that call `fact` of body `b` of target
/// `t` reaches.
fn callee<'c, 'a>(
    run: &AnalysisRun<'_>,
    funcs: &Funcs<'c, 'a>,
    t: usize,
    b: BodyId,
    fact: usize,
) -> Option<&'c Func<'a>> {
    let edge = run.edges_from(t, b).get(fact)?;
    let call = matches!(edge.kind, PathKind::Call(_)) && edge.fact == fact;
    funcs.get(&edge.item.filter(|_| call)?).copied()
}

/// Tells whether a function is behind no `cfg`. A function behind one
/// often picks one platform's code.
fn plain(run: &AnalysisRun<'_>, func: &Func<'_>) -> bool {
    run.meta(func.item).is_some_and(|meta| meta.cfgs.is_empty())
}

/// Tells whether a function name is `of` or starts with `of` and `_`,
/// such as `parse_with` for `parse`.
fn variant_of(name: &str, of: &str) -> bool {
    name.strip_prefix(of)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('_'))
}

/// Returns an item's path below its crate's root, joined by `::`.
fn shown(run: &AnalysisRun<'_>, item: ItemId) -> String {
    run.path(item).join("::")
}

/// Returns the function of the run that a body's one call reaches, when
/// that call passes each of the body's parameters on unchanged, once, and
/// nothing else, and the body names no parameter elsewhere; a `self` that
/// the body never names is no parameter here.
fn pure_call<'c, 'a>(
    run: &AnalysisRun<'_>,
    funcs: &Funcs<'c, 'a>,
    item: ItemId,
    b: BodyId,
    facts: &BodyFacts,
) -> Option<&'c Func<'a>> {
    let [call] = facts.calls.as_slice() else {
        return None;
    };
    let passed = |name: &str| {
        (call.passed_params.iter())
            .filter(|(_, n)| n == name)
            .count()
    };
    let params: Vec<_> = (facts.params.iter())
        .filter(|p| p.name != "self" || p.uses > 0)
        .collect();
    let each_once = (params.iter()).all(|p| p.uses == 1 && passed(&p.name) == 1);
    let only = call.args == call.passed_params.len() && call.args == params.len();
    if call.role != Role::Call || params.is_empty() || !each_once || !only {
        return None;
    }
    callee(run, funcs, item.target, b, 0).filter(|f| f.item != item)
}

/// `forwarder`: a free or inherent function whose body is one call to a
/// function of the run that passes each parameter on unchanged, once, and
/// nothing else; or a private method whose body only reads a private field
/// of its own type through `self`, which each of its callers can read. A
/// function behind a `cfg`, which picks one platform's code, is left out,
/// and so is an access seam: a function more visible than its callee,
/// which opens the callee to wider callers. Returns the functions it
/// finds.
fn forwarder(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    funcs: &Funcs<'_, '_>,
    out: &mut Vec<Lead>,
) -> BTreeSet<ItemId> {
    let mut found = BTreeSet::new();
    for func in (catalog.funcs.iter()).filter(|f| f.is_own() && plain(run, f)) {
        let Some((b, facts)) = body(run, func.item) else {
            continue;
        };
        let evidence = match &facts.single {
            Some(Single::Call(_)) => match pure_call(run, funcs, func.item, b, facts) {
                Some(to) if func.vis < to.vis => {
                    found.insert(func.item);
                    continue;
                }
                Some(to) => format!(
                    "one call to `{}` that passes every parameter on unchanged",
                    shown(run, to.item)
                ),
                None => continue,
            },
            Some(Single::Field(name)) if private_read(catalog, func, name) => {
                format!("a private method that only reads the private field `{name}`")
            }
            _ => continue,
        };
        found.insert(func.item);
        let lead = Lead::new("forwarder", Mode::DirectSyntax, evidence);
        out.push(lead.item(run, func.item));
    }
    found
}

/// Tells whether a function is a private method of no parameter besides
/// `self` that reaches only its type's private field `name`.
fn private_read(catalog: &Catalog<'_>, func: &Func<'_>, name: &str) -> bool {
    let (FnKind::Inherent, Some(d), [only]) = (func.kind, func.decl, func.self_fields) else {
        return false;
    };
    let own = ItemId::decl(func.item.target, d);
    let field = (catalog.fields_of(own, None)).find(|f| f.named && f.name == name);
    only == name
        && func.vis == Vis::Private
        && func.has_receiver()
        && func.args.is_empty()
        && field.is_some_and(|f| f.vis == Vis::Private)
}

/// Returns the function of the run that a body passes its parameter `name`
/// on to, with the name of the parameter it becomes there, when the body
/// names it only once: as an argument of that call, bare or behind `&`.
fn passed_once<'c, 'a>(
    run: &AnalysisRun<'_>,
    funcs: &Funcs<'c, 'a>,
    item: ItemId,
    b: BodyId,
    facts: &BodyFacts,
    name: &str,
) -> Option<(&'c Func<'a>, String)> {
    let uses = facts.params.iter().find(|p| p.name == name)?.uses;
    let mut sites = (facts.calls.iter().enumerate()).flat_map(|(k, call)| {
        let at = call.passed_params.iter().filter(|(_, n)| n == name);
        at.map(move |(position, _)| (k, *position))
    });
    let (Some((k, position)), None, 1) = (sites.next(), sites.next(), uses) else {
        return None;
    };
    let to = callee(run, funcs, item.target, b, k).filter(|f| f.item != item)?;
    let index = position.checked_sub(usize::from(to.has_receiver()))?;
    let becomes = to.args.get(index)?.name.clone()?;
    Some((to, becomes))
}

/// `tramp-param`: a parameter, other than `self`, of a free or inherent
/// function that is no forwarder and is behind no `cfg`, which the body
/// names only to pass it on unchanged to one function of the run that in
/// turn only passes it on to one function of the run. A callee named after
/// the function that calls it, such as `parse` or `parse_with` for
/// `parse`, is a variant of it, which `wrapper-name-family` covers, and a
/// chain back to the function is a recursion that carries its context;
/// both are left out. One lead per chain names every function whose
/// parameter takes it.
fn tramp_param(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    funcs: &Funcs<'_, '_>,
    forwarders: &BTreeSet<ItemId>,
    out: &mut Vec<Lead>,
) {
    let candidate = |f: &&Func<'_>| f.is_own() && plain(run, f) && !forwarders.contains(&f.item);
    let mut chains: BTreeMap<Chain, Vec<(ItemId, &str)>> = BTreeMap::new();
    for func in catalog.funcs.iter().filter(candidate) {
        let Some((b, facts)) = body(run, func.item) else {
            continue;
        };
        for param in facts.params.iter().filter(|p| p.name != "self") {
            let Some((to, becomes)) = passed_once(run, funcs, func.item, b, facts, &param.name)
            else {
                continue;
            };
            if variant_of(&to.name, &func.name) {
                continue;
            }
            let Some((next_b, next_facts)) = body(run, to.item) else {
                continue;
            };
            let Some((last, _)) = passed_once(run, funcs, to.item, next_b, next_facts, &becomes)
            else {
                continue;
            };
            if last.item == func.item || variant_of(&last.name, &to.name) {
                continue;
            }
            let chain = chains.entry((to.item, becomes, last.item)).or_default();
            chain.push((func.item, param.name.as_str()));
        }
    }
    for ((to, _, last), takers) in chains {
        let names: BTreeSet<String> = takers.iter().map(|(_, name)| format!("`{name}`")).collect();
        let verb = if names.len() == 1 { "goes" } else { "go" };
        let evidence = format!(
            "{} only {verb} to `{}`, which only passes it on to `{}`",
            names.into_iter().collect::<Vec<_>>().join(", "),
            shown(run, to),
            shown(run, last)
        );
        let mut lead = Lead::new("tramp-param", Mode::DirectSyntax, evidence);
        for &(item, _) in &takers {
            lead = lead.item(run, item);
        }
        out.push(lead);
    }
}

/// Tells whether a name's last word is `err` or `error`, such as
/// `ParseError` or `map_err`.
fn names_error(name: &str) -> bool {
    matches!(
        words(name).last().map(String::as_str),
        Some("err" | "error")
    )
}

/// Tells whether a body writes a direct error path: `?`, `Err`, an `Err`
/// pattern, a path, macro, macro argument, or method named as an error, a
/// macro that builds one, or a method that builds one or carries one
/// through.
fn has_error_path(facts: &BodyFacts) -> bool {
    let paths =
        (facts.calls.iter().map(|c| &c.path)).chain(facts.constructs.iter().map(|p| &p.path));
    let last = |path: &str| path.rsplit("::").next().unwrap_or(path).to_owned();
    facts.counts.tries > 0
        || facts.counts.errs > 0
        || paths.flatten().any(|segment| names_error(segment))
        || (facts.patterns.iter()).any(|p| p.path.last().is_some_and(|l| l == "Err"))
        || (facts.macros.iter().map(|m| last(m)))
            .any(|m| ERROR_MACROS.contains(&m.as_str()) || names_error(&m))
        || facts.macro_idents.iter().any(|ident| names_error(ident))
        || (facts.methods.iter()).any(|m| ERROR_METHODS.contains(&m.as_str()) || names_error(m))
}

/// `no-direct-error-path`: a `pub` free or inherent function that returns
/// a `Result` and whose body calls `Ok` but writes no direct error path
/// and calls no function of the run that returns a `Result`. A method or
/// a function outside the run whose `Result` the body returns may still
/// fail, and so may code that macros write.
fn no_direct_error_path(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    funcs: &Funcs<'_, '_>,
    out: &mut Vec<Lead>,
) {
    let fallible = |func: &Func<'_>| func.ret.as_ref().is_some_and(Ty::is_result);
    for func in (catalog.funcs.iter()).filter(|f| f.is_own() && f.vis == Vis::Public && fallible(f))
    {
        let Some((b, facts)) = body(run, func.item) else {
            continue;
        };
        let ok = (facts.calls.iter())
            .any(|c| c.role == Role::Call && c.path.last().is_some_and(|l| l == "Ok"));
        let calls_fallible = run.edges_from(func.item.target, b).iter().any(|edge| {
            let reached = edge.item.and_then(|item| funcs.get(&item));
            matches!(edge.kind, PathKind::Call(_)) && reached.is_some_and(|f| fallible(f))
        });
        if !ok || calls_fallible || has_error_path(facts) {
            continue;
        }
        let ret = func.ret.as_ref().map(Ty::shown).unwrap_or_default();
        let evidence = format!(
            "returns `{ret}` and calls `Ok`, with no `?`, `Err`, or error constructor; a returned callee may still fail"
        );
        let lead = Lead::new("no-direct-error-path", Mode::Resolved, evidence);
        out.push(
            lead.item(run, func.item)
                .limits(&["dynamic-dispatch", "macros"]),
        );
    }
}

/// Counts the body paths of the run by last segment, with whether one of
/// them reaches nothing.
fn paths_by_last<'a>(run: &AnalysisRun<'a>) -> BTreeMap<&'a str, (usize, bool)> {
    let mut out: BTreeMap<&'a str, (usize, bool)> = BTreeMap::new();
    for edge in &run.edges {
        let facts = &run.krates[edge.target].bodies[edge.body].facts;
        let path = match edge.kind {
            PathKind::Call(_) => facts.calls.get(edge.fact).map(|c| &c.path),
            PathKind::Construct => facts.constructs.get(edge.fact).map(|p| &p.path),
            PathKind::Pattern => facts.patterns.get(edge.fact).map(|p| &p.path),
        };
        if let Some(last) = path.and_then(|path| path.last()) {
            let entry = out.entry(last.as_str()).or_default();
            entry.0 += 1;
            entry.1 |= edge.item.is_none();
        }
    }
    out
}

/// `constant-arg`: a private free or inherent function that bodies of the
/// run call [`CONSTANT_ARG_CALLS`] or more times, always through a path,
/// each call passing the same literal in one position. Every hit of its
/// name in the scanned sources is a declaration or a body path that
/// resolves, no body passes it as a value, and no attribute, re-export, or
/// test, bench, example, or doc test names it; code that macros write goes
/// unseen.
fn constant_arg(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let by_last = paths_by_last(run);
    for func in (catalog.funcs.iter()).filter(|f| f.is_own() && f.vis == Vis::Private) {
        let edges: Vec<_> = run.edges_to(func.item).collect();
        let called = edges.iter().all(|e| e.kind == PathKind::Call(Role::Call));
        if edges.len() < CONSTANT_ARG_CALLS || !called {
            continue;
        }
        let accounted = by_last.get(func.name.as_str()).copied().unwrap_or_default();
        let hits = run.name_count(func.item).map(|count| count.uses());
        let named_elsewhere = run.uses_of(func.item).iter().any(|u| {
            matches!(u.kind, UseKind::Attr | UseKind::Reexport) || u.context != Context::Production
        });
        if hits != Some(accounted.0) || accounted.1 || named_elsewhere {
            continue;
        }
        let calls: Vec<&CallFact> = (edges.iter())
            .filter_map(|e| run.krates[e.target].bodies[e.body].facts.calls.get(e.fact))
            .collect();
        let found = same_literals(func, &calls);
        if calls.len() == edges.len() && !found.is_empty() {
            let evidence = format!(
                "all {} observed calls pass {}",
                calls.len(),
                found.join(", ")
            );
            let lead = Lead::new("constant-arg", Mode::Resolved, evidence);
            out.push(lead.item(run, func.item).limits(&["macros"]));
        }
    }
}

/// Returns, for each position where every call passes one literal, that
/// literal and the parameter it fills, such as `` `4` as `width` ``.
fn same_literals(func: &Func<'_>, calls: &[&CallFact]) -> Vec<String> {
    let Some(first) = calls.first() else {
        return Vec::new();
    };
    if calls.iter().any(|c| c.args != first.args) {
        return Vec::new();
    }
    let receiver = usize::from(func.has_receiver());
    let mut found = Vec::new();
    for (position, literal) in &first.literal_args {
        let same = |c: &&CallFact| {
            c.literal_args
                .iter()
                .any(|l| l.0 == *position && l.1 == *literal)
        };
        if !calls.iter().all(same) {
            continue;
        }
        let param = match position.checked_sub(receiver) {
            Some(index) => (func.args.get(index))
                .and_then(|arg| arg.name.clone())
                .unwrap_or_else(|| format!("argument {}", position + 1)),
            None => "self".to_owned(),
        };
        found.push(format!("`{literal}` as `{param}`"));
    }
    found
}
