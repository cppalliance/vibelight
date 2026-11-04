//! The detectors of placement and visibility: one path for a module and a
//! function, or one name for several kinds of type, a type's inherent
//! impls across many files, a module that holds one item or only impls, a
//! `pub` item no path from its library's root reaches, a `pub mod` on the
//! surface, `non_exhaustive` off every surface, modules that use each
//! other's items, and a restricted visibility spelled narrower than it is.

use std::collections::{BTreeMap, BTreeSet};

use crate::access::spelled;
use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode, Subject};
use crate::leads_census_scope::module_name;
use crate::leads_words::words;
use crate::macros;
use crate::model::{
    Crate, DeclKind, FileId, ItemKind, Loc, MemberKind, ModId, Shape, ValueRef, Vis,
};
use crate::order_graph::strongly_connected;
use crate::root::TargetKind;
use crate::test_support::is_test_support;
use crate::typerefs::strip_raw;

/// The fewest files of a type's inherent impls for `impl-spread`.
const SPREAD_FROM: usize = 3;

pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    name_collision(run, out);
    for (t, krate) in run.krates.iter().enumerate() {
        impl_spread(run, t, krate, out);
        single_item_module(run, t, krate, out);
        if run.targets[t].kind == TargetKind::Lib {
            pub_off_surface(run, t, krate, out);
        }
        hidden_non_exhaustive(run, t, krate, out);
        module_cycle(run, t, krate, out);
        visibility_spelling(run, t, krate, out);
    }
}

/// `impl-spread`: a type whose inherent impls are in [`SPREAD_FROM`] or
/// more files.
fn impl_spread(run: &AnalysisRun<'_>, t: usize, krate: &Crate, out: &mut Vec<Lead>) {
    for (d, impls) in run.impls_of[t].iter().enumerate() {
        let inherent: Vec<usize> = (impls.iter().copied())
            .filter(|&i| krate.impls[i].trait_.is_none())
            .collect();
        let files: BTreeSet<FileId> = (inherent.iter())
            .map(|&i| krate.real_loc(krate.impls[i].loc).file)
            .collect();
        if files.len() < SPREAD_FROM {
            continue;
        }
        let rels: Vec<&str> = files.iter().map(|&f| krate.files[f].rel.as_str()).collect();
        let evidence = format!(
            "inherent impls in {} files: {}",
            files.len(),
            rels.join(", ")
        );
        let at = ItemId::decl(t, d);
        let mut lead = Lead::new("impl-spread", Mode::DirectSyntax, evidence).item(run, at);
        for &i in &inherent {
            lead = lead.at(t, krate.impls[i].loc);
        }
        out.push(lead);
    }
}

/// `single-item-module`: a module other than the root, outside test
/// support, that holds one item that is no re-export, or no item and only
/// impls. Its items are its declarations, functions, constants, statics,
/// extern items, macro definitions, and child modules. A module named
/// after its one item, such as `idle_reader` for `IdleReader`, is the
/// common file per type and is left out.
fn single_item_module(run: &AnalysisRun<'_>, t: usize, krate: &Crate, out: &mut Vec<Lead>) {
    let mut held: Vec<Vec<(String, String)>> = vec![Vec::new(); krate.modules.len()];
    let mut impls = vec![0usize; krate.modules.len()];
    for decl in &krate.decls {
        held[decl.module].push((decl.name.clone(), decl.name.clone()));
    }
    for (m, module) in krate.modules.iter().enumerate() {
        for item in &module.items {
            let defines = match &item.kind {
                ItemKind::Mark(text) => macros::definition_name(text).is_some(),
                kind => crate::analysis_items::is_value(kind),
            };
            if defines {
                held[m].push((item.name.clone(), item.name.clone()));
            }
        }
        if let (Some(p), Some(name)) = (krate.parent_module(m), module.path.last()) {
            held[p].push((format!("mod {name}"), name.clone()));
        }
    }
    for i in 0..krate.impls.len() {
        impls[krate.impls[i].module] += 1;
    }
    for (m, module) in krate.modules.iter().enumerate().skip(1) {
        let own: BTreeSet<String> = module
            .path
            .last()
            .map(|name| words(name))
            .unwrap_or_default()
            .into_iter()
            .collect();
        let evidence = match (held[m].as_slice(), impls[m]) {
            ([(_, name)], _) if own.is_subset(&words(name).into_iter().collect()) => continue,
            ([(only, _)], _) => format!("holds only `{only}`"),
            ([], n) if n > 0 => format!("holds only {n} impls"),
            _ => continue,
        };
        if module.test_support {
            continue;
        }
        let at = ItemId {
            target: t,
            item: ItemRef::Module(m),
        };
        out.push(Lead::new("single-item-module", Mode::DirectSyntax, evidence).item(run, at));
    }
}

/// `pub-off-surface`: a `pub` module, declaration, or module value of a
/// library, outside test support, that no public path from the crate root
/// reaches. A trait named `Sealed` is the common sealed-trait marker and is
/// left out; a re-export that a macro writes goes unseen.
fn pub_off_surface(run: &AnalysisRun<'_>, t: usize, krate: &Crate, out: &mut Vec<Lead>) {
    let mut items = Vec::new();
    for (m, module) in krate.modules.iter().enumerate().skip(1) {
        if module.vis == Vis::Public && !module.test_support {
            items.push(ItemRef::Module(m));
        }
    }
    for (d, decl) in krate.decls.iter().enumerate() {
        let sealed = decl.kind == DeclKind::Trait && strip_raw(&decl.name) == "Sealed";
        if decl.vis == Vis::Public && !decl.test_support && !sealed {
            items.push(ItemRef::Decl(d));
        }
    }
    for (m, module) in krate.modules.iter().enumerate() {
        for (k, item) in module.items.iter().enumerate() {
            let support = module.test_support || is_test_support(&item.meta.cfgs);
            if item.vis == Vis::Public && crate::analysis_items::is_value(&item.kind) && !support {
                items.push(ItemRef::Value(ValueRef { module: m, item: k }));
            }
        }
    }
    for item in items {
        let at = ItemId { target: t, item };
        if !run.on_surface(at) {
            let evidence = "`pub`, but no public path from the crate root reaches it";
            let lead = Lead::new("pub-off-surface", Mode::Resolved, evidence).item(run, at);
            out.push(lead.limits(&["macros"]));
        } else if matches!(item, ItemRef::Module(_)) {
            let evidence = "`pub mod` on the surface: its `pub` items reach users by its path";
            out.push(Lead::new("pub-mod-surface", Mode::Resolved, evidence).item(run, at));
        }
    }
}

/// `hidden-non-exhaustive`: a `non_exhaustive` type or variant that is on
/// no surface, which a binary never has. Whether the package may be
/// published is shown, never taken as proof that it is.
fn hidden_non_exhaustive(run: &AnalysisRun<'_>, t: usize, krate: &Crate, out: &mut Vec<Lead>) {
    let marked = |attrs: &[String]| attrs.iter().any(|attr| attr == "non_exhaustive");
    let mut items = Vec::new();
    for (d, decl) in krate.decls.iter().enumerate() {
        if marked(&decl.meta.attrs) {
            items.push(ItemRef::Decl(d));
        }
        if let Shape::Enum(variants) = &decl.shape {
            let variants = variants
                .iter()
                .enumerate()
                .filter(|(_, v)| marked(&v.meta.attrs));
            items.extend(variants.map(|(k, _)| ItemRef::Variant(d, k)));
        }
    }
    let package = &run.packages[run.targets[t].package];
    for item in items {
        let at = ItemId { target: t, item };
        if !run.on_surface(at) {
            let evidence = format!(
                "`#[non_exhaustive]` off every surface; publish_allowed is {}",
                package.facts.publish_allowed
            );
            let lead = Lead::new("hidden-non-exhaustive", Mode::Resolved, evidence).item(run, at);
            out.push(lead.limits(&["macros"]));
        }
    }
}

/// `module-cycle`: modules of a target, none inside another, that use each
/// other's items, directly or around a longer cycle, as the module graph
/// of the observed uses shows. The lead is lexical when an edge of the
/// cycle rests only on qualified paths of the census.
fn module_cycle(run: &AnalysisRun<'_>, t: usize, krate: &Crate, out: &mut Vec<Lead>) {
    let graph = &run.graph[t];
    let n = krate.modules.len();
    let edges: Vec<BTreeSet<ModId>> = (graph.iter())
        .map(|to| to.keys().copied().collect())
        .collect();
    let nodes: Vec<ModId> = (0..n).collect();
    for mut cycle in strongly_connected(n, &edges, &nodes) {
        if cycle.len() < 2 {
            continue;
        }
        cycle.sort_unstable();
        let name = |m: ModId| krate.modules[m].path.join("::");
        let mut uses = Vec::new();
        let mut lexical = false;
        for &x in &cycle {
            for (&y, edge) in graph[x].iter().filter(|(y, _)| cycle.contains(y)) {
                uses.push(format!("`{}` uses `{}`", name(x), name(y)));
                lexical |= !edge.resolved;
            }
        }
        let mode = if lexical {
            Mode::Lexical
        } else {
            Mode::Resolved
        };
        let mut lead = Lead::new("module-cycle", mode, uses.join("; "));
        for m in cycle {
            lead = lead.item(
                run,
                ItemId {
                    target: t,
                    item: ItemRef::Module(m),
                },
            );
        }
        out.push(lead);
    }
}

/// `visibility-spelling`: a restricted visibility whose spelling says less
/// than it reaches. A `pub(super)` in a module directly under the crate
/// root reaches the whole crate, which `pub(crate)` says, and a
/// `pub(in path)` that names the declaring module, its parent, or the root
/// says what `pub(self)`, `pub(super)`, or `pub(crate)` says. A `use`
/// spells one visibility for all the names it binds, so one lead names
/// them all.
fn visibility_spelling(run: &AnalysisRun<'_>, t: usize, krate: &Crate, out: &mut Vec<Lead>) {
    const ID: &str = "visibility-spelling";
    let mut nodes: Vec<(ItemRef, Vis, ModId, String)> = Vec::new();
    let mut uses: BTreeMap<(ModId, Loc, &str), (Vis, Vec<&str>)> = BTreeMap::new();
    for (d, decl) in krate.decls.iter().enumerate() {
        nodes.push((
            ItemRef::Decl(d),
            decl.vis,
            decl.module,
            decl.meta.vis.clone(),
        ));
        for (k, member) in decl.members.iter().enumerate() {
            if matches!(member.kind, MemberKind::Field(_)) {
                let spelled = member.meta.vis.clone();
                nodes.push((ItemRef::Own(d, k), member.vis, decl.module, spelled));
            }
        }
    }
    for (m, module) in krate.modules.iter().enumerate() {
        if let (Some(parent), Vis::Super | Vis::In(_)) = (module.parent, module.vis) {
            let spelled = spelled(krate, module.vis);
            nodes.push((ItemRef::Module(m), module.vis, parent, spelled));
        }
        for (k, item) in module.items.iter().enumerate() {
            if matches!(item.kind, ItemKind::Reexport { .. }) {
                let names = uses.entry((m, item.loc, item.meta.vis.as_str()));
                names.or_insert((item.vis, Vec::new())).1.push(&item.name);
            } else {
                let at = ItemRef::Value(ValueRef { module: m, item: k });
                nodes.push((at, item.vis, m, item.meta.vis.clone()));
            }
        }
    }
    for (i, imp) in krate.impls.iter().enumerate() {
        for (k, member) in imp.members.iter().enumerate() {
            let spelled = member.meta.vis.clone();
            nodes.push((ItemRef::Inherent(i, k), member.vis, imp.module, spelled));
        }
    }
    for (item, vis, from, spelled) in nodes {
        if let Some(evidence) = spelling_evidence(run, t, krate, vis, from, &spelled) {
            let at = ItemId { target: t, item };
            out.push(Lead::new(ID, Mode::DirectSyntax, evidence).item(run, at));
        }
    }
    for ((m, loc, spelled), (vis, names)) in uses {
        let Some(evidence) = spelling_evidence(run, t, krate, vis, m, spelled) else {
            continue;
        };
        let mut lead = Lead::new(ID, Mode::DirectSyntax, evidence);
        for name in names {
            let mut path = krate.modules[m].path.clone();
            path.push(name.to_owned());
            lead.subjects.push(Subject {
                target: t,
                module: m,
                path,
            });
        }
        out.push(lead.at(t, loc));
    }
}

/// Returns the evidence of a visibility `vis`, spelled `spelled` in module
/// `from`, whose spelling says less than it reaches; `None` when the
/// spelling says what it reaches.
fn spelling_evidence(
    run: &AnalysisRun<'_>,
    t: usize,
    krate: &Crate,
    vis: Vis,
    from: ModId,
    spelled: &str,
) -> Option<String> {
    let parent = krate.parent_module(from);
    let canonical = match vis {
        Vis::Super if parent == Some(0) => "pub(crate)",
        Vis::In(m) if m == from => "pub(self)",
        Vis::In(m) if Some(m) == parent => "pub(super)",
        Vis::In(0) => "pub(crate)",
        _ => return None,
    };
    Some(format!(
        "`{spelled}` in `{}` reaches what `{canonical}` says",
        module_name(run, t, from)
    ))
}

/// Returns the functions that module `p` of a crate holds or re-exports
/// under `name`.
fn functions_named(krate: &Crate, p: ModId, name: &str) -> Vec<ValueRef> {
    let mut out = Vec::new();
    for (k, item) in krate.modules[p].items.iter().enumerate() {
        if strip_raw(&item.name) != name {
            continue;
        }
        let at = match &item.kind {
            ItemKind::Fn(_) => Some(ValueRef { module: p, item: k }),
            ItemKind::Reexport { text, .. } => {
                text.refs().next().and_then(|r| krate.refs[r].value())
            }
            _ => None,
        };
        let function = at.filter(|at| {
            matches!(
                krate.modules[at.module].items[at.item].kind,
                ItemKind::Fn(_)
            )
        });
        out.extend(function);
    }
    out
}

/// `name-collision`: one path that names both a module and a function, the
/// function beside the module in its parent or re-exported there, or one
/// name in one target for two or more kinds of type or trait.
fn name_collision(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    const ID: &str = "name-collision";
    for (t, krate) in run.krates.iter().enumerate() {
        let at = |item| ItemId { target: t, item };
        for (m, module) in krate.modules.iter().enumerate() {
            let (Some(name), Some(p)) = (module.path.last(), krate.parent_module(m)) else {
                continue;
            };
            let name = strip_raw(name);
            let functions = functions_named(krate, p, &name);
            if functions.is_empty() {
                continue;
            }
            let parent = &krate.modules[p].path;
            let place = if parent.is_empty() {
                "the crate root".to_owned()
            } else {
                format!("`{}`", parent.join("::"))
            };
            let evidence = format!("`{name}` names a module and a function in {place}");
            let mut lead =
                Lead::new(ID, Mode::DirectSyntax, evidence).item(run, at(ItemRef::Module(m)));
            for function in functions {
                lead = lead.item(run, at(ItemRef::Value(function)));
            }
            out.push(lead);
        }
        let mut types: BTreeMap<String, Vec<(&str, usize)>> = BTreeMap::new();
        for (d, decl) in krate.decls.iter().enumerate() {
            let kind = match decl.kind {
                DeclKind::Struct => "struct",
                DeclKind::Enum => "enum",
                DeclKind::Union => "union",
                DeclKind::Trait => "trait",
                DeclKind::TraitAlias => "trait alias",
                DeclKind::Alias => "type alias",
            };
            types
                .entry(strip_raw(&decl.name))
                .or_default()
                .push((kind, d));
        }
        for (name, decls) in types {
            let kinds: BTreeSet<&str> = decls.iter().map(|(kind, _)| *kind).collect();
            if kinds.len() < 2 {
                continue;
            }
            let shown: Vec<&str> = kinds.into_iter().collect();
            let evidence = format!("`{name}` names a {}", shown.join(" and a "));
            let mut lead = Lead::new(ID, Mode::DirectSyntax, evidence);
            for (_, d) in decls {
                lead = lead.item(run, at(ItemRef::Decl(d)));
            }
            out.push(lead);
        }
    }
}
