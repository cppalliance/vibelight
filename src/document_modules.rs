//! The module order the HTML's body follows.

use std::cmp::Ordering;

use crate::document::Builder;
use crate::model::{ModId, path_order};

impl Builder<'_> {
    /// Returns every module in path order.
    pub(crate) fn sorted_modules(&self) -> Vec<ModId> {
        let mut out: Vec<ModId> = (0..self.krate.modules.len()).collect();
        out.sort_by(|&x, &y| self.module_order(x, y));
        out
    }

    /// Compares two modules by path, then by id.
    pub(crate) fn module_order(&self, x: ModId, y: ModId) -> Ordering {
        let modules = &self.krate.modules;
        path_order(&modules[x].path, &modules[y].path).then(self.mod_ids[x].cmp(&self.mod_ids[y]))
    }
}
