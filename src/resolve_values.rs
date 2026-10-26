//! Indexes the functions, constants, and statics of each module for name
//! resolution, and tests how a name that may be a value finds them.

use std::collections::BTreeMap;

use crate::model::{Crate, ItemKind};
use crate::typerefs::strip_raw;

/// Returns each module's functions, constants, and statics by name, as
/// indices into the module's items. Items that share a name, such as one
/// function under two `cfg`s, are ordered by their `cfg` lines, so a name
/// reaches the same one whatever order the source holds them in.
pub(crate) fn value_index(krate: &Crate) -> Vec<BTreeMap<String, Vec<usize>>> {
    krate
        .modules
        .iter()
        .map(|module| {
            let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (at, item) in module.items.iter().enumerate() {
                let value = matches!(
                    item.kind,
                    ItemKind::Fn(_) | ItemKind::Const { .. } | ItemKind::Static { .. }
                );
                if value {
                    index.entry(strip_raw(&item.name)).or_default().push(at);
                }
            }
            for list in index.values_mut() {
                list.sort_by_key(|&at| module.items[at].meta.cfgs.join(" "));
            }
            index
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use crate::model::{Crate, ItemKind, Reach, TypeRef};
    use crate::resolve::{Tally, resolve};

    /// Resolves a crate made of one `lib.rs`.
    fn resolved(lib: &str) -> (tempfile::TempDir, Crate, Tally) {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), lib).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        let tally = resolve(&mut krate);
        (tmp, krate, tally)
    }

    /// The first reference written as `text` in `module`.
    fn reference<'a>(krate: &'a Crate, module: &str, text: &str) -> &'a TypeRef {
        krate
            .refs
            .iter()
            .find(|r| krate.modules[r.module].path.join("::") == module && r.text == text)
            .expect("the module writes it")
    }

    /// Where a reference's value is, as `module::name`, and what kind it is.
    fn value_of(krate: &Crate, r: &TypeRef) -> Option<String> {
        let at = r.value?;
        let module = &krate.modules[at.module];
        let item = &module.items[at.item];
        let kind = match item.kind {
            ItemKind::Fn(_) => "fn",
            ItemKind::Const { .. } => "const",
            ItemKind::Static { .. } => "static",
            _ => "other",
        };
        let mut path = module.path.clone();
        path.push(item.name.clone());
        Some(format!("{kind} {}", path.join("::")))
    }

    #[test]
    fn a_re_export_reaches_a_function_a_constant_and_a_static() {
        let (_tmp, krate, tally) = resolved(
            "pub mod tools { pub fn run() {} pub const LIMIT: u8 = 1; pub static TOTAL: u8 = 0; }\n\
             pub use tools::{run, LIMIT};\n\
             pub use tools::TOTAL as ALL;\n",
        );
        let value = |text: &str| value_of(&krate, reference(&krate, "", text));
        assert_eq!(value("tools::run").as_deref(), Some("fn tools::run"));
        assert_eq!(value("tools::LIMIT").as_deref(), Some("const tools::LIMIT"));
        assert_eq!(
            value("tools::TOTAL").as_deref(),
            Some("static tools::TOTAL")
        );
        assert_eq!(tally, Tally::default());
    }

    #[test]
    fn a_chain_of_re_exports_reaches_the_defining_module() {
        let (_tmp, krate, _) = resolved(
            "mod deep { pub fn run() {} }\n\
             mod middle { pub use crate::deep::run as go; }\n\
             pub use middle::go;\n\
             /// See [`go`] and [`crate::middle::go`].\n\
             pub struct Mark;\n",
        );
        for text in ["middle::go", "go", "crate::middle::go"] {
            let r = krate
                .refs
                .iter()
                .find(|r| r.text == text)
                .expect("the source writes it");
            assert_eq!(
                value_of(&krate, r).as_deref(),
                Some("fn deep::run"),
                "{text}"
            );
        }
    }

    #[test]
    fn a_type_wins_over_a_function_of_the_same_name() {
        let (_tmp, krate, _) = resolved(
            "pub mod shapes { pub fn Shape() {} pub struct Shape { pub sides: u8 } }\n\
             pub use shapes::Shape;\n\
             pub mod only { pub fn Lone() {} }\n\
             pub use only::Lone;\n",
        );
        let both = reference(&krate, "", "shapes::Shape");
        assert!(both.target.is_some(), "the type is preferred");
        assert_eq!(value_of(&krate, both), None);
        let lone = reference(&krate, "", "only::Lone");
        assert_eq!(lone.target, None);
        assert_eq!(value_of(&krate, lone).as_deref(), Some("fn only::Lone"));
    }

    #[test]
    fn a_module_and_a_function_of_the_same_name_resolve_to_the_function() {
        let (_tmp, krate, _) = resolved(
            "pub mod parse { pub fn inner() {} }\n\
             pub fn parse() {}\n\
             pub mod user { pub use crate::parse; pub use crate::parse::inner; }\n",
        );
        let r = reference(&krate, "user", "crate::parse");
        assert_eq!(value_of(&krate, r).as_deref(), Some("fn parse"));
        let inner = reference(&krate, "user", "crate::parse::inner");
        assert_eq!(value_of(&krate, inner).as_deref(), Some("fn parse::inner"));
    }

    #[test]
    fn a_name_in_a_type_position_never_resolves_to_a_value() {
        let (_tmp, krate, tally) = resolved(
            "pub mod tools { pub fn Gauge() {} }\n\
             pub use tools::Gauge;\n\
             pub struct Holder { pub gauge: Gauge }\n",
        );
        let field = krate
            .refs
            .iter()
            .find(|r| !r.comment && r.text == "Gauge")
            .expect("the field names it");
        assert_eq!((field.target, field.value), (None, None));
        assert_eq!(
            tally.unresolved, 0,
            "Gauge is imported, so it is not counted"
        );
        let export = reference(&krate, "", "tools::Gauge");
        assert_eq!(value_of(&krate, export).as_deref(), Some("fn tools::Gauge"));
    }

    #[test]
    fn a_derive_name_never_resolves_to_a_value() {
        let (_tmp, krate, _) = resolved("pub fn Clone() {}\n#[derive(Clone)]\npub struct S;\n");
        let derive = krate
            .refs
            .iter()
            .find(|r| r.text == "Clone")
            .expect("the derive writes it");
        assert_eq!(derive.value, None);
        assert!(!derive.may_be_value);
    }

    #[test]
    fn a_bare_name_in_a_comment_finds_only_a_value_in_scope() {
        let (_tmp, krate, tally) = resolved(
            "pub mod near {\n\
             /// Calls [`helper`] and [`far_helper`].\n\
             pub fn caller() {}\n\
             pub fn helper() {}\n\
             }\n\
             pub mod far { pub fn far_helper() {} }\n",
        );
        let helper = reference(&krate, "near", "helper");
        assert_eq!(value_of(&krate, helper).as_deref(), Some("fn near::helper"));
        let far = reference(&krate, "near", "far_helper");
        assert_eq!(
            value_of(&krate, far),
            None,
            "no unique-name fallback for values"
        );
        assert_eq!(tally, Tally::default());
    }

    #[test]
    fn functions_under_different_cfgs_resolve_to_the_same_one_in_any_order() {
        let first = "pub mod m {\n#[cfg(unix)]\npub fn f() {}\n#[cfg(windows)]\npub fn f() {}\n}\n\
                     pub use m::f;\n";
        let second = "pub mod m {\n#[cfg(windows)]\npub fn f() {}\n#[cfg(unix)]\npub fn f() {}\n}\n\
                      pub use m::f;\n";
        let cfg_of = |lib: &str| {
            let (_tmp, krate, _) = resolved(lib);
            let at = reference(&krate, "", "m::f").value.expect("it resolves");
            krate.modules[at.module].items[at.item].meta.cfgs.clone()
        };
        assert_eq!(cfg_of(first), ["unix"]);
        assert_eq!(cfg_of(second), ["unix"]);
    }

    #[test]
    fn a_name_reaches_the_defining_function_through_two_other_crates() {
        let tmp = tempfile::tempdir().unwrap();
        let mut krates = Vec::new();
        for (name, lib) in [
            (
                "origin",
                "pub mod gauge { pub fn measure() {} pub const LIMIT: u8 = 1; }\n",
            ),
            ("relay", "pub use origin::gauge::{LIMIT, measure};\n"),
            (
                "front",
                "pub use relay::measure;\n/// See [`measure`] and [`relay::LIMIT`].\npub struct S;\n",
            ),
        ] {
            let dir = tmp.path().join(name);
            fs::create_dir_all(dir.join("src")).unwrap();
            fs::write(dir.join("src").join("lib.rs"), lib).unwrap();
            let tree = crate::modtree::walk(&dir, "src/lib.rs").unwrap();
            let mut krate = crate::collect::collect(tree, &dir, name, name);
            resolve(&mut krate);
            krates.push(krate);
        }
        let deps = vec![
            BTreeMap::new(),
            BTreeMap::from([("origin".to_owned(), 0)]),
            BTreeMap::from([("relay".to_owned(), 1)]),
        ];
        crate::resolve_run::link(&mut krates, &deps);
        let reached = |krate: usize, text: &str| {
            let r = krates[krate]
                .refs
                .iter()
                .find(|r| r.text == text)
                .expect("the crate writes it");
            let cross = r.foreign.expect("it reaches another crate");
            let Reach::Value(at) = cross.reach else {
                panic!("{text} does not reach a value");
            };
            let name = &krates[cross.krate].modules[at.module].items[at.item].name;
            (cross.krate, name.clone())
        };
        let origin = |name: &str| (0, name.to_owned());
        assert_eq!(reached(1, "origin::gauge::measure"), origin("measure"));
        assert_eq!(reached(2, "relay::measure"), origin("measure"));
        assert_eq!(reached(2, "measure"), origin("measure"));
        assert_eq!(reached(2, "relay::LIMIT"), origin("LIMIT"));
    }

    #[test]
    fn a_raw_identifier_function_is_found_by_its_plain_name() {
        let (_tmp, krate, _) = resolved("pub mod m { pub fn r#fn() {} }\npub use m::r#fn;\n");
        let r = reference(&krate, "", "m::r#fn");
        assert_eq!(value_of(&krate, r).as_deref(), Some("fn m::r#fn"));
    }
}
