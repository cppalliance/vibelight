//! The module order: one section per module, holding its declarations, and
//! a Levels section that indexes every declaration by level.

use crate::document::{Builder, Inline, Item, Node, anchor};
use crate::layout_items::module_block;
use crate::model::{DeclId, ModId, path_order};
use crate::order::Order;

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

    /// Adds the Levels section, which lists the declarations of each level
    /// and then the test-support and unconnected ones, and returns its
    /// contents entry.
    pub(crate) fn levels_index(&mut self, order: &Order) -> Item {
        let mut groups: Vec<(String, &[DeclId])> = order
            .levels
            .iter()
            .enumerate()
            .map(|(index, level)| {
                let text = if index == 0 {
                    "Level 1: top-level types".to_owned()
                } else {
                    format!("Level {}", index + 1)
                };
                (text, level.as_slice())
            })
            .collect();
        groups.push(("Test support".to_owned(), &order.test_support));
        groups.push(("Unconnected types".to_owned(), &order.unconnected));
        let items = groups
            .into_iter()
            .filter(|(_, decls)| !decls.is_empty())
            .map(|(text, decls)| Item {
                inlines: vec![Inline::Text(text)],
                children: self.decl_items(decls),
            })
            .collect();
        self.heading(2, "levels", "Levels");
        self.nodes.push(Node::List {
            class: "levels",
            items,
        });
        Item {
            inlines: vec![anchor("Levels", "levels")],
            children: Vec::new(),
        }
    }

    /// Adds one H2 per module, in path order, with the module's docs and
    /// trailing block and then an H3 for each of its declarations, by level
    /// and then in the levels' order, and returns their contents entries.
    pub(crate) fn module_sections(&mut self, order: &Order, modules: &[ModId]) -> Vec<Item> {
        let sequence: Vec<DeclId> = order
            .levels
            .iter()
            .flatten()
            .chain(&order.test_support)
            .chain(&order.unconnected)
            .copied()
            .collect();
        let mut contents = Vec::new();
        for &m in modules {
            let decls: Vec<DeclId> = sequence
                .iter()
                .copied()
                .filter(|&d| self.krate.decls[d].module == m)
                .collect();
            self.module_head(m, 2);
            if let Some(block) = module_block(self.krate, m, self.notation) {
                self.code(block);
            }
            for &d in &decls {
                self.decl(d);
            }
            contents.push(Item {
                inlines: vec![anchor(&self.mod_heading(m), &self.mod_ids[m])],
                children: self.decl_items(&decls),
            });
        }
        contents
    }

    fn decl_items(&self, decls: &[DeclId]) -> Vec<Item> {
        decls
            .iter()
            .map(|&d| Item {
                inlines: vec![anchor(&self.decl_heading(d), &self.decl_ids[d])],
                children: Vec::new(),
            })
            .collect()
    }
}
