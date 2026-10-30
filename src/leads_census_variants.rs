//! The census detectors of variants and fields: a variant of a quiet enum
//! that no observed path builds, one that no observed pattern matches, and
//! a field of a quiet struct that no observed access reads. Only plain
//! declarations are read, since a derive, an attribute macro, or `serde`
//! writes code the run does not see.

use std::collections::BTreeSet;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::census::Form;
use crate::leads::{Lead, Mode};
use crate::leads_census_scope::{
    Candidate, Kind, Scope, edge_path, elsewhere, in_macro, limits, plain,
};
use crate::model::{DeclId, DeclKind};
use crate::model_body::Role;

/// The last segments of the body paths that reach nothing: those in
/// expression position or struct literals, and those in patterns. A
/// variant one of them names may be built or matched through a glob import
/// or a path the rules leave unresolved.
struct Loose {
    built: BTreeSet<String>,
    matched: BTreeSet<String>,
}

impl Loose {
    fn new(run: &AnalysisRun<'_>) -> Loose {
        let mut loose = Loose {
            built: BTreeSet::new(),
            matched: BTreeSet::new(),
        };
        for (e, edge) in run.edges.iter().enumerate() {
            let Some(last) = edge_path(run, e).last().filter(|_| edge.item.is_none()) else {
                continue;
            };
            let names = match edge.kind {
                PathKind::Pattern => &mut loose.matched,
                PathKind::Call(_) | PathKind::Construct => &mut loose.built,
            };
            names.insert(last.clone());
        }
        loose
    }
}

/// Finds the group's leads on variants and fields.
pub(crate) fn find(
    run: &AnalysisRun<'_>,
    scope: &Scope,
    candidates: &[Candidate<'_>],
    out: &mut Vec<Lead>,
) {
    let loose = Loose::new(run);
    for c in candidates {
        let t = c.item.target;
        let (Kind::Variant | Kind::Field, Some(d)) = (c.kind, c.decl) else {
            continue;
        };
        let quiet = scope.skipped(run, c) && !scope.unseen(run, c);
        if !quiet || c.support || !plain(run, t, d) {
            continue;
        }
        if elsewhere(&run.census.count(&c.name)) || in_macro(run, &c.name) {
            continue;
        }
        if c.kind == Kind::Variant {
            variant(run, c, d, &loose, out);
        } else {
            field(run, c, d, out);
        }
    }
}

/// Returns a declaration's name as written.
fn owner_name<'a>(run: &AnalysisRun<'a>, t: usize, d: DeclId) -> &'a str {
    &run.krates[t].decls[d].name
}

/// `no-observed-construction` and `no-observed-pattern`: a variant that no
/// body path in expression position or struct literal reaches, and a
/// variant that body paths build but that no pattern and no comparison
/// operand reaches. A path the rules leave unresolved that ends in the
/// variant's name counts as both, and the leads of a variant whose name
/// test code or macro arguments write are left out, since the census
/// cannot tell a build from a match there.
fn variant(
    run: &AnalysisRun<'_>,
    c: &Candidate<'_>,
    d: DeclId,
    loose: &Loose,
    out: &mut Vec<Lead>,
) {
    let t = c.item.target;
    let edges: Vec<PathKind> = run.edges_to(c.item).map(|edge| edge.kind).collect();
    let built = (edges.iter())
        .filter(|kind| matches!(kind, PathKind::Call(_) | PathKind::Construct))
        .count();
    let matched = (edges.iter())
        .filter(|kind| {
            matches!(
                kind,
                PathKind::Pattern | PathKind::Call(Role::OtherReference)
            )
        })
        .count();
    let shown = format!("{}::{}", owner_name(run, t, d), c.name);
    let base = limits(run, c.item, &["generated-code", "macros"]);
    if built == 0 && !loose.built.contains(&c.name) {
        let evidence = format!(
            "no observed path builds `{shown}`; {matched} patterns or comparisons match it"
        );
        let lead = Lead::new("no-observed-construction", Mode::Resolved, evidence);
        out.push(lead.item(run, c.item).limits(&base));
    }
    if built > 0 && matched == 0 && !loose.matched.contains(&c.name) {
        let evidence = format!(
            "{built} observed paths build `{shown}`, and no pattern or comparison matches it"
        );
        let lead = Lead::new("no-observed-pattern", Mode::Resolved, evidence);
        out.push(lead.item(run, c.item).limits(&base));
    }
}

/// `no-observed-field-read`: a named field of a struct without `repr`
/// whose name no scanned source writes after a dot, `.name`, and whose
/// struct no body pattern destructures. Struct literals set the field, so
/// its other hits are writes.
fn field(run: &AnalysisRun<'_>, c: &Candidate<'_>, d: DeclId, out: &mut Vec<Lead>) {
    let t = c.item.target;
    let decl = &run.krates[t].decls[d];
    let laid_out = decl.meta.attrs.iter().any(|attr| attr.starts_with("repr"));
    if decl.kind != DeclKind::Struct || laid_out {
        return;
    }
    let dotted = (run.census.token_hits(&c.name).iter()).any(|hit| hit.form == Form::Field);
    let owner = ItemId {
        target: t,
        item: ItemRef::Decl(d),
    };
    let kinds: Vec<PathKind> = run.edges_to(owner).map(|edge| edge.kind).collect();
    if dotted || kinds.contains(&PathKind::Pattern) {
        return;
    }
    let built = kinds
        .iter()
        .filter(|&&kind| kind == PathKind::Construct)
        .count();
    let evidence = format!(
        "{built} struct literals of `{}` set it, and no `.{}` access or pattern reads it",
        decl.name, c.name
    );
    let base = limits(run, c.item, &["generated-code", "macros"]);
    let lead = Lead::new("no-observed-field-read", Mode::Lexical, evidence);
    out.push(lead.item(run, c.item).limits(&base));
}
