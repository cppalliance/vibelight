//! Walks `mod` declarations from a root file and parses every module file.

use std::fs;
use std::ops::Range;
use std::path::Path;

use anyhow::{Context, Result};
use syn::{Attribute, Item, ItemMod};

use crate::attrs;
use crate::model::{FileId, ModId, Warning};
use crate::source::SourceFile;

/// One module found by the walk.
pub(crate) struct ModSource {
    /// The path below the crate root; empty for the root.
    pub(crate) path: Vec<String>,
    /// The file holding the module's items.
    pub(crate) file: FileId,
    /// The `mod` item that declared the module: its parent module and its
    /// index among the parent's items.
    pub(crate) decl: Option<(ModId, usize)>,
    pub(crate) items: Vec<Item>,
    /// Inner attributes: a file's `//!` docs, or an inline module's.
    pub(crate) inner: Vec<Attribute>,
    /// The byte range of `file` that holds the items.
    pub(crate) scope: Range<usize>,
    /// The line of an inline module's `mod` keyword.
    pub(crate) inline_line: Option<u32>,
    /// Why the module has no items: a missing or unparsable file.
    pub(crate) problem: Option<String>,
}

/// Every file and module of one target.
pub(crate) struct Tree {
    pub(crate) files: Vec<SourceFile>,
    pub(crate) modules: Vec<ModSource>,
    pub(crate) warnings: Vec<Warning>,
    /// The files that test-only `mod` items, and the modules inside them,
    /// load, crate-relative, sorted. They never join the tree.
    pub(crate) test_files: Vec<String>,
}

/// Walks the module tree from `root`, a crate-relative file path.
///
/// # Errors
/// Returns an error when the root file cannot be read.
pub(crate) fn walk(crate_dir: &Path, root: &str) -> Result<Tree> {
    let mut walker = Walker {
        crate_dir,
        tree: Tree {
            files: Vec::new(),
            modules: Vec::new(),
            warnings: Vec::new(),
            test_files: Vec::new(),
        },
        chain: Vec::new(),
    };
    let text = fs::read_to_string(crate_dir.join(root))
        .with_context(|| format!("cannot read root file {}", crate_dir.join(root).display()))?;
    walker.file_module(Vec::new(), None, root, &text, true);
    walker.tree.test_files.sort();
    Ok(walker.tree)
}

struct Walker<'a> {
    crate_dir: &'a Path,
    tree: Tree,
    /// Files on the current path from the root, to stop a `#[path]` loop.
    chain: Vec<String>,
}

impl Walker<'_> {
    /// Adds a module held in its own file and walks its children.
    fn file_module(
        &mut self,
        path: Vec<String>,
        decl: Option<(ModId, usize)>,
        rel: &str,
        text: &str,
        mod_rs: bool,
    ) {
        let file = SourceFile::new(rel.to_owned(), text);
        let parsed = syn::parse_file(&file.text);
        let scope = 0..file.text.len();
        let id = self.tree.files.len();
        self.tree.files.push(file);
        let module = self.tree.modules.len();
        let name = path
            .last()
            .map(|n| crate::typerefs::strip_raw(n))
            .unwrap_or_default();
        match parsed {
            Ok(parsed) => {
                self.tree.modules.push(ModSource {
                    path,
                    file: id,
                    decl,
                    items: parsed.items.clone(),
                    inner: parsed.attrs,
                    scope,
                    inline_line: None,
                    problem: None,
                });
                let dirs = Dirs::of_file(rel, &name, mod_rs);
                self.chain.push(rel.to_owned());
                self.children(module, &parsed.items, &dirs);
                self.chain.pop();
            }
            Err(error) => {
                let line = u32::try_from(error.span().start().line).unwrap_or(0);
                self.warn(rel, line, format!("cannot parse this file: {error}"));
                self.tree.modules.push(ModSource {
                    path,
                    file: id,
                    decl,
                    items: Vec::new(),
                    inner: Vec::new(),
                    scope,
                    inline_line: None,
                    problem: Some("unparsable file".to_owned()),
                });
            }
        }
    }

    /// Adds every child module declared among `items` of `module`.
    fn children(&mut self, module: ModId, items: &[Item], dirs: &Dirs) {
        let file = self.tree.modules[module].file;
        for (index, item) in items.iter().enumerate() {
            let Item::Mod(m) = item else {
                continue;
            };
            if attrs::is_test_only(&m.attrs) {
                self.test_module(m, dirs);
                continue;
            }
            let mut path = self.tree.modules[module].path.clone();
            path.push(m.ident.to_string());
            let decl = Some((module, index));
            let line = u32::try_from(m.mod_token.span.start().line).unwrap_or(0);
            let name = crate::typerefs::strip_raw(&m.ident.to_string());
            match self.resolve(m, &name, dirs) {
                Body::Inline(inline, (brace, inner_items)) => {
                    let source = &self.tree.files[file];
                    let scope = source.end(brace.span.open())..source.start(brace.span.close());
                    let (_, inner) = attrs::split_inner(&m.attrs);
                    let id = self.tree.modules.len();
                    self.tree.modules.push(ModSource {
                        path,
                        file,
                        decl,
                        items: inner_items.clone(),
                        inner,
                        scope,
                        inline_line: Some(line),
                        problem: None,
                    });
                    self.children(id, inner_items, &inline);
                }
                Body::File(Loaded { file: found, text }) => {
                    self.file_module(path, decl, &found.rel, &text, found.mod_rs);
                }
                Body::Absent { warning, why } => {
                    let declaring = self.tree.files[file].rel.clone();
                    self.warn(&declaring, line, warning);
                    self.missing(path, decl, file, line, why);
                }
            }
        }
    }

    /// Finds where the items of the `mod` item `m`, named `name`, live:
    /// inline, or in the first candidate file that loads and is not already
    /// on the current path from the root.
    fn resolve<'m>(&self, m: &'m ItemMod, name: &str, dirs: &Dirs) -> Body<'m> {
        let path_value = attrs::path_attr(&m.attrs);
        if let Some(content) = &m.content {
            return Body::Inline(dirs.inline(path_value.as_deref(), name), content);
        }
        let tried = candidates(path_value.as_deref(), name, dirs);
        let absent = |warning, why| Body::Absent { warning, why };
        match self.load(&tried) {
            Some(loaded) if self.chain.contains(&loaded.file.rel) => absent(
                format!("module `{name}` loads {} again", loaded.file.rel),
                "module file loop",
            ),
            Some(loaded) => Body::File(loaded),
            None => absent(
                format!("no file found for module `{name}` ({})", tried[0].rel),
                "missing file",
            ),
        }
    }

    /// Reads the first candidate file that exists.
    fn load(&self, candidates: &[Candidate]) -> Option<Loaded> {
        candidates.iter().find_map(|candidate| {
            let text = fs::read_to_string(self.crate_dir.join(&candidate.rel)).ok()?;
            let file = candidate.clone();
            Some(Loaded { file, text })
        })
    }

    /// Records the file a test-only module loads, and the files of the
    /// modules inside it, without parsing any of them into the tree.
    fn test_module(&mut self, m: &ItemMod, dirs: &Dirs) {
        let name = crate::typerefs::strip_raw(&m.ident.to_string());
        let Loaded { file: found, text } = match self.resolve(m, &name, dirs) {
            Body::Inline(inline, (_, items)) => {
                self.test_children(items, &inline);
                return;
            }
            Body::File(loaded) => loaded,
            Body::Absent { .. } => return,
        };
        if self.tree.test_files.contains(&found.rel) {
            return;
        }
        self.tree.test_files.push(found.rel.clone());
        let file = SourceFile::new(found.rel.clone(), &text);
        let Ok(parsed) = syn::parse_file(&file.text) else {
            return;
        };
        let dirs = Dirs::of_file(&found.rel, &name, found.mod_rs);
        self.test_children(&parsed.items, &dirs);
    }

    /// Records the files of every module declared among a test module's
    /// items.
    fn test_children(&mut self, items: &[Item], dirs: &Dirs) {
        for item in items {
            if let Item::Mod(m) = item {
                self.test_module(m, dirs);
            }
        }
    }

    /// Adds a placeholder module, located at its `mod` line, for a file that
    /// cannot be loaded.
    fn missing(
        &mut self,
        path: Vec<String>,
        decl: Option<(ModId, usize)>,
        file: FileId,
        line: u32,
        why: &str,
    ) {
        self.tree.modules.push(ModSource {
            path,
            file,
            decl,
            items: Vec::new(),
            inner: Vec::new(),
            scope: 0..0,
            inline_line: Some(line),
            problem: Some(why.to_owned()),
        });
    }

    fn warn(&mut self, file: &str, line: u32, message: String) {
        self.tree.warnings.push(Warning {
            file: file.to_owned(),
            line,
            message,
        });
    }
}

/// The directories a module's `mod` items resolve against: `child_dir` for
/// `mod name;` files, `path_base` for `#[path]` values.
struct Dirs {
    child_dir: String,
    path_base: String,
}

impl Dirs {
    /// Returns the directories of a module held in the file `rel`: its
    /// children live in the file's folder when the file is `mod.rs`-like,
    /// else in a folder named for the module.
    fn of_file(rel: &str, name: &str, mod_rs: bool) -> Self {
        let path_base = dir_of(rel);
        let child_dir = join(&path_base, if mod_rs { "" } else { name });
        Self {
            child_dir,
            path_base,
        }
    }

    /// Returns the directories of an inline module, which uses one
    /// directory for both: its `#[path]` value, or its name below the
    /// `child_dir` of the module that declares it.
    fn inline(&self, path_value: Option<&str>, name: &str) -> Self {
        let dir = match path_value {
            Some(value) => join(&self.path_base, value),
            None => join(&self.child_dir, name),
        };
        Self {
            child_dir: dir.clone(),
            path_base: dir,
        }
    }
}

/// A file a `mod name;` item may load, and whether it holds its children
/// the way a `mod.rs` does.
#[derive(Clone)]
struct Candidate {
    rel: String,
    mod_rs: bool,
}

/// A candidate file that was read.
struct Loaded {
    file: Candidate,
    text: String,
}

/// Where a `mod` item's items live.
enum Body<'m> {
    /// Written in braces: its children's directories, and the content.
    Inline(Dirs, &'m (syn::token::Brace, Vec<Item>)),
    /// Read from a file.
    File(Loaded),
    /// In no usable file: the warning to give, and the placeholder's problem.
    Absent { warning: String, why: &'static str },
}

/// Returns the files a `mod name;` item may load: the `#[path]` value, or
/// `name.rs`, then `name/mod.rs`.
fn candidates(path_value: Option<&str>, name: &str, dirs: &Dirs) -> Vec<Candidate> {
    let candidate = |rel, mod_rs| Candidate { rel, mod_rs };
    match path_value {
        Some(value) => vec![candidate(join(&dirs.path_base, value), true)],
        None => vec![
            candidate(join(&dirs.child_dir, &format!("{name}.rs")), false),
            candidate(join(&dirs.child_dir, &format!("{name}/mod.rs")), true),
        ],
    }
}

/// Returns the directory part of a crate-relative path, or `""`.
fn dir_of(rel: &str) -> String {
    rel.rfind('/')
        .map_or_else(String::new, |i| rel[..i].to_owned())
}

/// Joins crate-relative paths with forward slashes, resolving `.` and `..`
/// lexically.
fn join(base: &str, rel: &str) -> String {
    let mut parts: Vec<&str> = base.split('/').filter(|p| !p.is_empty()).collect();
    for part in rel.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn paths(tree: &Tree) -> Vec<(String, String, Option<String>)> {
        tree.modules
            .iter()
            .map(|m| {
                (
                    m.path.join("::"),
                    tree.files[m.file].rel.clone(),
                    m.problem.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn walks_files_mod_rs_inline_and_path_modules() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write(
            dir,
            "src/lib.rs",
            "mod a;\nmod b;\nmod c { mod d; }\n#[cfg(test)] mod tests;\n",
        );
        write(dir, "src/a.rs", "mod nested;\n");
        write(
            dir,
            "src/a/nested.rs",
            "#[path = \"config-limits.rs\"] mod limits;\n",
        );
        write(dir, "src/a/config-limits.rs", "pub struct Limit;\n");
        write(dir, "src/b/mod.rs", "mod under;\n");
        write(dir, "src/b/under.rs", "");
        write(dir, "src/c/d.rs", "");
        let tree = walk(dir, "src/lib.rs").unwrap();
        let s = |x: &str| x.to_owned();
        assert_eq!(
            paths(&tree),
            [
                (s(""), s("src/lib.rs"), None),
                (s("a"), s("src/a.rs"), None),
                (s("a::nested"), s("src/a/nested.rs"), None),
                (s("a::nested::limits"), s("src/a/config-limits.rs"), None),
                (s("b"), s("src/b/mod.rs"), None),
                (s("b::under"), s("src/b/under.rs"), None),
                (s("c"), s("src/lib.rs"), None),
                (s("c::d"), s("src/c/d.rs"), None),
            ]
        );
        assert!(tree.warnings.is_empty(), "{:?}", tree.warnings);
        let inline = &tree.modules[6];
        assert_eq!(inline.inline_line, Some(3));
        assert_eq!(
            &tree.files[inline.file].text[inline.scope.clone()],
            " mod d; "
        );
    }

    #[test]
    fn missing_and_unparsable_files_get_placeholders() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write(dir, "lib.rs", "mod gone;\nmod broken;\n");
        write(dir, "broken.rs", "fn (\n");
        let tree = walk(dir, "lib.rs").unwrap();
        let problems: Vec<_> = tree.modules.iter().map(|m| m.problem.clone()).collect();
        assert_eq!(
            problems,
            [
                None,
                Some("missing file".into()),
                Some("unparsable file".into())
            ]
        );
        let lines: Vec<_> = tree
            .warnings
            .iter()
            .map(|w| (w.file.as_str(), w.line))
            .collect();
        assert_eq!(lines, [("lib.rs", 1), ("broken.rs", 1)]);
    }

    #[test]
    fn test_only_modules_record_their_files_without_joining_the_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write(
            dir,
            "src/lib.rs",
            "mod a;\n#[cfg(test)]\nmod tests;\n#[cfg(test)]\nmod inline { mod helper; }\n",
        );
        write(
            dir,
            "src/a.rs",
            "#[cfg(test)]\n#[path = \"a_tests.rs\"]\nmod tests;\n",
        );
        write(dir, "src/a_tests.rs", "");
        write(dir, "src/tests.rs", "mod deep;\n");
        write(dir, "src/tests/deep.rs", "");
        write(dir, "src/inline/helper.rs", "");
        let tree = walk(dir, "src/lib.rs").unwrap();
        assert_eq!(
            tree.test_files,
            [
                "src/a_tests.rs",
                "src/inline/helper.rs",
                "src/tests.rs",
                "src/tests/deep.rs"
            ]
        );
        let files: Vec<&str> = tree.files.iter().map(|f| f.rel.as_str()).collect();
        assert_eq!(files, ["src/lib.rs", "src/a.rs"]);
        assert!(tree.warnings.is_empty(), "{:?}", tree.warnings);
    }

    #[test]
    fn unreadable_root_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(walk(tmp.path(), "src/lib.rs").is_err());
    }

    #[test]
    fn join_resolves_dots() {
        assert_eq!(join("src/a", "../b.rs"), "src/b.rs");
        assert_eq!(join("", "./x/y.rs"), "x/y.rs");
        assert_eq!(join("src", "..\\..\\up.rs"), "../up.rs");
    }
}
