//! Finds a library's public surface: every item a user of the crate can
//! name, by the path they name it with.

use crate::model::{Crate, DeclId, Foreign, ItemKind, ModId, Seg, Text, Vis, path_order};
use crate::typerefs::strip_raw;

/// One path of the public surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The path a user of the crate writes, below the crate root.
    pub(crate) path: Vec<String>,
    pub(crate) kind: Kind,
}

/// What a surface path names, and so where its entry links.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Module(ModId),
    Decl(DeclId),
    /// A function, which links to the module that declares it.
    Function(ModId),
    /// A constant, a static, or an extern type, which links to the module
    /// that declares it.
    Value(ModId),
}

/// Walks from the crate root through public modules and public re-exports,
/// and returns every public path in path order.
pub(crate) fn walk(krate: &Crate) -> Vec<Entry> {
    let mut walk = Walk {
        krate,
        out: Vec::new(),
        stack: Vec::new(),
    };
    walk.module(0, &[]);
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
}

impl Walk<'_> {
    /// Adds what module `m` makes public, each path starting with `prefix`.
    fn module(&mut self, m: ModId, prefix: &[String]) {
        if self.stack.contains(&m) {
            return;
        }
        self.stack.push(m);
        let krate = self.krate;
        for (d, decl) in krate.decls.iter().enumerate() {
            if decl.module == m && decl.vis == Vis::Public {
                self.push(prefix, &decl.name, Kind::Decl(d));
            }
        }
        for child in children(krate, m) {
            if krate.modules[child].vis == Vis::Public {
                let name = krate.modules[child]
                    .path
                    .last()
                    .cloned()
                    .unwrap_or_default();
                self.push(prefix, &name, Kind::Module(child));
                self.module(child, &joined(prefix, &name));
            }
        }
        for item in &krate.modules[m].items {
            if item.vis != Vis::Public {
                continue;
            }
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
                    self.reexport(prefix, &item.name, text);
                    continue;
                }
                ItemKind::Mark(_) | ItemKind::Unsupported(_) => continue,
            };
            self.push(prefix, &item.name, kind);
        }
        self.stack.pop();
    }

    /// Adds a re-exported type, alias, or module under its new name, and
    /// walks a re-exported module under that name.
    fn reexport(&mut self, prefix: &[String], name: &str, text: &Text) {
        let krate = self.krate;
        let Some(id) = text.0.iter().find_map(|seg| match seg {
            Seg::Ref(id) => Some(*id),
            Seg::Plain(_) => None,
        }) else {
            return;
        };
        let reference = &krate.refs[id];
        if let Some(d) = reference.target {
            self.push(prefix, name, Kind::Decl(d));
        } else if let Some(m) =
            module_at(krate, reference.module, &reference.path, reference.global)
        {
            self.push(prefix, name, Kind::Module(m));
            self.module(m, &joined(prefix, name));
        }
    }

    fn push(&mut self, prefix: &[String], name: &str, kind: Kind) {
        self.out.push(Entry {
            path: joined(prefix, name),
            kind,
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
            "super" => parent(krate, at)?,
            name => *children(krate, at).iter().find(|&&c| {
                krate.modules[c].path.last().map(|s| strip_raw(s)) == Some(strip_raw(name))
            })?,
        };
    }
    Some(at)
}

fn parent(krate: &Crate, m: ModId) -> Option<ModId> {
    let path = &krate.modules[m].path;
    let (_, up) = path.split_last()?;
    (0..krate.modules.len()).find(|&p| krate.modules[p].path == up)
}
