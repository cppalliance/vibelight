//! The detectors of parameters and returns: parameters that travel together
//! through several functions, tuple returns, `&mut` primitive parameters,
//! several parameters of one primitive type, and parameters named as
//! unused. A trait impl's signature is its trait's, so none is read here.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, FnKind, Func, escapes};
use crate::leads_types::{Tok, Ty, leaf};

/// The most parameters of one function that `param-clump` combines.
const CLUMP_ARGS: usize = 8;

/// The fewest functions that one clump of parameters is in.
const CLUMP_FROM: usize = 3;

/// A clump of parameters: its target and its two or three parameter ids,
/// padded with `u32::MAX`.
type Clump = (usize, [u32; 3]);

/// The fewest parameters of one primitive type for `same-type-params`.
const SAME_TYPE_FROM: usize = 3;

pub(crate) fn find(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    param_clump(run, catalog, out);
    for func in catalog.funcs.iter().filter(|f| f.kind != FnKind::TraitImpl) {
        tuple_return(run, func, out);
        out_param_cursor(run, func, out);
        same_type_params(run, func, out);
        if func.is_own() {
            underscore_param(run, func, out);
        }
    }
}

/// `param-clump`: two or three parameters, each a name and a type, that
/// [`CLUMP_FROM`] or more functions of one target all take, and that half
/// or more of the functions taking each of them take together, so that a
/// context parameter most functions take clumps with nothing. A pair whose
/// functions are exactly a triple's is left to the triple, and, widest and
/// most shared first, a clump whose every function a kept clump already
/// names is left out.
fn param_clump(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    const NONE: u32 = u32::MAX;
    let mut ids: BTreeMap<(usize, String, String), u32> = BTreeMap::new();
    let mut shown: Vec<String> = Vec::new();
    let mut taken: Vec<usize> = Vec::new();
    let mut clumps: BTreeMap<Clump, Vec<usize>> = BTreeMap::new();
    for (f, func) in catalog.funcs.iter().enumerate() {
        if func.kind == FnKind::TraitImpl {
            continue;
        }
        let t = func.item.target;
        let mut keys: Vec<u32> = Vec::new();
        for arg in &func.args {
            let Some(name) = &arg.name else {
                continue;
            };
            let next = u32::try_from(ids.len()).unwrap_or(NONE);
            let id = *ids
                .entry((t, name.clone(), arg.ty.key()))
                .or_insert_with(|| {
                    shown.push(format!("`{name}: {}`", arg.ty.shown()));
                    taken.push(0);
                    next
                });
            keys.push(id);
        }
        keys.sort_unstable();
        keys.dedup();
        for &id in &keys {
            if let Some(count) = usize::try_from(id).ok().and_then(|i| taken.get_mut(i)) {
                *count += 1;
            }
        }
        keys.truncate(CLUMP_ARGS);
        for (i, &a) in keys.iter().enumerate() {
            for (j, &b) in keys.iter().enumerate().skip(i + 1) {
                clumps.entry((t, [a, b, NONE])).or_default().push(f);
                for &c in &keys[j + 1..] {
                    clumps.entry((t, [a, b, c])).or_default().push(f);
                }
            }
        }
    }
    clumps.retain(|&(_, ids), funcs| {
        let together = (ids.iter().filter(|&&id| id != NONE))
            .filter_map(|&id| taken.get(usize::try_from(id).ok()?))
            .all(|&alone| funcs.len() * 2 >= alone);
        funcs.len() >= CLUMP_FROM && together
    });
    let mut covered: BTreeSet<Clump> = BTreeSet::new();
    for (&(t, [a, b, c]), funcs) in &clumps {
        if c == NONE {
            continue;
        }
        for pair in [[a, b, NONE], [a, c, NONE], [b, c, NONE]] {
            if clumps.get(&(t, pair)) == Some(funcs) {
                covered.insert((t, pair));
            }
        }
    }
    let width = |key: &Clump| key.1.iter().filter(|&&id| id != NONE).count();
    let mut kept: Vec<(&Clump, &Vec<usize>)> = clumps
        .iter()
        .filter(|(key, _)| !covered.contains(key))
        .collect();
    kept.sort_by(|a, b| {
        (width(b.0).cmp(&width(a.0)))
            .then(b.1.len().cmp(&a.1.len()))
            .then(a.0.cmp(b.0))
    });
    let mut named: BTreeSet<usize> = BTreeSet::new();
    for (key, funcs) in kept {
        if funcs.iter().all(|f| named.contains(f)) {
            continue;
        }
        named.extend(funcs.iter().copied());
        let params: Vec<&str> = (key.1.iter())
            .filter(|&&id| id != NONE)
            .filter_map(|&id| shown.get(usize::try_from(id).ok()?).map(String::as_str))
            .collect();
        let evidence = format!("{} in {} functions", params.join(", "), funcs.len());
        let mut lead = Lead::new("param-clump", Mode::DirectSyntax, evidence);
        for &f in funcs {
            lead = lead.item(run, catalog.funcs[f].item);
        }
        out.push(lead);
    }
}

/// `tuple-return`: a function that returns a tuple of three or more parts,
/// or of two parts of one type, bare or in a `Result` or an `Option`.
fn tuple_return(run: &AnalysisRun<'_>, func: &Func<'_>, out: &mut Vec<Lead>) {
    let Some(ret) = &func.ret else {
        return;
    };
    let held = match ret.generic() {
        Some((head, args)) if matches!(leaf(head).as_str(), "Option" | "Result") => {
            args.into_iter().next()
        }
        _ => None,
    };
    let Some(parts) = ret.tuple().or_else(|| held.as_ref().and_then(Ty::tuple)) else {
        return;
    };
    let evidence = match parts.as_slice() {
        [a, b] if a == b => format!("returns two `{}`", a.shown()),
        [_, _] => return,
        _ => {
            let shown: Vec<String> = parts.iter().map(Ty::shown).collect();
            format!(
                "returns a tuple of {}: `({})`",
                parts.len(),
                shown.join(", ")
            )
        }
    };
    out.push(Lead::new("tuple-return", Mode::DirectSyntax, evidence).item(run, func.item));
}

/// Returns a parameter as the evidence shows it.
fn arg_text(name: Option<&str>, ty: &Ty) -> String {
    format!("`{}: {}`", name.unwrap_or("_"), ty.shown())
}

/// `out-param-cursor`: a parameter of type `&mut` and a primitive.
fn out_param_cursor(run: &AnalysisRun<'_>, func: &Func<'_>, out: &mut Vec<Lead>) {
    let cursors: Vec<String> = (func.args.iter())
        .filter(|arg| {
            let mutable = matches!(arg.ty.0.as_slice(), [Tok::Code(amp), Tok::Code(m), ..] if amp == "&" && m == "mut");
            mutable && arg.ty.deref().primitive().is_some()
        })
        .map(|arg| arg_text(arg.name.as_deref(), &arg.ty))
        .collect();
    if !cursors.is_empty() {
        let evidence = format!("`&mut` primitive parameters {}", cursors.join(", "));
        let lead = Lead::new("out-param-cursor", Mode::DirectSyntax, evidence);
        out.push(lead.item(run, func.item));
    }
}

/// `same-type-params`: [`SAME_TYPE_FROM`] or more parameters of one
/// primitive type, references included.
fn same_type_params(run: &AnalysisRun<'_>, func: &Func<'_>, out: &mut Vec<Lead>) {
    let mut by_type: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for arg in &func.args {
        if let Some(primitive) = arg.ty.deref().primitive() {
            let name = arg.name.as_deref().unwrap_or("_");
            by_type.entry(primitive.to_owned()).or_default().push(name);
        }
    }
    for (primitive, names) in by_type {
        if names.len() >= SAME_TYPE_FROM {
            let evidence = format!(
                "{} `{primitive}` parameters: {}",
                names.len(),
                names.join(", ")
            );
            let lead = Lead::new("same-type-params", Mode::DirectSyntax, evidence);
            out.push(lead.item(run, func.item));
        }
    }
}

/// `underscore-param`: a parameter named with a leading underscore, which
/// marks it unused, in a function whose signature no trait sets and that
/// no body passes as a value, such as a handler whose callback sets it.
fn underscore_param(run: &AnalysisRun<'_>, func: &Func<'_>, out: &mut Vec<Lead>) {
    if escapes(run, func.item) {
        return;
    }
    let names: Vec<String> = (func.args.iter())
        .filter_map(|arg| arg.name.as_deref())
        .filter(|name| name.len() > 1 && name.starts_with('_'))
        .map(|name| format!("`{name}`"))
        .collect();
    if !names.is_empty() {
        let evidence = format!("parameters {}", names.join(", "));
        let lead = Lead::new("underscore-param", Mode::DirectSyntax, evidence);
        out.push(lead.item(run, func.item));
    }
}
