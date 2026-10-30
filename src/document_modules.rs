//! The module order the HTML's Modules section follows.

use crate::document::Builder;
use crate::model::{ModId, path_order};

impl Builder<'_> {
    /// Returns every module in path order.
    pub(crate) fn sorted_modules(&self) -> Vec<ModId> {
        let modules = &self.krate.modules;
        let mut out: Vec<ModId> = (0..modules.len()).collect();
        out.sort_by(|&x, &y| {
            path_order(&modules[x].path, &modules[y].path)
                .then(self.mod_ids[x].cmp(&self.mod_ids[y]))
        });
        out
    }
}
