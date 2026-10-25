//! The names the standard prelude and the language provide, and the `std`
//! path of each item from outside the crate.

/// Names the standard prelude and the language provide, which never count
/// as unresolved, each with the `std` path it stands for.
const PRELUDE: &[(&str, &str)] = &[
    ("AsMut", "std::convert::AsMut"),
    ("AsRef", "std::convert::AsRef"),
    ("Box", "std::boxed::Box"),
    ("Clone", "std::clone::Clone"),
    ("Copy", "std::marker::Copy"),
    ("Default", "std::default::Default"),
    ("DoubleEndedIterator", "std::iter::DoubleEndedIterator"),
    ("Drop", "std::ops::Drop"),
    ("Eq", "std::cmp::Eq"),
    ("ExactSizeIterator", "std::iter::ExactSizeIterator"),
    ("Extend", "std::iter::Extend"),
    ("Fn", "std::ops::Fn"),
    ("FnMut", "std::ops::FnMut"),
    ("FnOnce", "std::ops::FnOnce"),
    ("From", "std::convert::From"),
    ("FromIterator", "std::iter::FromIterator"),
    ("Future", "std::future::Future"),
    ("Into", "std::convert::Into"),
    ("IntoFuture", "std::future::IntoFuture"),
    ("IntoIterator", "std::iter::IntoIterator"),
    ("Iterator", "std::iter::Iterator"),
    ("Option", "std::option::Option"),
    ("Ord", "std::cmp::Ord"),
    ("PartialEq", "std::cmp::PartialEq"),
    ("PartialOrd", "std::cmp::PartialOrd"),
    ("Result", "std::result::Result"),
    ("Send", "std::marker::Send"),
    ("Sized", "std::marker::Sized"),
    ("String", "std::string::String"),
    ("Sync", "std::marker::Sync"),
    ("ToOwned", "std::borrow::ToOwned"),
    ("ToString", "std::string::ToString"),
    ("TryFrom", "std::convert::TryFrom"),
    ("TryInto", "std::convert::TryInto"),
    ("Unpin", "std::marker::Unpin"),
    ("Vec", "std::vec::Vec"),
    ("bool", "std::primitive::bool"),
    ("char", "std::primitive::char"),
    ("f32", "std::primitive::f32"),
    ("f64", "std::primitive::f64"),
    ("i128", "std::primitive::i128"),
    ("i16", "std::primitive::i16"),
    ("i32", "std::primitive::i32"),
    ("i64", "std::primitive::i64"),
    ("i8", "std::primitive::i8"),
    ("isize", "std::primitive::isize"),
    ("str", "std::primitive::str"),
    ("u128", "std::primitive::u128"),
    ("u16", "std::primitive::u16"),
    ("u32", "std::primitive::u32"),
    ("u64", "std::primitive::u64"),
    ("u8", "std::primitive::u8"),
    ("usize", "std::primitive::usize"),
];

/// Returns the `std` path a bare prelude name stands for.
pub(crate) fn prelude_path(name: &str) -> Option<&'static str> {
    PRELUDE
        .iter()
        .find(|(prelude, _)| *prelude == name)
        .map(|(_, path)| *path)
}

/// Returns the full path of the item a canonical path names: a bare prelude
/// name stands for its `std` path, and a `core` or `alloc` path for the
/// `std` path that re-exports it.
pub(crate) fn item_path(path: &[String]) -> Option<String> {
    let joined = match path {
        [name] => prelude_path(name)?.to_owned(),
        _ => path.join("::"),
    };
    Some(match joined.split_once("::") {
        Some(("core" | "alloc", rest)) => format!("std::{rest}"),
        _ => joined,
    })
}

/// Tells whether a canonical path names an item of `std`, `core`, `alloc`,
/// or the prelude, whose canonical path is its bare name.
pub(crate) fn is_standard(path: &[String]) -> bool {
    match path {
        [name] => prelude_path(name).is_some(),
        [first, ..] => matches!(first.as_str(), "std" | "core" | "alloc"),
        [] => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_paths_start_with_std_core_or_alloc_or_name_a_prelude_item() {
        let path = |s: &str| -> Vec<String> { s.split("::").map(str::to_owned).collect() };
        for standard in [
            "std::fmt::Debug",
            "core::cmp::Ord",
            "alloc::vec::Vec",
            "Option",
        ] {
            assert!(is_standard(&path(standard)), "{standard}");
        }
        for other in [
            "serde::Serialize",
            "runvil_lua::ErrorValue",
            "Named",
            "Debug",
        ] {
            assert!(!is_standard(&path(other)), "{other}");
        }
        assert!(!is_standard(&[]));
    }
}
