//! The census detectors of reach: an item away from its users, a library
//! item that no other target of the run names, and an item whose observed
//! uses need less reach than it has. No compiler warns about a visibility
//! wider than the uses need, or about an item away from its users.

use crate::access::{Access, covers, encloses, join, reexported, spell, written};
use crate::analysis::AnalysisRun;
use crate::analysis_uses::UseKind;
use crate::census::{Context, SourceId};
use crate::md::leads::census::place::defined_away;
use crate::md::leads::census::scope::{Candidate, Kind, Scope, consumers, in_macro};
use crate::md::leads::{Lead, Mode};
use crate::model::{ItemKind, Vis};
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
        visibility_slack(run, c, out);
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

/// `visibility-slack`: a declaration, function, constant, static, or
/// associated function or constant whose observed uses need less reach
/// than it has. The need is the lowest module that holds the item's own
/// module and every use, widened to the reach of each re-export of it.
/// Every use must be in its own target's tree, and every hit of its name,
/// which the run declares once, in a file that the need holds whole. A
/// library's surface item, a `pub` item of a library, which
/// `pub-off-surface` names, and a declaration that a signature names stay
/// out. The lead is lexical when only qualified paths of the census widen
/// the need.
fn visibility_slack(run: &AnalysisRun<'_>, c: &Candidate<'_>, out: &mut Vec<Lead>) {
    let t = c.item.target;
    let krate = &run.krates[t];
    let listed = matches!(
        c.kind,
        Kind::Decl(_) | Kind::Function | Kind::Value | Kind::Associated
    );
    let lib = run.targets[t].kind == TargetKind::Lib;
    let unique = run.census.declared(&c.name) == 1;
    let open = run.on_surface(c.item) || (lib && c.vis == Vis::Public);
    if !listed || !unique || open || in_macro(run, &c.name) {
        return;
    }
    let Some(reach) = run.access_of(c.item) else {
        return;
    };
    let uses = consumers(run, c.item);
    let signed = uses.iter().any(|u| u.kind == UseKind::Signature);
    if uses.is_empty() || (signed && matches!(c.kind, Kind::Decl(_))) {
        return;
    }
    let mut need = Access::Within(c.module);
    let mut resolved = need;
    for u in &uses {
        let source = &run.census.sources[u.source];
        if source.target != Some(t) || source.context == Context::Doctest {
            return;
        }
        let at = Access::Within(u.module.unwrap_or(0));
        need = join(krate, need, at);
        if !matches!(u.kind, UseKind::Qualified(_)) {
            resolved = join(krate, resolved, at);
        }
    }
    for (m, module) in krate.modules.iter().enumerate() {
        for item in &module.items {
            if let ItemKind::Reexport { text, .. } = &item.kind {
                if reexported(krate, m, text) == Some(c.item.item) {
                    need = join(krate, need, written(krate, item.vis, m));
                }
            }
        }
    }
    let Access::Within(holder) = need else {
        return;
    };
    if need == reach || !covers(krate, reach, need) {
        return;
    }
    for hit in run.census.token_hits(&c.name) {
        let source = &run.census.sources[hit.source];
        let tree = source.target == Some(t) && source.context != Context::Doctest;
        let Some(f) = source.file.filter(|_| tree) else {
            return;
        };
        if !(run.scopes[t][f].iter()).all(|&(_, m)| encloses(krate, holder, m)) {
            return;
        }
    }
    let mode = if covers(krate, resolved, need) {
        Mode::Resolved
    } else {
        Mode::Lexical
    };
    let reaches = spell(krate, reach, c.module);
    let written = if reaches == c.meta.vis {
        format!("`{reaches}`")
    } else {
        format!("`{}` reaches `{reaches}`", c.meta.vis)
    };
    let evidence = format!(
        "{written}, but every observed use is inside what `{}` reaches",
        spell(krate, need, c.module)
    );
    let lead = Lead::new("visibility-slack", mode, evidence).item(run, c.item);
    out.push(lead.limits(&["generated-code", "macros", "method-calls"]));
}
