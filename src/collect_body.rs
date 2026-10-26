//! Collects the type names function bodies write. They keep a type out of
//! Unconnected (rule 26) and name nothing else.

use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{Block, Macro, Path};

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
                target: None,
                value: None,
                canonical: None,
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
