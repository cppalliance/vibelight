//! Resolves type names to declarations by rule 39.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Crate, DeclId, ModId, TypeRef, ValueRef};
use crate::prelude::prelude_path;
use crate::resolve_values::value_index;
use crate::typerefs::strip_raw;

/// What a path names.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Res {
    Decl(DeclId),
    /// A function, constant, or static, reachable only by a name that may
    /// be a value.
    Value(ValueRef),
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

/// What one name's lookup found.
#[derive(Default)]
struct Found {
    target: Option<DeclId>,
    value: Option<ValueRef>,
    canonical: Option<Vec<String>>,
}

impl Found {
    fn decl(id: DeclId) -> Found {
        Found {
            target: Some(id),
            ..Found::default()
        }
    }

    fn value(at: ValueRef) -> Found {
        Found {
            value: Some(at),
            ..Found::default()
        }
    }

    fn outside(path: Vec<String>) -> Found {
        Found {
            canonical: Some(path),
            ..Found::default()
        }
    }
}

/// The state of one name's lookup.
#[derive(Default)]
struct Seen {
    /// The (module, name, may be a value) lookups already made.
    done: BTreeSet<(ModId, String, bool)>,
    /// True once the lookup passed a glob from another crate, which may be
    /// where an unfound name comes from.
    external_glob: bool,
}

/// The names one crate declares, by module, for lookups.
pub(crate) struct Scope<'a> {
    pub(crate) krate: &'a Crate,
    /// Each module's declarations by name.
    pub(crate) decls: Vec<BTreeMap<String, Vec<DeclId>>>,
    /// Each module's functions, constants, and statics by name, as indices
    /// into the module's items.
    pub(crate) values: Vec<BTreeMap<String, Vec<usize>>>,
    /// Each module's child modules by name.
    pub(crate) children: Vec<BTreeMap<String, Vec<ModId>>>,
    pub(crate) parents: Vec<Option<ModId>>,
    unique: BTreeMap<String, Vec<DeclId>>,
}

/// Fills in the target, the value, or the canonical path of every type
/// reference and returns how many names stayed plain.
pub(crate) fn resolve(krate: &mut Crate) -> Tally {
    let scope = Scope::new(krate);
    let mut tally = Tally::default();
    let found: Vec<Found> = krate
        .refs
        .iter()
        .map(|r| scope.lookup(r, &mut tally))
        .collect();
    for (r, found) in krate.refs.iter_mut().zip(found) {
        r.target = found.target;
        r.value = found.value;
        r.canonical = found.canonical;
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
            values: value_index(krate),
            children,
            parents,
            unique,
        }
    }

    /// Finds the declaration a name resolves to, or the full path of a name
    /// from outside the crate. A bare name that no `use`, glob, or
    /// declaration of the module provides is the prelude's when the prelude
    /// has it, as in Rust, before any declaration in another module. A name
    /// unique across the target counts only after that, and only when no
    /// glob from another crate could have brought the name in. A name that
    /// may be a value finds a function, constant, or static only where no
    /// type of that name is found, and never by the unique-name fallback.
    fn lookup(&self, r: &TypeRef, tally: &mut Tally) -> Found {
        let Some(first) = r.path.first() else {
            return Found::default();
        };
        if !r.global && (r.generics.contains(first) || first == "Self") {
            return Found::default();
        }
        if r.global {
            return Found::outside(r.path.clone());
        }
        let mut seen = Seen::default();
        match self.walk(r.module, &r.path, &mut seen, r.may_be_value) {
            Some(Res::Decl(id)) => return Found::decl(id),
            Some(Res::Value(at)) => return Found::value(at),
            Some(Res::External(path)) => return Found::outside(path),
            Some(Res::Module(_)) => return Found::default(),
            None => {}
        }
        if r.path.len() == 1 {
            if prelude_path(first).is_some() {
                return Found::outside(r.path.clone());
            }
            let unique = if seen.external_glob {
                None
            } else {
                self.unique.get(first).map(Vec::as_slice)
            };
            match unique {
                Some([only]) => return Found::decl(*only),
                Some([_, _, ..]) => {
                    if !r.comment {
                        tally.ambiguous += 1;
                    }
                    return Found::default();
                }
                _ => {}
            }
            if !r.comment && !seen.external_glob {
                tally.unresolved += 1;
            }
            return Found::default();
        }
        if !r.comment {
            tally.unresolved += 1;
        }
        Found::default()
    }

    /// Walks a path from `module`; `None` means a local name was not found.
    /// With `values` set, the last name may be a function, constant, or
    /// static.
    fn walk(&self, module: ModId, path: &[String], seen: &mut Seen, values: bool) -> Option<Res> {
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
            let last = i + 1 == rest.len();
            let found = self.lookup_in(current, segment, seen, values && last);
            let Some(found) = found else {
                return if i == 0 && !prefixed && rest.len() > 1 {
                    Some(Res::External(rest.to_vec()))
                } else {
                    None
                };
            };
            if last {
                return Some(found);
            }
            match found {
                Res::Module(next) => current = next,
                Res::Decl(_) | Res::Value(_) => return None,
                Res::External(mut outside) => {
                    outside.extend(rest[i + 1..].iter().cloned());
                    return Some(Res::External(outside));
                }
            }
        }
        None
    }

    /// Looks up one name in a module: its `use` items, then its globs, then
    /// its declarations, then, with `values` set, its functions, constants,
    /// and statics, then its child modules. A name already looked up in this
    /// module during the same query is not found again, which stops `use`
    /// and glob cycles.
    fn lookup_in(&self, module: ModId, name: &str, seen: &mut Seen, values: bool) -> Option<Res> {
        if !seen.done.insert((module, name.to_owned(), values)) {
            return None;
        }
        let uses = &self.krate.modules[module].uses;
        if let Some(u) = uses.iter().find(|u| u.name.as_deref() == Some(name)) {
            if u.global {
                return Some(Res::External(u.path.clone()));
            }
            return Some(
                self.walk(module, &u.path, seen, values)
                    .unwrap_or_else(|| Res::External(u.path.clone())),
            );
        }
        for u in uses.iter().filter(|u| u.name.is_none()) {
            match self.walk(module, &u.path, seen, false) {
                Some(Res::Module(source)) if source != module => {
                    if let Some(found) = self.lookup_in(source, name, seen, values) {
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
        if values {
            let first = self.values[module]
                .get(name)
                .and_then(|items| items.first());
            if let Some(&item) = first {
                return Some(Res::Value(ValueRef { module, item }));
            }
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
    use crate::prelude::item_path;

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

    /// Resolves a crate made of one `lib.rs`.
    fn resolved(lib: &str) -> (tempfile::TempDir, Crate, Tally) {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), lib).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        let tally = resolve(&mut krate);
        (tmp, krate, tally)
    }

    /// The declaration and the outside item that the first reference to
    /// `name` in `module` resolves to, as paths.
    fn found(krate: &Crate, module: &str, name: &str) -> (Option<String>, Option<String>) {
        let r = krate
            .refs
            .iter()
            .find(|r| krate.modules[r.module].path.join("::") == module && r.path == [name])
            .expect("the module names it");
        let decl = r.target.map(|d| {
            let decl = &krate.decls[d];
            let mut path = krate.modules[decl.module].path.clone();
            path.push(decl.name.clone());
            path.join("::")
        });
        (decl, r.canonical.as_deref().and_then(item_path))
    }

    #[test]
    fn a_bare_prelude_name_never_resolves_to_a_declaration_in_another_module() {
        let (_tmp, krate, tally) = resolved(
            "pub mod a { type Result<T> = std::result::Result<T, ()>; pub struct Vec; }\n\
             pub mod b { type Result<T> = std::result::Result<T, ()>; pub struct Config; }\n\
             pub mod c { pub struct C { pub r: Result<u8, String>, pub v: Vec<u8>, pub k: Config } }\n",
        );
        let prelude = |item: &str| (None, Some(item.to_owned()));
        assert_eq!(found(&krate, "c", "Result"), prelude("std::result::Result"));
        assert_eq!(found(&krate, "c", "Vec"), prelude("std::vec::Vec"));
        assert_eq!(
            found(&krate, "c", "Config"),
            (Some("b::Config".to_owned()), None),
            "a name outside the prelude still falls back to the one in the crate"
        );
        assert_eq!(tally, Tally::default());
    }

    #[test]
    fn a_declaration_a_use_or_a_glob_in_the_module_wins_over_the_prelude() {
        let (_tmp, krate, tally) = resolved(
            "pub mod own { pub type Result<T> = std::result::Result<T, ()>; \
             pub struct O { pub r: Result<u8> } }\n\
             pub mod used { use crate::own::Result; pub struct U { pub r: Result<u8> } }\n\
             pub mod starred { use crate::own::*; pub struct S { pub r: Result<u8> } }\n",
        );
        for module in ["own", "used", "starred"] {
            assert_eq!(
                found(&krate, module, "Result"),
                (Some("own::Result".to_owned()), None),
                "{module}"
            );
        }
        assert_eq!(tally, Tally::default());
    }

    #[test]
    fn the_futures_of_the_2024_prelude_resolve_without_a_use() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            "pub fn run<F: Future<Output = ()>, I: IntoFuture>(f: F, i: I) {}\n",
        )
        .unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        let tally = resolve(&mut krate);
        let item = |name: &str| {
            krate
                .refs
                .iter()
                .find(|r| r.path.first().is_some_and(|first| first == name))
                .and_then(|r| r.canonical.as_deref())
                .and_then(item_path)
        };
        assert_eq!(item("Future").as_deref(), Some("std::future::Future"));
        assert_eq!(
            item("IntoFuture").as_deref(),
            Some("std::future::IntoFuture")
        );
        assert_eq!(tally, Tally::default());
    }
}
