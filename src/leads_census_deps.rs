//! The census detector of manifests: a dependency entry whose crate name
//! no source that can use the entry writes. A normal entry counts only
//! production code, a build entry only the build script, and a dev entry
//! only tests, benches, examples, and doc tests, so a hit in one context
//! never hides an unused entry of another.

use crate::analysis::AnalysisRun;
use crate::census::Context;
use crate::leads::{Lead, Mode, Subject};
use crate::leads_census_scope::Scope;
use crate::manifest_facts::{DepEntry, DepKind};
use crate::root::TargetKind;

/// The suffixes of a package that links a native library, which a crate
/// may depend on only to link it.
const NATIVE_SUFFIXES: [&str; 2] = ["-sys", "_sys"];

/// Returns the contexts whose code can use a dependency entry.
fn contexts(kind: DepKind) -> &'static [Context] {
    match kind {
        DepKind::Normal => &[Context::Production],
        DepKind::Build => &[Context::Build],
        DepKind::Dev => &[
            Context::Test,
            Context::Bench,
            Context::Example,
            Context::Doctest,
        ],
    }
}

/// Returns the table a dependency entry is in, as a manifest writes it.
fn table(dep: &DepEntry) -> String {
    let kind = match dep.kind {
        DepKind::Normal => "dependencies",
        DepKind::Dev => "dev-dependencies",
        DepKind::Build => "build-dependencies",
    };
    match &dep.target {
        Some(target) => format!("target.'{target}'.{kind}"),
        None => kind.to_owned(),
    }
}

/// `no-observed-dependency-use`: a dependency entry of a package whose
/// code the census reads in full, so of none that uses `include!`, when no
/// source of the package in a context that can use the entry writes the
/// crate name the entry gives, with `-` read as `_` or dropped. A native
/// `-sys` package, the package itself, which a dev entry names to turn on
/// features, and a dev entry beside a normal or build entry of one name,
/// which may only add features, are left out.
pub(crate) fn find(run: &AnalysisRun<'_>, scope: &Scope, out: &mut Vec<Lead>) {
    for (p, package) in run.packages.iter().enumerate() {
        let lib = (package.targets.iter()).find(|&&t| run.targets[t].kind == TargetKind::Lib);
        let Some(&home) = lib.or_else(|| package.targets.first()) else {
            continue;
        };
        if scope.unread(p) {
            continue;
        }
        let facts = &package.facts;
        let deps = &facts.deps;
        for dep in deps {
            let native = NATIVE_SUFFIXES.iter().any(|s| dep.name.ends_with(s));
            let named = dep.package.as_ref().unwrap_or(&dep.name);
            let itself = facts.name.as_ref() == Some(named);
            let augments = dep.kind == DepKind::Dev
                && (deps.iter()).any(|d| d.name == dep.name && d.kind != DepKind::Dev);
            if native || itself || augments {
                continue;
            }
            let written = [dep.name.replace('-', "_"), dep.name.replace('-', "")];
            let wanted = contexts(dep.kind);
            let used = written.iter().any(|name| {
                (run.census.token_hits(name).iter()).any(|hit| {
                    wanted.contains(&hit.context) && run.census.sources[hit.source].package == p
                })
            });
            if used {
                continue;
            }
            let place = match dep.kind {
                DepKind::Normal => "production",
                DepKind::Build => "build script",
                DepKind::Dev => "test, bench, example, or doc test",
            };
            let evidence = format!(
                "`{}` in `[{}]`, but no {place} source of the package writes `{}`",
                dep.name,
                table(dep),
                written[0]
            );
            let mut lead = Lead::new("no-observed-dependency-use", Mode::Lexical, evidence);
            lead.subjects.push(Subject {
                target: home,
                module: 0,
                path: vec![dep.name.clone()],
            });
            let mut limits = vec!["generated-code", "macros"];
            if dep.target.is_some() {
                limits.push("cfg-selection");
            }
            out.push(lead.limits(&limits));
        }
    }
}
