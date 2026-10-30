//! Reads function bodies: the type names they write, which keep a type out
//! of the HTML's Unconnected types, the declarations that nothing names, and
//! name nothing else, and the fields a method reaches through `self`, which
//! the detectors group.

use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprField, Macro, Path};

use crate::collect::{Collector, Here};
use crate::model::{BodyName, BodyOwner, TypeRef};
use crate::typerefs::strip_raw;

impl Collector<'_> {
    /// Records each path `block` writes, and each such path without its last
    /// segment, which names the type of an associated item such as
    /// `Listed::new`. Each resolves like any other name.
    pub(crate) fn body_names(&mut self, here: &Here, block: &Block, owner: BodyOwner) {
        let mut paths = Paths::default();
        paths.visit_block(block);
        for (global, path) in paths.found {
            let name = self.refs.len();
            self.refs.push(TypeRef {
                text: path.join("::"),
                path,
                global,
                module: here.module,
                generics: here.generics.clone(),
                comment: true,
                may_be_value: false,
                res: None,
                written: None,
                foreign: None,
            });
            self.body_names.push(BodyName { name, owner });
        }
    }
}

/// The paths a body writes, each once. Macro arguments are tokens, so their
/// paths are not read.
#[derive(Default)]
struct Paths {
    found: BTreeSet<(bool, Vec<String>)>,
}

impl<'ast> Visit<'ast> for Paths {
    fn visit_path(&mut self, path: &'ast Path) {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|s| strip_raw(&s.ident.to_string()))
            .collect();
        if segments.first().is_some_and(|s| s != "Self" && s != "self") {
            let global = path.leading_colon.is_some();
            if segments.len() > 1 {
                self.found
                    .insert((global, segments[..segments.len() - 1].to_vec()));
            }
            self.found.insert((global, segments));
        }
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, _: &'ast Macro) {}
}

/// Returns the fields a method body reaches through `self`, such as `tasks`
/// in `self.tasks.len()`, sorted, each once. Macro arguments are not read.
pub(crate) fn self_fields(block: &Block) -> Vec<String> {
    let mut fields = SelfFields::default();
    fields.visit_block(block);
    fields.found.into_iter().collect()
}

#[derive(Default)]
struct SelfFields {
    found: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for SelfFields {
    fn visit_expr_field(&mut self, node: &'ast ExprField) {
        let on_self = matches!(&*node.base, Expr::Path(base) if base.path.is_ident("self"));
        if let (true, syn::Member::Named(ident)) = (on_self, &node.member) {
            self.found.insert(strip_raw(&ident.to_string()));
        }
        visit::visit_expr_field(self, node);
    }

    fn visit_macro(&mut self, _: &'ast Macro) {}
}
