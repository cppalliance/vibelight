//! The detectors of containers and access profiles: which collections a
//! type's fields hold and what each is keyed by, and how bodies use the
//! collections that fields and parameters hold. This file holds the group's
//! registry and the detectors that read one type's fields; its siblings
//! hold the keys shared across owned types and the operation chains of
//! bodies.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::leads::{Lead, Mode};
use crate::leads_items::{Catalog, Field};
use crate::leads_types::{Tok, Ty, leaf};
use crate::{leads_containers_ops, leads_containers_owners};

/// The group's lead ids, in glossary order.
pub(crate) const IDS: &[&str] = &[
    "container-smell",
    "parallel-collections",
    "shared-key",
    "linear-search",
    "front-ops-on-vec",
    "sort-after-push",
    "sorted-unordered-map",
    "co-mutated",
];

/// The ids whose leads rest on what the run does not observe: none, since
/// every lead of the group points at a written type or operation.
pub(crate) const ABSENT: &[&str] = &[];

/// The maps and sets whose first generic argument is their key, by the
/// last segment of their path.
const KEYED: [&str; 6] = [
    "BTreeMap", "BTreeSet", "HashMap", "HashSet", "IndexMap", "IndexSet",
];

/// The maps among [`KEYED`].
const MAPS: [&str; 3] = ["BTreeMap", "HashMap", "IndexMap"];

/// The sequences whose element is what they are keyed by, by the last
/// segment of their path.
const SEQUENCES: [&str; 2] = ["Vec", "VecDeque"];

/// The fewest collection fields of one key in one type for
/// `parallel-collections`.
const PARALLEL_FROM: usize = 2;

/// Finds the group's leads.
pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let catalog = Catalog::new(run);
    container_smell(run, &catalog, out);
    let keyed = keyed(&catalog);
    parallel_collections(run, &keyed, out);
    leads_containers_owners::find(run, &keyed, out);
    leads_containers_ops::find(run, &catalog, out);
}

/// One collection field of a struct or union, with the type it is keyed
/// by.
pub(crate) struct Keyed<'c, 'a> {
    pub(crate) field: &'c Field<'a>,
    /// The key, as [`Ty::key`] spells it.
    pub(crate) key: String,
    /// The key as a reader sees it.
    pub(crate) shown: String,
}

impl Keyed<'_, '_> {
    /// Returns the field as the evidence shows it.
    pub(crate) fn text(&self) -> String {
        self.field.ty.typed(&self.field.name)
    }
}

/// Returns the head of a type written as one name and its generic
/// arguments, as the last segment of its path, with the arguments.
pub(crate) fn head(ty: &Ty) -> Option<(String, Vec<Ty>)> {
    let (head, args) = ty.generic()?;
    Some((leaf(head), args))
}

/// How a collection holds its key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Keying {
    /// As a map's or set's key.
    Key,
    /// As the first part of a sequence's tuple element.
    Pair,
    /// As a sequence's element.
    Element,
}

/// Returns what a field's collection is keyed by, at its core, and how: a
/// map's or set's key, the first part of a sequence's element that is a
/// tuple, or a sequence's element. A key made only of `std` types, such as
/// a number, a string, or a path, names no identity of the crate's own, so
/// two collections of it are common and say nothing; such a key, a generic
/// one, and an `impl` or `dyn` one count as none.
fn key_of(ty: &Ty) -> Option<(Ty, Keying)> {
    let (head, args) = head(&ty.core())?;
    let first = args.into_iter().next()?;
    let (key, keying) = if KEYED.contains(&head.as_str()) {
        (first, Keying::Key)
    } else if SEQUENCES.contains(&head.as_str()) {
        match first.tuple() {
            Some(parts) => (parts.into_iter().next()?, Keying::Pair),
            None => (first, Keying::Element),
        }
    } else {
        return None;
    };
    let key = key.core();
    let named = key.names().next().is_some();
    (named && !key.is_std() && !key.is_generic() && !key.is_opaque()).then_some((key, keying))
}

/// Returns the keyed collection fields of each struct and union, in field
/// order. A sequence's plain element counts as a key only when a map, a
/// set, or a sequence of pairs of the same target is keyed by its type, so
/// that two lists of one record type are no lead while two lists of one
/// identity are. An enum's variants are alternatives rather than parts
/// held together, so their fields are left out.
fn keyed<'c, 'a>(catalog: &'c Catalog<'a>) -> BTreeMap<ItemId, Vec<Keyed<'c, 'a>>> {
    let found: Vec<(&Field<'a>, Ty, Keying)> = (catalog.fields.iter())
        .filter_map(|field| key_of(&field.ty).map(|(key, keying)| (field, key, keying)))
        .collect();
    let known: BTreeSet<(usize, String)> = (found.iter())
        .filter(|(_, _, keying)| *keying != Keying::Element)
        .map(|(field, key, _)| (field.decl.target, key.key()))
        .collect();
    let mut out: BTreeMap<ItemId, Vec<Keyed<'c, 'a>>> = BTreeMap::new();
    for (field, key, keying) in found {
        let spelled = key.key();
        let identity =
            keying != Keying::Element || known.contains(&(field.decl.target, spelled.clone()));
        if field.variant.is_none() && identity {
            out.entry(field.decl).or_default().push(Keyed {
                field,
                key: spelled,
                shown: key.shown(),
            });
        }
    }
    out
}

/// Returns the pair a `Vec` of pairs holds, behind references and an
/// `Option`, when its first part may be a key: a number, a `bool`, or a
/// `char` first is a position or a measure, such as the ends of a range.
fn pair_of(ty: &Ty) -> Option<[Ty; 2]> {
    let ty = ty.deref();
    let ty = ty.option().map_or(ty, |held| held.deref());
    let (head, args) = head(&ty)?;
    let [k, v]: [Ty; 2] = args
        .first()
        .filter(|_| head == "Vec")?
        .tuple()?
        .try_into()
        .ok()?;
    let measure = k.deref().primitive().is_some_and(|p| p != "str");
    (!measure).then_some([k, v])
}

/// Returns a map whose values are maps, at its core, as the evidence
/// shows it.
fn nested_map(ty: &Ty) -> Option<String> {
    let (outer, args) = head(&ty.core()).filter(|(outer, _)| MAPS.contains(&outer.as_str()))?;
    let (inner, _) = head(&args.get(1)?.core())?;
    MAPS.contains(&inner.as_str())
        .then(|| format!("a `{inner}` inside a `{outer}`"))
}

/// Returns what about a field's type is a container smell: a
/// `LinkedList`, a `Vec` of pairs, a map of maps, or an `Rc<RefCell<_>>`,
/// each named from outside the run.
fn smells(ty: &Ty) -> Vec<String> {
    let outside = |name: &str| !name.starts_with('#');
    let mut found = Vec::new();
    if ty
        .names()
        .any(|name| outside(name) && leaf(name) == "LinkedList")
    {
        found.push("a `LinkedList`".to_owned());
    }
    if let Some([k, v]) = pair_of(ty) {
        found.push(format!("a `Vec` of `({}, {})`", k.shown(), v.shown()));
    }
    found.extend(nested_map(ty));
    let shared = ty.0.windows(3).any(|w| match w {
        [Tok::Name(rc), Tok::Code(open), Tok::Name(cell)] => {
            outside(rc) && leaf(rc) == "Rc" && open == "<" && leaf(cell) == "RefCell"
        }
        _ => false,
    });
    if shared {
        found.push("an `Rc<RefCell<_>>`".to_owned());
    }
    found
}

/// `container-smell`: a field of a struct, union, or variant whose type
/// holds a `LinkedList`, a `Vec` of pairs whose first part is no number,
/// `bool`, or `char`, a map whose values are maps, or an `Rc<RefCell<_>>`.
fn container_smell(run: &AnalysisRun<'_>, catalog: &Catalog<'_>, out: &mut Vec<Lead>) {
    for field in &catalog.fields {
        let found = smells(&field.ty);
        if found.is_empty() {
            continue;
        }
        let evidence = format!(
            "`{}: {}` holds {}",
            field.name,
            field.ty.shown(),
            found.join(" and ")
        );
        let lead = Lead::new("container-smell", Mode::DirectSyntax, evidence);
        out.push(field.add(run, lead));
    }
}

/// `parallel-collections`: [`PARALLEL_FROM`] or more collection fields of
/// one struct or union keyed by one type, as [`keyed`] reads them.
fn parallel_collections(
    run: &AnalysisRun<'_>,
    keyed: &BTreeMap<ItemId, Vec<Keyed<'_, '_>>>,
    out: &mut Vec<Lead>,
) {
    for fields in keyed.values() {
        let mut by_key: BTreeMap<&str, Vec<&Keyed<'_, '_>>> = BTreeMap::new();
        for field in fields {
            by_key.entry(field.key.as_str()).or_default().push(field);
        }
        for group in by_key.values().filter(|g| g.len() >= PARALLEL_FROM) {
            let shown: Vec<String> = group.iter().map(|k| k.text()).collect();
            let evidence = format!("{} are keyed by `{}`", shown.join(", "), group[0].shown);
            let mut lead = Lead::new("parallel-collections", Mode::DirectSyntax, evidence);
            for keyed in group {
                lead = keyed.field.add(run, lead);
            }
            out.push(lead);
        }
    }
}
