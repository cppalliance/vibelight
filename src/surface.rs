//! Finds a library's public surface: every item a user of the crate can
//! name, by the path they name it with.

use crate::model::{Crate, DeclId, Foreign, ItemKind, ModId, Seg, Text, Vis, path_order};
use crate::test_support::is_test_support;
use crate::typerefs::strip_raw;

/// One path of the public surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The path a user of the crate writes, below the crate root.
    pub(crate) path: Vec<String>,
    pub(crate) kind: Kind,
    /// Whether the path names test support (`test_support::mark`).
    pub(crate) test_support: bool,
}

/// What a surface path names, and so where its entry links.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Module(ModId),
    Decl(DeclId),
    /// A function, which links to the module that declares it.
    Function(ModId),
    /// A constant, a static, or an extern type, which links to the module
    /// that declares it.
    Value(ModId),
    /// A `#[macro_export]` macro, which links to the module that defines
    /// it.
    Macro(ModId),
    /// A re-export of another crate's item, or one the walk cannot place:
    /// the path as written, with no link.
    Unlinked(String),
}

/// What a `use` path names inside the crate.
enum Found {
    Decl(DeclId),
    Module(ModId),
    /// A function, with whether it is test support.
    Function(ModId, bool),
    /// A constant, a static, or an extern type, with whether it is test
    /// support.
    Value(ModId, bool),
}

/// Walks from the crate root through public modules and public re-exports,
/// and returns every public path in path order.
pub(crate) fn walk(krate: &Crate) -> Vec<Entry> {
    let mut walk = Walk {
        krate,
        out: Vec::new(),
        stack: Vec::new(),
        inherited: false,
    };
    walk.module(0, &[]);
    for (name, m) in &krate.exported_macros {
        walk.push(&[], name, Kind::Macro(*m), krate.modules[*m].test_support);
    }
    let mut out = walk.out;
    out.sort_by(|a, b| path_order(&a.path, &b.path).then(a.kind.cmp(&b.kind)));
    out.dedup();
    out
}

struct Walk<'a> {
    krate: &'a Crate,
    out: Vec<Entry>,
    /// The modules being walked, which guards against re-export cycles.
    stack: Vec<ModId>,
    /// Whether everything the walk now adds is test support, as inside a
    /// test-support module or a test-support glob.
    inherited: bool,
}

impl Walk<'_> {
    /// Adds what module `m` makes public, each path starting with `prefix`.
    fn module(&mut self, m: ModId, prefix: &[String]) {
        if self.stack.contains(&m) {
            return;
        }
        self.stack.push(m);
        let krate = self.krate;
        let outer = self.inherited;
        self.inherited |= krate.modules[m].test_support;
        for (d, decl) in krate.decls.iter().enumerate() {
            if decl.module == m && decl.vis == Vis::Public {
                self.push(prefix, &decl.name, Kind::Decl(d), decl.test_support);
            }
        }
        for child in children(krate, m) {
            if krate.modules[child].vis == Vis::Public {
                let name = krate.modules[child]
                    .path
                    .last()
                    .cloned()
                    .unwrap_or_default();
                let support = krate.modules[child].test_support;
                self.push(prefix, &name, Kind::Module(child), support);
                self.module(child, &joined(prefix, &name));
            }
        }
        for item in &krate.modules[m].items {
            if item.vis != Vis::Public {
                continue;
            }
            let support = is_test_support(&item.meta.cfgs);
            let kind = match &item.kind {
                ItemKind::Fn(_)
                | ItemKind::Foreign {
                    item: Foreign::Fn(_),
                    ..
                } => Kind::Function(m),
                ItemKind::Const { .. } | ItemKind::Static { .. } | ItemKind::Foreign { .. } => {
                    Kind::Value(m)
                }
                ItemKind::Reexport { text, .. } => {
                    self.reexport(m, prefix, &item.name, text, support);
                    continue;
                }
                ItemKind::Mark(_) | ItemKind::Unsupported(_) => continue,
            };
            self.push(prefix, &item.name, kind, support);
        }
        self.inherited = outer;
        self.stack.pop();
    }

    /// Adds what a re-export in module `m` names under its new name, and
    /// walks a re-exported module under that name. A glob adds the target
    /// module's public items under `prefix` itself. An entry is test
    /// support when the re-export is or when what it names is.
    fn reexport(&mut self, m: ModId, prefix: &[String], name: &str, text: &Text, support: bool) {
        let krate = self.krate;
        let Some(id) = text.0.iter().find_map(|seg| match seg {
            Seg::Ref(id) => Some(*id),
            Seg::Plain(_) => None,
        }) else {
            let outer = self.inherited;
            self.inherited |= support;
            self.glob(m, prefix, &text.visible(&krate.refs));
            self.inherited = outer;
            return;
        };
        let reference = &krate.refs[id];
        let found = match reference.decl() {
            Some(d) => Some(Found::Decl(d)),
            None => lookup(
                krate,
                reference.module,
                &reference.path,
                reference.global,
                &mut Vec::new(),
            ),
        };
        match found {
            Some(Found::Decl(d)) => {
                let support = support || krate.decls[d].test_support;
                self.push(prefix, name, Kind::Decl(d), support);
            }
            Some(Found::Module(target)) => {
                let support = support || krate.modules[target].test_support;
                self.push(prefix, name, Kind::Module(target), support);
                let outer = self.inherited;
                self.inherited |= support;
                self.module(target, &joined(prefix, name));
                self.inherited = outer;
            }
            Some(Found::Function(at, own)) => {
                self.push(prefix, name, Kind::Function(at), support || own);
            }
            Some(Found::Value(at, own)) => self.push(prefix, name, Kind::Value(at), support || own),
            None => {
                let kind = Kind::Unlinked(reference.text.clone());
                self.push(prefix, name, kind, support);
            }
        }
    }

    /// Adds the public items of the module a glob such as `inner::*` names,
    /// or the glob itself, unlinked, when it names no module of the crate.
    fn glob(&mut self, m: ModId, prefix: &[String], written: &str) {
        let path = written.strip_suffix("::*").unwrap_or(written);
        let global = path.starts_with("::");
        let segments: Vec<String> = path
            .trim_start_matches("::")
            .split("::")
            .map(strip_raw)
            .collect();
        match module_at(self.krate, m, &segments, global) {
            Some(target) => self.module(target, prefix),
            None => self.push(prefix, "*", Kind::Unlinked(written.to_owned()), false),
        }
    }

    fn push(&mut self, prefix: &[String], name: &str, kind: Kind, support: bool) {
        self.out.push(Entry {
            path: joined(prefix, name),
            kind,
            test_support: self.inherited || support,
        });
    }
}

fn joined(prefix: &[String], name: &str) -> Vec<String> {
    let mut path = prefix.to_vec();
    path.push(name.to_owned());
    path
}

/// Returns the modules directly inside module `m`.
fn children(krate: &Crate, m: ModId) -> Vec<ModId> {
    let parent = &krate.modules[m].path;
    (0..krate.modules.len())
        .filter(|&c| {
            let path = &krate.modules[c].path;
            path.len() == parent.len() + 1 && path.starts_with(parent)
        })
        .collect()
}

/// Returns the module a `use` path written in module `from` names, through
/// `crate`, `self`, `super`, and child modules.
pub(crate) fn module_at(
    krate: &Crate,
    from: ModId,
    path: &[String],
    global: bool,
) -> Option<ModId> {
    if global {
        return None;
    }
    let mut at = from;
    for (index, segment) in path.iter().enumerate() {
        at = match segment.as_str() {
            "crate" if index == 0 => 0,
            "self" if index == 0 => at,
            "super" => krate.parent_module(at)?,
            name => *children(krate, at).iter().find(|&&c| {
                krate.modules[c].path.last().map(|s| strip_raw(s)) == Some(strip_raw(name))
            })?,
        };
    }
    Some(at)
}

/// Finds what a `use` path written in module `from` names: a declaration,
/// a module, a function, a constant, a static, or an extern type of the
/// crate. A name the target module itself re-exports is followed, with
/// `seen` guarding against cycles.
fn lookup(
    krate: &Crate,
    from: ModId,
    path: &[String],
    global: bool,
    seen: &mut Vec<(ModId, String)>,
) -> Option<Found> {
    if let Some(m) = module_at(krate, from, path, global) {
        return Some(Found::Module(m));
    }
    let (name, parent) = path.split_last()?;
    let m = module_at(krate, from, parent, global)?;
    let name = strip_raw(name);
    if seen.contains(&(m, name.clone())) {
        return None;
    }
    seen.push((m, name.clone()));
    if let Some(d) = (0..krate.decls.len())
        .find(|&d| krate.decls[d].module == m && strip_raw(&krate.decls[d].name) == name)
    {
        return Some(Found::Decl(d));
    }
    for item in krate.modules[m]
        .items
        .iter()
        .filter(|i| strip_raw(&i.name) == name)
    {
        let support = krate.modules[m].test_support || is_test_support(&item.meta.cfgs);
        match &item.kind {
            ItemKind::Fn(_)
            | ItemKind::Foreign {
                item: Foreign::Fn(_),
                ..
            } => return Some(Found::Function(m, support)),
            ItemKind::Const { .. } | ItemKind::Static { .. } | ItemKind::Foreign { .. } => {
                return Some(Found::Value(m, support));
            }
            ItemKind::Reexport { text, .. } => {
                let next = text.0.iter().find_map(|seg| match seg {
                    Seg::Ref(id) => Some(&krate.refs[*id]),
                    Seg::Plain(_) => None,
                });
                if let Some(next) = next {
                    if let Some(d) = next.decl() {
                        return Some(Found::Decl(d));
                    }
                    if let Some(found) = lookup(krate, next.module, &next.path, next.global, seen) {
                        return Some(found);
                    }
                }
            }
            ItemKind::Mark(_) | ItemKind::Unsupported(_) => {}
        }
    }
    None
}
