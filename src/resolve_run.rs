//! Resolves the names a crate writes into the other crates of a run. A path
//! that starts with a dependency's name walks into that crate's modules,
//! through its `use` items and globs, to the declaration, function,
//! constant, or static it names.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Crate, CrossRef, DeclId, ModId, Reach, TypeRef, ValueRef};
use crate::resolve::Scope;

/// Where a path leads in a run: a declaration, a function, constant, or
/// static, or a module of some crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hop {
    Decl(usize, DeclId),
    Value(usize, ValueRef),
    Module(usize, ModId),
}

/// The (crate, module, name, may be a value) lookups one query has made,
/// which stops `use` and glob cycles inside a crate and across crates.
type Seen = BTreeSet<(usize, ModId, String, bool)>;

struct World<'a> {
    scopes: Vec<Scope<'a>>,
    deps: &'a [BTreeMap<String, usize>],
}

/// Fills in the foreign item of every name that reaches another crate of
/// the run. `deps[c]` maps each dependency name that crate `c` may write to
/// that dependency's index in `krates`. A name already resolved inside its
/// own crate keeps that target, and `canonical` never changes.
pub(crate) fn link(krates: &mut [Crate], deps: &[BTreeMap<String, usize>]) {
    let found = cross_refs(krates, deps);
    for (krate, found) in krates.iter_mut().zip(found) {
        for (r, foreign) in krate.refs.iter_mut().zip(found) {
            r.foreign = foreign;
        }
    }
}

/// Returns, for each crate and each of its names, the item in another
/// crate of the run that the name reaches through `deps`, as `link` would
/// fill it in, and changes no crate. A name resolved inside its own crate
/// reaches nothing here.
pub(crate) fn cross_refs(
    krates: &[Crate],
    deps: &[BTreeMap<String, usize>],
) -> Vec<Vec<Option<CrossRef>>> {
    let world = World {
        scopes: krates.iter().map(Scope::new).collect(),
        deps,
    };
    krates
        .iter()
        .enumerate()
        .map(|(c, krate)| krate.refs.iter().map(|r| world.foreign(c, r)).collect())
        .collect()
}

impl World<'_> {
    fn foreign(&self, c: usize, r: &TypeRef) -> Option<CrossRef> {
        if r.decl().is_some() || r.value().is_some() || self.deps[c].is_empty() {
            return None;
        }
        let first = r.path.first()?;
        if !r.global && (r.generics.contains(first) || first == "Self") {
            return None;
        }
        let mut seen = Seen::new();
        let may_be_value = r.may_be_value;
        let found = if r.global {
            self.extern_path(c, &r.path, &mut seen, may_be_value)
        } else {
            self.walk(c, r.module, &r.path, &mut seen, may_be_value)
        };
        match found? {
            Hop::Decl(krate, decl) if krate != c => Some(CrossRef {
                krate,
                reach: Reach::Decl(decl),
            }),
            Hop::Value(krate, value) if krate != c => Some(CrossRef {
                krate,
                reach: Reach::Value(value),
            }),
            _ => None,
        }
    }

    /// Walks a path whose first name is one of crate `c`'s dependencies.
    fn extern_path(
        &self,
        c: usize,
        path: &[String],
        seen: &mut Seen,
        may_be_value: bool,
    ) -> Option<Hop> {
        let (first, rest) = path.split_first()?;
        let &dep = self.deps[c].get(first)?;
        self.follow(Hop::Module(dep, 0), rest, seen, may_be_value)
    }

    /// Walks a path from `module` of crate `c` as `resolve` does, except
    /// that a first name not found there may be a dependency, so any step
    /// can lead into another crate. With `may_be_value` set, the last name
    /// may be a function, constant, or static.
    fn walk(
        &self,
        c: usize,
        module: ModId,
        path: &[String],
        seen: &mut Seen,
        may_be_value: bool,
    ) -> Option<Hop> {
        match path.first()?.as_str() {
            "crate" => self.follow(Hop::Module(c, 0), &path[1..], seen, may_be_value),
            "self" => self.follow(Hop::Module(c, module), &path[1..], seen, may_be_value),
            "super" => {
                let mut current = module;
                let mut i = 0;
                while path.get(i).map(String::as_str) == Some("super") {
                    current = self.scopes[c].parents[current]?;
                    i += 1;
                }
                self.follow(Hop::Module(c, current), &path[i..], seen, may_be_value)
            }
            first => {
                match self.lookup_in(c, module, first, seen, may_be_value && path.len() == 1) {
                    Some(found) => self.follow(found, &path[1..], seen, may_be_value),
                    None => self.extern_path(c, path, seen, may_be_value),
                }
            }
        }
    }

    /// Looks up each name of `rest` in turn, starting from `at`.
    fn follow(
        &self,
        mut at: Hop,
        rest: &[String],
        seen: &mut Seen,
        may_be_value: bool,
    ) -> Option<Hop> {
        for (i, name) in rest.iter().enumerate() {
            let Hop::Module(c, module) = at else {
                return None;
            };
            at = self.lookup_in(c, module, name, seen, may_be_value && i + 1 == rest.len())?;
        }
        Some(at)
    }

    /// Looks up one name in a module in the order of `Scope::lookup_in`, but
    /// across the crates of the run.
    fn lookup_in(
        &self,
        c: usize,
        module: ModId,
        name: &str,
        seen: &mut Seen,
        may_be_value: bool,
    ) -> Option<Hop> {
        if !seen.insert((c, module, name.to_owned(), may_be_value)) {
            return None;
        }
        let scope = &self.scopes[c];
        let uses = &scope.krate.modules[module].uses;
        if let Some(u) = uses.iter().find(|u| u.name.as_deref() == Some(name)) {
            return if u.global {
                self.extern_path(c, &u.path, seen, may_be_value)
            } else {
                self.walk(c, module, &u.path, seen, may_be_value)
            };
        }
        for u in uses.iter().filter(|u| u.name.is_none()) {
            let source = if u.global {
                self.extern_path(c, &u.path, seen, false)
            } else {
                self.walk(c, module, &u.path, seen, false)
            };
            if let Some(Hop::Module(k, m)) = source {
                if (k, m) != (c, module) {
                    if let Some(found) = self.lookup_in(k, m, name, seen, may_be_value) {
                        return Some(found);
                    }
                }
            }
        }
        if let Some(&id) = scope.decls[module].get(name).and_then(|ids| ids.first()) {
            return Some(Hop::Decl(c, id));
        }
        if may_be_value {
            let first = scope.values[module]
                .get(name)
                .and_then(|items| items.first());
            if let Some(&item) = first {
                return Some(Hop::Value(c, ValueRef { module, item }));
            }
        }
        scope.children[module]
            .get(name)
            .and_then(|ids| ids.first())
            .map(|&id| Hop::Module(c, id))
    }
}
