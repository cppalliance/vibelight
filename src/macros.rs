//! Finds macros: derive lists, attribute macros, and invocations.

use std::collections::BTreeSet;
use std::ops::Range;

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, Expr, Macro, MacroDelimiter, Meta, Path, Token};

use crate::model::name_order;
use crate::source::SourceFile;
use crate::typerefs::strip_raw;

/// Attributes the compiler or a tool owns, which are never macros.
const BUILTIN: &[&str] = &[
    "allow",
    "automatically_derived",
    "bench",
    "cfg",
    "cfg_attr",
    "cold",
    "collapse_debuginfo",
    "coverage",
    "crate_name",
    "crate_type",
    "debugger_visualizer",
    "default",
    "deny",
    "deprecated",
    "derive",
    "diagnostic",
    "doc",
    "expect",
    "export_name",
    "feature",
    "forbid",
    "global_allocator",
    "ignore",
    "inline",
    "instruction_set",
    "link",
    "link_name",
    "link_ordinal",
    "link_section",
    "macro_export",
    "macro_use",
    "must_use",
    "naked",
    "no_builtins",
    "no_implicit_prelude",
    "no_main",
    "no_mangle",
    "no_std",
    "non_exhaustive",
    "optimize",
    "panic_handler",
    "path",
    "proc_macro",
    "proc_macro_attribute",
    "proc_macro_derive",
    "recursion_limit",
    "register_tool",
    "repr",
    "should_panic",
    "target_feature",
    "test",
    "test_runner",
    "track_caller",
    "type_length_limit",
    "unsafe",
    "used",
    "warn",
    "windows_subsystem",
];

/// Tools whose attributes are never macros.
const TOOLS: &[&str] = &[
    "clippy",
    "diagnostic",
    "rust_analyzer",
    "rustdoc",
    "rustfmt",
];

/// Returns the attributes an item really has: every attribute, with each
/// `cfg_attr` that is not test-only replaced by the attributes it holds.
pub(crate) fn effective(attrs: &[Attribute]) -> Vec<Meta> {
    let mut out = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("cfg_attr") {
            out.push(attr.meta.clone());
            continue;
        }
        let Ok(parts) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        let mut parts = parts.into_iter();
        let Some(predicate) = parts.next() else {
            continue;
        };
        if test_only(&predicate) {
            continue;
        }
        out.extend(parts);
    }
    out
}

/// Tells whether a `cfg` predicate is test-only: `test`, or an `all(...)`
/// that holds a test-only predicate.
pub(crate) fn test_only(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("test"),
        Meta::List(list) if list.path.is_ident("all") => list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .is_ok_and(|items| items.iter().any(test_only)),
        _ => false,
    }
}

/// Tells whether a `macro_rules!` definition is exported at the crate
/// root, through `#[macro_export]` or a `cfg_attr` that holds it.
pub(crate) fn is_exported(attrs: &[Attribute]) -> bool {
    effective(attrs)
        .iter()
        .any(|meta| meta.path().is_ident("macro_export"))
}

/// The prefix of the item text of a `macro_rules!` definition.
const DEFINITION: &str = "macro_rules! ";

/// Returns the item text of the `macro_rules!` definition of `ident`.
pub(crate) fn definition_text(ident: &str) -> String {
    format!("{DEFINITION}{ident}")
}

/// Returns the macro name when `text` is the item text of a `macro_rules!`
/// definition.
pub(crate) fn definition_name(text: &str) -> Option<&str> {
    text.strip_prefix(DEFINITION)
}

/// Returns the traits from every derive attribute, merged and sorted.
pub(crate) fn derives(attrs: &[Attribute]) -> Vec<String> {
    let mut names = Vec::new();
    for meta in effective(attrs) {
        let Meta::List(list) = meta else {
            continue;
        };
        if !list.path.is_ident("derive") {
            continue;
        }
        if let Ok(paths) = list.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated) {
            names.extend(paths.iter().map(path_text));
        }
    }
    sorted(names)
}

/// Returns the attribute macros among `attrs`, sorted. A single-segment
/// unknown attribute is a derive helper when `helpers` is true.
pub(crate) fn attribute_macros(attrs: &[Attribute], helpers: bool) -> Vec<String> {
    let mut names = Vec::new();
    for meta in effective(attrs) {
        let path = meta.path();
        let first = path
            .segments
            .first()
            .map(|s| strip_raw(&s.ident.to_string()))
            .unwrap_or_default();
        let counts = if path.segments.len() == 1 {
            !BUILTIN.contains(&first.as_str()) && !first.starts_with("rustc_") && !helpers
        } else {
            !TOOLS.contains(&first.as_str())
        };
        if counts {
            names.push(path_text(path));
        }
    }
    sorted(names)
}

/// The kind in the mark text of a list of attribute macros.
pub(crate) const ATTRIBUTE: &str = "attribute";

/// The kind in the mark text of a list of derives.
pub(crate) const DERIVE: &str = "derive";

/// Returns the mark text of a list of names: the `kind`, a colon, and the
/// names joined by commas, as in `attribute: a, b`.
pub(crate) fn mark_text(kind: &str, names: &[String]) -> String {
    format!("{kind}: {}", names.join(", "))
}

/// Takes mark text apart into its kind and its names, or returns `None`
/// when `text` is not mark text.
pub(crate) fn mark_parts(text: &str) -> Option<(&str, Vec<&str>)> {
    let (kind, names) = text.split_once(": ")?;
    Some((kind, names.split(", ").collect()))
}

/// Returns a path as written: its segments joined by `::`.
pub(crate) fn path_text(path: &Path) -> String {
    let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    let joined = segments.join("::");
    if path.leading_colon.is_some() {
        format!("::{joined}")
    } else {
        joined
    }
}

/// Returns the name of every macro invoked inside an expression, in order,
/// without repeats.
pub(crate) fn expr_macros(expr: &Expr) -> Vec<String> {
    struct Calls(Vec<String>);
    impl<'ast> Visit<'ast> for Calls {
        fn visit_macro(&mut self, node: &'ast Macro) {
            let name = format!("{}!", path_text(&node.path));
            if !self.0.contains(&name) {
                self.0.push(name);
            }
            visit::visit_macro(self, node);
        }
    }
    let mut calls = Calls(Vec::new());
    calls.visit_expr(expr);
    calls.0
}

/// Returns every identifier in a token stream, with any `r#` prefix removed.
pub(crate) fn idents(tokens: &TokenStream) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![tokens.clone()];
    while let Some(stream) = stack.pop() {
        for tree in stream {
            match tree {
                TokenTree::Ident(ident) => {
                    out.insert(strip_raw(&ident.to_string()));
                }
                TokenTree::Group(group) => stack.push(group.stream()),
                TokenTree::Punct(_) | TokenTree::Literal(_) => {}
            }
        }
    }
    out
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort_by(|a, b| name_order(a, b));
    names.dedup();
    names
}

/// Returns the bytes inside a macro's delimiters, delimiters included.
pub(crate) fn delimiter_range(delimiter: &MacroDelimiter, file: &SourceFile) -> Range<usize> {
    let span = match delimiter {
        MacroDelimiter::Paren(d) => d.span,
        MacroDelimiter::Brace(d) => d.span,
        MacroDelimiter::Bracket(d) => d.span,
    };
    file.start(span.open())..file.end(span.close())
}

/// Returns the span of a token stream's first token.
pub(crate) fn tokens_span(tokens: &TokenStream) -> Span {
    tokens
        .clone()
        .into_iter()
        .next()
        .map_or_else(Span::call_site, |t| t.span())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(text: &str) -> Vec<Attribute> {
        let item: syn::ItemStruct = syn::parse_str(text).unwrap();
        item.attrs
    }

    #[test]
    fn derives_merge_cfg_attr_and_skip_test_only() {
        let list = attrs(
            "#[derive(Debug, serde::Serialize)] #[cfg_attr(feature = \"x\", derive(Clone))] \
             #[cfg_attr(test, derive(PartialEq))] struct A;",
        );
        assert_eq!(derives(&list), ["Clone", "Debug", "serde::Serialize"]);
    }

    #[test]
    fn attribute_macros_skip_builtins_tools_and_helpers() {
        let list = attrs(
            "#[derive(Debug)] #[serde(rename = \"a\")] #[repr(C)] #[rustfmt::skip] \
             #[tokio::main] #[must_use] struct A;",
        );
        assert_eq!(attribute_macros(&list, true), ["tokio::main"]);
        let list = attrs("#[async_trait] #[clippy::msrv] struct A;");
        assert_eq!(attribute_macros(&list, false), ["async_trait"]);
    }

    #[test]
    fn mark_text_is_taken_apart_into_kind_and_names() {
        let names = ["tokio::main".to_owned(), "::a::b".to_owned()];
        let text = mark_text(ATTRIBUTE, &names);
        assert_eq!(text, "attribute: tokio::main, ::a::b");
        assert_eq!(
            mark_parts(&text),
            Some((ATTRIBUTE, vec!["tokio::main", "::a::b"]))
        );
        assert_eq!(
            mark_parts(&mark_text(DERIVE, &["Clone".to_owned()])),
            Some((DERIVE, vec!["Clone"]))
        );
        assert_eq!(mark_parts("vec!"), None);
        assert_eq!(mark_parts("attribute::x!"), None);
    }

    #[test]
    fn macros_in_expressions() {
        let expr: Expr = syn::parse_str("concat!(\"a\", env!(\"B\")) + f(vec![1])").unwrap();
        assert_eq!(expr_macros(&expr), ["concat!", "vec!"]);
    }
}
