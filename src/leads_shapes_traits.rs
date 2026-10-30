//! The detectors of what a type's traits and observed uses show: a
//! hand-written `Deserialize`, a trait of the run with one observed impl, a
//! one-field wrapper that one module uses, and the error types of a target
//! that share variant names.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::analysis_uses::{Use, UseKind};
use crate::census::Context;
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, impls_of_trait, is_deserialize, is_wire, trait_of};
use crate::leads_words::words;
use crate::model::{DeclId, DeclKind, MemberKind, ModId, Shape, VariantFields};
use crate::typerefs::strip_raw;

/// The most unit variants of an enum whose hand-written `Deserialize` is a
/// `manual-serde` lead.
const TINY_ENUM: usize = 4;

pub(crate) fn find(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    manual_serde(run, out);
    one_observed_trait_impl(run, out);
    thin_wrapper(run, catalog, out);
    error_family(run, out);
}

/// Returns the declaration that the `deserialize` member of trait impl `i`
/// of target `t` reads through `Other::deserialize`, other than `own`.
fn wire_twin(run: &AnalysisRun<'_>, t: usize, i: usize, own: ItemId) -> Option<ItemId> {
    let krate = &run.krates[t];
    let members = krate.impls[i].trait_members.iter().enumerate();
    let mut fns =
        members.filter(|(_, m)| m.name == "deserialize" && matches!(m.kind, MemberKind::Fn(_)));
    fns.find_map(|(k, _)| {
        let item = ItemId {
            target: t,
            item: ItemRef::TraitImpl(i, k),
        };
        let body = run.body_of(item)?;
        let calls = &krate.bodies[body].facts.calls;
        run.edges_from(t, body).iter().find_map(|edge| {
            let named = matches!(edge.kind, PathKind::Call(_))
                && (calls.get(edge.fact))
                    .is_some_and(|call| call.path.last().is_some_and(|last| last == "deserialize"));
            edge.prefix.filter(|&prefix| named && prefix != own)
        })
    })
}

/// `manual-serde`: a hand-written `Deserialize` for a type of the run that
/// reads another declaration of the run through its own `deserialize`, a
/// wire twin, or for an enum of at most [`TINY_ENUM`] unit variants.
fn manual_serde(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    const ID: &str = "manual-serde";
    for (t, krate) in run.krates.iter().enumerate() {
        for (i, imp) in krate.impls.iter().enumerate() {
            let Some(d) = run.attached[t][i] else {
                continue;
            };
            if !trait_of(run, t, i).is_some_and(|name| is_deserialize(&name)) {
                continue;
            }
            let at = ItemId::decl(t, d);
            let decl = &krate.decls[d];
            let lead = if let Some(twin) = wire_twin(run, t, i, at) {
                let other = run.name(twin).unwrap_or_default();
                let evidence = format!(
                    "a hand-written Deserialize for `{}` reads `{other}::deserialize`",
                    decl.name
                );
                Lead::new(ID, Mode::Resolved, evidence)
                    .item(run, at)
                    .item(run, twin)
            } else {
                let Shape::Enum(variants) = &decl.shape else {
                    continue;
                };
                let unit = variants
                    .iter()
                    .all(|v| matches!(v.fields, VariantFields::Unit));
                if !unit || variants.is_empty() || variants.len() > TINY_ENUM {
                    continue;
                }
                let evidence = format!(
                    "a hand-written Deserialize for an enum of {} unit variants",
                    variants.len()
                );
                Lead::new(ID, Mode::DirectSyntax, evidence).item(run, at)
            };
            out.push(lead.at(t, imp.loc));
        }
    }
}

/// `one-observed-trait-impl`: a trait of the run that one impl in the run
/// implements for one of the run's own types. A blanket impl, an extension
/// trait for an outside type, and a trait that tests, benches, or examples
/// name are left out; impls by macros and outside the run go unseen.
fn one_observed_trait_impl(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let mut impls: BTreeMap<ItemId, Vec<(usize, usize)>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (i, imp) in krate.impls.iter().enumerate() {
            let path = imp.trait_.as_ref().and_then(|base| base.path_ref);
            if let Some(item) = path.and_then(|r| run.item_of_ref(t, r)) {
                impls.entry(item).or_default().push((t, i));
            }
        }
    }
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let at = ItemId::decl(t, d);
            let found = impls.get(&at).map_or(&[][..], Vec::as_slice);
            let (DeclKind::Trait, [(it, i)]) = (decl.kind, found) else {
                continue;
            };
            let (it, i) = (*it, *i);
            let Some(owner) = run.attached[it][i] else {
                continue;
            };
            let imp = &run.krates[it].impls[i];
            let blanket = imp.self_text.refs().next().is_some_and(|r| {
                let reference = &run.krates[it].refs[r];
                reference.path.len() == 1 && reference.generics.contains(&reference.path[0])
            });
            let tested = (run.census.token_hits(&strip_raw(&decl.name)).iter()).any(|hit| {
                matches!(
                    hit.context,
                    Context::Test | Context::Bench | Context::Example
                )
            });
            if blanket || tested {
                continue;
            }
            let evidence = format!("one impl in the run: `{}`", imp.header);
            let lead = Lead::new("one-observed-trait-impl", Mode::Resolved, evidence);
            let owner = ItemId::decl(it, owner);
            let lead = lead.item(run, at).item(run, owner).at(it, imp.loc);
            out.push(lead.limits(&["external-impls", "macros"]));
        }
    }
}

/// Tells whether declaration `d` of target `t` has a method in any impl.
fn has_methods(run: &AnalysisRun<'_>, t: usize, d: DeclId) -> bool {
    run.impls_of[t][d].iter().any(|&i| {
        let imp = &run.krates[t].impls[i];
        (imp.members.iter().chain(&imp.trait_members)).any(|m| matches!(m.kind, MemberKind::Fn(_)))
    })
}

/// `thin-wrapper-one-observed-user`: a struct of one field with no
/// methods, off every surface and with no `serde` derive or attribute,
/// whose observed uses outside docs, re-exports, and derives are all in
/// one module. The evidence is lexical when every use is a qualified path
/// of the census. Macros and outside users go unseen.
fn thin_wrapper(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        for (d, decl) in krate.decls.iter().enumerate() {
            let at = ItemId::decl(t, d);
            let fields: Vec<_> = catalog.fields_of(at, None).collect();
            let ([field], DeclKind::Struct, false) =
                (fields.as_slice(), decl.kind, decl.test_support)
            else {
                continue;
            };
            if is_wire(run, at) || has_methods(run, t, d) || run.on_surface(at) {
                continue;
            }
            let uses: Vec<&Use> = (run.uses_of(at).iter())
                .filter(|u| !matches!(u.kind, UseKind::Doc | UseKind::Reexport | UseKind::Derive))
                .collect();
            let places: BTreeSet<(Option<usize>, Option<ModId>)> = (uses.iter())
                .map(|u| (run.census.sources[u.source].target, u.module))
                .collect();
            let [(Some(holder), Some(m))] = places.into_iter().collect::<Vec<_>>()[..] else {
                continue;
            };
            let path = &run.krates[holder].modules[m].path;
            let module = if path.is_empty() {
                run.targets[holder].crate_name.clone()
            } else {
                path.join("::")
            };
            let evidence = format!(
                "one field `{}`, no methods, {} observed uses, all in `{module}`",
                field.ty.shown(),
                uses.len()
            );
            let lexical = uses.iter().all(|u| matches!(u.kind, UseKind::Qualified(_)));
            let mode = if lexical {
                Mode::Lexical
            } else {
                Mode::Resolved
            };
            let lead = Lead::new("thin-wrapper-one-observed-user", mode, evidence);
            out.push(lead.item(run, at).limits(&["external-consumers", "macros"]));
        }
    }
}

/// `error-family`: the error types of a target, the structs and enums
/// whose name ends in `Error` or that implement `std::error::Error`, when
/// two or more of them have a variant of one name.
fn error_family(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    for (t, krate) in run.krates.iter().enumerate() {
        let errors: Vec<DeclId> = (0..krate.decls.len())
            .filter(|&d| {
                let decl = &krate.decls[d];
                let typed = matches!(decl.kind, DeclKind::Struct | DeclKind::Enum);
                let named = words(&decl.name).last().is_some_and(|w| w == "error");
                typed
                    && (named
                        || impls_of_trait(run, t, d, "std::error::Error")
                            .next()
                            .is_some())
            })
            .collect();
        let mut shared: BTreeMap<String, Vec<DeclId>> = BTreeMap::new();
        for &d in &errors {
            if let Shape::Enum(variants) = &krate.decls[d].shape {
                for variant in variants {
                    let users = shared.entry(strip_raw(&variant.name)).or_default();
                    if users.last() != Some(&d) {
                        users.push(d);
                    }
                }
            }
        }
        shared.retain(|_, users| users.len() > 1);
        if shared.is_empty() {
            continue;
        }
        let name = |d: &DeclId| format!("`{}`", krate.decls[*d].name);
        let found: Vec<String> = (shared.iter())
            .map(|(variant, users)| {
                let users: Vec<String> = users.iter().map(name).collect();
                format!("`{variant}` in {}", users.join(", "))
            })
            .collect();
        let evidence = format!("{} error types; {}", errors.len(), found.join("; "));
        let mut lead = Lead::new("error-family", Mode::DirectSyntax, evidence);
        let members: BTreeSet<DeclId> = shared.into_values().flatten().collect();
        for d in members {
            lead = lead.item(run, ItemId::decl(t, d));
        }
        out.push(lead);
    }
}
