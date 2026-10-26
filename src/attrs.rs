//! Reads the attributes the outline cares about: `cfg`, `path`, and docs.

use syn::punctuated::Punctuated;
use syn::{
    AttrStyle, Attribute, Expr, ExprLit, Lit, MacroDelimiter, Meta, MetaList, Token, Visibility,
};

use crate::model::name_order;
use crate::source::SourceFile;

/// Tells whether a `cfg` on the item is test-only: `test`, or an `all(...)`
/// that holds a test-only predicate.
pub(crate) fn is_test_only(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg") && attr.parse_args::<Meta>().is_ok_and(|meta| test_only(&meta))
    })
}

fn test_only(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("test"),
        Meta::List(list) if list.path.is_ident("all") => list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .is_ok_and(|items| items.iter().any(test_only)),
        _ => false,
    }
}

/// Returns each `cfg` predicate as written, normalized and sorted.
pub(crate) fn cfgs(attrs: &[Attribute], file: &SourceFile) -> Vec<String> {
    let mut out: Vec<String> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .filter_map(|attr| match &attr.meta {
            Meta::List(list) => Some(list_text(list, file)),
            _ => None,
        })
        .collect();
    sort_marks(&mut out);
    out
}

/// Sorts `cfg` lines by name and drops repeats.
pub(crate) fn sort_marks(marks: &mut Vec<String>) {
    marks.sort_by(|a, b| name_order(a, b));
    marks.dedup();
}

/// Tells whether a visibility is plain `pub`.
pub(crate) fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// Returns the normalized source text inside a meta list's delimiters.
pub(crate) fn list_text(list: &MetaList, file: &SourceFile) -> String {
    let span = match &list.delimiter {
        MacroDelimiter::Paren(d) => d.span,
        MacroDelimiter::Brace(d) => d.span,
        MacroDelimiter::Bracket(d) => d.span,
    };
    file.clean(file.end(span.open())..file.start(span.close()))
}

/// Returns the value of a `#[path = "..."]` attribute.
pub(crate) fn path_attr(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(nv) if nv.path.is_ident("path") => string_value(&nv.value),
        _ => None,
    })
}

/// Returns the string of a string literal expression.
pub(crate) fn string_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Some(s.value()),
        _ => None,
    }
}

/// Splits attributes into outer and inner ones.
pub(crate) fn split_inner(attrs: &[Attribute]) -> (Vec<Attribute>, Vec<Attribute>) {
    attrs
        .iter()
        .cloned()
        .partition(|attr| matches!(attr.style, AttrStyle::Outer))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(text: &str) -> (Vec<Attribute>, SourceFile) {
        let file = SourceFile::new("a.rs".into(), text);
        let item: syn::ItemStruct = syn::parse_str(&file.text).unwrap();
        (item.attrs, file)
    }

    #[test]
    fn test_only_predicates() {
        assert!(is_test_only(&attrs("#[cfg(test)] struct A;").0));
        assert!(is_test_only(&attrs("#[cfg(all(test, unix))] struct A;").0));
        assert!(is_test_only(
            &attrs("#[cfg(all(unix, all(test)))] struct A;").0
        ));
        assert!(!is_test_only(&attrs("#[cfg(any(test, unix))] struct A;").0));
        assert!(!is_test_only(&attrs("#[cfg(not(test))] struct A;").0));
        assert!(!is_test_only(
            &attrs("#[cfg_attr(test, derive(Debug))] struct A;").0
        ));
    }

    #[test]
    fn cfg_text_is_normalized_and_sorted() {
        let (list, file) =
            attrs("#[cfg(feature = \"x\")]\n#[cfg(all( unix ,\n not(test) ))]\nstruct A;");
        assert_eq!(
            cfgs(&list, &file),
            ["all(unix, not(test))", "feature = \"x\""]
        );
    }

    #[test]
    fn path_attribute_value() {
        let file = SourceFile::new("a.rs".into(), "#[path = \"config-limits.rs\"] mod limits;");
        let item: syn::ItemMod = syn::parse_str(&file.text).unwrap();
        assert_eq!(path_attr(&item.attrs).as_deref(), Some("config-limits.rs"));
    }
}
