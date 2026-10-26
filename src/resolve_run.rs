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
    let found: Vec<Vec<Option<CrossRef>>> = {
        let shared: &[Crate] = krates;
        let world = World {
            scopes: shared.iter().map(Scope::new).collect(),
            deps,
        };
        shared
            .iter()
            .enumerate()
            .map(|(c, krate)| krate.refs.iter().map(|r| world.foreign(c, r)).collect())
            .collect()
    };
    for (krate, found) in krates.iter_mut().zip(found) {
        for (r, foreign) in krate.refs.iter_mut().zip(found) {
            r.foreign = foreign;
        }
    }
}

impl World<'_> {
    fn foreign(&self, c: usize, r: &TypeRef) -> Option<CrossRef> {
        if r.target.is_some() || r.value.is_some() || self.deps[c].is_empty() {
            return None;
        }
        let first = r.path.first()?;
        if !r.global && (r.generics.contains(first) || first == "Self") {
            return None;
        }
        let mut seen = Seen::new();
        let values = r.may_be_value;
        let found = if r.global {
            self.extern_path(c, &r.path, &mut seen, values)
        } else {
            self.walk(c, r.module, &r.path, &mut seen, values)
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
    fn extern_path(&self, c: usize, path: &[String], seen: &mut Seen, values: bool) -> Option<Hop> {
        let (first, rest) = path.split_first()?;
        let &dep = self.deps[c].get(first)?;
        self.follow(Hop::Module(dep, 0), rest, seen, values)
    }

    /// Walks a path from `module` of crate `c` as `resolve` does, except
    /// that a first name not found there may be a dependency, so any step
    /// can lead into another crate. With `values` set, the last name may be
    /// a function, constant, or static.
    fn walk(
        &self,
        c: usize,
        module: ModId,
        path: &[String],
        seen: &mut Seen,
        values: bool,
    ) -> Option<Hop> {
        match path.first()?.as_str() {
            "crate" => self.follow(Hop::Module(c, 0), &path[1..], seen, values),
            "self" => self.follow(Hop::Module(c, module), &path[1..], seen, values),
            "super" => {
                let mut current = module;
                let mut i = 0;
                while path.get(i).map(String::as_str) == Some("super") {
                    current = self.scopes[c].parents[current]?;
                    i += 1;
                }
                self.follow(Hop::Module(c, current), &path[i..], seen, values)
            }
            first => match self.lookup_in(c, module, first, seen, values && path.len() == 1) {
                Some(found) => self.follow(found, &path[1..], seen, values),
                None => self.extern_path(c, path, seen, values),
            },
        }
    }

    /// Looks up each name of `rest` in turn, starting from `at`.
    fn follow(&self, mut at: Hop, rest: &[String], seen: &mut Seen, values: bool) -> Option<Hop> {
        for (i, name) in rest.iter().enumerate() {
            let Hop::Module(c, module) = at else {
                return None;
            };
            at = self.lookup_in(c, module, name, seen, values && i + 1 == rest.len())?;
        }
        Some(at)
    }

    /// Looks up one name in a module: its `use` items, then its globs, then
    /// its declarations, then, with `values` set, its functions, constants,
    /// and statics, then its child modules.
    fn lookup_in(
        &self,
        c: usize,
        module: ModId,
        name: &str,
        seen: &mut Seen,
        values: bool,
    ) -> Option<Hop> {
        if !seen.insert((c, module, name.to_owned(), values)) {
            return None;
        }
        let scope = &self.scopes[c];
        let uses = &scope.krate.modules[module].uses;
        if let Some(u) = uses.iter().find(|u| u.name.as_deref() == Some(name)) {
            return if u.global {
                self.extern_path(c, &u.path, seen, values)
            } else {
                self.walk(c, module, &u.path, seen, values)
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
                    if let Some(found) = self.lookup_in(k, m, name, seen, values) {
                        return Some(found);
                    }
                }
            }
        }
        if let Some(&id) = scope.decls[module].get(name).and_then(|ids| ids.first()) {
            return Some(Hop::Decl(c, id));
        }
        if values {
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
