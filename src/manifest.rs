//! Reads what a run needs from Cargo manifests: the members of a workspace
//! and the path dependencies of each crate.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use toml::{Table, Value};

use crate::glob;
use crate::manifest_facts::DepKind;

/// The dependency tables and the kind each holds: the hyphen name of each
/// kind first, then the underscore spelling Cargo still reads.
pub(crate) const DEP_TABLES: [(&str, DepKind); 5] = [
    ("dependencies", DepKind::Normal),
    ("dev-dependencies", DepKind::Dev),
    ("dev_dependencies", DepKind::Dev),
    ("build-dependencies", DepKind::Build),
    ("build_dependencies", DepKind::Build),
];

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
    /// The `[workspace.dependencies]` entries that have a path.
    pub(crate) deps: Vec<Dep>,
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

/// Joins a relative path written in a file to that file's directory.
pub(crate) fn join(dir: &Path, rel: &str) -> PathBuf {
    lexical(&dir.join(rel))
}

/// Returns `dir` below `base` with forward slashes, or `None` when it is
/// not below `base`.
pub(crate) fn relative(base: &Path, dir: &Path) -> Option<String> {
    let rest = dir.strip_prefix(base).ok()?;
    let names: Vec<String> = rest
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(names.join("/"))
}

/// Returns the crate directories below `dir` that a glob such as
/// `crates/*` matches: the matched folders that hold a `Cargo.toml`, sorted.
pub(crate) fn crate_dirs(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    let parts: Vec<&str> = pattern
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let mut matched = Vec::new();
    expand(dir, &parts, &mut matched);
    let found: BTreeSet<PathBuf> = matched
        .into_iter()
        .filter(|dir| dir.join("Cargo.toml").is_file())
        .collect();
    found.into_iter().collect()
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

pub(crate) fn strings(value: Option<&Value>) -> Vec<String> {
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

fn shared_deps(root: &Path, deps: &Table) -> Vec<Dep> {
    deps.iter()
        .filter_map(|(key, value)| {
            let entry = value.as_table()?;
            let path = entry.get("path").and_then(Value::as_str)?;
            Some(Dep {
                key: key.clone(),
                renamed: entry.contains_key("package"),
                dir: join(root, path),
            })
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
        found.extend(
            crate_dirs(&workspace.root, member)
                .into_iter()
                .filter(|dir| !excluded.iter().any(|e| dir.starts_with(e))),
        );
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
    deps_in(dir, table, workspace, &[DepKind::Normal, DepKind::Build])
}

/// Returns the path dependencies a manifest names in the tables of the
/// `kinds`, each under every spelling Cargo reads, such as
/// `dev_dependencies`, and in the same tables under every `[target.*]`,
/// sorted.
pub(crate) fn deps_in(
    dir: &Path,
    table: &Table,
    workspace: Option<&Workspace>,
    kinds: &[DepKind],
) -> Vec<Dep> {
    let spellings: Vec<&str> = (DEP_TABLES.iter())
        .filter(|(_, kind)| kinds.contains(kind))
        .map(|&(name, _)| name)
        .collect();
    let mut holders = vec![table];
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        holders.extend(targets.values().filter_map(Value::as_table));
    }
    let tables: Vec<&Table> = holders
        .into_iter()
        .flat_map(|holder| {
            (spellings.iter()).filter_map(move |name| holder.get(*name).and_then(Value::as_table))
        })
        .collect();
    let mut deps = BTreeSet::new();
    for (key, value) in tables.into_iter().flatten() {
        let Some(entry) = value.as_table() else {
            continue;
        };
        let found = if entry.get("workspace").and_then(Value::as_bool) == Some(true) {
            workspace
                .and_then(|w| w.deps.iter().find(|d| d.key == *key))
                .cloned()
        } else {
            entry.get("path").and_then(Value::as_str).map(|path| Dep {
                key: key.clone(),
                renamed: entry.contains_key("package"),
                dir: join(dir, path),
            })
        };
        if let Some(dep) = found {
            deps.insert(dep);
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
    fn underscore_dependency_tables_are_followed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = normalize(tmp.path()).unwrap();
        write(
            &root,
            "app/Cargo.toml",
            "[package]\nname = \"app\"\n\
             [build_dependencies]\nbuilder = { path = \"../builder\" }\n\
             [dev_dependencies]\ntesting = { path = \"../testing\" }\n\
             [target.'cfg(unix)'.build_dependencies]\nunix = { path = \"../unix\" }\n",
        );
        let dir = root.join("app");
        let table = read(&dir.join("Cargo.toml")).unwrap();
        let keys: Vec<String> = path_deps(&dir, &table, None)
            .into_iter()
            .map(|d| d.key)
            .collect();
        assert_eq!(keys, ["builder", "unix"], "build counts, dev never does");
    }

    #[test]
    fn every_dependency_kind_names_its_hyphen_table() {
        let tables = [DepKind::Normal, DepKind::Dev, DepKind::Build].map(DepKind::table);
        assert_eq!(
            tables,
            ["dependencies", "dev-dependencies", "build-dependencies"]
        );
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

    #[test]
    fn a_root_manifest_that_is_also_a_package_is_a_member() {
        let tmp = tempfile::tempdir().unwrap();
        let root = normalize(tmp.path()).unwrap();
        write(
            &root,
            "Cargo.toml",
            "[package]\nname = \"top\"\n[workspace]\nmembers = [\"crates/*\"]\n",
        );
        write(&root, "crates/a/Cargo.toml", "[package]\nname = \"a\"\n");
        let workspace = find_workspace(&root.join("crates").join("a"))
            .unwrap()
            .unwrap();
        assert!(workspace.package);
        assert_eq!(
            members(&workspace),
            [root.clone(), root.join("crates").join("a")]
        );
    }
}
