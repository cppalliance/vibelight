//! Checks the analysis facts that sit on nodes rather than bodies: the
//! test-only ranges and files, the trait impl members, and every node's raw
//! doc text and lint attributes.

use std::cmp::Ordering;
use std::ops::Range;

use crate::model::{
    Crate, Foreign, ItemKind, MemberKind, Meta, Shape, Sig, VariantFields, name_order,
};

/// Checks a crate's node facts, naming the first broken invariant.
pub(crate) fn check_nodes(krate: &Crate) -> Result<(), String> {
    let files = krate.files.len();
    let in_file = |(file, range): &(usize, Range<usize>)| {
        *file < files && range.start <= range.end && range.end <= krate.files[*file].text.len()
    };
    let key = |(file, range): &(usize, Range<usize>)| (*file, range.start, range.end);
    let ranges = &krate.test_ranges;
    if !ranges.iter().all(in_file) || !ranges.windows(2).all(|w| key(&w[0]) <= key(&w[1])) {
        return Err("the test-only ranges are out of their files or not sorted".to_owned());
    }
    let tests = &krate.test_files;
    if tests.iter().any(|t| t.contains('\\')) || !tests.windows(2).all(|w| w[0] < w[1]) {
        return Err("the test files are not sorted, each once, with forward slashes".to_owned());
    }
    for (i, item) in krate.impls.iter().enumerate() {
        if item.trait_.is_none() && !item.trait_members.is_empty() {
            return Err(format!("inherent impl {i} has trait members"));
        }
        for member in &item.trait_members {
            let name = &member.name;
            let refs = (member.kind.refs())
                .ok_or_else(|| format!("trait member {name} of impl {i} has no signature"))?;
            if refs
                .iter()
                .any(|&r| krate.refs.get(r).is_none_or(|t| !t.comment))
            {
                return Err(format!(
                    "trait member {name} of impl {i} writes a counted reference"
                ));
            }
        }
    }
    check_metas(krate)
}

/// Checks the raw doc text and lint attributes of every node collect built
/// them for.
fn check_metas(krate: &Crate) -> Result<(), String> {
    let mut metas: Vec<&Meta> = Vec::new();
    let mut sigs: Vec<&Sig> = Vec::new();
    for decl in &krate.decls {
        metas.push(&decl.meta);
        match &decl.shape {
            Shape::Tuple(params) => metas.extend(params.iter().map(|p| &p.meta)),
            Shape::Enum(variants) => {
                for variant in variants {
                    metas.push(&variant.meta);
                    if let VariantFields::Tuple(params) | VariantFields::Named(params) =
                        &variant.fields
                    {
                        metas.extend(params.iter().map(|p| &p.meta));
                    }
                }
            }
            _ => {}
        }
    }
    let impl_members = krate
        .impls
        .iter()
        .flat_map(|i| i.members.iter().chain(&i.trait_members));
    for member in krate
        .decls
        .iter()
        .flat_map(|d| &d.members)
        .chain(impl_members)
    {
        metas.push(&member.meta);
        if let MemberKind::Fn(sig) = &member.kind {
            sigs.push(sig);
        }
    }
    for item in krate.modules.iter().flat_map(|m| &m.items) {
        metas.push(&item.meta);
        if let ItemKind::Fn(sig)
        | ItemKind::Foreign {
            item: Foreign::Fn(sig),
            ..
        } = &item.kind
        {
            sigs.push(sig);
        }
    }
    metas.extend(
        sigs.iter()
            .flat_map(|sig| sig.params.iter().map(|p| &p.meta)),
    );
    let modules = krate.modules.iter().map(|m| (&m.raw_doc, &m.lints));
    for (raw_doc, lints) in metas.iter().map(|m| (&m.raw_doc, &m.lints)).chain(modules) {
        let sorted = lints
            .windows(2)
            .all(|w| name_order(&w[0], &w[1]) == Ordering::Less);
        let trimmed = raw_doc
            .as_ref()
            .is_none_or(|d| !d.is_empty() && !d.starts_with('\n') && !d.ends_with('\n'));
        if !sorted || !trimmed {
            return Err("a node's lints are not sorted or its raw doc is not trimmed".to_owned());
        }
    }
    Ok(())
}
