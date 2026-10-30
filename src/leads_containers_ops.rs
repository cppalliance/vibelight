//! The detectors of how bodies use collections: the operation chains that
//! start directly at a field of a method's type, through `self`, or at a
//! typed parameter, read against the type declared there. A local
//! variable's provenance is never inferred, so a collection reached
//! through a `let` goes unseen.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_containers::head;
use crate::leads_items::{Catalog, Field, Func};
use crate::leads_types::Ty;
use crate::model_body::{OpChain, Root};

/// The methods that start iterating a sequence.
const ITERATES: [&str; 3] = ["into_iter", "iter", "iter_mut"];

/// The iterator adapters that may sit between [`ITERATES`] and a search.
const ADAPTERS: [&str; 6] = ["cloned", "copied", "enumerate", "filter", "map", "rev"];

/// The iterator methods that search item by item.
const SEARCHES: [&str; 3] = ["any", "find", "position"];

/// The sequence methods that scan every item.
const SCANS: [&str; 2] = ["contains", "retain"];

/// The `Vec` methods that add at the end.
const PUSHES: [&str; 4] = ["append", "extend", "extend_from_slice", "push"];

/// The slice methods that sort in place.
const SORTS: [&str; 7] = [
    "sort",
    "sort_by",
    "sort_by_cached_key",
    "sort_by_key",
    "sort_unstable",
    "sort_unstable_by",
    "sort_unstable_by_key",
];

/// The map methods that start iterating its entries, keys, or values.
const MAP_ITERATES: [&str; 9] = [
    "drain",
    "into_iter",
    "into_keys",
    "into_values",
    "iter",
    "iter_mut",
    "keys",
    "values",
    "values_mut",
];

/// The collection methods besides [`SORTS`] that change what a collection
/// holds or its order, or give mutable access to its items.
const MUTATORS: [&str; 34] = [
    "append",
    "clear",
    "dedup",
    "dedup_by",
    "dedup_by_key",
    "drain",
    "entry",
    "extend",
    "extend_from_slice",
    "first_mut",
    "get_mut",
    "insert",
    "iter_mut",
    "last_mut",
    "pop",
    "pop_back",
    "pop_first",
    "pop_front",
    "pop_last",
    "push",
    "push_back",
    "push_front",
    "remove",
    "remove_entry",
    "resize",
    "retain",
    "retain_mut",
    "reverse",
    "shift_remove",
    "split_off",
    "swap",
    "swap_remove",
    "truncate",
    "values_mut",
];

/// The fewest methods of a type that search one of its fields for a
/// `linear-search` lead outside a loop.
const SEARCHED_FROM: usize = 2;

/// The fewest methods that mutate both fields for `co-mutated`.
const CO_MUTATED_SHARED: usize = 2;

/// The least share, in percent, of the methods that mutate either field
/// that mutate both, for `co-mutated`.
const CO_MUTATED_PERCENT: usize = 80;

/// A body's item, with what its chains may start at.
struct Site<'c, 'a> {
    item: ItemId,
    /// The type that the body is a method of, whose fields `self` reaches.
    decl: Option<ItemId>,
    func: Option<&'c Func<'a>>,
}

/// What a chain starts at: a field, or a parameter by name, with its
/// declared type.
enum Start<'c, 'a> {
    Field(&'c Field<'a>),
    Param(&'c str, &'c Ty),
}

impl Start<'_, '_> {
    fn ty(&self) -> &Ty {
        match self {
            Start::Field(field) => &field.ty,
            Start::Param(_, ty) => ty,
        }
    }

    /// Returns the start as the body writes it, such as `self.items`.
    fn written(&self) -> String {
        match self {
            Start::Field(field) => format!("self.{}", field.name),
            Start::Param(name, _) => (*name).to_owned(),
        }
    }
}

impl<'c, 'a> Site<'c, 'a> {
    /// Returns what a chain's root is, when its type is declared.
    fn start(&self, catalog: &'c Catalog<'a>, root: &'c Root) -> Option<Start<'c, 'a>> {
        match root {
            Root::Field(name) => (catalog.fields_of(self.decl?, None))
                .find(|f| f.name == *name)
                .map(Start::Field),
            Root::Param(name) => {
                let args = &self.func?.args;
                let arg = args.iter().find(|a| a.name.as_deref() == Some(name))?;
                Some(Start::Param(name, &arg.ty))
            }
        }
    }

    /// Builds a lead of the body's item and, for a field, the field.
    fn lead(
        &self,
        run: &AnalysisRun<'_>,
        id: &'static str,
        start: &Start<'_, '_>,
        seen: &[String],
    ) -> Lead {
        let lead = Lead::new(id, Mode::DirectSyntax, seen.join(", ")).item(run, self.item);
        match start {
            Start::Field(field) => field.add(run, lead),
            Start::Param(..) => lead,
        }
    }
}

/// Returns the head of a type without the `&` and `mut` before it, with
/// a slice as `[]`.
fn held(ty: &Ty) -> Option<String> {
    let ty = ty.deref();
    match head(&ty) {
        Some((head, _)) => Some(head),
        None => ty.is_collection().then(|| "[]".to_owned()),
    }
}

/// Shows a chain from its start up to and including method `last`, then
/// `args` as written after it, with ` in a loop` when it sits in one.
fn shown(start: &Start<'_, '_>, chain: &OpChain, last: usize, args: &str) -> String {
    let methods = chain.methods[..=last].join("().");
    let place = if chain.in_loop { " in a loop" } else { "" };
    format!("`{}.{methods}{args}`{place}", start.written())
}

/// Finds the group's leads in operation chains.
pub(crate) fn find(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    let funcs: BTreeMap<ItemId, &Func<'_>> = catalog.funcs.iter().map(|f| (f.item, f)).collect();
    let mut methods: BTreeMap<ItemId, Vec<(ItemId, &[OpChain])>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (b, body) in krate.bodies.iter().enumerate() {
            let Some(item) = run.body_items[t][b] else {
                continue;
            };
            let at = ItemId { target: t, item };
            let decl = match item {
                ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _) => run.attached[t][i],
                _ => None,
            };
            let site = Site {
                item: at,
                decl: decl.map(|d| ItemId::decl(t, d)),
                func: funcs.get(&at).copied(),
            };
            let chains = body.facts.chains.as_slice();
            body_leads(run, catalog, &site, chains, out);
            if let Some(decl) = site.decl {
                methods.entry(decl).or_default().push((at, chains));
            }
        }
    }
    for (&decl, bodies) in &methods {
        searched_fields(run, catalog, decl, bodies, out);
        sort_after_push(run, catalog, decl, bodies, out);
        co_mutated(run, catalog, decl, bodies, out);
    }
}

/// `linear-search` at the `Vec` and slice fields of one type: one lead per
/// field, with every method that searches it, when a search sits in a
/// loop or [`SEARCHED_FROM`] or more methods search it, since a field that
/// many methods look up by scanning may want a map.
fn searched_fields(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    decl: ItemId,
    bodies: &[(ItemId, &[OpChain])],
    out: &mut Vec<Lead>,
) {
    for field in fields_held(catalog, decl, |ty| is_sequence(held(ty).as_deref())) {
        let start = Start::Field(field);
        let mut searches: BTreeMap<ItemId, Vec<String>> = BTreeMap::new();
        let mut looped = false;
        for &(method, chains) in bodies {
            for chain in on_field(chains, &field.name) {
                if let Some(last) = linear_search(chain) {
                    looped |= chain.in_loop;
                    let seen = searches.entry(method).or_default();
                    seen.push(shown(&start, chain, last, ""));
                }
            }
        }
        if searches.is_empty() || (!looped && searches.len() < SEARCHED_FROM) {
            continue;
        }
        let shown: Vec<String> = (searches.iter())
            .map(|(&method, seen)| {
                let name = run.name(method).unwrap_or_default();
                format!("`{name}` with {}", seen.join(", "))
            })
            .collect();
        let mut lead = Lead::new("linear-search", Mode::DirectSyntax, shown.join("; "));
        for &method in searches.keys() {
            lead = lead.item(run, method);
        }
        out.push(field.add(run, lead));
    }
}

/// Finds `front-ops-on-vec` and `sorted-unordered-map` in one body, and
/// `linear-search` at its parameters, one lead of each per root. A
/// parameter's search counts only inside a loop, since a lookup in a short
/// list that a caller passes is common.
fn body_leads(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    site: &Site<'_, '_>,
    chains: &[OpChain],
    out: &mut Vec<Lead>,
) {
    let mut found: BTreeMap<(&'static str, &Root), Vec<String>> = BTreeMap::new();
    for chain in chains {
        let Some(start) = site.start(catalog, &chain.root) else {
            continue;
        };
        let head = held(start.ty());
        let mut note = |id: &'static str, last: usize, args: &str| {
            let seen = found.entry((id, &chain.root)).or_default();
            seen.push(shown(&start, chain, last, args));
        };
        let param = matches!(start, Start::Param(..)) && chain.in_loop;
        if let Some(last) = linear_search(chain).filter(|_| param && is_sequence(head.as_deref())) {
            note("linear-search", last, "");
        }
        if let Some(args) = front_op(chain).filter(|_| head.as_deref() == Some("Vec")) {
            note("front-ops-on-vec", 0, args);
        }
        let hashed = head.as_deref().is_some_and(|h| h.ends_with("HashMap"));
        if let Some(last) = sorted_iteration(chain).filter(|_| hashed) {
            note("sorted-unordered-map", last, "");
        }
    }
    for ((id, root), seen) in found {
        if let Some(start) = site.start(catalog, root) {
            out.push(site.lead(run, id, &start, &seen));
        }
    }
}

/// Tells whether a head that [`held`] returns is a `Vec` or a slice.
fn is_sequence(head: Option<&str>) -> bool {
    matches!(head, Some("Vec" | "[]"))
}

/// `linear-search`: a chain that scans a sequence with `contains` or
/// `retain`, or iterates it and searches with `any`, `find`, or
/// `position`. Returns the index of the searching method.
fn linear_search(chain: &OpChain) -> Option<usize> {
    let methods = &chain.methods;
    let first = methods.first()?.as_str();
    if SCANS.contains(&first) {
        return Some(0);
    }
    if !ITERATES.contains(&first) {
        return None;
    }
    let at = 1 + methods[1..]
        .iter()
        .position(|m| !ADAPTERS.contains(&m.as_str()))?;
    SEARCHES.contains(&methods[at].as_str()).then_some(at)
}

/// `front-ops-on-vec`: a chain that calls `remove(0)` or `insert(0, _)`.
/// Returns its arguments as the evidence shows them.
fn front_op(chain: &OpChain) -> Option<&'static str> {
    if !chain.first_literal.as_deref().is_some_and(is_zero) {
        return None;
    }
    match chain.methods.first()?.as_str() {
        "remove" => Some("(0)"),
        "insert" => Some("(0, _)"),
        _ => None,
    }
}

/// Tells whether a literal as written is the number 0, with or without a
/// suffix such as `usize`.
fn is_zero(literal: &str) -> bool {
    let digits = literal.trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '_');
    digits == "0"
}

/// `sorted-unordered-map`: a chain that iterates a map and then sorts what
/// it collects, with a `sorted` method or a sort after `collect`. Returns
/// the index of the sorting method.
fn sorted_iteration(chain: &OpChain) -> Option<usize> {
    let methods = &chain.methods;
    if !MAP_ITERATES.contains(&methods.first()?.as_str()) {
        return None;
    }
    let collected = methods.iter().position(|m| m == "collect");
    methods.iter().enumerate().skip(1).find_map(|(i, m)| {
        let after = collected.is_some_and(|c| c < i) && SORTS.contains(&m.as_str());
        (m.starts_with("sorted") || after).then_some(i)
    })
}

/// Returns the fields of a declaration whose type, without `&` and `mut`,
/// `keep` accepts.
fn fields_held<'c, 'a>(
    catalog: &'c Catalog<'a>,
    decl: ItemId,
    keep: impl Fn(&Ty) -> bool,
) -> Vec<&'c Field<'a>> {
    (catalog.fields_of(decl, None))
        .filter(|f| keep(&f.ty.deref()))
        .collect()
}

/// Tells whether a type is a `Vec`.
fn is_vec(ty: &Ty) -> bool {
    head(ty).is_some_and(|(head, _)| head == "Vec")
}

/// Returns the chains of one body that start at field `name`.
fn on_field<'b>(chains: &'b [OpChain], name: &'b str) -> impl Iterator<Item = &'b OpChain> {
    chains
        .iter()
        .filter(move |c| matches!(&c.root, Root::Field(f) if f == name))
}

/// Returns the first method of a chain when it is one of `set`.
fn first_in(chain: &OpChain, set: &[&str]) -> bool {
    chain
        .methods
        .first()
        .is_some_and(|m| set.contains(&m.as_str()))
}

/// Joins the names of methods with commas.
fn names(run: &AnalysisRun<'_>, methods: &BTreeSet<ItemId>) -> String {
    let names: Vec<String> = (methods.iter())
        .map(|&m| format!("`{}`", run.name(m).unwrap_or_default()))
        .collect();
    names.join(", ")
}

/// Returns whether each chain of one body that starts at field `name` and
/// calls one of `set` first sits in a loop, each answer once.
fn loop_levels(chains: &[OpChain], name: &str, set: &[&str]) -> BTreeSet<bool> {
    (on_field(chains, name))
        .filter(|c| first_in(c, set))
        .map(|c| c.in_loop)
        .collect()
}

/// `sort-after-push`: a `Vec` field that the same methods of its type
/// push to and sort, where one of them pushes and sorts at the same loop
/// level. A method that pushes in a loop and sorts once after it builds a
/// batch, which is no lead alone.
fn sort_after_push(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    decl: ItemId,
    bodies: &[(ItemId, &[OpChain])],
    out: &mut Vec<Lead>,
) {
    for field in fields_held(catalog, decl, is_vec) {
        let mut pushers = BTreeSet::new();
        let mut sorters = BTreeSet::new();
        let mut level = false;
        for &(method, chains) in bodies {
            let push_levels = loop_levels(chains, &field.name, &PUSHES);
            let sort_levels = loop_levels(chains, &field.name, &SORTS);
            if !push_levels.is_empty() {
                pushers.insert(method);
            }
            if !sort_levels.is_empty() {
                sorters.insert(method);
            }
            level |= !push_levels.is_disjoint(&sort_levels);
        }
        if pushers.is_empty() || pushers != sorters || !level {
            continue;
        }
        let evidence = format!(
            "`self.{}` is pushed to and sorted by {}",
            field.name,
            names(run, &pushers)
        );
        let mut lead = Lead::new("sort-after-push", Mode::DirectSyntax, evidence);
        for &method in &pushers {
            lead = lead.item(run, method);
        }
        out.push(field.add(run, lead));
    }
}

/// `co-mutated`: two collection fields of one type whose sets of mutating
/// methods share [`CO_MUTATED_SHARED`] or more methods and
/// [`CO_MUTATED_PERCENT`] percent or more of their union. A method
/// mutates a field when a chain from it assigns it, borrows it as `&mut`,
/// or calls one of [`MUTATORS`] or [`SORTS`] on it first.
fn co_mutated(
    run: &AnalysisRun<'_>,
    catalog: &Catalog<'_>,
    decl: ItemId,
    bodies: &[(ItemId, &[OpChain])],
    out: &mut Vec<Lead>,
) {
    let fields = fields_held(catalog, decl, Ty::is_collection);
    let mutators: Vec<BTreeSet<ItemId>> = (fields.iter())
        .map(|field| {
            (bodies.iter())
                .filter(|(_, chains)| {
                    on_field(chains, &field.name)
                        .any(|c| c.write || first_in(c, &MUTATORS) || first_in(c, &SORTS))
                })
                .map(|&(method, _)| method)
                .collect()
        })
        .collect();
    for (i, a) in fields.iter().enumerate() {
        for (j, b) in fields.iter().enumerate().skip(i + 1) {
            let both: BTreeSet<ItemId> = mutators[i].intersection(&mutators[j]).copied().collect();
            let either = mutators[i].union(&mutators[j]).count();
            if both.len() < CO_MUTATED_SHARED || both.len() * 100 < CO_MUTATED_PERCENT * either {
                continue;
            }
            let evidence = format!(
                "`{}` and `{}` are mutated by {} of {} methods: {}",
                a.name,
                b.name,
                both.len(),
                either,
                names(run, &both)
            );
            let lead = Lead::new("co-mutated", Mode::DirectSyntax, evidence);
            out.push(b.add(run, a.add(run, lead)));
        }
    }
}
