//! Finds the items outside the crate that the declarations write in more
//! than one way, such as `fmt::Debug` beside `std::fmt::Debug`.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Crate, Loc, RefId, Seg, TypeRef, name_order, path_order};
use crate::prelude::item_path;
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
/// A derive name never counts either. A derive is written bare in practice,
/// so it never matches a path in an impl header or a bound, and the entry
/// it made could not be fixed. Each entry names the spelling to prefer,
/// when it has one.
pub(crate) fn find(krate: &Crate) -> Vec<Entry> {
    find_hiding(krate, |i| krate.impls[i].attached.is_some())
}

/// Does what [`find`] does, with `attached` telling which impls are
/// attached to their types, so that it also works on a crate that attach
/// has not changed yet.
pub(crate) fn find_hiding(krate: &Crate, attached: impl Fn(usize) -> bool) -> Vec<Entry> {
    let hidden: BTreeSet<RefId> = krate
        .impls
        .iter()
        .enumerate()
        .filter(|&(i, _)| attached(i))
        .flat_map(|(_, i)| i.self_text.0.iter())
        .filter_map(|seg| match seg {
            Seg::Ref(id) => Some(*id),
            Seg::Plain(_) => None,
        })
        .collect();
    let mut seen = BTreeSet::new();
    let mut groups: BTreeMap<String, BTreeMap<String, (usize, Vec<Loc>)>> = BTreeMap::new();
    for (id, r) in krate.refs.iter().enumerate() {
        if r.comment || hidden.contains(&id) {
            continue;
        }
        let (Some(written), Some(item)) = (r.written, item(r)) else {
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
/// prelude name stands for its `std` path.
fn item(r: &TypeRef) -> Option<String> {
    if r.target.is_some() {
        return None;
    }
    r.canonical.as_deref().and_then(item_path)
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
    use std::fs;

    use super::*;

    /// Runs the crate source through the same steps as a real run, and
    /// returns each entry's item with its spellings and their counts.
    fn entries(lib: &str) -> Vec<(String, Vec<(String, usize)>)> {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), lib).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        crate::resolve::resolve(&mut krate);
        crate::attach::attach(&mut krate);
        find(&krate)
            .into_iter()
            .map(|entry| {
                let spellings = entry.spellings.into_iter();
                (entry.item, spellings.map(|s| (s.text, s.count)).collect())
            })
            .collect()
    }

    fn spelled(texts: &[(&str, usize)]) -> Vec<(String, usize)> {
        texts.iter().map(|&(t, n)| (t.to_owned(), n)).collect()
    }

    #[test]
    fn a_derive_beside_one_impl_spelling_is_no_entry() {
        let lib = "use std::fmt;\n\
                   #[derive(Debug)]\n\
                   pub struct Gauge;\n\
                   pub struct Dial;\n\
                   impl fmt::Debug for Dial {}\n";
        assert_eq!(entries(lib), Vec::new());
    }

    #[test]
    fn a_derive_adds_no_count_to_an_entry() {
        let lib = "use std::fmt;\n\
                   #[derive(Debug)]\n\
                   pub struct Gauge;\n\
                   pub struct Dial;\n\
                   pub struct Knob;\n\
                   impl fmt::Debug for Dial {}\n\
                   impl std::fmt::Debug for Knob {}\n";
        assert_eq!(
            entries(lib),
            [(
                "std::fmt::Debug".to_owned(),
                spelled(&[("fmt::Debug", 1), ("std::fmt::Debug", 1)])
            )]
        );
    }

    #[test]
    fn a_derive_is_left_out_however_it_is_written_or_attached() {
        let lib = "use serde::Serialize;\n\
                   #[derive(Serialize)]\n\
                   pub struct A;\n\
                   #[derive(serde::Serialize)]\n\
                   pub struct B;\n\
                   #[cfg_attr(feature = \"x\", derive(Clone))]\n\
                   pub struct C;\n\
                   #[derive(core::clone::Clone)]\n\
                   pub struct D;\n\
                   impl std::clone::Clone for E {}\n\
                   pub struct E;\n";
        assert_eq!(entries(lib), Vec::new());
    }

    #[test]
    fn names_that_are_not_derives_still_count() {
        let lib = "use serde::Serialize;\n\
                   pub struct A { pub one: Option<u8>, pub two: std::option::Option<u8> }\n\
                   impl Serialize for A {}\n\
                   impl serde::Serialize for B {}\n\
                   pub struct B;\n";
        assert_eq!(
            entries(lib),
            [
                (
                    "serde::Serialize".to_owned(),
                    spelled(&[("serde::Serialize", 1), ("Serialize", 1)])
                ),
                (
                    "std::option::Option".to_owned(),
                    spelled(&[("Option", 1), ("std::option::Option", 1)])
                ),
            ]
        );
    }

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
