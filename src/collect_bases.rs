//! Collects what routing needs to send a trait to a special member or a
//! base list: the trait's path reference, its type argument, supertrait
//! entries, and a reference for each derive name.

use syn::visit::Visit;
use syn::{GenericArgument, Path, PathArguments, Type, TypeParamBound};

use crate::collect::{Collector, Here};
use crate::model::{Base, RefId, Text, TypeRef};
use crate::typerefs::{range_of, strip_raw};

/// Returns the one type argument of a path's last segment, such as the `T`
/// of `From<T>`.
pub(crate) fn single_arg(path: &Path) -> Option<&Type> {
    let PathArguments::AngleBracketed(args) = &path.segments.last()?.arguments else {
        return None;
    };
    match args.args.iter().collect::<Vec<_>>().as_slice() {
        [GenericArgument::Type(ty)] => Some(ty),
        _ => None,
    }
}

impl Collector<'_> {
    /// Slices a special member's argument type, adding `&` unless the type
    /// is a reference already.
    pub(crate) fn argument(&mut self, here: &Here, ty: &Type) -> Text {
        let mut text = self.slicer(here).node(ty, |f| f.visit_type(ty));
        if !matches!(ty, Type::Reference(_)) {
            text.push_str("&");
        }
        text
    }

    /// Slices one supertrait bound into a base-list entry.
    pub(crate) fn bound_base(&mut self, here: &Here, bound: &TypeParamBound) -> Base {
        let files = self.files;
        let file = &files[here.file];
        let (text, ids) = self
            .slicer(here)
            .range(range_of(bound, file), |f| f.visit_type_param_bound(bound));
        match bound {
            TypeParamBound::Trait(t) => {
                let start = range_of(&t.path, file).start;
                Base {
                    name: t
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default(),
                    path_ref: ids
                        .iter()
                        .find(|(r, _)| r.start == start)
                        .map(|(_, id)| *id),
                    text,
                    ..Base::default()
                }
            }
            _ => Base {
                name: text.visible(&self.refs),
                text,
                ..Base::default()
            },
        }
    }

    /// Records a derive name as a reference for resolve to fill in, so
    /// routing can tell a standard derive from another crate's.
    pub(crate) fn derive_ref(&mut self, here: &Here, name: &str) -> RefId {
        self.refs.push(TypeRef {
            text: name.to_owned(),
            path: name
                .trim_start_matches("::")
                .split("::")
                .map(strip_raw)
                .collect(),
            global: name.starts_with("::"),
            module: here.module,
            generics: Vec::new(),
            comment: true,
            may_be_value: false,
            res: None,
            written: None,
            foreign: None,
        });
        self.refs.len() - 1
    }
}
