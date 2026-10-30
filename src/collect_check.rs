//! Checks the analysis facts before anything reads them, so a broken fact
//! stops a debug run as an internal error rather than reaching an analysis:
//! the facts collection keeps for each crate, and the manifest facts of each
//! package.

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use anyhow::{Context, Result, anyhow, bail};

use crate::collect_attrs::SERDE_PATH_KEYS;
use crate::collect_body_tokens::{MIN_TOKENS, SHINGLE};
use crate::collect_check_nodes::check_nodes;
use crate::inputs::Package;
use crate::model::{Crate, DeclKind, ItemKind, Member, MemberKind, RefId, Shape};
use crate::model_body::{Body, BodyFacts, BodyId, BodyKind, Owner, Placement, Role, Root};
use crate::root::TargetKind;

/// Checks the analysis facts of one collected, unattached crate: each
/// body's key, owner, and facts, the test-only ranges and files, the trait
/// impl members, every node's raw doc text and lint attributes, and the
/// paths `serde` attributes name.
///
/// # Errors
/// Returns an internal error that names the first broken invariant.
pub(crate) fn validate_body_facts(krate: &Crate) -> Result<()> {
    check_crate(krate).map_err(|why| {
        anyhow!(
            "internal error: the analysis facts of {} break an invariant: {why}",
            krate.name
        )
    })
}

/// Checks one package's analysis facts: its manifest facts, and that only a
/// lone target of a package with no name is a single file.
///
/// # Errors
/// Returns an internal error that names the first broken invariant.
pub(crate) fn validate_package_facts(package: &Package) -> Result<()> {
    package.facts.validate().with_context(|| {
        format!(
            "internal error: the manifest facts of {} break an invariant",
            package.dir.display()
        )
    })?;
    let targets = &package.input.targets;
    for target in targets {
        let single = target.kind == TargetKind::SingleFile;
        let alone = targets.len() == 1 && package.facts.name.is_none();
        if single && !alone {
            bail!(
                "internal error: target {} is a single file but not alone in a package without a name",
                target.stem
            );
        }
    }
    Ok(())
}

fn check_crate(krate: &Crate) -> Result<(), String> {
    let mut keys = BTreeSet::new();
    for (id, body) in krate.bodies.iter().enumerate() {
        if !keys.insert((body.loc.file, body.start, body.kind)) {
            return Err(format!(
                "body {id} repeats the file, start, and kind of another"
            ));
        }
        check_body(krate, id).map_err(|why| format!("body {id}: {why}"))?;
    }
    check_nodes(krate)?;
    for path in &krate.attr_paths {
        check_ref(krate, path.path_ref, &path.path, true)?;
        if !SERDE_PATH_KEYS.contains(&path.key.as_str()) || path.loc.file >= krate.files.len() {
            return Err(format!(
                "the serde path {} is malformed",
                path.path.join("::")
            ));
        }
    }
    Ok(())
}

fn check_body(krate: &Crate, id: BodyId) -> Result<(), String> {
    let body = &krate.bodies[id];
    let file = krate
        .files
        .get(body.loc.file)
        .ok_or("its file is out of range")?;
    let line = file.line_of(body.start);
    if body.start >= file.text.len() || body.loc.line != line {
        return Err("its start and line disagree".to_owned());
    }
    if !owner_holds(krate, body) {
        return Err(format!("{:?} does not hold a {:?}", body.owner, body.kind));
    }
    check_paths(krate, &body.facts)?;
    check_shape(&body.facts, line..=file.line_of(file.text.len()))
}

/// Tells whether a body's owner exists and holds a body of its kind.
fn owner_holds(krate: &Crate, body: &Body) -> bool {
    let wanted = |kind: &MemberKind| match body.kind {
        BodyKind::AssocConst => matches!(kind, MemberKind::Const { .. }),
        _ => matches!(kind, MemberKind::Fn(_)),
    };
    let named =
        |members: &[Member], name: &str| members.iter().any(|m| m.name == name && wanted(&m.kind));
    match (&body.owner, body.kind) {
        (Owner::Item(at), BodyKind::Fn | BodyKind::Const | BodyKind::Static) => {
            let item = krate
                .modules
                .get(at.module)
                .and_then(|m| m.items.get(at.item));
            matches!(
                (item.map(|i| &i.kind), body.kind),
                (Some(ItemKind::Fn(_)), BodyKind::Fn)
                    | (Some(ItemKind::Const { .. }), BodyKind::Const)
                    | (Some(ItemKind::Static { .. }), BodyKind::Static)
            )
        }
        (Owner::Anon(module), BodyKind::AnonConst) => *module < krate.modules.len(),
        (
            Owner::Impl(i, name),
            BodyKind::Method | BodyKind::TraitImplMethod | BodyKind::AssocConst,
        ) => krate
            .impls
            .get(*i)
            .is_some_and(|item| match (body.kind, item.trait_.is_some()) {
                (BodyKind::Method | BodyKind::AssocConst, false) => named(&item.members, name),
                (BodyKind::TraitImplMethod | BodyKind::AssocConst, true) => {
                    named(&item.trait_members, name)
                }
                _ => false,
            }),
        (Owner::Trait(d, name), BodyKind::TraitDefault | BodyKind::AssocConst) => krate
            .decls
            .get(*d)
            .is_some_and(|decl| decl.kind == DeclKind::Trait && named(&decl.members, name)),
        (Owner::Variant(d, name), BodyKind::Discriminant) => {
            krate.decls.get(*d).is_some_and(|decl| match &decl.shape {
                Shape::Enum(variants) => variants.iter().any(|v| v.name == *name),
                _ => false,
            })
        }
        _ => false,
    }
}

/// Checks a body's call, construct, and pattern facts, and its chains.
fn check_paths(krate: &Crate, facts: &BodyFacts) -> Result<(), String> {
    let uses = |name: &str| {
        facts
            .params
            .iter()
            .find(|p| p.name == name)
            .is_some_and(|p| p.uses > 0)
    };
    let names: BTreeSet<&str> = facts.params.iter().map(|p| p.name.as_str()).collect();
    if names.len() != facts.params.len() {
        return Err("a parameter name repeats".to_owned());
    }
    for call in &facts.calls {
        check_ref(krate, call.full_ref, &call.path, true)?;
        check_prefix(krate, call.prefix_ref, &call.path)?;
        let placed = |list: &[(usize, String)]| {
            list.windows(2).all(|w| w[0].0 < w[1].0) && list.iter().all(|(i, _)| *i < call.args)
        };
        let literals = call.literal_args.iter().all(|(_, text)| !text.is_empty());
        let passed = call.passed_params.iter().all(|(_, name)| uses(name));
        let called = call.role == Role::Call || call.args == 0;
        if !(placed(&call.literal_args)
            && placed(&call.passed_params)
            && literals
            && passed
            && called)
        {
            return Err(format!(
                "the call fact of {} is malformed",
                call.path.join("::")
            ));
        }
    }
    for path in &facts.constructs {
        check_ref(krate, path.full_ref, &path.path, false)?;
        check_prefix(krate, path.prefix_ref, &path.path)?;
    }
    for path in &facts.patterns {
        check_ref(krate, path.full_ref, &path.path, true)?;
        check_prefix(krate, path.prefix_ref, &path.path)?;
    }
    if !facts.chains.windows(2).all(|w| w[0] < w[1]) {
        return Err("the operation chains are not sorted, each once".to_owned());
    }
    for chain in &facts.chains {
        let rooted = match &chain.root {
            Root::Field(_) => true,
            Root::Param(name) => name != "self" && uses(name),
        };
        let literal = chain.first_literal.is_none() || !chain.methods.is_empty();
        let looped = !chain.in_loop || facts.loops > 0;
        if !(rooted && literal && looped) {
            return Err(format!("the chain {chain:?} is malformed"));
        }
    }
    Ok(())
}

/// Checks a body's lexical lists, counts, statements, fingerprint, and
/// comments against each other and against the body's `lines`.
fn check_shape(facts: &BodyFacts, lines: RangeInclusive<u32>) -> Result<(), String> {
    let strict = |list: &[String]| list.windows(2).all(|w| w[0] < w[1]);
    let lists = [
        &facts.macros,
        &facts.macro_idents,
        &facts.strings,
        &facts.numbers,
    ];
    if !facts.methods.windows(2).all(|w| w[0] <= w[1]) || !lists.iter().all(|l| strict(l)) {
        return Err("a name or literal list is not sorted".to_owned());
    }
    let tokens = facts.fingerprint.tokens;
    let counts = facts.counts;
    let counted = counts.tries + counts.errs + counts.nones + counts.awaits + counts.panics;
    if tokens > 0 && (counted > tokens || facts.loops > tokens) {
        return Err("more markers or loops are counted than the body has tokens".to_owned());
    }
    if facts.loop_depth > facts.loops {
        return Err("loops nest deeper than there are loops".to_owned());
    }
    if facts.statements > facts.stmt_lines.len()
        || (facts.single.is_some() && facts.statements != 1)
    {
        return Err("the statement count disagrees with the statements".to_owned());
    }
    let inside = |&(first, last): &(u32, u32)| lines.contains(&first) && first <= last;
    if !facts.stmt_lines.iter().all(inside) || facts.stmt_lines.iter().any(|s| s.1 > *lines.end()) {
        return Err("a statement lies outside the body's lines".to_owned());
    }
    let shingles = &facts.fingerprint.shingles;
    let sized = if tokens < MIN_TOKENS {
        shingles.is_empty()
    } else {
        shingles.len() <= tokens + 1 - SHINGLE
    };
    if !shingles.windows(2).all(|w| w[0] < w[1]) || !sized {
        return Err("the shingles are not sorted, each once, or outnumber the tokens".to_owned());
    }
    if !facts.comments.windows(2).all(|w| w[0].line <= w[1].line) {
        return Err("the comments are not in source order".to_owned());
    }
    for comment in &facts.comments {
        let line = comment.line;
        let placed = match comment.stmt {
            None => comment.placement == Placement::Inner && comment.fingerprint.is_empty(),
            Some(s) => {
                facts
                    .stmt_lines
                    .get(s)
                    .is_some_and(|&(first, last)| match comment.placement {
                        Placement::Leading => first >= line,
                        Placement::Trailing => last <= line,
                        Placement::Inner => first <= line && line <= last,
                    })
            }
        };
        let written = !comment.text.is_empty() && lines.contains(&line);
        if !(placed && written && strict(&comment.fingerprint)) {
            return Err(format!("the comment on line {line} is misplaced"));
        }
    }
    Ok(())
}

/// Checks that a fact's reference is the comment reference for its path,
/// with `value` saying whether it may be a value.
fn check_ref(krate: &Crate, id: RefId, path: &[String], value: bool) -> Result<(), String> {
    let fits = krate
        .refs
        .get(id)
        .is_some_and(|r| r.comment && r.path == path && r.may_be_value == value);
    if fits {
        Ok(())
    } else {
        Err(format!(
            "reference {id} is not the comment reference of {}",
            path.join("::")
        ))
    }
}

/// Checks that a path of two or more segments has a reference for its
/// prefix, and a path of one segment none.
fn check_prefix(krate: &Crate, prefix: Option<RefId>, path: &[String]) -> Result<(), String> {
    let short = path.split_last().map_or(&[][..], |(_, rest)| rest);
    match prefix {
        Some(id) if !short.is_empty() => check_ref(krate, id, short, false),
        None if short.is_empty() => Ok(()),
        _ => Err(format!(
            "the prefix reference of {} is wrong",
            path.join("::")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect_body_facts::tests::collected;

    const LIB: &str = "pub struct Gauge { level: u8 }\n\
                       impl Gauge {\n\
                       /// Reads it.\n\
                       #[allow(clippy::len_without_is_empty)]\n\
                       pub fn read(&self, scale: u8) -> u8 {\n\
                       // scale it\n\
                       let raw = self.level.checked_mul(scale).unwrap_or(0);\n\
                       for step in 0..3 { if raw > step * 40 { return raw - 7; } }\n\
                       raw + helper(scale, \"twenty-five characters long\")\n\
                       }\n\
                       }\n\
                       impl Default for Gauge { fn default() -> Self { Gauge { level: 1 } } }\n\
                       fn helper(x: u8, _name: &str) -> u8 { x }\n";

    /// Collects `LIB`, breaks it with `edit`, and checks that validation
    /// fails with an internal error that says `why`.
    fn fails(edit: impl FnOnce(&mut Crate), why: &str) {
        let (_tmp, mut krate) = collected(LIB);
        edit(&mut krate);
        let error = validate_body_facts(&krate).unwrap_err().to_string();
        assert!(error.starts_with("internal error"), "{error}");
        assert!(error.contains(why), "{error} does not say {why}");
    }

    #[test]
    fn a_collected_crate_keeps_every_invariant() {
        let (_tmp, krate) = collected(LIB);
        validate_body_facts(&krate).unwrap();
        let read = &krate.bodies[0].facts;
        assert!(!read.fingerprint.shingles.is_empty(), "read is long enough");
        assert_eq!(read.comments.len(), 1);
        assert_eq!(
            krate.impls[0].members[0].meta.lints,
            ["allow(clippy::len_without_is_empty)"]
        );
    }

    #[test]
    fn broken_facts_are_internal_errors() {
        fails(
            |krate| {
                let first = krate.bodies[0].clone();
                krate.bodies.push(first);
            },
            "repeats",
        );
        fails(
            |krate| krate.bodies[0].facts.fingerprint.shingles.reverse(),
            "shingles",
        );
        fails(
            |krate| krate.bodies[0].facts.calls[0].full_ref = usize::MAX,
            "reference",
        );
        fails(
            |krate| krate.bodies[0].owner = Owner::Anon(0),
            "does not hold",
        );
        fails(
            |krate| krate.bodies[0].facts.comments[0].stmt = Some(usize::MAX),
            "misplaced",
        );
        fails(
            |krate| krate.modules[0].lints = vec!["warn(a)".to_owned(), "allow(b)".to_owned()],
            "lints",
        );
    }
}
