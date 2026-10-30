//! Builds the census of a run. It lexes each target's module tree, where
//! the hits inside a test-only node count as test code, the files that the
//! target's test-only `mod` items load, each package's tests, benches,
//! examples, and build script, and the doc tests of each library, and it
//! counts the names that the run declares.

use std::collections::BTreeMap;
use std::ops::Range;

use proc_macro2::LineColumn;

use crate::analysis::{Extra, PackageInfo, TargetInfo};
use crate::analysis_items::{Scopes, innermost};
use crate::census::{Census, Context, Form, QualifiedPathHit, Scan, Source, TokenHit};
use crate::census_docs::{doc_texts, doctests};
use crate::census_lex::lex;
use crate::macros;
use crate::model::{Crate, FileId, ItemKind, MemberKind, ModId, Seg, Shape, VariantFields};
use crate::root::TargetKind;
use crate::source::SourceFile;

/// The test-only spans of one file as token positions, sorted and apart.
type Spans = Vec<(LineColumn, LineColumn)>;

/// What one token hit is counted by inside its source: its context, its
/// form, and whether it is in macro arguments.
type Key = (Context, Form, bool);

/// A file of a module tree, with the modules it holds and their bytes,
/// which place each qualified path.
type Placed<'a> = (&'a SourceFile, &'a [(Range<usize>, ModId)]);

/// Builds the census of a run's crates and the inputs analysis gathered
/// beside them. `scopes` holds, for each target and file, the modules
/// there with their bytes.
pub(crate) fn build(
    krates: &[Crate],
    targets: &[TargetInfo],
    packages: &[PackageInfo],
    scopes: &[Scopes],
) -> Census {
    let mut census = Census::default();
    for (t, krate) in krates.iter().enumerate() {
        census.tree.push(census.sources.len());
        for (f, file) in krate.files.iter().enumerate() {
            let source = Source {
                package: targets[t].package,
                target: Some(t),
                rel: file.rel.clone(),
                file: Some(f),
                context: Context::Production,
                line: 1,
                scan: Scan::Lexed,
            };
            let tree = (file, scopes[t][f].as_slice());
            add(
                &mut census,
                source,
                Some(&file.text),
                &spans(krate, f),
                Some(tree),
            );
        }
    }
    for (t, info) in targets.iter().enumerate() {
        for extra in &info.files {
            let source = outside(info.package, Some(t), extra);
            add(&mut census, source, extra.text.as_deref(), &[], None);
        }
    }
    for (p, package) in packages.iter().enumerate() {
        for extra in &package.files {
            let source = outside(p, None, extra);
            add(&mut census, source, extra.text.as_deref(), &[], None);
        }
    }
    for (t, krate) in krates.iter().enumerate() {
        if targets[t].kind != TargetKind::Lib {
            continue;
        }
        for (loc, doc) in doc_texts(krate) {
            for code in doctests(doc) {
                let source = Source {
                    package: targets[t].package,
                    target: Some(t),
                    rel: krate.files[loc.file].rel.clone(),
                    file: Some(loc.file),
                    context: Context::Doctest,
                    line: loc.line,
                    scan: Scan::Lexed,
                };
                add(&mut census, source, Some(&code), &[], None);
            }
        }
    }
    census.paths.sort();
    for (p, path) in census.paths.iter().enumerate() {
        if let (Some(first), Some(last)) = (path.segments.first(), path.segments.last()) {
            census.by_first.entry(first.clone()).or_default().push(p);
            census.by_last.entry(last.clone()).or_default().push(p);
        }
    }
    census.declarations = declarations(krates);
    census
}

/// Returns the source of a file outside a target's module tree.
fn outside(package: usize, target: Option<usize>, extra: &Extra) -> Source {
    Source {
        package,
        target,
        rel: extra.rel.clone(),
        file: None,
        context: extra.context,
        line: 1,
        scan: Scan::Lexed,
    }
}

/// Lexes one source's text into the census; a text that is missing or
/// does not lex leaves only the source, marked. Inside `spans`, hits count
/// as test code.
fn add(
    census: &mut Census,
    mut source: Source,
    text: Option<&str>,
    spans: &[(LineColumn, LineColumn)],
    tree: Option<Placed<'_>>,
) {
    let id = census.sources.len();
    let lexed = text.map(|text| lex(text, spans));
    source.scan = match &lexed {
        None => Scan::Unreadable,
        Some(None) => Scan::Unlexable,
        Some(Some(_)) => Scan::Lexed,
    };
    let (base, line) = (source.context, source.line);
    census.sources.push(source);
    let Some(Some(lexed)) = lexed else {
        return;
    };
    let context = |in_test: bool| if in_test { Context::Test } else { base };
    let mut counts: BTreeMap<&str, Vec<(Key, usize)>> = BTreeMap::new();
    for name in &lexed.names {
        let key = (context(name.in_test), name.form, name.in_macro);
        let keys = counts.entry(&lexed.words[name.text.clone()]).or_default();
        match keys.iter_mut().find(|(k, _)| *k == key) {
            Some((_, count)) => *count += 1,
            None => keys.push((key, 1)),
        }
    }
    for (name, mut keys) in counts {
        keys.sort_unstable();
        let hits = match census.tokens.get_mut(name) {
            Some(hits) => hits,
            None => census.tokens.entry(name.to_owned()).or_default(),
        };
        hits.extend(
            keys.into_iter()
                .map(|((context, form, in_macro), count)| TokenHit {
                    source: id,
                    context,
                    form,
                    in_macro,
                    count,
                }),
        );
    }
    for path in lexed.paths {
        let at_line = u32::try_from(path.at.line).unwrap_or(u32::MAX);
        census.paths.push(QualifiedPathHit {
            source: id,
            line: if base == Context::Doctest {
                line
            } else {
                at_line
            },
            context: context(path.in_test),
            module: tree.and_then(|(file, modules)| innermost(modules, file.offset(path.at))),
            segments: path.segments,
            global: path.global,
            in_macro: path.in_macro,
        });
    }
}

/// Returns the test-only spans of a crate's file, merged.
fn spans(krate: &Crate, f: FileId) -> Spans {
    let file = &krate.files[f];
    let ranges = &krate.test_ranges;
    let first = ranges.partition_point(|(at, _)| *at < f);
    let mut out: Spans = Vec::new();
    for (_, range) in ranges[first..].iter().take_while(|(at, _)| *at == f) {
        let span = (line_column(file, range.start), line_column(file, range.end));
        match out.last_mut() {
            Some(last) if span.0 <= last.1 => last.1 = last.1.max(span.1),
            _ => out.push(span),
        }
    }
    out
}

/// Returns the token position of a byte offset: its line, and its column
/// in characters.
fn line_column(file: &SourceFile, offset: usize) -> LineColumn {
    let offset = offset.min(file.text.len());
    let start = file.text[..offset].rfind('\n').map_or(0, |n| n + 1);
    LineColumn {
        line: usize::try_from(file.line_of(offset)).unwrap_or(usize::MAX),
        column: file.text[start..offset].chars().count(),
    }
}

/// Counts how many items of the run declare each name: declarations, their
/// variants and named variant fields, the members of declarations and
/// impls, modules, and module items other than re-exports and macro
/// invocations. Each declaration writes its name once.
fn declarations(krates: &[Crate]) -> BTreeMap<String, usize> {
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    let mut add = |name: &str| {
        let name = name.strip_prefix("r#").unwrap_or(name);
        match out.get_mut(name) {
            Some(count) => *count += 1,
            None if !name.is_empty() => {
                out.insert(name.to_owned(), 1);
            }
            None => {}
        }
    };
    for krate in krates {
        for decl in &krate.decls {
            add(&decl.name);
            let Shape::Enum(variants) = &decl.shape else {
                continue;
            };
            for variant in variants {
                add(&variant.name);
                if let VariantFields::Named(fields) = &variant.fields {
                    for field in fields {
                        if let Some(Seg::Plain(text)) = field.text.0.first() {
                            add(text.split(':').next().unwrap_or_default().trim());
                        }
                    }
                }
            }
        }
        let impls = krate
            .impls
            .iter()
            .flat_map(|i| i.members.iter().chain(&i.trait_members));
        for member in krate.decls.iter().flat_map(|d| &d.members).chain(impls) {
            if !matches!(
                member.kind,
                MemberKind::Mark(_) | MemberKind::Unsupported(_)
            ) {
                add(&member.name);
            }
        }
        for module in &krate.modules {
            if let Some(name) = module.path.last() {
                add(name);
            }
            for item in &module.items {
                let declares = match &item.kind {
                    ItemKind::Mark(text) => macros::definition_name(text).is_some(),
                    ItemKind::Reexport { .. } | ItemKind::Unsupported(_) => false,
                    ItemKind::Fn(_)
                    | ItemKind::Const { .. }
                    | ItemKind::Static { .. }
                    | ItemKind::Foreign { .. } => true,
                };
                if declares {
                    add(&item.name);
                }
            }
        }
    }
    out
}
