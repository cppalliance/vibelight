//! The lexical census of a run: how often each scanned source writes each
//! name, split by source and context, and the qualified paths it writes,
//! kept apart from the bare names. A hit shows that code writes a name,
//! never that it uses an item: macro expansion, generated files, disabled
//! `cfg` selection, dynamic dispatch, and code outside the run go unseen.

use std::collections::BTreeMap;

use crate::model::{FileId, ModId};

/// Index into [`Census::sources`].
pub(crate) type SourceId = usize;

/// The kind of code a hit is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Context {
    /// A target's module tree, outside its test-only nodes.
    Production,
    /// Test-only code: a `cfg(test)` node, a file that a test-only `mod`
    /// item loads, or a package test.
    Test,
    /// A package bench.
    Bench,
    /// A package example.
    Example,
    /// A package build script.
    Build,
    /// A compiled Rust code block in a library's doc text.
    Doctest,
}

/// How a hit writes its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Form {
    /// Any other place: a path segment, a type, a binding, or an attribute.
    Plain,
    /// The name of a method call, `.name(` or `.name::<`.
    Method,
    /// A name after a dot that no call follows, such as a field, `.name`.
    Field,
    /// The name of a macro invocation, `name!`.
    Macro,
}

/// Whether the census read a source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scan {
    Lexed,
    /// The file cannot be read.
    Unreadable,
    /// The text does not lex as Rust tokens.
    Unlexable,
}

/// One text the census reads.
#[derive(Clone, Debug)]
pub(crate) struct Source {
    pub(crate) package: usize,
    /// The target whose module tree, test-only `mod` items, or doc text
    /// holds the text; `None` for a package's tests, benches, examples, and
    /// build script.
    pub(crate) target: Option<usize>,
    /// The crate-relative path, with forward slashes.
    pub(crate) rel: String,
    /// The target's file that holds the text: a file of its module tree, or
    /// the file of a documented node.
    pub(crate) file: Option<FileId>,
    /// What the text's hits count as. In a file of a module tree, the hits
    /// inside test-only nodes count as `Test` instead.
    pub(crate) context: Context,
    /// The line the text starts on: 1 for a file, and the documented node's
    /// line for a doc test.
    pub(crate) line: u32,
    pub(crate) scan: Scan,
}

/// How often one source writes one name in one context and form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TokenHit {
    pub(crate) source: SourceId,
    pub(crate) context: Context,
    pub(crate) form: Form,
    /// Whether the name is inside a macro's arguments or a `macro_rules!`
    /// body.
    pub(crate) in_macro: bool,
    pub(crate) count: usize,
}

/// A path of two or more segments that a source writes, such as
/// `parse::Args` or `crate::run`. A `use` group gives one path for each
/// name it brings in.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct QualifiedPathHit {
    pub(crate) source: SourceId,
    /// The line of its first segment; the paths of a doc test carry the
    /// documented node's line.
    pub(crate) line: u32,
    /// The segments, without `r#`.
    pub(crate) segments: Vec<String>,
    /// True for a path that starts with `::`.
    pub(crate) global: bool,
    pub(crate) context: Context,
    pub(crate) in_macro: bool,
    /// The innermost module of the target's tree that holds the path;
    /// `None` outside the files of the tree.
    pub(crate) module: Option<ModId>,
}

/// How often the scanned sources write one name, beside how many items of
/// the run declare it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NameCount {
    /// The items of the run with this name; each declaration writes the
    /// name once.
    pub(crate) declared: usize,
    /// The hits by package, target, and context.
    pub(crate) by_place: BTreeMap<(usize, Option<usize>, Context), usize>,
}

impl NameCount {
    /// Returns every token hit of the name.
    pub(crate) fn total(&self) -> usize {
        self.by_place.values().sum()
    }

    /// Returns the name-use count: the hits that are not declarations.
    pub(crate) fn uses(&self) -> usize {
        self.total().saturating_sub(self.declared)
    }
}

/// The census of a run.
#[derive(Debug, Default)]
pub(crate) struct Census {
    /// Each target's tree files, in target and file order, then the files
    /// that the test-only `mod` items of each target load, then each
    /// package's files, then the doc tests.
    pub(crate) sources: Vec<Source>,
    /// The first source of each target's tree files.
    pub(crate) tree: Vec<SourceId>,
    /// Each name's hits, sorted by source, context, form, and macro flag.
    /// Keywords and lifetimes are no names.
    pub(crate) tokens: BTreeMap<String, Vec<TokenHit>>,
    /// Sorted by source, line, and segments.
    pub(crate) paths: Vec<QualifiedPathHit>,
    /// The index in `paths` of each path, by its last segment.
    pub(crate) by_last: BTreeMap<String, Vec<usize>>,
    /// The index in `paths` of each path, by its first segment.
    pub(crate) by_first: BTreeMap<String, Vec<usize>>,
    /// How many items of the run declare each name.
    pub(crate) declarations: BTreeMap<String, usize>,
}

impl Census {
    /// Returns the source of target `t`'s tree file `f`.
    pub(crate) fn tree_source(&self, t: usize, f: FileId) -> SourceId {
        self.tree[t] + f
    }

    /// Returns the ids of the sources that could not be read or lexed, in
    /// source order.
    pub(crate) fn unread(&self) -> impl Iterator<Item = SourceId> + '_ {
        (self.sources.iter().enumerate())
            .filter(|(_, source)| source.scan != Scan::Lexed)
            .map(|(id, _)| id)
    }

    /// Tells whether every source was read and lexed.
    pub(crate) fn fully_read(&self) -> bool {
        self.unread().next().is_none()
    }

    /// Returns every token hit of a name written without `r#`.
    pub(crate) fn token_hits(&self, name: &str) -> &[TokenHit] {
        self.tokens.get(name).map_or(&[], Vec::as_slice)
    }

    /// Returns the qualified paths whose last segment is `name`.
    pub(crate) fn paths_ending(&self, name: &str) -> impl Iterator<Item = &QualifiedPathHit> {
        let found = self.by_last.get(name).map_or(&[][..], Vec::as_slice);
        found.iter().map(|&p| &self.paths[p])
    }

    /// Returns the qualified paths whose first segment is `name`, such as
    /// the paths that start with a dependency's crate name.
    pub(crate) fn paths_starting(&self, name: &str) -> impl Iterator<Item = &QualifiedPathHit> {
        let found = self.by_first.get(name).map_or(&[][..], Vec::as_slice);
        found.iter().map(|&p| &self.paths[p])
    }

    /// Returns how many items of the run declare a name.
    pub(crate) fn declared(&self, name: &str) -> usize {
        self.declarations.get(name).copied().unwrap_or(0)
    }

    /// Counts a name's hits by package, target, and context, beside its
    /// declarations.
    pub(crate) fn count(&self, name: &str) -> NameCount {
        let mut count = NameCount {
            declared: self.declared(name),
            ..NameCount::default()
        };
        for hit in self.token_hits(name) {
            let source = &self.sources[hit.source];
            *count
                .by_place
                .entry((source.package, source.target, hit.context))
                .or_default() += hit.count;
        }
        count
    }
}
