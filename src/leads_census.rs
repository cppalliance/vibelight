//! The detectors of the source-observed census: what the run's resolved
//! uses, qualified paths, and name hits do not show. They report only what
//! the compilers leave quiet: an item on a library's surface, an item that
//! an `allow` or `expect` of `dead_code` reaches, a visibility wider than
//! the observed uses, and uses that only tests make. Every lead rests on
//! what the run does not see, so each names its blind spots. This file
//! holds the group's registry and the detectors of name hits; its siblings
//! hold the scope they share and the rest.

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_uses::{Use, UseKind};
use crate::census::{Context, NameCount};
use crate::leads::{Lead, Mode};
use crate::leads_census_scope::{
    Candidate, Kind, Scope, candidates, consumers, edge_path, generated_part, hits_in, in_macro,
    limits,
};
use crate::model::Shape;
use crate::{leads_census_deps, leads_census_paths, leads_census_reach, leads_census_variants};

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "no-observed-name-use",
    "no-observed-path-use",
    "test-only-observed-use",
    "no-observed-construction",
    "no-observed-pattern",
    "no-observed-field-read",
    "defined-away-from-observed-users",
    "no-observed-external-use",
    "no-observed-outside-module-use",
    "no-observed-reexport-use",
    "one-observed-caller",
    "guarded-error-path",
    "one-observed-alias-use",
    "no-observed-dependency-use",
];

/// The ids whose leads rest on what the run does not observe: all of them.
pub(crate) const ABSENT: &[&str] = IDS;

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let scope = Scope::new(run);
    let candidates = candidates(run);
    for c in &candidates {
        if scope.skipped(run, c) && !scope.unseen(run, c) {
            let count = run.census.count(&c.name);
            no_observed_name_use(run, c, &count, out);
            no_observed_path_use(run, c, &count, out);
            test_only_observed_use(run, c, &count, out);
        }
    }
    leads_census_variants::find(run, &scope, &candidates, out);
    leads_census_reach::find(run, &scope, &candidates, out);
    leads_census_paths::find(run, &scope, &candidates, out);
    leads_census_deps::find(run, &scope, out);
}

/// Starts a lead on a candidate with the limits it names.
fn lead(
    run: &AnalysisRun<'_>,
    id: &'static str,
    mode: Mode,
    c: &Candidate<'_>,
    evidence: String,
    base: &[&'static str],
) -> Lead {
    let limits = limits(run, c.item, base);
    Lead::new(id, mode, evidence)
        .item(run, c.item)
        .limits(&limits)
}

/// `no-observed-name-use`: a quiet candidate whose name no scanned source
/// writes beyond the run's declarations of it. A field or variant of a
/// declaration that is not plain is left out, since a derive or `serde`
/// reads and builds it unseen.
fn no_observed_name_use(
    run: &AnalysisRun<'_>,
    c: &Candidate<'_>,
    count: &NameCount,
    out: &mut Vec<Lead>,
) {
    if count.uses() > 0 || generated_part(run, c) {
        return;
    }
    let evidence = format!(
        "no hit of `{}` in the scanned sources beyond its {} declaration{}",
        c.name,
        count.declared,
        if count.declared == 1 { "" } else { "s" }
    );
    let base = ["generated-code", "macros"];
    out.push(lead(
        run,
        "no-observed-name-use",
        Mode::Lexical,
        c,
        evidence,
        &base,
    ));
}

/// Tells whether a body path that the run leaves unresolved may reach an
/// associated candidate: a path whose prefix names the candidate's type
/// and whose last segment is its name, such as `Self::new` in a trait impl.
fn loose_member(run: &AnalysisRun<'_>, c: &Candidate<'_>) -> bool {
    let Some(d) = c.decl else {
        return false;
    };
    let owner = ItemId::decl(c.item.target, d);
    run.uses_of(owner).iter().any(|u| {
        let UseKind::Qualifier(e) = u.kind else {
            return false;
        };
        run.edges[e].item.is_none() && edge_path(run, e).last() == Some(&c.name)
    })
}

/// `no-observed-path-use`: a quiet declaration, module value, or
/// associated item whose name production code writes outside macros, yet
/// no consuming use, resolved or a unique qualified path, reaches it. A
/// declaration counts as reached when one of its members or variants is.
fn no_observed_path_use(
    run: &AnalysisRun<'_>,
    c: &Candidate<'_>,
    count: &NameCount,
    out: &mut Vec<Lead>,
) {
    let pathed = matches!(
        c.kind,
        Kind::Decl(_) | Kind::Function | Kind::Value | Kind::Associated
    );
    let production = (count.by_place.keys()).all(|(_, _, c)| *c == Context::Production);
    if !pathed || count.uses() == 0 || !production || in_macro(run, &c.name) {
        return;
    }
    if !consumers(run, c.item).is_empty() || loose_member(run, c) {
        return;
    }
    if let (Kind::Decl(_), Some(d)) = (c.kind, c.decl) {
        let t = c.item.target;
        let mut parts: Vec<ItemRef> = run.members(t, d).map(|(item, _)| item).collect();
        if let Shape::Enum(variants) = &run.krates[t].decls[d].shape {
            parts.extend((0..variants.len()).map(|k| ItemRef::Variant(d, k)));
        }
        let at = |item| ItemId { target: t, item };
        if parts
            .into_iter()
            .any(|item| !consumers(run, at(item)).is_empty())
        {
            return;
        }
    }
    let evidence = format!(
        "{} production hits of `{}` outside macros, none a resolved or unique qualified path to it",
        count.uses(),
        c.name
    );
    out.push(lead(
        run,
        "no-observed-path-use",
        Mode::Resolved,
        c,
        evidence,
        &["macros"],
    ));
}

/// `test-only-observed-use`: a quiet candidate outside test support whose
/// every consuming use is a qualified path in test, bench, example, or
/// doc test code, and whose name production code writes nowhere beyond
/// its declarations, so that no bare token hides a production use. A
/// field or variant of a declaration that is not plain is left out, since
/// `serde` or a derive may build it in production unseen.
fn test_only_observed_use(
    run: &AnalysisRun<'_>,
    c: &Candidate<'_>,
    count: &NameCount,
    out: &mut Vec<Lead>,
) {
    let produced = hits_in(count, Context::Production) > count.declared;
    if c.support || produced || generated_part(run, c) {
        return;
    }
    let uses = consumers(run, c.item);
    let tested = |u: &&Use| {
        matches!(
            u.context,
            Context::Test | Context::Bench | Context::Example | Context::Doctest
        )
    };
    if uses.is_empty() || !uses.iter().all(tested) {
        return;
    }
    let mut contexts: Vec<&str> = (uses.iter())
        .map(|u| match u.context {
            Context::Bench => "bench",
            Context::Example => "example",
            Context::Doctest => "doc test",
            _ => "test",
        })
        .collect();
    contexts.sort_unstable();
    contexts.dedup();
    let evidence = format!(
        "{} qualified paths, all in {} code, and no production hit beyond its declarations",
        uses.len(),
        contexts.join(" and ")
    );
    out.push(lead(
        run,
        "test-only-observed-use",
        Mode::Lexical,
        c,
        evidence,
        &["macros"],
    ));
}
