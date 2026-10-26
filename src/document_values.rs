//! Gives each module-level function, constant, and static its HTML id.

use std::collections::BTreeMap;

use crate::document::suffixed;
use crate::layout_items::item_text;
use crate::model::{Crate, ItemKind, ValueRef};
use crate::typerefs::strip_raw;

/// Returns the id of each function, constant, and static of a module:
/// `fn.`, `const.`, or `static.`, then the crate, the module path, and the
/// name, joined with dots, with case kept and raw prefixes removed. An id
/// that would repeat, such as one function under two `cfg`s, gets `-2`,
/// `-3`, and so on, ordered by the items' rendered text. Methods and extern
/// block items get none.
pub(crate) fn value_ids(krate: &Crate) -> BTreeMap<ValueRef, String> {
    let mut places = Vec::new();
    let mut base = Vec::new();
    for (module, holder) in krate.modules.iter().enumerate() {
        for (item, entry) in holder.items.iter().enumerate() {
            let kind = match entry.kind {
                ItemKind::Fn(_) => "fn",
                ItemKind::Const { .. } => "const",
                ItemKind::Static { .. } => "static",
                _ => continue,
            };
            let mut parts = vec![krate.name.clone()];
            parts.extend(holder.path.iter().map(|s| strip_raw(s)));
            parts.push(strip_raw(&entry.name));
            places.push(ValueRef { module, item });
            base.push(format!("{kind}.{}", parts.join(".")));
        }
    }
    let ids = suffixed(&base, |a, b| {
        item_text(krate, places[a])
            .cmp(&item_text(krate, places[b]))
            .then(a.cmp(&b))
    });
    places.into_iter().zip(ids).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /// The ids of a crate named `t` made of one `lib.rs`, with the item
    /// each one names, as `module::name`.
    fn ids_of(lib: &str) -> Vec<(String, String)> {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), lib).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        value_ids(&krate)
            .into_iter()
            .map(|(at, id)| {
                let module = &krate.modules[at.module];
                let mut path = module.path.clone();
                path.push(module.items[at.item].name.clone());
                (path.join("::"), id)
            })
            .collect()
    }

    fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect()
    }

    #[test]
    fn an_id_joins_the_kind_crate_module_path_and_name_with_dots() {
        let found = ids_of(
            "pub fn Run() {}\n\
             pub mod a { pub mod b { pub const LIMIT: u8 = 1; pub static mut Total: u8 = 0; } }\n",
        );
        assert_eq!(
            found,
            pairs(&[
                ("Run", "fn.t.Run"),
                ("a::b::LIMIT", "const.t.a.b.LIMIT"),
                ("a::b::Total", "static.t.a.b.Total"),
            ])
        );
    }

    #[test]
    fn raw_prefixes_are_removed_from_the_module_path_and_the_name() {
        let found = ids_of("pub mod r#impl { pub fn r#fn() {} }\n");
        assert_eq!(found, pairs(&[("r#impl::r#fn", "fn.t.impl.fn")]));
    }

    #[test]
    fn private_items_get_ids_and_methods_and_extern_items_get_none() {
        let found = ids_of(
            "fn hidden() {}\n\
             pub struct S;\n\
             impl S { pub fn method(&self) {} }\n\
             extern \"C\" { pub fn outside(); pub static LIMIT: u8; }\n",
        );
        assert_eq!(found, pairs(&[("hidden", "fn.t.hidden")]));
    }

    #[test]
    fn a_repeated_id_gets_a_number_ordered_by_the_rendered_text() {
        let found = ids_of(
            "/// Zeta.\n#[cfg(unix)]\npub fn f() {}\n\
             /// Alpha.\n#[cfg(windows)]\npub fn f() {}\n\
             /// Middle.\n#[cfg(wasm)]\npub fn f() {}\n",
        );
        let ids: Vec<&str> = found.iter().map(|(_, id)| id.as_str()).collect();
        assert_eq!(
            ids,
            ["fn.t.f-3", "fn.t.f", "fn.t.f-2"],
            "in source order: Zeta, Alpha, Middle"
        );
    }

    #[test]
    fn a_type_and_a_function_of_one_name_have_separate_ids() {
        let found = ids_of("pub struct Shape { pub sides: u8 }\npub fn Shape() {}\n");
        assert_eq!(found, pairs(&[("Shape", "fn.t.Shape")]));
    }
}
