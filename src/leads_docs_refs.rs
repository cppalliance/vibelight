//! The doc detectors of names that nothing reaches: explicit intra-doc
//! links that resolve to no item, and backticked paths that the run's
//! code, strings, attributes, and manifests never write. Both rest on what
//! the run does not see, so both name macros as their blind spot.

use std::collections::BTreeSet;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::census_lex::{PATH_KEYWORDS, is_keyword};
use crate::leads::{Lead, Mode};
use crate::leads_docs::{Doc, Docs, Node};
use crate::leads_docs_std::is_std_name;
use crate::leads_docs_text::{Kind, identifiers, quote};
use crate::leads_items::is_wire;
use crate::model::{Crate, DeclId, ItemKind, Meta, Param, RefId, Shape, TypeRef, VariantFields};
use crate::prelude::prelude_path;
use crate::typerefs::strip_raw;

/// What the leads of both detectors cannot see.
const LIMITS: &[&str] = &["macros"];

/// Version 1 of the words that make a sentence's names examples, such as
/// "an output stem, such as `foo_bar`".
const EXAMPLE_WORDS: [&str; 5] = ["e.g", "example", "instance", "like", "such"];

/// Finds the leads of names in docs that nothing reaches.
pub(crate) fn find(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    let known = Known::new(run);
    for doc in &docs.all {
        let linked = unresolved_links(run, &known, doc, out);
        unresolved_backticks(run, (&known, docs), doc, &linked, out);
    }
}

/// The names that the run writes outside its docs.
struct Known<'r, 'a> {
    run: &'r AnalysisRun<'a>,
    /// The identifiers of every body's string literals and of every
    /// attribute, and the names of every target, package, feature, and
    /// dependency.
    written: BTreeSet<String>,
    /// The crate names that a path may start with: every target's, and
    /// every dependency's.
    crates: BTreeSet<String>,
}

impl<'r, 'a> Known<'r, 'a> {
    fn new(run: &'r AnalysisRun<'a>) -> Known<'r, 'a> {
        let mut written = BTreeSet::new();
        let mut add = |text: &str| written.extend(identifiers(text).map(strip_raw));
        for krate in run.krates {
            for body in &krate.bodies {
                body.facts.strings.iter().for_each(|s| add(s));
            }
            for meta in metas(krate) {
                meta.attrs.iter().for_each(|attr| add(attr));
            }
        }
        let mut crates: BTreeSet<String> = ["alloc", "core", "proc_macro", "std", "test"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        for target in run.targets {
            crates.insert(target.crate_name.clone());
        }
        for package in run.packages {
            let facts = &package.facts;
            for feature in &facts.features {
                add(&feature.name);
            }
            for dep in &facts.deps {
                crates.insert(dep.name.replace('-', "_"));
                crates.extend(dep.package.iter().map(|p| p.replace('-', "_")));
                dep.features.iter().for_each(|feature| add(feature));
            }
            facts.name.iter().for_each(|name| add(name));
        }
        written.extend(crates.iter().cloned());
        Known {
            run,
            written,
            crates,
        }
    }

    /// Tells whether a name is a keyword, a primitive, a prelude or `std`
    /// name, or shaped like a generic parameter, such as `T` or `K2`.
    fn standard(name: &str) -> bool {
        let mut chars = name.chars();
        let generic = chars.next().is_some_and(|c| c.is_ascii_uppercase())
            && chars.all(|c| c.is_ascii_digit());
        generic || is_keyword(name) || prelude_path(name).is_some() || is_std_name(name)
    }

    /// Tells whether an item of the run or a crate has a name, or the name
    /// is standard.
    fn declared(&self, name: &str) -> bool {
        Known::standard(name) || self.run.census.declared(name) > 0 || self.crates.contains(name)
    }

    /// Tells whether the run writes a name anywhere outside its docs, or
    /// declares a name of the same words in `CamelCase`, as a wire spelling
    /// such as `step_failed` for `StepFailed` is.
    fn observed(&self, name: &str) -> bool {
        let camel: String = (name.split('_'))
            .map(|word| {
                let mut chars = word.chars();
                let first = chars.next().map(|c| c.to_ascii_uppercase());
                first.into_iter().chain(chars).collect::<String>()
            })
            .collect();
        self.declared(name)
            || !self.run.census.token_hits(name).is_empty()
            || self.written.contains(name)
            || (name.contains('_') && self.run.census.declared(&camel) > 0)
    }
}

/// Tells whether a backticked path is shaped like a Rust path to an item:
/// two or more segments, a call or a macro, or a `snake_case` name with an
/// underscore inside. A lone `CamelCase`, `camelCase`, or `SCREAMING_CASE`
/// name or a lone lowercase word is as often the name of something outside
/// the code, such as a type of another system or a key of a format.
fn rust_shaped(reference: &TypeRef) -> bool {
    let called = reference.text.ends_with("()") || reference.text.ends_with('!');
    match reference.path.as_slice() {
        [one] => {
            let snake = one.contains('_')
                && !one.starts_with('_')
                && !one.ends_with('_')
                && (one.chars()).all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            called || snake
        }
        _ => true,
    }
}

/// Tells whether a doc describes an outside format: the doc of a wire
/// type, of an item with a `repr` attribute, or of an alias of an
/// `extern` type, of a field or a variant of one, of an `extern` function,
/// or of a node in a module that declares an extern item, or the doc of a
/// module that declares a wire type.
fn outside_format(run: &AnalysisRun<'_>, docs: &Docs<'_>, doc: &Doc<'_>) -> bool {
    let t = doc.item.target;
    let krate = &run.krates[t];
    let outside = |d: DeclId| {
        let at = ItemId::decl(t, d);
        let decl = &krate.decls[d];
        let foreign = match &decl.shape {
            Shape::Alias(target) => target.visible(&krate.refs).contains("extern"),
            _ => false,
        };
        foreign || is_wire(run, at) || decl.meta.attrs.iter().any(|a| a.starts_with("repr"))
    };
    let owner = match doc.node {
        Node::Decl(d) | Node::Variant(d, _) => Some(d),
        Node::Field(f) => match docs.catalog.fields[f].decl.item {
            ItemRef::Decl(d) => Some(d),
            _ => None,
        },
        Node::Func(f) => {
            let quals = &docs.catalog.funcs[f].sig.quals;
            if quals.iter().any(|q| q.starts_with("extern")) {
                return true;
            }
            None
        }
        Node::Value | Node::Module => None,
    };
    let module = krate.modules.get(doc.module);
    let externs = module.is_some_and(|m| {
        (m.items.iter()).any(|item| matches!(item.kind, ItemKind::Foreign { .. }))
    });
    let wired = (0..krate.decls.len()).any(|d| krate.decls[d].module == doc.module && outside(d));
    owner.is_some_and(outside) || externs || (doc.node == Node::Module && wired)
}

/// Returns every node's meta of a crate.
fn metas(krate: &Crate) -> impl Iterator<Item = &Meta> {
    fn params(params: &[Param]) -> impl Iterator<Item = &Meta> {
        params.iter().map(|p| &p.meta)
    }
    let decls = krate.decls.iter().flat_map(|decl| {
        let mut found = vec![&decl.meta];
        found.extend(decl.members.iter().map(|m| &m.meta));
        match &decl.shape {
            Shape::Tuple(fields) => found.extend(params(fields)),
            Shape::Enum(variants) => {
                for variant in variants {
                    found.push(&variant.meta);
                    if let VariantFields::Tuple(fields) | VariantFields::Named(fields) =
                        &variant.fields
                    {
                        found.extend(params(fields));
                    }
                }
            }
            _ => {}
        }
        found
    });
    let items = (krate.modules.iter()).flat_map(|m| m.items.iter().map(|item| &item.meta));
    let impls = (krate.impls.iter())
        .flat_map(|i| i.members.iter().chain(&i.trait_members).map(|m| &m.meta));
    decls.chain(items).chain(impls)
}

/// Returns the path a link destination or a code span names, without a
/// disambiguator such as `struct@`, a trailing `()` or `!`, or a leading
/// `::`, and without `r#`; `None` when it is no path, such as a URL.
pub(crate) fn intra_path(dest: &str) -> Option<Vec<String>> {
    let dest = dest.trim().trim_matches('`');
    let dest = match dest.split_once('@') {
        Some((kind, rest)) if !kind.is_empty() && kind.chars().all(|c| c.is_ascii_lowercase()) => {
            rest
        }
        _ => dest,
    };
    let dest = (dest.strip_suffix("()"))
        .or_else(|| dest.strip_suffix('!'))
        .unwrap_or(dest);
    let body = dest.strip_prefix("::").unwrap_or(dest);
    let valid = !body.is_empty()
        && body.split("::").all(|segment| {
            let segment = segment.strip_prefix("r#").unwrap_or(segment);
            segment.starts_with(|c: char| c.is_alphabetic() || c == '_')
                && segment.chars().all(|c| c.is_alphanumeric() || c == '_')
        });
    valid.then(|| body.split("::").map(strip_raw).collect())
}

/// Tells whether a doc's reference of target `t` reaches something: an
/// item of the run, a path outside the crate, or a generic parameter.
fn reached(run: &AnalysisRun<'_>, t: usize, r: RefId) -> bool {
    let reference = &run.krates[t].refs[r];
    let generic = (reference.path.first()).is_some_and(|first| reference.generics.contains(first));
    generic || reference.outside().is_some() || run.item_of_ref(t, r).is_some()
}

/// `doc-unresolved-link`: explicit intra-doc links of a doc whose path
/// resolves to nothing, whose last name no item of the run and no
/// standard name has, and whose first name, for a path of two or more,
/// is a path keyword or a name the run declares. A bare `[word]` in
/// lowercase is prose, and a one-name link in a module with a glob import
/// may reach through the glob. Returns the paths of every link the doc
/// writes.
fn unresolved_links(
    run: &AnalysisRun<'_>,
    known: &Known<'_, '_>,
    doc: &Doc<'_>,
    out: &mut Vec<Lead>,
) -> Vec<Vec<String>> {
    let t = doc.item.target;
    let krate = &run.krates[t];
    let mut linked = Vec::new();
    let mut missing = Vec::new();
    for link in &doc.prose.links {
        let Some(path) = intra_path(&link.dest) else {
            continue;
        };
        linked.push(path.clone());
        let label = &doc.prose.toks[link.label.clone()];
        let coded = matches!(label, [tok] if tok.kind == Kind::Code);
        let capital = path[0].starts_with(char::is_uppercase);
        if link.shortcut && !coded && path.len() == 1 && !capital {
            continue;
        }
        let found = doc.refs().find(|&r| krate.refs[r].path == path);
        if found.is_some_and(|r| reached(run, t, r)) {
            continue;
        }
        let module = found.map_or(doc.module, |r| krate.refs[r].module);
        let globbed =
            (krate.modules.get(module)).is_some_and(|m| m.uses.iter().any(|u| u.name.is_none()));
        let local = |name: &String| PATH_KEYWORDS.contains(&name.as_str()) || known.declared(name);
        let outside = path.len() > 1 && found.is_none() && !local(&path[0]);
        let last = &path[path.len() - 1];
        if known.declared(last) || outside || (path.len() == 1 && globbed) {
            continue;
        }
        missing.push(quote(&doc.prose.doc[link.span.clone()]));
    }
    if !missing.is_empty() {
        missing.dedup();
        let evidence = format!(
            "{} reach no item, and no item of the run has the last name",
            missing.join(", ")
        );
        let lead = Lead::new("doc-unresolved-link", Mode::Resolved, evidence).limits(LIMITS);
        out.push(doc.add(run, lead));
    }
    linked
}

/// `doc-unresolved-backtick`: backticked paths of a doc, outside its
/// explicit links and outside the docs of outside formats, that are shaped
/// like Rust paths, resolve to nothing, and have a name that the run's
/// code, string literals, attributes, manifests, and the doc's own code
/// blocks never write and that no standard name has. A path written only
/// in sentences of examples, or with a one-letter word that marks a
/// placeholder, such as `get_x`, is left out.
fn unresolved_backticks(
    run: &AnalysisRun<'_>,
    (known, docs): (&Known<'_, '_>, &Docs<'_>),
    doc: &Doc<'_>,
    linked: &[Vec<String>],
    out: &mut Vec<Lead>,
) {
    let t = doc.item.target;
    let krate = &run.krates[t];
    if outside_format(run, docs, doc) {
        return;
    }
    let blocks: BTreeSet<&str> = (doc.prose.blocks.iter())
        .flat_map(|block| identifiers(&doc.prose.doc[block.clone()]))
        .collect();
    let prose = &doc.prose;
    let example = |path: &[String]| {
        let mut sentences = (prose.toks.iter())
            .filter(|tok| tok.kind == Kind::Code && intra_path(&tok.text).as_deref() == Some(path))
            .map(|tok| tok.sentence)
            .peekable();
        let cued = |s: usize| {
            (prose.toks.iter()).any(|tok| {
                tok.sentence == s
                    && tok.kind == Kind::Word
                    && EXAMPLE_WORDS.contains(&tok.text.as_str())
            })
        };
        sentences.peek().is_some() && sentences.all(cued)
    };
    let placeholder = |path: &[String]| {
        (path.iter()).any(|name| name.split('_').any(|word| word.chars().count() == 1))
    };
    let mut unseen: Vec<String> = Vec::new();
    for r in doc.refs() {
        let reference = &krate.refs[r];
        let skipped = linked.contains(&reference.path)
            || !rust_shaped(reference)
            || placeholder(&reference.path)
            || example(&reference.path);
        if skipped || reached(run, t, r) {
            continue;
        }
        let mut names = (reference.path.iter()).filter(|n| !PATH_KEYWORDS.contains(&n.as_str()));
        if names.any(|n| !known.observed(n) && !blocks.contains(n.as_str())) {
            let shown = format!("`{}`", reference.path.join("::"));
            if !unseen.contains(&shown) {
                unseen.push(shown);
            }
        }
    }
    if !unseen.is_empty() {
        let evidence = format!(
            "{} name nothing that the run's code, strings, or manifests write",
            unseen.join(", ")
        );
        let lead = Lead::new("doc-unresolved-backtick", Mode::Lexical, evidence).limits(LIMITS);
        out.push(doc.add(run, lead));
    }
}
