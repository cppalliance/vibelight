//! Structure rules for the package, shared by `build.rs` and `tests/it/structure.rs`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) const SECOND_MANIFEST: &str =
    "the package is one crate, so its only Cargo.toml is the one at the root";
pub(crate) const WORKSPACE: &str = "the package is one crate, so Cargo.toml declares no workspace";
pub(crate) const PATH_DEPENDENCY: &str =
    "the package is one crate, so Cargo.toml has no path dependency";
pub(crate) const LIB_TARGET: &str = "the package is one binary crate, so it has no src/lib.rs";
pub(crate) const LONG_FILE: &str = "a .rs file under src, tests, or build has at most 500 lines";
pub(crate) const MOD_RS: &str = "a module file is named after its module, never mod.rs";
pub(crate) const MODULE_DOC: &str = "every .rs file under src begins with a //! line";
pub(crate) const LINT_LEVEL: &str =
    "lint levels belong in the [lints] tables of Cargo.toml, not in #![...] attributes";
pub(crate) const PLAN_NAME: &str =
    "plans/ holds only files named YYYY-MM-DD-<one to three kebab words>.plan.md";

const MAX_LINES: usize = 500;

/// Directories never walked, relative to the package root. The fixtures are
/// source trees for the tool to read, so they break these rules on purpose.
const SKIPPED: [&str; 3] = [".git", "target", "tests/fixtures"];

const LINT_ATTRIBUTES: [&str; 5] = [
    "#![allow(",
    "#![deny(",
    "#![expect(",
    "#![forbid(",
    "#![warn(",
];

/// One broken rule: where it is broken and how to fix it.
pub(crate) struct Violation {
    /// Relative to the package root, with forward slashes.
    pub(crate) path: String,
    pub(crate) rule: &'static str,
    pub(crate) fix: String,
}

impl Violation {
    fn new(path: impl Into<String>, rule: &'static str, fix: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            rule,
            fix: fix.into(),
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}. Fix: {}.", self.path, self.rule, self.fix)
    }
}

/// Whether `root` is an unpacked published package. `cargo package` adds
/// `Cargo.toml.orig` to every tarball, and a checkout never has one.
pub(crate) fn is_packaged(root: &Path) -> bool {
    root.join("Cargo.toml.orig").is_file()
}

/// Every violation in the package at `root`, sorted by path and then rule, or
/// none for a published package.
pub(crate) fn check(root: &Path) -> io::Result<Vec<Violation>> {
    if is_packaged(root) {
        return Ok(Vec::new());
    }
    let mut violations = extra_manifests(root)?;
    violations.extend(root_manifest(root)?);
    violations.extend(lib_target(root));
    violations.extend(long_files(root)?);
    violations.extend(mod_rs(root)?);
    violations.extend(module_docs(root)?);
    violations.extend(lint_levels(root)?);
    violations.extend(plan_names(root)?);
    violations.sort_by(|a, b| (&a.path, a.rule).cmp(&(&b.path, b.rule)));
    Ok(violations)
}

/// A `Cargo.toml` anywhere but the package root.
pub(crate) fn extra_manifests(root: &Path) -> io::Result<Vec<Violation>> {
    let fix = "move that crate's code into modules of this one and remove the manifest";
    Ok(files(root, "")?
        .into_keys()
        .filter(|name| name.ends_with("/Cargo.toml"))
        .map(|name| Violation::new(name, SECOND_MANIFEST, fix))
        .collect())
}

/// A `[workspace]` table, or a path dependency, in the root `Cargo.toml`.
pub(crate) fn root_manifest(root: &Path) -> io::Result<Vec<Violation>> {
    let mut found = Vec::new();
    let mut table = String::new();
    for (index, line) in read(&root.join("Cargo.toml"))?.lines().enumerate() {
        let number = index + 1;
        let code = line.split('#').next().unwrap_or_default().trim();
        if code.starts_with('[') {
            table = table_name(code);
            if table == "workspace" || table.starts_with("workspace.") {
                let fix = format!("remove the [{table}] table at line {number}");
                found.push(Violation::new("Cargo.toml", WORKSPACE, fix));
            }
        } else if is_dependency_table(&table) && sets_path(code) {
            let fix = format!(
                "replace the path dependency at line {number} with a published version, \
                 or move that code into a module"
            );
            found.push(Violation::new("Cargo.toml", PATH_DEPENDENCY, fix));
        }
    }
    Ok(found)
}

/// A `src/lib.rs`, which adds a library crate beside the binary.
pub(crate) fn lib_target(root: &Path) -> Vec<Violation> {
    if root.join("src").join("lib.rs").is_file() {
        let fix = "move its code into modules declared from src/main.rs";
        vec![Violation::new("src/lib.rs", LIB_TARGET, fix)]
    } else {
        Vec::new()
    }
}

/// A `.rs` file under `src`, `tests`, or `build` with more than 500 lines.
pub(crate) fn long_files(root: &Path) -> io::Result<Vec<Violation>> {
    let mut found = Vec::new();
    for dir in ["build", "src", "tests"] {
        for (name, path) in rust_files(root, dir)? {
            let count = read(&path)?.lines().count();
            if count > MAX_LINES {
                let fix = format!("split its {count} lines into smaller modules");
                found.push(Violation::new(name, LONG_FILE, fix));
            }
        }
    }
    Ok(found)
}

/// A file named `mod.rs` anywhere in the package.
pub(crate) fn mod_rs(root: &Path) -> io::Result<Vec<Violation>> {
    let mut found = Vec::new();
    for name in files(root, "")?.into_keys() {
        let fix = match name.strip_suffix("/mod.rs") {
            Some(dir) => format!("rename it to {dir}.rs"),
            None if name == "mod.rs" => "rename it after its module".to_string(),
            None => continue,
        };
        found.push(Violation::new(name, MOD_RS, fix));
    }
    Ok(found)
}

/// A `.rs` file under `src` whose first line does not start with `//!`.
pub(crate) fn module_docs(root: &Path) -> io::Result<Vec<Violation>> {
    let mut found = Vec::new();
    for (name, path) in rust_files(root, "src")? {
        let text = read(&path)?;
        // rustc skips a leading byte order mark, so the first line starts after it.
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        if !text.starts_with("//!") {
            let fix = "add a first line starting with //! that says what the file is for";
            found.push(Violation::new(name, MODULE_DOC, fix));
        }
    }
    Ok(found)
}

/// A lint-level inner attribute, such as `#![allow(...)]`, in a `.rs` file under `src`.
pub(crate) fn lint_levels(root: &Path) -> io::Result<Vec<Violation>> {
    let mut found = Vec::new();
    for (name, path) in rust_files(root, "src")? {
        for (index, line) in read(&path)?.lines().enumerate() {
            let attribute = line.trim();
            if LINT_ATTRIBUTES
                .iter()
                .any(|start| attribute.starts_with(start))
            {
                let fix = format!(
                    "move `{attribute}` at line {} to the [lints] tables of Cargo.toml, \
                     or to an outer attribute on the item that needs it",
                    index + 1
                );
                found.push(Violation::new(name.clone(), LINT_LEVEL, fix));
            }
        }
    }
    Ok(found)
}

/// A file in `plans/` that `is_plan_name` rejects, or any folder there.
pub(crate) fn plan_names(root: &Path) -> io::Result<Vec<Violation>> {
    let dir = root.join("plans");
    let mut found = Vec::new();
    if !dir.is_dir() {
        return Ok(found);
    }
    for entry in with_path(&dir, fs::read_dir(&dir))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_dir() {
            let fix = "move its plans up into plans/ and remove the folder";
            found.push(Violation::new(format!("plans/{name}/"), PLAN_NAME, fix));
        } else if !is_plan_name(&name) {
            let fix = "rename it, as in plans/2026-10-25-vibelight-tool.plan.md";
            found.push(Violation::new(format!("plans/{name}"), PLAN_NAME, fix));
        }
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}

/// Whether `name` is `YYYY-MM-DD-`, then one to three words of `a-z` and `0-9`
/// joined by single hyphens, then `.plan.md`.
fn is_plan_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".plan.md") else {
        return false;
    };
    let parts: Vec<&str> = stem.split('-').collect();
    if !(4..=6).contains(&parts.len()) {
        return false;
    }
    let digits =
        |part: &str, count: usize| part.len() == count && part.bytes().all(|b| b.is_ascii_digit());
    let word = |part: &&str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    };
    digits(parts[0], 4) && digits(parts[1], 2) && digits(parts[2], 2) && parts[3..].iter().all(word)
}

/// The table a header line opens, with whitespace removed, so that
/// `[ target.'cfg(unix)'.dependencies ]` gives `target.'cfg(unix)'.dependencies`.
fn table_name(header: &str) -> String {
    let inside = header
        .trim_start_matches('[')
        .split(']')
        .next()
        .unwrap_or_default();
    inside.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Whether a table lists dependencies, as `[dependencies]`, `[dev-dependencies]`,
/// and `[target.'cfg(unix)'.dependencies]` do, or is one dependency, as
/// `[dependencies.foo]` is.
fn is_dependency_table(table: &str) -> bool {
    table.ends_with("dependencies")
        || table
            .rsplit_once('.')
            .is_some_and(|(parent, _)| parent.ends_with("dependencies"))
}

/// Whether a manifest line, without its comment, sets a `path` key:
/// `path = ...`, `foo.path = ...`, or `path = ...` inside an inline table.
fn sets_path(code: &str) -> bool {
    code.match_indices("path").any(|(start, key)| {
        let before = code[..start].chars().next_back();
        let after = code[start + key.len()..].trim_start();
        before.is_none_or(|c| c.is_whitespace() || matches!(c, '{' | ',' | '.'))
            && after.starts_with('=')
    })
}

/// Every file under `dir` (empty for the whole package), keyed by its path
/// relative to `root` with forward slashes, skipping the `SKIPPED` directories.
fn files(root: &Path, dir: &str) -> io::Result<BTreeMap<String, PathBuf>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![(dir.to_string(), root.join(dir))];
    while let Some((name, path)) = pending.pop() {
        if !path.is_dir() {
            continue;
        }
        for entry in with_path(&path, fs::read_dir(&path))? {
            let entry = entry?;
            let child = entry.file_name().to_string_lossy().into_owned();
            let child = if name.is_empty() {
                child
            } else {
                format!("{name}/{child}")
            };
            if entry.file_type()?.is_dir() {
                if !SKIPPED.contains(&child.as_str()) {
                    pending.push((child, entry.path()));
                }
            } else {
                found.insert(child, entry.path());
            }
        }
    }
    Ok(found)
}

/// The `.rs` files that `files` finds under `dir`.
fn rust_files(root: &Path, dir: &str) -> io::Result<BTreeMap<String, PathBuf>> {
    let mut found = files(root, dir)?;
    found.retain(|_, path| path.extension().is_some_and(|ext| ext == "rs"));
    Ok(found)
}

fn read(path: &Path) -> io::Result<String> {
    with_path(path, fs::read_to_string(path))
}

/// Names `path` in an I/O error, which otherwise does not say which file failed.
fn with_path<T>(path: &Path, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|err| io::Error::new(err.kind(), format!("{}: {err}", path.display())))
}
