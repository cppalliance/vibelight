//! Reads what a run needs from Cargo manifests: the members of a workspace
//! and the path dependencies of each crate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use toml::{Table, Value};

use crate::glob;

/// A path dependency as a crate's manifest writes it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Dep {
    /// The table key, which the source uses as the crate's name when
    /// `package = "..."` renames the dependency.
    pub(crate) key: String,
    /// True when `package = "..."` renames the dependency.
    pub(crate) renamed: bool,
    /// The dependency's directory, absolute and normalized.
    pub(crate) dir: PathBuf,
}

/// A workspace root manifest.
#[derive(Clone, Debug, Default)]
pub(crate) struct Workspace {
    /// The directory of the root manifest, absolute and normalized.
    pub(crate) root: PathBuf,
    pub(crate) members: Vec<String>,
    pub(crate) exclude: Vec<String>,
    /// True when the root manifest is also a package, which makes the root
    /// a member.
    pub(crate) package: bool,
    /// The `[workspace.dependencies]` entries that have a path: each key
    /// with its directory and whether `package = "..."` renames it.
    pub(crate) deps: BTreeMap<String, (PathBuf, bool)>,
}

/// Reads and parses one manifest.
pub(crate) fn read(path: &Path) -> Result<Table> {
    let text =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    text.parse()
        .with_context(|| format!("cannot parse {}", path.display()))
}

/// Makes a path absolute and resolves its `.` and `..` segments without
/// asking the file system, so the result has no `\\?\` prefix on Windows.
pub(crate) fn normalize(path: &Path) -> Result<PathBuf> {
    let absolute =
        std::path::absolute(path).with_context(|| format!("cannot resolve {}", path.display()))?;
    Ok(lexical(&absolute))
}

fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Joins a manifest's relative path to the manifest's directory.
fn join(dir: &Path, rel: &str) -> PathBuf {
    lexical(&dir.join(rel))
}

/// Finds the workspace that holds `dir`: the first manifest with a
/// `[workspace]` table at `dir` or above it. A manifest that cannot be read
/// or parsed is passed over, so an unrelated file above a crate never stops
/// a run.
pub(crate) fn find_workspace(dir: &Path) -> Result<Option<Workspace>> {
    let start = normalize(dir)?;
    for root in start.ancestors() {
        let path = root.join("Cargo.toml");
        if !path.is_file() {
            continue;
        }
        let Ok(table) = read(&path) else {
            continue;
        };
        let Some(workspace) = table.get("workspace").and_then(Value::as_table) else {
            continue;
        };
        return Ok(Some(Workspace {
            root: root.to_path_buf(),
            members: strings(workspace.get("members")),
            exclude: strings(workspace.get("exclude")),
            package: table.contains_key("package"),
            deps: workspace
                .get("dependencies")
                .and_then(Value::as_table)
                .map(|deps| shared_deps(root, deps))
                .unwrap_or_default(),
        }));
    }
    Ok(None)
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn shared_deps(root: &Path, deps: &Table) -> BTreeMap<String, (PathBuf, bool)> {
    deps.iter()
        .filter_map(|(key, value)| {
            let entry = value.as_table()?;
            let path = entry.get("path").and_then(Value::as_str)?;
            Some((
                key.clone(),
                (join(root, path), entry.contains_key("package")),
            ))
        })
        .collect()
}

/// Returns the member directories, sorted: each member written as a plain
/// path, and each directory a glob member matches that holds a
/// `Cargo.toml`, unless it lies in an excluded directory. The root is a
/// member when it is also a package.
pub(crate) fn members(workspace: &Workspace) -> Vec<PathBuf> {
    let excluded: Vec<PathBuf> = workspace
        .exclude
        .iter()
        .map(|e| join(&workspace.root, e))
        .collect();
    let mut found = BTreeSet::new();
    if workspace.package {
        found.insert(workspace.root.clone());
    }
    for member in &workspace.members {
        if !glob::is_glob(member) {
            found.insert(join(&workspace.root, member));
            continue;
        }
        let parts: Vec<&str> = member
            .split(['/', '\\'])
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        let mut matched = Vec::new();
        expand(&workspace.root, &parts, &mut matched);
        found.extend(matched.into_iter().filter(|dir| {
            dir.join("Cargo.toml").is_file() && !excluded.iter().any(|e| dir.starts_with(e))
        }));
    }
    found.into_iter().collect()
}

/// Adds every directory below `dir` that the pattern segments match.
fn expand(dir: &Path, parts: &[&str], found: &mut Vec<PathBuf>) {
    let Some((first, rest)) = parts.split_first() else {
        found.push(dir.to_path_buf());
        return;
    };
    if *first == "**" {
        expand(dir, rest, found);
        for child in subdirs(dir) {
            expand(&child, parts, found);
        }
    } else if glob::is_glob(first) {
        for child in subdirs(dir) {
            let name = child.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if glob::segment(first, name) {
                expand(&child, rest, found);
            }
        }
    } else {
        let child = dir.join(first);
        if child.is_dir() {
            expand(&child, rest, found);
        }
    }
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Returns the path dependencies a manifest names in `[dependencies]` and
/// `[build-dependencies]`, and in the same tables under every `[target.*]`,
/// sorted. A `workspace = true` entry takes its path from the workspace.
/// Dev-dependencies never count.
pub(crate) fn path_deps(dir: &Path, table: &Table, workspace: Option<&Workspace>) -> Vec<Dep> {
    let kinds = ["dependencies", "build-dependencies"];
    let mut tables: Vec<&Table> = kinds
        .iter()
        .filter_map(|kind| table.get(*kind).and_then(Value::as_table))
        .collect();
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        for target in targets.values().filter_map(Value::as_table) {
            tables.extend(
                kinds
                    .iter()
                    .filter_map(|kind| target.get(*kind).and_then(Value::as_table)),
            );
        }
    }
    let mut deps = BTreeSet::new();
    for (key, value) in tables.into_iter().flatten() {
        let Some(entry) = value.as_table() else {
            continue;
        };
        let found = if entry.get("workspace").and_then(Value::as_bool) == Some(true) {
            workspace.and_then(|w| w.deps.get(key)).cloned()
        } else {
            entry
                .get("path")
                .and_then(Value::as_str)
                .map(|path| (join(dir, path), entry.contains_key("package")))
        };
        if let Some((dir, renamed)) = found {
            deps.insert(Dep {
                key: key.clone(),
                renamed,
                dir,
            });
        }
    }
    deps.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn path_dependencies_come_from_every_table_but_dev_dependencies() {
        let tmp = tempfile::tempdir().unwrap();
        let root = normalize(tmp.path()).unwrap();
        write(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\n[workspace.dependencies]\n\
             shared = { path = \"crates/shared\" }\n\
             alias = { package = \"real\", path = \"crates/real\" }\nversioned = \"1\"\n",
        );
        write(
            &root,
            "crates/app/Cargo.toml",
            "[package]\nname = \"app\"\n[dependencies]\nlocal = { path = \"../local\" }\n\
             b = { package = \"beta\", path = \"../beta\" }\nregistry = \"1\"\n\
             shared.workspace = true\nalias = { workspace = true, features = [\"x\"] }\n\
             versioned.workspace = true\n[dev-dependencies]\ntesting = { path = \"../testing\" }\n\
             [build-dependencies]\nbuilder = { path = \"../builder\" }\n\
             [target.'cfg(unix)'.dependencies]\nunix = { path = \"../unix\" }\n",
        );
        let dir = root.join("crates").join("app");
        let table = read(&dir.join("Cargo.toml")).unwrap();
        let workspace = find_workspace(&dir).unwrap().unwrap();
        let deps: Vec<(String, bool, PathBuf)> = path_deps(&dir, &table, Some(&workspace))
            .into_iter()
            .map(|d| (d.key, d.renamed, d.dir))
            .collect();
        let crates = root.join("crates");
        let expected = [
            ("alias", true, "real"),
            ("b", true, "beta"),
            ("builder", false, "builder"),
            ("local", false, "local"),
            ("shared", false, "shared"),
            ("unix", false, "unix"),
        ]
        .map(|(key, renamed, dir)| (key.to_owned(), renamed, crates.join(dir)));
        assert_eq!(deps, expected);
    }

    #[test]
    fn members_expand_globs_and_prune_excluded_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = normalize(tmp.path()).unwrap();
        write(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\", \"crates/family/kept\", \"tools/**\"]\n\
             exclude = [\"crates/family\", \"crates/skipped\"]\n",
        );
        for dir in [
            "crates/a",
            "crates/skipped",
            "crates/family/kept",
            "crates/family/dropped",
            "tools/x",
            "tools/nested/y",
        ] {
            write(
                &root,
                &format!("{dir}/Cargo.toml"),
                "[package]\nname = \"p\"\n",
            );
        }
        fs::create_dir_all(root.join("crates").join("notes")).unwrap();
        let workspace = find_workspace(&root.join("crates").join("a"))
            .unwrap()
            .unwrap();
        assert_eq!(workspace.root, root);
        let found: Vec<String> = members(&workspace)
            .iter()
            .map(|dir| {
                let rel = dir.strip_prefix(&root).unwrap();
                rel.to_string_lossy().replace('\\', "/")
            })
            .collect();
        assert_eq!(
            found,
            [
                "crates/a",
                "crates/family/kept",
                "tools/nested/y",
                "tools/x"
            ]
        );
    }
}
