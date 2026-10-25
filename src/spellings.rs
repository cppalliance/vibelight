//! Finds the items outside the crate that the declarations write in more
//! than one way, such as `fmt::Debug` beside `std::fmt::Debug`.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Crate, Loc, RefId, Seg, TypeRef, name_order, path_order};
use crate::resolve::item_path;
use crate::route::derive_path;
use crate::typerefs::strip_raw;

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
    /// The spelling to prefer: the bare name of a standard item, as a base
    /// list prints it, when that name stands for nothing else in the crate.
    pub(crate) preferred: Option<String>,
    /// Sorted by spelling.
    pub(crate) spellings: Vec<Spelling>,
}

/// Groups every name in a declaration that resolves outside the crate by
/// the item it names, and returns the items written more than one way,
/// sorted by path. Names from comments and re-exports never count, and
/// neither does the self type of an impl attached to its type, which the
/// outline does not show and which each split of the impl writes again.
/// Each entry names the spelling to prefer, when it has one.
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
    let declared: BTreeSet<String> = krate.decls.iter().map(|d| strip_raw(&d.name)).collect();
    let mut bare_names: BTreeMap<String, usize> = BTreeMap::new();
    for item in groups.keys() {
        if let Some(bare) = bare_name(item) {
            *bare_names.entry(bare.to_owned()).or_default() += 1;
        }
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
            Entry {
                preferred: preferred(&item, &declared, &bare_names),
                item,
                spellings,
            }
        })
        .collect();
    entries.sort_by(|a, b| path_order(&segments(&a.item), &segments(&b.item)));
    entries
}

fn segments(path: &str) -> Vec<String> {
    path.split("::").map(str::to_owned).collect()
}

/// Returns the last segment of a standard item's path, such as `Display`
/// for `std::fmt::Display`.
fn bare_name(item: &str) -> Option<&str> {
    item.strip_prefix("std::")?.rsplit("::").next()
}

/// Returns the bare name to prefer for an item, unless the item is not
/// standard, the crate declares a type or trait with that name, or another
/// standard item the crate uses has the same bare name.
fn preferred(
    item: &str,
    declared: &BTreeSet<String>,
    bare_names: &BTreeMap<String, usize>,
) -> Option<String> {
    let bare = bare_name(item)?;
    (!declared.contains(bare) && bare_names.get(bare) == Some(&1)).then(|| bare.to_owned())
}

/// Returns the full path of the item a name outside the crate stands for,
/// or `None` for a name in the crate or one that does not resolve. A bare
/// prelude or derive name stands for its `std` path.
fn item(r: &TypeRef, derive: bool) -> Option<String> {
    if r.target.is_some() {
        return None;
    }
    match (r.canonical.as_deref(), r.path.as_slice()) {
        (Some(path), _) => item_path(path),
        (None, [name]) if derive => derive_path(name).map(str::to_owned),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_standard_item_prefers_its_bare_name_unless_the_name_is_taken() {
        let declared: BTreeSet<String> = ["Duration".to_owned()].into();
        let bare_names: BTreeMap<String, usize> = [("Display", 1), ("Duration", 1), ("Write", 2)]
            .map(|(name, count)| (name.to_owned(), count))
            .into();
        let prefer = |item: &str| preferred(item, &declared, &bare_names);
        assert_eq!(prefer("std::fmt::Display").as_deref(), Some("Display"));
        assert_eq!(
            prefer("std::time::Duration"),
            None,
            "the crate declares one"
        );
        assert_eq!(
            prefer("std::fmt::Write"),
            None,
            "std::io::Write ends the same"
        );
        assert_eq!(prefer("serde_json::Value"), None, "not a standard item");
    }
}
