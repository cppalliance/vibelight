//! Finds the items outside the crate that the declarations write in more
//! than one way, such as `fmt::Debug` beside `std::fmt::Debug`.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Crate, Loc, RefId, Seg, TypeRef, name_order, path_order};
use crate::resolve::prelude_path;
use crate::route::derive_path;

/// One way an item is written: the path as written, how many times, and
/// each line it is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Spelling {
    pub(crate) text: String,
    pub(crate) count: usize,
    /// Sorted by file and line, each line once.
    pub(crate) locs: Vec<Loc>,
}

/// An item written more than one way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The item's full path, with a `core` or `alloc` path written as the
    /// `std` path that re-exports it.
    pub(crate) item: String,
    /// Sorted by spelling.
    pub(crate) spellings: Vec<Spelling>,
}

/// Groups every name in a declaration that resolves outside the crate by
/// the item it names, and returns the items written more than one way,
/// sorted by path. Names from comments and re-exports never count, and
/// neither does the self type of an impl attached to its type, which the
/// outline does not show and which each split of the impl writes again.
pub(crate) fn find(krate: &Crate) -> Vec<Entry> {
    let derives: BTreeSet<RefId> = krate
        .decls
        .iter()
        .flat_map(|d| d.derive_refs.iter().copied())
        .collect();
    let hidden: BTreeSet<RefId> = krate
        .impls
        .iter()
        .filter(|i| i.attached.is_some())
        .flat_map(|i| i.self_text.0.iter())
        .filter_map(|seg| match seg {
            Seg::Ref(id) => Some(*id),
            Seg::Plain(_) => None,
        })
        .collect();
    let mut seen = BTreeSet::new();
    let mut groups: BTreeMap<String, BTreeMap<String, (usize, Vec<Loc>)>> = BTreeMap::new();
    for (id, r) in krate.refs.iter().enumerate() {
        let derive = derives.contains(&id);
        if (r.comment && !derive) || hidden.contains(&id) {
            continue;
        }
        let (Some(written), Some(item)) = (r.written, item(r, derive)) else {
            continue;
        };
        if !seen.insert((written.loc.file, written.offset)) {
            continue;
        }
        let slot = groups
            .entry(item)
            .or_default()
            .entry(spelling(r))
            .or_default();
        slot.0 += 1;
        slot.1.push(written.loc);
    }
    let mut entries: Vec<Entry> = groups
        .into_iter()
        .filter(|(_, spellings)| spellings.len() > 1)
        .map(|(item, spellings)| {
            let mut spellings: Vec<Spelling> = spellings
                .into_iter()
                .map(|(text, (count, mut locs))| {
                    locs.sort_by(|a, b| {
                        krate.files[a.file]
                            .rel
                            .cmp(&krate.files[b.file].rel)
                            .then(a.line.cmp(&b.line))
                    });
                    locs.dedup();
                    Spelling { text, count, locs }
                })
                .collect();
            spellings.sort_by(|a, b| name_order(&a.text, &b.text));
            Entry { item, spellings }
        })
        .collect();
    entries.sort_by(|a, b| path_order(&segments(&a.item), &segments(&b.item)));
    entries
}

fn segments(path: &str) -> Vec<String> {
    path.split("::").map(str::to_owned).collect()
}

/// Returns the full path of the item a name outside the crate stands for,
/// or `None` for a name in the crate or one that does not resolve. A bare
/// prelude or derive name stands for its `std` path.
fn item(r: &TypeRef, derive: bool) -> Option<String> {
    if r.target.is_some() {
        return None;
    }
    let path = match (r.canonical.as_deref(), r.path.as_slice()) {
        (Some([name]), _) => prelude_path(name)?.to_owned(),
        (Some(path), _) => path.join("::"),
        (None, [name]) if derive => derive_path(name)?.to_owned(),
        _ => return None,
    };
    Some(match path.split_once("::") {
        Some(("core" | "alloc", rest)) => format!("std::{rest}"),
        _ => path,
    })
}

/// Returns a name as written, without generic arguments or `r#` prefixes.
fn spelling(r: &TypeRef) -> String {
    let joined = r.path.join("::");
    if r.global {
        format!("::{joined}")
    } else {
        joined
    }
}
