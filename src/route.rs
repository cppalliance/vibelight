//! Sends each trait a type implements or derives, and each supertrait, to a
//! C++ special member, the short base list, or the long base list.

use std::collections::BTreeSet;

use crate::model::{
    Base, Crate, DeclId, Holder, MacroKind, Seg, Shape, Special, SpecialKind, Text, TypeRef,
};
use crate::resolve::is_standard;
use crate::typerefs::strip_raw;

/// The derive macros of the standard prelude. A derive macro is not a type,
/// so resolve finds no path for one written bare.
const DERIVES: [&str; 9] = [
    "Clone",
    "Copy",
    "Debug",
    "Default",
    "Eq",
    "Hash",
    "Ord",
    "PartialEq",
    "PartialOrd",
];

/// Returns the special member a trait impl becomes: a standard trait with a
/// C++ equivalent, in an impl with no generic parameters or `where` clause
/// of its own.
pub(crate) fn special_kind(refs: &[TypeRef], base: &Base) -> Option<SpecialKind> {
    let reference = &refs[base.path_ref?];
    if base.generic || reference.target.is_some() {
        return None;
    }
    let path = reference.canonical.as_deref().filter(|p| is_standard(p))?;
    let kind = kind_of(path.last()?)?;
    (kind != SpecialKind::From || base.arg.is_some()).then_some(kind)
}

fn kind_of(name: &str) -> Option<SpecialKind> {
    Some(match name {
        "Default" => SpecialKind::Default,
        "Clone" => SpecialKind::Clone,
        "From" => SpecialKind::From,
        "Drop" => SpecialKind::Drop,
        "PartialEq" => SpecialKind::PartialEq,
        "PartialOrd" => SpecialKind::PartialOrd,
        _ => return None,
    })
}

/// Routes every declaration's derives, then gives each base-list entry and
/// each supertrait its printed form.
pub(crate) fn finish(krate: &mut Crate) {
    for id in 0..krate.decls.len() {
        route_derives(krate, id);
    }
    let paths: Vec<String> = (0..krate.decls.len())
        .map(|d| krate.decl_path(d).join("::"))
        .collect();
    let names: BTreeSet<String> = krate.decls.iter().map(|d| strip_raw(&d.name)).collect();
    for decl in &mut krate.decls {
        let bases = match &mut decl.shape {
            Shape::Trait { supertraits, .. } => supertraits,
            _ => &mut decl.bases,
        };
        spell(&mut krate.refs, &paths, &names, bases);
    }
}

/// Sends each standard derive to a `= default` special member or to the
/// short list, and leaves the rest on the `derive:` mark, which goes away
/// with its kill list entry when no name is left.
fn route_derives(krate: &mut Crate, id: DeclId) {
    let decl = &mut krate.decls[id];
    let mut kept = Vec::new();
    for (name, &r) in decl.derives.iter().zip(&decl.derive_refs) {
        let reference = &krate.refs[r];
        let path = reference
            .canonical
            .clone()
            .unwrap_or_else(|| reference.path.clone());
        let standard = reference.target.is_none()
            && match &reference.canonical {
                Some(canonical) => is_standard(canonical),
                None => matches!(path.as_slice(), [one] if DERIVES.contains(&one.as_str())),
            };
        let last = path.last().cloned().unwrap_or_default();
        match kind_of(&last) {
            _ if !standard => kept.push(name.clone()),
            Some(
                kind @ (SpecialKind::Default
                | SpecialKind::Clone
                | SpecialKind::PartialEq
                | SpecialKind::PartialOrd),
            ) => decl.special.push(Special {
                kind,
                arg: None,
                derived: true,
                notes: Vec::new(),
                loc: decl.loc,
            }),
            Some(_) => kept.push(name.clone()),
            None => decl.bases.push(Base {
                text: Text::plain(last.clone()),
                name: last,
                ..Base::default()
            }),
        }
    }
    decl.derives = kept;
    let text = format!("derive: {}", decl.derives.join(", "));
    let gone = decl.derives.is_empty();
    let this =
        |m: &crate::model::MacroUse| m.kind == MacroKind::Derive && m.holder == Holder::Decl(id);
    if gone {
        krate.macros.retain(|m| !this(m));
    } else {
        for mark in krate.macros.iter_mut().filter(|m| this(m)) {
            mark.text.clone_from(&text);
        }
    }
}

/// Gives each entry its printed text and list. A standard trait keeps its
/// short name, or its full path when that name is also a type in the crate
/// or another entry's name. A trait of this crate prints its crate-relative
/// path and stays a link, and any other trait prints the path it resolves
/// to, or the text as written when nothing resolves.
fn spell(refs: &mut Vec<TypeRef>, paths: &[String], names: &BTreeSet<String>, bases: &mut [Base]) {
    let mut short: Vec<(usize, usize, String, String)> = Vec::new();
    for (index, base) in bases.iter_mut().enumerate() {
        let Some(id) = base.path_ref else {
            continue;
        };
        let Some(at) = base
            .text
            .0
            .iter()
            .position(|s| matches!(s, Seg::Ref(r) if *r == id))
        else {
            continue;
        };
        let reference = refs[id].clone();
        match (reference.target, &reference.canonical) {
            (Some(target), _) => {
                base.long = true;
                refs.push(TypeRef {
                    text: paths[target].clone(),
                    ..reference
                });
                base.text.0[at] = Seg::Ref(refs.len() - 1);
            }
            (None, Some(path)) if is_standard(path) => {
                let name = path.last().cloned().unwrap_or_default();
                short.push((index, at, name, path.join("::")));
            }
            (None, Some(path)) => {
                base.long = true;
                base.text.0[at] = Seg::Plain(path.join("::"));
            }
            (None, None) => base.long = true,
        }
    }
    for (index, at, name, full) in &short {
        let repeated = short.iter().filter(|other| other.2 == *name).count() > 1;
        let printed = if repeated || names.contains(name) {
            full.clone()
        } else {
            name.clone()
        };
        bases[*index].text.0[*at] = Seg::Plain(printed);
        bases[*index].name.clone_from(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(text: &str, canonical: Option<&str>, target: Option<DeclId>) -> TypeRef {
        let split = |s: &str| s.split("::").map(str::to_owned).collect();
        TypeRef {
            text: text.to_owned(),
            path: split(text),
            global: false,
            module: 0,
            generics: Vec::new(),
            comment: false,
            target,
            canonical: canonical.map(split),
        }
    }

    fn base(id: usize, name: &str) -> Base {
        Base {
            text: Text(vec![Seg::Ref(id)]),
            name: name.to_owned(),
            path_ref: Some(id),
            ..Base::default()
        }
    }

    #[test]
    fn only_a_standard_trait_in_an_impl_without_generics_is_a_special_member() {
        let refs = vec![
            reference("From", Some("From"), None),
            reference("fmt::Debug", Some("std::fmt::Debug"), None),
            reference("Clone", None, Some(0)),
            reference("cmp::PartialEq", Some("core::cmp::PartialEq"), None),
        ];
        let mut from = base(0, "From");
        assert_eq!(special_kind(&refs, &from), None, "From needs its argument");
        from.arg = Some(Text::plain("u8&"));
        assert_eq!(special_kind(&refs, &from), Some(SpecialKind::From));
        from.generic = true;
        assert_eq!(special_kind(&refs, &from), None);
        assert_eq!(special_kind(&refs, &base(1, "Debug")), None);
        assert_eq!(special_kind(&refs, &base(2, "Clone")), None);
        assert_eq!(
            special_kind(&refs, &base(3, "PartialEq")),
            Some(SpecialKind::PartialEq)
        );
    }

    #[test]
    fn a_short_entry_keeps_its_full_path_only_on_a_collision() {
        let mut refs = vec![
            reference("fmt::Write", Some("std::fmt::Write"), None),
            reference("io::Write", Some("std::io::Write"), None),
            reference("error::Error", Some("std::error::Error"), None),
            reference("fmt::Debug", Some("std::fmt::Debug"), None),
            reference("Named", None, Some(0)),
            reference("serde::Serialize", Some("serde::Serialize"), None),
            reference("Unknown", None, None),
        ];
        let names: BTreeSet<String> = ["Error".to_owned(), "Named".to_owned()].into();
        let paths = vec!["traits::Named".to_owned()];
        let mut bases: Vec<Base> = [
            "Write",
            "Write",
            "Error",
            "Debug",
            "Named",
            "Serialize",
            "Unknown",
        ]
        .iter()
        .enumerate()
        .map(|(id, name)| base(id, name))
        .collect();
        spell(&mut refs, &paths, &names, &mut bases);
        let printed: Vec<(String, bool)> = bases
            .iter()
            .map(|b| (b.text.visible(&refs), b.long))
            .collect();
        let expected = [
            ("std::fmt::Write", false),
            ("std::io::Write", false),
            ("std::error::Error", false),
            ("Debug", false),
            ("traits::Named", true),
            ("serde::Serialize", true),
            ("Unknown", true),
        ]
        .map(|(text, long)| (text.to_owned(), long));
        assert_eq!(printed, expected);
    }
}
