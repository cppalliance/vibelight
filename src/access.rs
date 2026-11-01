//! Effective visibility: how far each module, declaration, and module value
//! of a crate can be named from, through the modules around it and every
//! re-export that names it.

use crate::analysis_items::is_value;
use crate::model::{Crate, DeclId, ItemKind, ModId, Text, ValueRef, Vis};
use crate::surface::module_at;
use crate::typerefs::strip_raw;

/// Where a node can be named from: anywhere, or only inside one module and
/// the modules below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    /// Anywhere the crate can be named.
    World,
    /// Inside a module and the modules below it.
    Within(ModId),
}

/// How far each node of one crate can be named from: the widest access
/// over its own path and every re-export that names it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Accesses {
    pub(crate) modules: Vec<Access>,
    pub(crate) decls: Vec<Access>,
    /// Each module value's access, by module and then by item index.
    pub(crate) values: Vec<Vec<Access>>,
}

impl Accesses {
    /// Returns a module value's access.
    pub(crate) fn value(&self, at: ValueRef) -> Access {
        self.values[at.module][at.item]
    }
}

/// Returns the access a visibility written in module `m` gives: `pub`
/// reaches anywhere, `pub(crate)` the crate, `pub(super)` the parent's
/// subtree, `pub(in path)` the named module's, and no keyword `m`'s own.
pub(crate) fn written(krate: &Crate, vis: Vis, m: ModId) -> Access {
    match vis {
        Vis::Public => Access::World,
        Vis::Crate => Access::Within(0),
        Vis::Super => Access::Within(krate.parent_module(m).unwrap_or(0)),
        Vis::In(target) => Access::Within(target),
        Vis::Private => Access::Within(m),
    }
}

/// Tells whether module `outer` is module `inner` or holds it.
pub(crate) fn encloses(krate: &Crate, outer: ModId, inner: ModId) -> bool {
    let mut at = Some(inner);
    while let Some(m) = at {
        if m == outer {
            return true;
        }
        at = krate.parent_module(m);
    }
    false
}

/// Tells whether access `outer` reaches everywhere access `inner` does.
pub(crate) fn covers(krate: &Crate, outer: Access, inner: Access) -> bool {
    match (outer, inner) {
        (Access::World, _) => true,
        (Access::Within(_), Access::World) => false,
        (Access::Within(a), Access::Within(b)) => encloses(krate, a, b),
    }
}

/// Returns the narrower of two accesses, or `None` when neither reaches
/// everywhere the other does.
fn narrow(krate: &Crate, a: Access, b: Access) -> Option<Access> {
    if covers(krate, a, b) {
        Some(b)
    } else if covers(krate, b, a) {
        Some(a)
    } else {
        None
    }
}

/// Returns the narrowest access that reaches everywhere both do: the
/// lowest module that holds both, or anywhere when one of them is.
pub(crate) fn join(krate: &Crate, a: Access, b: Access) -> Access {
    let (Access::Within(x), Access::Within(y)) = (a, b) else {
        return Access::World;
    };
    let mut at = Some(x);
    while let Some(m) = at {
        if encloses(krate, m, y) {
            return Access::Within(m);
        }
        at = krate.parent_module(m);
    }
    Access::Within(0)
}

/// Returns how far every module, declaration, and module value of a crate
/// can be named from. The root is named anywhere. A node's access through
/// a path is the path's access narrowed by the node's written visibility,
/// and a re-export or glob names its targets with its own access narrowed
/// the same way, so no node is ever wider than it is written.
pub(crate) fn accesses(krate: &Crate) -> Accesses {
    let mut walk = Walk::new(krate);
    walk.slots.modules[0] = Some(Access::World);
    loop {
        walk.changed = false;
        for m in 0..krate.modules.len() {
            if let Some(here) = walk.slots.modules[m] {
                walk.visit(m, here);
            }
        }
        if !walk.changed {
            break;
        }
    }
    walk.finish()
}

/// What a re-export names inside its crate.
#[derive(Clone, Copy)]
enum Target {
    Module(ModId),
    Decl(DeclId),
    Value(ValueRef),
    /// Every item a glob of this module imports.
    Glob(ModId),
}

/// One node whose access the walk widens.
#[derive(Clone, Copy)]
enum Slot {
    Module(ModId),
    Decl(DeclId),
    Value(ValueRef),
}

/// The accesses found so far; `None` for a node no path reached yet.
struct Slots {
    modules: Vec<Option<Access>>,
    decls: Vec<Option<Access>>,
    values: Vec<Vec<Option<Access>>>,
}

/// The fixed-point walk over a crate's modules and re-exports.
struct Walk<'a> {
    krate: &'a Crate,
    /// The modules directly inside each module.
    children: Vec<Vec<ModId>>,
    /// The declarations of each module.
    decls_of: Vec<Vec<DeclId>>,
    slots: Slots,
    changed: bool,
}

/// The most re-exports a chain of re-exports is followed through.
const CHAIN: usize = 16;

impl<'a> Walk<'a> {
    fn new(krate: &'a Crate) -> Self {
        let n = krate.modules.len();
        let mut children = vec![Vec::new(); n];
        for (m, module) in krate.modules.iter().enumerate() {
            if let Some(parent) = module.parent {
                children[parent].push(m);
            }
        }
        let mut decls_of = vec![Vec::new(); n];
        for (d, decl) in krate.decls.iter().enumerate() {
            decls_of[decl.module].push(d);
        }
        Walk {
            krate,
            children,
            decls_of,
            slots: Slots {
                modules: vec![None; n],
                decls: vec![None; krate.decls.len()],
                values: (krate.modules.iter())
                    .map(|module| vec![None; module.items.len()])
                    .collect(),
            },
            changed: false,
        }
    }

    /// Widens what module `m` declares and re-exports, from its access.
    fn visit(&mut self, m: ModId, here: Access) {
        let krate = self.krate;
        for i in 0..self.children[m].len() {
            let c = self.children[m][i];
            let reach = narrow(krate, here, written(krate, krate.modules[c].vis, m));
            self.widen(Slot::Module(c), reach);
        }
        for i in 0..self.decls_of[m].len() {
            let d = self.decls_of[m][i];
            let reach = narrow(krate, here, written(krate, krate.decls[d].vis, m));
            self.widen(Slot::Decl(d), reach);
        }
        for (k, item) in krate.modules[m].items.iter().enumerate() {
            let reach = narrow(krate, here, written(krate, item.vis, m));
            self.widen(Slot::Value(ValueRef { module: m, item: k }), reach);
            if let (ItemKind::Reexport { text, .. }, Some(reach)) = (&item.kind, reach) {
                if let Some(target) = target(krate, m, text) {
                    self.reexport(m, target, reach, 0);
                }
            }
        }
    }

    /// Widens what a re-export written in module `m` names, from the
    /// re-export's own access.
    fn reexport(&mut self, m: ModId, target: Target, reach: Access, depth: usize) {
        let krate = self.krate;
        match target {
            Target::Module(t) => {
                let parent = krate.parent_module(t).unwrap_or(0);
                let own = written(krate, krate.modules[t].vis, parent);
                self.widen(Slot::Module(t), narrow(krate, reach, own));
            }
            Target::Decl(d) => {
                let decl = &krate.decls[d];
                let own = written(krate, decl.vis, decl.module);
                self.widen(Slot::Decl(d), narrow(krate, reach, own));
            }
            Target::Value(at) => self.value(at, reach, depth),
            Target::Glob(g) => self.glob(m, g, reach, depth),
        }
    }

    /// Widens a module value that a re-export or a glob names, and follows
    /// it when it is a re-export itself.
    fn value(&mut self, at: ValueRef, reach: Access, depth: usize) {
        let krate = self.krate;
        let item = &krate.modules[at.module].items[at.item];
        let Some(reach) = narrow(krate, reach, written(krate, item.vis, at.module)) else {
            return;
        };
        self.widen(Slot::Value(at), Some(reach));
        if let ItemKind::Reexport { text, .. } = &item.kind {
            if depth < CHAIN {
                if let Some(next) = target(krate, at.module, text) {
                    self.reexport(at.module, next, reach, depth + 1);
                }
            }
        }
    }

    /// Widens the items of module `g` that a glob written in module `m`
    /// imports: those whose visibility reaches `m`.
    fn glob(&mut self, m: ModId, g: ModId, reach: Access, depth: usize) {
        let krate = self.krate;
        let seen = |access: Access| covers(krate, access, Access::Within(m));
        for i in 0..self.children[g].len() {
            let c = self.children[g][i];
            let own = written(krate, krate.modules[c].vis, g);
            if seen(own) {
                self.widen(Slot::Module(c), narrow(krate, reach, own));
            }
        }
        for i in 0..self.decls_of[g].len() {
            let d = self.decls_of[g][i];
            let own = written(krate, krate.decls[d].vis, g);
            if seen(own) {
                self.widen(Slot::Decl(d), narrow(krate, reach, own));
            }
        }
        for (k, item) in krate.modules[g].items.iter().enumerate() {
            let named = is_value(&item.kind) || matches!(item.kind, ItemKind::Reexport { .. });
            if named && seen(written(krate, item.vis, g)) && depth < CHAIN {
                self.value(ValueRef { module: g, item: k }, reach, depth + 1);
            }
        }
    }

    /// Joins `access` into a node's access, and notes a change.
    fn widen(&mut self, slot: Slot, access: Option<Access>) {
        let Some(access) = access else {
            return;
        };
        let krate = self.krate;
        let cell = match slot {
            Slot::Module(m) => &mut self.slots.modules[m],
            Slot::Decl(d) => &mut self.slots.decls[d],
            Slot::Value(at) => &mut self.slots.values[at.module][at.item],
        };
        let next = cell.map_or(access, |old| join(krate, old, access));
        if *cell != Some(next) {
            *cell = Some(next);
            self.changed = true;
        }
    }

    /// Returns the accesses, with a node that no path reached private to
    /// its own module.
    fn finish(self) -> Accesses {
        let krate = self.krate;
        Accesses {
            modules: (self.slots.modules.into_iter().enumerate())
                .map(|(m, access)| access.unwrap_or(Access::Within(m)))
                .collect(),
            decls: (self.slots.decls.into_iter().enumerate())
                .map(|(d, access)| access.unwrap_or(Access::Within(krate.decls[d].module)))
                .collect(),
            values: (self.slots.values.into_iter().enumerate())
                .map(|(m, items)| {
                    let own = Access::Within(m);
                    items.into_iter().map(|a| a.unwrap_or(own)).collect()
                })
                .collect(),
        }
    }
}

/// Returns what a re-export written in module `m` names inside the crate:
/// a module, a declaration, a module value, or, for `path::*`, a glob of a
/// module.
fn target(krate: &Crate, m: ModId, text: &Text) -> Option<Target> {
    let Some(id) = text.refs().next() else {
        let shown = text.visible(&krate.refs);
        let path = shown.strip_suffix("::*")?;
        let global = path.starts_with("::");
        let segments: Vec<String> = (path.trim_start_matches("::").split("::"))
            .map(strip_raw)
            .collect();
        return module_at(krate, m, &segments, global).map(Target::Glob);
    };
    let reference = &krate.refs[id];
    if let Some(d) = reference.decl() {
        return Some(Target::Decl(d));
    }
    if let Some(at) = reference.value() {
        return Some(Target::Value(at));
    }
    let (from, path, global) = (reference.module, &reference.path, reference.global);
    if let Some(t) = module_at(krate, from, path, global) {
        return Some(Target::Module(t));
    }
    let (name, parent) = path.split_last()?;
    let at = module_at(krate, from, parent, global)?;
    let name = strip_raw(name);
    if let Some(d) = (0..krate.decls.len())
        .find(|&d| krate.decls[d].module == at && strip_raw(&krate.decls[d].name) == name)
    {
        return Some(Target::Decl(d));
    }
    let k = krate.modules[at].items.iter().position(|item| {
        strip_raw(&item.name) == name
            && (is_value(&item.kind) || matches!(item.kind, ItemKind::Reexport { .. }))
    })?;
    Some(Target::Value(ValueRef {
        module: at,
        item: k,
    }))
}
