//! The facts that the records of one target's outline carry: what names
//! each declaration in its signatures, what re-exports each item, where
//! other targets and test code are observed to use an item, who calls a
//! function directly, and what each body holds. Every list is sorted, each
//! entry once.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::analysis_paths::PathKind;
use crate::analysis_uses::UseKind;
use crate::census::Context;
use crate::model::{
    Crate, DeclId, ItemKind, MemberKind, ModId, Note, Shape, Text, VariantFields, path_order,
};
use crate::model_body::{BodyFacts, Role, Root, Single};
use crate::render_slop::AnalysisTarget;
use crate::render_slop_catalog::{BodyComment, BodyRecord};

/// Returns a list, or `None` when it is empty.
pub(crate) fn some<T>(list: Vec<T>) -> Option<Vec<T>> {
    (!list.is_empty()).then_some(list)
}

/// Returns `Some(true)` for a set flag, so a clear one is left out.
pub(crate) fn flag(set: bool) -> Option<bool> {
    set.then_some(true)
}

/// Sorts names that are paths segment by segment, each once.
pub(crate) fn sorted_paths(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let set: BTreeSet<String> = names.into_iter().collect();
    let mut out: Vec<String> = set.into_iter().collect();
    out.sort_by(|a, b| path_order(&segments(a), &segments(b)).then_with(|| a.cmp(b)));
    out
}

fn segments(path: &str) -> Vec<String> {
    path.split("::").map(str::to_owned).collect()
}

/// Returns a module's path as an outline writes it: `crate` for the root.
pub(crate) fn module_path(krate: &Crate, m: ModId) -> String {
    let path = &krate.modules[m].path;
    if path.is_empty() {
        "crate".to_owned()
    } else {
        path.join("::")
    }
}

/// Returns a path below a target's root as the outline of target `own`
/// writes it: as it is in its own target, and after `::` and the other
/// target's stem elsewhere, such as `::alpha::Meter`.
pub(crate) fn subject(run: &AnalysisRun<'_>, own: usize, target: usize, path: &[String]) -> String {
    let joined = path.join("::");
    match (target == own, joined.is_empty()) {
        (true, true) => "crate".to_owned(),
        (true, false) => joined,
        (false, true) => format!("::{}", run.targets[target].stem),
        (false, false) => format!("::{}::{joined}", run.targets[target].stem),
    }
}

/// Returns the plain comments of a node's notes, and its trailing comment,
/// one line after another.
pub(crate) fn comment(notes: &[Note], trailing: Option<&String>) -> Option<String> {
    let mut lines: Vec<&str> = notes
        .iter()
        .filter_map(|note| match note {
            Note::Plain(lines) => Some(lines),
            Note::Doc(_) => None,
        })
        .flatten()
        .map(String::as_str)
        .collect();
    lines.extend(trailing.map(String::as_str));
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// Returns the doc lines of notes as their text shows, for a node that keeps
/// no raw doc, such as an impl.
pub(crate) fn note_doc(notes: &[Note], krate: &Crate) -> Option<String> {
    let lines: Vec<String> = notes
        .iter()
        .filter_map(|note| match note {
            Note::Doc(lines) => Some(lines),
            Note::Plain(_) => None,
        })
        .flatten()
        .map(|line| line.visible(&krate.refs))
        .collect();
    let text = lines.join("\n");
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// The facts of one target's outline.
pub(crate) struct Facts<'a> {
    target: AnalysisTarget<'a>,
    /// For each declaration of the target, the declarations and modules
    /// whose signatures name it.
    named_by: Vec<BTreeSet<String>>,
    /// For each item of the run that a re-export reaches, the modules that
    /// re-export it, as this outline writes them.
    reexported_by: BTreeMap<ItemId, BTreeSet<String>>,
}

impl<'a> Facts<'a> {
    pub(crate) fn new(target: AnalysisTarget<'a>) -> Facts<'a> {
        let AnalysisTarget { run, target: t } = target;
        let krate = &run.krates[t];
        let mut named_by = vec![BTreeSet::new(); krate.decls.len()];
        let mut note = |r: usize, user: &str, own: Option<DeclId>| {
            let reference = &krate.refs[r];
            if let Some(d) = reference.decl().filter(|_| !reference.comment) {
                if Some(d) != own {
                    named_by[d].insert(user.to_owned());
                }
            }
        };
        for d in 0..krate.decls.len() {
            let user = krate.decl_path(d).join("::");
            for text in decl_texts(run, t, d) {
                text.refs().for_each(|r| note(r, &user, Some(d)));
            }
        }
        for (m, module) in krate.modules.iter().enumerate() {
            let user = module_path(krate, m);
            for item in module.items.iter().filter(|i| !is_reexport(&i.kind)) {
                for text in crate::order::item_texts(&item.kind) {
                    text.refs().for_each(|r| note(r, &user, None));
                }
            }
        }
        let mut reexported_by: BTreeMap<ItemId, BTreeSet<String>> = BTreeMap::new();
        for (u, other) in run.krates.iter().enumerate() {
            for (m, module) in other.modules.iter().enumerate() {
                for item in &module.items {
                    let ItemKind::Reexport { text, .. } = &item.kind else {
                        continue;
                    };
                    for r in text.refs() {
                        if let Some(found) = run.item_of_ref(u, r) {
                            let shown = subject(run, t, u, &other.modules[m].path);
                            reexported_by.entry(found).or_default().insert(shown);
                        }
                    }
                }
            }
        }
        Facts {
            target,
            named_by,
            reexported_by,
        }
    }

    /// Returns what names declaration `d` of the target in signatures.
    pub(crate) fn named_by(&self, d: DeclId) -> Option<Vec<String>> {
        some(sorted_paths(self.named_by[d].iter().cloned()))
    }

    /// Returns the modules that re-export an item of the target.
    pub(crate) fn reexported_by(&self, item: ItemRef) -> Option<Vec<String>> {
        let at = ItemId {
            target: self.target.target,
            item,
        };
        let found = self.reexported_by.get(&at)?;
        some(sorted_paths(found.iter().cloned()))
    }

    /// Returns the other targets observed to use an item of the target, and
    /// the kinds of code other than production code observed to use it;
    /// links in docs do not count.
    pub(crate) fn observed(&self, item: ItemRef) -> (Option<Vec<String>>, Option<Vec<String>>) {
        let AnalysisTarget { run, target: t } = self.target;
        let at = ItemId { target: t, item };
        let mut crates = BTreeSet::new();
        let mut tests = BTreeSet::new();
        for found in run.uses_of(at).iter().filter(|u| u.kind != UseKind::Doc) {
            let source = &run.census.sources[found.source];
            if let Some(other) = source.target.filter(|&other| other != t) {
                crates.insert(run.targets[other].stem.clone());
            }
            if let Some(name) = context_name(found.context) {
                tests.insert(name.to_owned());
            }
        }
        (
            some(crates.into_iter().collect()),
            some(tests.into_iter().collect()),
        )
    }

    /// Returns the bodies of the run whose direct calls resolve to an item
    /// of the target.
    pub(crate) fn direct_callers(&self, item: ItemRef) -> Option<Vec<String>> {
        let AnalysisTarget { run, target: t } = self.target;
        let at = ItemId { target: t, item };
        let callers = run
            .edges_to(at)
            .filter(|edge| edge.kind == PathKind::Call(Role::Call))
            .map(|edge| {
                let caller = run.body_items[edge.target][edge.body].map(|item| ItemId {
                    target: edge.target,
                    item,
                });
                let path = caller.map_or_else(
                    || {
                        let body = &run.krates[edge.target].bodies[edge.body];
                        let m = run
                            .module_at(edge.target, body.loc.file, body.start)
                            .unwrap_or(0);
                        let mut path = run.krates[edge.target].modules[m].path.clone();
                        path.push("_".to_owned());
                        path
                    },
                    |caller| run.path(caller),
                );
                subject(run, t, edge.target, &path)
            });
        some(sorted_paths(callers))
    }

    /// Returns the facts of an item's body: its counts, its direct uses of
    /// `self` fields and parameters, and its comments.
    pub(crate) fn body(
        &self,
        item: ItemRef,
    ) -> (
        Option<BodyRecord>,
        Option<Vec<String>>,
        Option<Vec<BodyComment>>,
    ) {
        let AnalysisTarget { run, target: t } = self.target;
        let at = ItemId { target: t, item };
        let Some(b) = run.body_of(at) else {
            return (None, None, None);
        };
        let facts = &run.krates[t].bodies[b].facts;
        (Some(counts(facts)), some(ops(facts)), comments(facts))
    }

    /// Returns the comments of an item's body alone.
    pub(crate) fn comments(&self, item: ItemRef) -> Option<Vec<BodyComment>> {
        self.body(item).2
    }
}

/// Returns a context's name, or `None` for production code.
fn context_name(context: Context) -> Option<&'static str> {
    match context {
        Context::Production => None,
        Context::Test => Some("test"),
        Context::Bench => Some("bench"),
        Context::Example => Some("example"),
        Context::Build => Some("build"),
        Context::Doctest => Some("doctest"),
    }
}

fn is_reexport(kind: &ItemKind) -> bool {
    matches!(kind, ItemKind::Reexport { .. })
}

/// Returns the texts of declaration `d` of target `t` that can name types:
/// its generics, `where` clause, fields, variants, alias target,
/// supertraits, and own items, and the members and traits of the impls
/// that belong to it.
fn decl_texts<'a>(run: &AnalysisRun<'a>, t: usize, d: DeclId) -> Vec<&'a Text> {
    let krate: &'a Crate = &run.krates[t];
    let decl = &krate.decls[d];
    let mut texts = vec![&decl.generics];
    texts.extend(decl.where_clause.iter().flat_map(|clause| clause.0.iter()));
    match &decl.shape {
        Shape::Tuple(params) => texts.extend(params.iter().map(|p| &p.text)),
        Shape::Enum(variants) => {
            for variant in variants {
                if let VariantFields::Tuple(params) | VariantFields::Named(params) = &variant.fields
                {
                    texts.extend(params.iter().map(|p| &p.text));
                }
            }
        }
        Shape::Alias(target) => texts.push(target),
        Shape::Trait { supertraits, .. } => texts.extend(supertraits.iter().map(|b| &b.text)),
        Shape::Named | Shape::Unit => {}
    }
    let impls = run.impls_of[t][d].iter().map(|&i| &krate.impls[i]);
    let mut members: Vec<&'a crate::model::Member> = decl.members.iter().collect();
    for item in impls {
        match &item.trait_ {
            Some(base) => texts.push(&base.text),
            None => members.extend(&item.members),
        }
    }
    for member in members {
        match &member.kind {
            MemberKind::Field(ty) | MemberKind::Type(ty) | MemberKind::Const { ty, .. } => {
                texts.push(ty);
            }
            MemberKind::Fn(sig) => texts.extend(sig.texts()),
            MemberKind::Mark(_) | MemberKind::Unsupported(_) => {}
        }
    }
    texts
}

/// Returns the texts that can name types for every declaration of target
/// `t`, with each declaration's path, for the external items.
pub(crate) fn texts_by_decl<'a>(run: &AnalysisRun<'a>, t: usize) -> Vec<(String, Vec<&'a Text>)> {
    let krate = &run.krates[t];
    (0..krate.decls.len())
        .map(|d| (krate.decl_path(d).join("::"), decl_texts(run, t, d)))
        .collect()
}

/// Returns a body's counts.
fn counts(facts: &BodyFacts) -> BodyRecord {
    let count = |n: usize| (n > 0).then_some(n);
    BodyRecord {
        statements: facts.statements,
        tokens: facts.fingerprint.tokens,
        loops: count(facts.loops),
        loop_depth: (facts.loop_depth > 1).then_some(facts.loop_depth),
        tries: count(facts.counts.tries),
        errs: count(facts.counts.errs),
        nones: count(facts.counts.nones),
        awaits: count(facts.counts.awaits),
        panics: count(facts.counts.panics),
        macros: some(facts.macros.clone()),
        single: facts.single.as_ref().map(|single| match single {
            Single::Call(path) => format!("call {}", path.join("::")),
            Single::Method(name) => format!("method {name}"),
            Single::Field(name) => format!("field {name}"),
        }),
    }
}

/// Returns a body's direct uses of `self` fields and parameters, such as
/// `self.items.remove(0) [loop]`.
fn ops(facts: &BodyFacts) -> Vec<String> {
    let mut out: Vec<String> = facts
        .chains
        .iter()
        .map(|chain| {
            let mut text = match &chain.root {
                Root::Field(name) => format!("self.{name}"),
                Root::Param(name) => name.clone(),
            };
            for (i, method) in chain.methods.iter().enumerate() {
                text.push('.');
                text.push_str(method);
                if let (0, Some(first)) = (i, &chain.first_literal) {
                    let _ = write!(text, "({first})");
                }
            }
            if chain.in_loop {
                text.push_str(" [loop]");
            }
            if chain.write {
                text.push_str(" [write]");
            }
            text
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Returns a body's comments, each with its lines joined.
fn comments(facts: &BodyFacts) -> Option<Vec<BodyComment>> {
    some(
        facts
            .comments
            .iter()
            .map(|comment| BodyComment {
                text: comment.text.join("\n"),
                line: comment.line,
            })
            .collect(),
    )
}
