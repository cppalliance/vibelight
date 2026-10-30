//! The census detectors of reach: an item away from its users, a library
//! item that no other target of the run names, and a `pub(crate)` item
//! that only its own module names. No compiler warns about a visibility
//! wider than the uses need, or about an item away from its users.

use crate::analysis::AnalysisRun;
use crate::analysis_uses::{Use, UseKind};
use crate::census::{Context, SourceId};
use crate::leads::{Lead, Mode};
use crate::leads_census_place::defined_away;
use crate::leads_census_scope::{Candidate, Kind, Scope, consumers, in_macro, module_name};
use crate::model::Vis;
use crate::root::TargetKind;

/// Finds the group's leads on placement and visibility.
pub(crate) fn find(
    run: &AnalysisRun<'_>,
    scope: &Scope,
    candidates: &[Candidate<'_>],
    out: &mut Vec<Lead>,
) {
    for c in candidates {
        if c.support || scope.unseen(run, c) {
            continue;
        }
        defined_away(run, c, out);
        no_observed_external_use(run, c, out);
        no_observed_outside_module_use(run, c, out);
    }
}

/// `no-observed-external-use`: a declaration or module value on the
/// surface of a library whose package does not allow publishing, so that
/// every consumer is a crate of its repository, without `doc(hidden)`,
/// whose consuming uses are all in its own target's tree and whose name
/// no other target, package test, bench, example, or doc test writes. A
/// declaration that a signature names stays out, since a public signature
/// may expose it to users who never write its name.
fn no_observed_external_use(run: &AnalysisRun<'_>, c: &Candidate<'_>, out: &mut Vec<Lead>) {
    let t = c.item.target;
    let info = &run.targets[t];
    let listed = matches!(c.kind, Kind::Decl(_) | Kind::Function | Kind::Value);
    let published = run.packages[info.package].facts.publish_allowed;
    if !listed || info.kind != TargetKind::Lib || published || !run.on_surface(c.item) {
        return;
    }
    if c.meta.attrs.iter().any(|attr| attr == "doc(hidden)") {
        return;
    }
    let inside = |source: SourceId| {
        let source = &run.census.sources[source];
        source.target == Some(t) && source.context != Context::Doctest
    };
    let uses = consumers(run, c.item);
    if uses.is_empty() || !uses.iter().all(|u| inside(u.source)) {
        return;
    }
    let signed = uses.iter().any(|u| u.kind == UseKind::Signature);
    let named = (run.census.token_hits(&c.name).iter()).any(|hit| !inside(hit.source));
    if (signed && matches!(c.kind, Kind::Decl(_))) || named || in_macro(run, &c.name) {
        return;
    }
    let evidence = format!(
        "{} observed uses, all inside `{}`; no other target of the run writes `{}`",
        uses.len(),
        info.crate_name,
        c.name
    );
    let lead = Lead::new("no-observed-external-use", Mode::Lexical, evidence);
    out.push(
        lead.item(run, c.item)
            .limits(&["external-consumers", "macros"]),
    );
}

/// `no-observed-outside-module-use`: an item written `pub(crate)`,
/// `pub(super)`, or `pub(in path)` whose name the run declares once and
/// whose every hit is in its own file, a file that holds only its module,
/// outside macros. A declaration that a signature names stays out, since a
/// wider item's signature may expose it.
fn no_observed_outside_module_use(run: &AnalysisRun<'_>, c: &Candidate<'_>, out: &mut Vec<Lead>) {
    let t = c.item.target;
    let listed = !matches!(c.kind, Kind::Variant | Kind::TraitItem);
    let unique = run.census.declared(&c.name) == 1;
    if !listed || c.vis != Vis::Internal || !unique || in_macro(run, &c.name) {
        return;
    }
    let Some(loc) = run.loc(c.item) else {
        return;
    };
    let krate = &run.krates[t];
    let alone = matches!(run.scopes[t][loc.file].as_slice(), [(_, m)] if *m == c.module);
    if !alone || krate.modules[c.module].file != loc.file {
        return;
    }
    let source = run.census.tree_source(t, loc.file);
    let hits = run.census.token_hits(&c.name);
    if run.census.count(&c.name).uses() == 0 || hits.iter().any(|hit| hit.source != source) {
        return;
    }
    let uses = consumers(run, c.item);
    let outside =
        |u: &&Use| run.census.sources[u.source].target != Some(t) || u.module != Some(c.module);
    let signed = uses.iter().any(|u| u.kind == UseKind::Signature);
    if uses.iter().any(outside) || (signed && matches!(c.kind, Kind::Decl(_))) {
        return;
    }
    let evidence = format!(
        "`{}`, but every hit of `{}` is in `{}`",
        c.meta.vis,
        c.name,
        module_name(run, t, c.module)
    );
    let lead = Lead::new("no-observed-outside-module-use", Mode::Lexical, evidence);
    out.push(lead.item(run, c.item).limits(&["generated-code", "macros"]));
}
