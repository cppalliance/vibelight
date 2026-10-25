//! Resolves type names to declarations by rule 39.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Crate, DeclId, ModId, TypeRef};
use crate::typerefs::strip_raw;

/// What a path names.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Res {
    Decl(DeclId),
    Module(ModId),
    /// A path into another crate, spelled out through the `use` items it
    /// passed.
    External(Vec<String>),
}

/// How many names stayed plain.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tally {
    pub(crate) unresolved: usize,
    pub(crate) ambiguous: usize,
}

/// The state of one name's lookup.
#[derive(Default)]
struct Seen {
    /// The (module, name) lookups already made.
    done: BTreeSet<(ModId, String)>,
    /// True once the lookup passed a glob from another crate, which may be
    /// where an unfound name comes from.
    external_glob: bool,
}

/// Names the standard prelude and the language provide, which never count
/// as unresolved, each with the `std` path it stands for.
const PRELUDE: &[(&str, &str)] = &[
    ("AsMut", "std::convert::AsMut"),
    ("AsRef", "std::convert::AsRef"),
    ("Box", "std::boxed::Box"),
    ("Clone", "std::clone::Clone"),
    ("Copy", "std::marker::Copy"),
    ("Default", "std::default::Default"),
    ("DoubleEndedIterator", "std::iter::DoubleEndedIterator"),
    ("Drop", "std::ops::Drop"),
    ("Eq", "std::cmp::Eq"),
    ("ExactSizeIterator", "std::iter::ExactSizeIterator"),
    ("Extend", "std::iter::Extend"),
    ("Fn", "std::ops::Fn"),
    ("FnMut", "std::ops::FnMut"),
    ("FnOnce", "std::ops::FnOnce"),
    ("From", "std::convert::From"),
    ("FromIterator", "std::iter::FromIterator"),
    ("Into", "std::convert::Into"),
    ("IntoIterator", "std::iter::IntoIterator"),
    ("Iterator", "std::iter::Iterator"),
    ("Option", "std::option::Option"),
    ("Ord", "std::cmp::Ord"),
    ("PartialEq", "std::cmp::PartialEq"),
    ("PartialOrd", "std::cmp::PartialOrd"),
    ("Result", "std::result::Result"),
    ("Send", "std::marker::Send"),
    ("Sized", "std::marker::Sized"),
    ("String", "std::string::String"),
    ("Sync", "std::marker::Sync"),
    ("ToOwned", "std::borrow::ToOwned"),
    ("ToString", "std::string::ToString"),
    ("TryFrom", "std::convert::TryFrom"),
    ("TryInto", "std::convert::TryInto"),
    ("Unpin", "std::marker::Unpin"),
    ("Vec", "std::vec::Vec"),
    ("bool", "std::primitive::bool"),
    ("char", "std::primitive::char"),
    ("f32", "std::primitive::f32"),
    ("f64", "std::primitive::f64"),
    ("i128", "std::primitive::i128"),
    ("i16", "std::primitive::i16"),
    ("i32", "std::primitive::i32"),
    ("i64", "std::primitive::i64"),
    ("i8", "std::primitive::i8"),
    ("isize", "std::primitive::isize"),
    ("str", "std::primitive::str"),
    ("u128", "std::primitive::u128"),
    ("u16", "std::primitive::u16"),
    ("u32", "std::primitive::u32"),
    ("u64", "std::primitive::u64"),
    ("u8", "std::primitive::u8"),
    ("usize", "std::primitive::usize"),
];

/// Returns the `std` path a bare prelude name stands for.
fn prelude_path(name: &str) -> Option<&'static str> {
    PRELUDE
        .iter()
        .find(|(prelude, _)| *prelude == name)
        .map(|(_, path)| *path)
}

/// Returns the full path of the item a canonical path names: a bare prelude
/// name stands for its `std` path, and a `core` or `alloc` path for the
/// `std` path that re-exports it.
pub(crate) fn item_path(path: &[String]) -> Option<String> {
    let joined = match path {
        [name] => prelude_path(name)?.to_owned(),
        _ => path.join("::"),
    };
    Some(match joined.split_once("::") {
        Some(("core" | "alloc", rest)) => format!("std::{rest}"),
        _ => joined,
    })
}

/// Tells whether a canonical path names an item of `std`, `core`, `alloc`,
/// or the prelude, whose canonical path is its bare name.
pub(crate) fn is_standard(path: &[String]) -> bool {
    match path {
        [name] => prelude_path(name).is_some(),
        [first, ..] => matches!(first.as_str(), "std" | "core" | "alloc"),
        [] => false,
    }
}

/// The names one crate declares, by module, for lookups.
pub(crate) struct Scope<'a> {
    pub(crate) krate: &'a Crate,
    /// Each module's declarations by name.
    pub(crate) decls: Vec<BTreeMap<String, Vec<DeclId>>>,
    /// Each module's child modules by name.
    pub(crate) children: Vec<BTreeMap<String, Vec<ModId>>>,
    pub(crate) parents: Vec<Option<ModId>>,
    unique: BTreeMap<String, Vec<DeclId>>,
}

/// Fills in the target or the canonical path of every type reference and
/// returns how many names stayed plain.
pub(crate) fn resolve(krate: &mut Crate) -> Tally {
    let scope = Scope::new(krate);
    let mut tally = Tally::default();
    let found: Vec<(Option<DeclId>, Option<Vec<String>>)> = krate
        .refs
        .iter()
        .map(|r| scope.lookup(r, &mut tally))
        .collect();
    for (r, (target, canonical)) in krate.refs.iter_mut().zip(found) {
        r.target = target;
        r.canonical = canonical;
    }
    tally
}

impl<'a> Scope<'a> {
    pub(crate) fn new(krate: &'a Crate) -> Scope<'a> {
        let n = krate.modules.len();
        let mut by_path: BTreeMap<&[String], ModId> = BTreeMap::new();
        for (id, module) in krate.modules.iter().enumerate() {
            by_path.entry(module.path.as_slice()).or_insert(id);
        }
        let mut parents = vec![None; n];
        let mut children: Vec<BTreeMap<String, Vec<ModId>>> = vec![BTreeMap::new(); n];
        for (id, module) in krate.modules.iter().enumerate() {
            if let Some((name, parent)) = module.path.split_last() {
                let parent = by_path.get(parent).copied();
                parents[id] = parent;
                if let Some(parent) = parent {
                    children[parent]
                        .entry(strip_raw(name))
                        .or_default()
                        .push(id);
                }
            }
        }
        let mut decls: Vec<BTreeMap<String, Vec<DeclId>>> = vec![BTreeMap::new(); n];
        let mut unique: BTreeMap<String, Vec<DeclId>> = BTreeMap::new();
        for (id, decl) in krate.decls.iter().enumerate() {
            let name = strip_raw(&decl.name);
            decls[decl.module].entry(name.clone()).or_default().push(id);
            unique.entry(name).or_default().push(id);
        }
        let key = |id: &DeclId| decl_key(krate, *id);
        for list in decls.iter_mut().flat_map(BTreeMap::values_mut) {
            list.sort_by_key(key);
        }
        Scope {
            krate,
            decls,
            children,
            parents,
            unique,
        }
    }

    /// Finds the declaration a name resolves to, or the full path of a name
    /// from outside the crate. A name unique across the target counts only
    /// when no glob from another crate could have brought the name in.
    fn lookup(&self, r: &TypeRef, tally: &mut Tally) -> (Option<DeclId>, Option<Vec<String>>) {
        let Some(first) = r.path.first() else {
            return (None, None);
        };
        if !r.global && (r.generics.contains(first) || first == "Self") {
            return (None, None);
        }
        if r.global {
            return (None, Some(r.path.clone()));
        }
        let mut seen = Seen::default();
        match self.walk(r.module, &r.path, &mut seen) {
            Some(Res::Decl(id)) => return (Some(id), None),
            Some(Res::External(path)) => return (None, Some(path)),
            Some(Res::Module(_)) => return (None, None),
            None => {}
        }
        if r.path.len() == 1 {
            let unique = if seen.external_glob {
                None
            } else {
                self.unique.get(first).map(Vec::as_slice)
            };
            match unique {
                Some([only]) => return (Some(*only), None),
                Some([_, _, ..]) => {
                    if !r.comment {
                        tally.ambiguous += 1;
                    }
                    return (None, None);
                }
                _ => {}
            }
            if prelude_path(first).is_some() {
                return (None, Some(r.path.clone()));
            }
            if !r.comment && !seen.external_glob {
                tally.unresolved += 1;
            }
            return (None, None);
        }
        if !r.comment {
            tally.unresolved += 1;
        }
        (None, None)
    }

    /// Walks a path from `module`; `None` means a local name was not found.
    fn walk(&self, module: ModId, path: &[String], seen: &mut Seen) -> Option<Res> {
        if path.is_empty() {
            return None;
        }
        let (mut current, rest, prefixed) = match path[0].as_str() {
            "crate" => (0, &path[1..], true),
            "self" => (module, &path[1..], true),
            "super" => {
                let mut current = module;
                let mut i = 0;
                while path.get(i).map(String::as_str) == Some("super") {
                    current = self.parents[current]?;
                    i += 1;
                }
                (current, &path[i..], true)
            }
            _ => (module, path, false),
        };
        if rest.is_empty() {
            return Some(Res::Module(current));
        }
        for (i, segment) in rest.iter().enumerate() {
            let found = self.lookup_in(current, segment, seen);
            let Some(found) = found else {
                return if i == 0 && !prefixed && rest.len() > 1 {
                    Some(Res::External(rest.to_vec()))
                } else {
                    None
                };
            };
            if i + 1 == rest.len() {
                return Some(found);
            }
            match found {
                Res::Module(next) => current = next,
                Res::Decl(_) => return None,
                Res::External(mut outside) => {
                    outside.extend(rest[i + 1..].iter().cloned());
                    return Some(Res::External(outside));
                }
            }
        }
        None
    }

    /// Looks up one name in a module: its `use` items, then its globs, then
    /// its declarations, then its child modules. A name already looked up in
    /// this module during the same query is not found again, which stops
    /// `use` and glob cycles.
    fn lookup_in(&self, module: ModId, name: &str, seen: &mut Seen) -> Option<Res> {
        if !seen.done.insert((module, name.to_owned())) {
            return None;
        }
        let uses = &self.krate.modules[module].uses;
        if let Some(u) = uses.iter().find(|u| u.name.as_deref() == Some(name)) {
            if u.global {
                return Some(Res::External(u.path.clone()));
            }
            return Some(
                self.walk(module, &u.path, seen)
                    .unwrap_or_else(|| Res::External(u.path.clone())),
            );
        }
        for u in uses.iter().filter(|u| u.name.is_none()) {
            match self.walk(module, &u.path, seen) {
                Some(Res::Module(source)) if source != module => {
                    if let Some(found) = self.lookup_in(source, name, seen) {
                        return Some(found);
                    }
                }
                Some(Res::External(_)) | None => seen.external_glob = true,
                Some(_) => {}
            }
        }
        if let Some(ids) = self.decls[module].get(name) {
            return ids.first().map(|&id| Res::Decl(id));
        }
        self.children[module]
            .get(name)
            .and_then(|ids| ids.first())
            .map(|&id| Res::Module(id))
    }
}

/// Orders declarations that share a path, such as one type declared under
/// two `cfg`s, by their `cfg` lines and generics, so a name links to the
/// same one whatever order the source holds them in.
fn decl_key(krate: &Crate, id: DeclId) -> String {
    let decl = &krate.decls[id];
    format!(
        "{} {}",
        decl.meta.cfgs.join(" "),
        decl.generics.visible(&krate.refs)
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn glob_cycles_and_external_globs_end_quickly() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), "pub mod a;\npub mod b;\npub mod c;\n").unwrap();
        fs::write(src.join("c.rs"), "pub struct Lone { pub gone: Gone }\n").unwrap();
        fs::write(
            src.join("a.rs"),
            "pub use crate::b::*;\nuse x::prelude::*;\nuse y::prelude::*;\nuse z::prelude::*;\n\
             pub struct Holder { pub found: Found, pub missing: Missing }\n",
        )
        .unwrap();
        fs::write(
            src.join("b.rs"),
            "pub use crate::a::*;\nuse w::prelude::*;\npub struct Found;\n",
        )
        .unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        let tally = resolve(&mut krate);
        let target = |name: &str| {
            krate
                .refs
                .iter()
                .find(|r| r.text == name)
                .and_then(|r| r.target)
        };
        assert_eq!(
            target("Found")
                .map(|d| krate.decls[d].name.clone())
                .as_deref(),
            Some("Found")
        );
        assert_eq!(target("Missing"), None);
        assert_eq!(target("Gone"), None);
        assert_eq!(
            tally.unresolved, 1,
            "only Gone counts; Missing may come from a glob"
        );
    }

    #[test]
    fn a_glob_from_another_crate_keeps_a_unique_name_from_resolving() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            "pub mod custom { pub struct Shown; }\n\
             pub mod globbed { use something::*; pub struct Starred { pub shown: Shown } }\n\
             pub mod plain { pub struct Bare { pub shown: Shown } }\n",
        )
        .unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        let tally = resolve(&mut krate);
        let targets: Vec<Option<DeclId>> = krate
            .refs
            .iter()
            .filter(|r| r.text == "Shown")
            .map(|r| r.target)
            .collect();
        let shown = krate.decls.iter().position(|d| d.name == "Shown");
        assert_eq!(targets, [None, shown], "the glob may bring its own Shown");
        assert_eq!(tally.unresolved, 0);
    }

    #[test]
    fn standard_paths_start_with_std_core_or_alloc_or_name_a_prelude_item() {
        let path = |s: &str| -> Vec<String> { s.split("::").map(str::to_owned).collect() };
        for standard in [
            "std::fmt::Debug",
            "core::cmp::Ord",
            "alloc::vec::Vec",
            "Option",
        ] {
            assert!(is_standard(&path(standard)), "{standard}");
        }
        for other in [
            "serde::Serialize",
            "runvil_lua::ErrorValue",
            "Named",
            "Debug",
        ] {
            assert!(!is_standard(&path(other)), "{other}");
        }
        assert!(!is_standard(&[]));
    }
}
