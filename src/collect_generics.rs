//! Slices generic parameter lists and `where` clauses into text with type
//! references.

use syn::{Generics, WhereClause};

use crate::collect::{Collector, Here};
use crate::model::{Text, Where};

impl Collector<'_> {
    /// Returns a generic parameter list as `<...>`, each parameter sliced
    /// on its own, or empty text.
    pub(crate) fn generics_text(&mut self, here: &Here, generics: &Generics) -> Text {
        let mut text = Text::default();
        if generics.params.is_empty() {
            return text;
        }
        text.push_str("<");
        for (i, param) in generics.params.iter().enumerate() {
            if i > 0 {
                text.push_str(", ");
            }
            let part = self
                .slicer(here)
                .node(param, |f| syn::visit::Visit::visit_generic_param(f, param));
            text.append(&part);
        }
        text.push_str(">");
        text
    }

    /// Returns a `where` clause, each predicate sliced on its own.
    pub(crate) fn where_text(
        &mut self,
        here: &Here,
        clause: Option<&WhereClause>,
    ) -> Option<Where> {
        let clause = clause.filter(|c| !c.predicates.is_empty())?;
        let mut predicates = Vec::new();
        for predicate in &clause.predicates {
            predicates.push(self.slicer(here).node(predicate, |f| {
                syn::visit::Visit::visit_where_predicate(f, predicate);
            }));
        }
        Some(Where(predicates))
    }
}
