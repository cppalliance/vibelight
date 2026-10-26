//! Finds test-support code: items that also compile for tests, behind a
//! `cfg` that lists `test` inside `any(...)`, such as
//! `cfg(any(test, feature = "test-support"))`. Test-only code never gets
//! this far, because collect skips it.

use syn::punctuated::Punctuated;
use syn::{Meta, Token};

use crate::model::Crate;

/// Marks each module and declaration whose own `cfg`s, or an enclosing
/// module's, make it test support.
pub(crate) fn mark(krate: &mut Crate) {
    let own: Vec<bool> = krate
        .modules
        .iter()
        .map(|m| is_test_support(&m.cfgs))
        .collect();
    let flags: Vec<bool> = krate
        .modules
        .iter()
        .map(|m| {
            krate
                .modules
                .iter()
                .zip(&own)
                .any(|(outer, &own)| own && m.path.starts_with(&outer.path))
        })
        .collect();
    for (module, flag) in krate.modules.iter_mut().zip(&flags) {
        module.test_support = *flag;
    }
    for decl in &mut krate.decls {
        decl.test_support = flags[decl.module] || is_test_support(&decl.meta.cfgs);
    }
}

/// Tells whether `cfg` predicates, as written, make an item test support:
/// one of them lists `test` inside `any(...)`. `not(...)` never does.
pub(crate) fn is_test_support(cfgs: &[String]) -> bool {
    cfgs.iter()
        .any(|cfg| syn::parse_str::<Meta>(cfg).is_ok_and(|meta| lists_test(&meta, false)))
}

fn lists_test(meta: &Meta, in_any: bool) -> bool {
    match meta {
        Meta::Path(path) => in_any && path.is_ident("test"),
        Meta::List(list) if list.path.is_ident("any") || list.path.is_ident("all") => {
            let in_any = in_any || list.path.is_ident("any");
            list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .is_ok_and(|items| items.iter().any(|item| lists_test(item, in_any)))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn support(cfg: &str) -> bool {
        is_test_support(&[cfg.to_owned()])
    }

    #[test]
    fn only_test_inside_any_is_test_support() {
        assert!(support("any(test, feature = \"test-support\")"));
        assert!(support("all(unix, any(test, feature = \"x\"))"));
        assert!(support("any(all(test, unix), feature = \"x\")"));
        assert!(!support("not(any(test, feature = \"x\"))"));
        assert!(!support("feature = \"test-support\""));
        assert!(!support("unix"));
        assert!(!is_test_support(&[]));
    }
}
