//! What a module declares, counted for both outlines: its types, functions,
//! values, and re-exports, extern items by their kind, and the same items
//! by written visibility. Macro marks and unsupported items count for
//! nothing.

use std::collections::BTreeMap;

use crate::access::spelled;
use crate::model::{Crate, Foreign, ItemKind, ModId};

/// One module's counts.
pub(crate) struct Anatomy {
    /// Its structs, enums, unions, traits, and aliases; its functions; its
    /// constants and statics; and its re-exports.
    pub(crate) counts: [usize; 4],
    /// The same items by their written visibility, `pub(self)` for none.
    pub(crate) mix: BTreeMap<String, usize>,
}

/// Counts what module `m` declares.
pub(crate) fn of(krate: &Crate, m: ModId) -> Anatomy {
    let mut counts = [0; 4];
    let mut mix: BTreeMap<String, usize> = BTreeMap::new();
    let mut count = |slot: usize, vis| {
        counts[slot] += 1;
        *mix.entry(spelled(krate, vis)).or_default() += 1;
    };
    for decl in krate.decls.iter().filter(|d| d.module == m) {
        count(0, decl.vis);
    }
    for item in &krate.modules[m].items {
        let slot = match &item.kind {
            ItemKind::Foreign {
                item: Foreign::Type,
                ..
            } => 0,
            ItemKind::Fn(_)
            | ItemKind::Foreign {
                item: Foreign::Fn(_),
                ..
            } => 1,
            ItemKind::Const { .. } | ItemKind::Static { .. } | ItemKind::Foreign { .. } => 2,
            ItemKind::Reexport { .. } => 3,
            ItemKind::Mark(_) | ItemKind::Unsupported(_) => continue,
        };
        count(slot, item.vis);
    }
    Anatomy { counts, mix }
}
