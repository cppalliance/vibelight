//! Gives a repository its first `vibelight.toml`, so a plain `vibelight`
//! needs no switches: it finds the root of the repository, leaves out the
//! crates that only support development, and writes the file there with a
//! reason beside each crate it leaves out.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml::{Table, Value};

use crate::config;
use crate::inputs::relative;
use crate::manifest::{self, Workspace};

/// The config a scaffold wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Written {
    pub(crate) path: PathBuf,
    /// How many crates its `exclude` leaves out.
    pub(crate) left_out: usize,
}

/// A crate the scaffold leaves out: its folder below the root, with
/// forward slashes, and the reason its line gives.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Left {
    dir: String,
    reason: String,
}

/// Writes `vibelight.toml` at the root of the repository that holds
/// `start`: the workspace root above it, or else the nearest folder with a
/// `Cargo.toml`. Returns `None` when no such folder exists.
///
/// # Errors
/// Returns an error when the folder cannot be read or the file cannot be
/// written.
pub(crate) fn write(start: &Path) -> Result<Option<Written>> {
    let (root, text, left_out) = if let Some(workspace) = manifest::find_workspace(start)? {
        let left = left_out(&workspace);
        (workspace.root.clone(), text(true, &left), left.len())
    } else {
        let start = manifest::normalize(start)?;
        let Some(root) = start
            .ancestors()
            .find(|dir| dir.join("Cargo.toml").is_file())
        else {
            return Ok(None);
        };
        (root.to_path_buf(), text(false, &[]), 0)
    };
    let path = root.join(config::FILE);
    fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;
    Ok(Some(Written { path, left_out }))
}

/// The text of the config: the workspace or the one crate at the root,
/// every path dependency below the root as context, and the crates left
/// out.
fn text(workspace: bool, left: &[Left]) -> String {
    let mut out = String::from(
        "# Written by vibelight. It chooses the crates a plain `vibelight` outlines:\n\
         # edit `exclude` to change which of them are analyzed.\n",
    );
    out.push_str(if workspace {
        "workspace = true\n"
    } else {
        "crates = [\".\"]\n"
    });
    out.push_str("follow = [\"**\"]\n");
    if left.is_empty() {
        out.push_str("exclude = []\n");
    } else {
        out.push_str("exclude = [\n");
        for crate_left in left {
            let _ = writeln!(out, "    \"{}\",  # {}", crate_left.dir, crate_left.reason);
        }
        out.push_str("]\n");
    }
    let _ = writeln!(out, "out = \"{}\"", config::OUT);
    out
}

/// One member of the workspace, as the rules read it.
struct Member {
    dir: PathBuf,
    rel: String,
    name: String,
    table: Table,
}

/// Finds the members that only support development, sorted by folder: a
/// crate a cargo alias runs, a crate whose name or folder says it is build
/// tooling, a fuzz, benchmark, or example crate, or a test fixture or
/// helper, and a crate that other members use only as a build or dev
/// dependency. A member whose manifest cannot be read is kept, so the run
/// reports it.
fn left_out(workspace: &Workspace) -> Vec<Left> {
    let members: Vec<Member> = manifest::members(workspace)
        .into_iter()
        .filter_map(|dir| {
            let table = manifest::read(&dir.join("Cargo.toml")).ok()?;
            let name = table
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let rel = relative(&workspace.root, &dir)?;
            Some(Member {
                dir,
                rel,
                name,
                table,
            })
        })
        .collect();
    let aliases = aliases(&workspace.root, &members);
    let used = dependents(workspace, &members);
    let mut left: Vec<Left> = members
        .iter()
        .filter_map(|member| {
            let reason = aliases
                .get(&member.dir)
                .map(|alias| format!("run by the cargo alias `{alias}`"))
                .or_else(|| by_name(member).map(str::to_owned))
                .or_else(|| by_use(used.get(&member.dir)).map(str::to_owned))?;
            Some(Left {
                dir: member.rel.clone(),
                reason,
            })
        })
        .collect();
    left.sort_by(|a, b| a.dir.cmp(&b.dir));
    left
}

/// Reads a member's name and folder as words split at `-` and `_`.
fn by_name(member: &Member) -> Option<&'static str> {
    let starts_with_build = [member.name.as_str(), last_part(&member.rel)]
        .iter()
        .filter_map(|name| name.split(['-', '_']).next())
        .any(|word| word.eq_ignore_ascii_case("build"));
    if starts_with_build {
        return Some("build tooling");
    }
    let words: BTreeSet<String> = member
        .rel
        .split('/')
        .chain([member.name.as_str()])
        .flat_map(|part| part.split(['-', '_']))
        .map(str::to_ascii_lowercase)
        .collect();
    let has = |set: &[&str]| set.iter().any(|word| words.contains(*word));
    if has(&["xtask", "xtasks"]) {
        Some("build tooling")
    } else if has(&["fuzz", "fuzzing", "fuzzer", "fuzzers"]) {
        Some("a fuzz crate")
    } else if has(&["bench", "benches", "benchmark", "benchmarks"]) {
        Some("a benchmark crate")
    } else if has(&["example", "examples"]) {
        Some("an example crate")
    } else if has(&[
        "test",
        "tests",
        "testing",
        "testutil",
        "testutils",
        "fixture",
        "fixtures",
    ]) {
        Some("a test fixture or helper")
    } else {
        None
    }
}

fn last_part(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// How the other members use one member.
#[derive(Debug, Default)]
struct Uses {
    normal: bool,
    build: bool,
    dev: bool,
}

/// Leaves out a member that other members use, but never as an ordinary
/// dependency.
fn by_use(uses: Option<&Uses>) -> Option<&'static str> {
    let uses = uses?;
    if uses.normal {
        None
    } else if uses.build {
        Some("used only as a build dependency")
    } else if uses.dev {
        Some("used only as a dev dependency")
    } else {
        None
    }
}

/// Records, for each member folder, how the other members depend on it.
fn dependents(workspace: &Workspace, members: &[Member]) -> BTreeMap<PathBuf, Uses> {
    let mut by_dir: BTreeMap<PathBuf, Uses> = BTreeMap::new();
    for member in members {
        for kind in ["dependencies", "build-dependencies", "dev-dependencies"] {
            let deps = manifest::deps_in(&member.dir, &member.table, Some(workspace), &[kind]);
            for dep in deps.into_iter().filter(|dep| dep.dir != member.dir) {
                let found = by_dir.entry(dep.dir).or_default();
                match kind {
                    "dependencies" => found.normal = true,
                    "build-dependencies" => found.build = true,
                    _ => found.dev = true,
                }
            }
        }
    }
    by_dir
}

/// Maps the folder of each member that a cargo alias in the root's
/// `.cargo/config.toml` runs, through `-p`, `--package`, or
/// `--manifest-path`, to the first such alias by name.
fn aliases(root: &Path, members: &[Member]) -> BTreeMap<PathBuf, String> {
    let mut found = BTreeMap::new();
    let config = [".cargo/config.toml", ".cargo/config"]
        .iter()
        .map(|rel| root.join(rel))
        .find(|path| path.is_file());
    let Some(table) = config.and_then(|path| manifest::read(&path).ok()) else {
        return found;
    };
    let Some(aliases) = table.get("alias").and_then(Value::as_table) else {
        return found;
    };
    for (alias, value) in aliases {
        let words: Vec<String> = match value {
            Value::String(text) => text.split_whitespace().map(str::to_owned).collect(),
            Value::Array(items) => items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            _ => continue,
        };
        for target in alias_targets(&words) {
            let member = members.iter().find(|m| match &target {
                Target::Package(name) => m.name == *name,
                Target::Manifest(path) => manifest::join(root, path) == m.dir.join("Cargo.toml"),
            });
            if let Some(member) = member {
                found
                    .entry(member.dir.clone())
                    .or_insert_with(|| alias.clone());
            }
        }
    }
    found
}

/// What one alias names to run.
enum Target {
    Package(String),
    Manifest(String),
}

fn alias_targets(words: &[String]) -> Vec<Target> {
    let mut out = Vec::new();
    let mut words = words.iter().map(String::as_str);
    while let Some(word) = words.next() {
        if word == "-p" || word == "--package" {
            out.extend(words.next().map(|name| Target::Package(name.to_owned())));
        } else if let Some(name) = word.strip_prefix("--package=") {
            out.push(Target::Package(name.to_owned()));
        } else if let Some(name) = word.strip_prefix("-p").filter(|n| !n.is_empty()) {
            out.push(Target::Package(name.to_owned()));
        } else if word == "--manifest-path" {
            out.extend(words.next().map(|path| Target::Manifest(path.to_owned())));
        } else if let Some(path) = word.strip_prefix("--manifest-path=") {
            out.push(Target::Manifest(path.to_owned()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn package(dir: &Path, rel: &str, name: &str, extra: &str) {
        write_file(
            dir,
            &format!("{rel}/Cargo.toml"),
            &format!("[package]\nname = \"{name}\"\n{extra}"),
        );
    }

    #[test]
    fn a_workspace_leaves_out_its_development_crates_with_their_reasons() {
        let tmp = tempfile::tempdir().unwrap();
        let root = manifest::normalize(tmp.path()).unwrap();
        write_file(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\", \"xtask\", \"fuzz\"]\n",
        );
        write_file(
            &root,
            ".cargo/config.toml",
            "[alias]\ngen = \"run -p codegen --\"\n",
        );
        package(
            &root,
            "crates/app",
            "app",
            "[dependencies]\ncore = { path = \"../core\" }\n\
             [build-dependencies]\nembed = { path = \"../embed\" }\n\
             [dev-dependencies]\nhelpers = { path = \"../helpers\" }\ncore = { path = \"../core\" }\n",
        );
        package(&root, "crates/core", "core", "");
        package(&root, "crates/embed", "embed", "");
        package(&root, "crates/helpers", "helpers", "");
        package(&root, "crates/codegen", "codegen", "");
        package(&root, "crates/build-ui", "build-ui", "");
        package(&root, "crates/test-plugin", "runvil-plugin", "");
        package(&root, "crates/bench-suite", "suite", "");
        package(&root, "xtask", "xtask", "");
        package(&root, "fuzz", "app-fuzz", "");
        let written = write(&root.join("crates").join("app")).unwrap().unwrap();
        assert_eq!(written.path, root.join("vibelight.toml"));
        assert_eq!(written.left_out, 8);
        let text = fs::read_to_string(&written.path).unwrap();
        assert_eq!(
            text,
            "# Written by vibelight. It chooses the crates a plain `vibelight` outlines:\n\
             # edit `exclude` to change which of them are analyzed.\n\
             workspace = true\n\
             follow = [\"**\"]\n\
             exclude = [\n    \
             \"crates/bench-suite\",  # a benchmark crate\n    \
             \"crates/build-ui\",  # build tooling\n    \
             \"crates/codegen\",  # run by the cargo alias `gen`\n    \
             \"crates/embed\",  # used only as a build dependency\n    \
             \"crates/helpers\",  # used only as a dev dependency\n    \
             \"crates/test-plugin\",  # a test fixture or helper\n    \
             \"fuzz\",  # a fuzz crate\n    \
             \"xtask\",  # build tooling\n\
             ]\n\
             out = \".vibelight\"\n"
        );
        let parsed = config::read(&written.path).unwrap();
        assert_eq!(parsed.exclude.len(), 8);
        assert_eq!(parsed.follow, ["**"]);
    }

    #[test]
    fn a_lone_crate_is_found_above_the_start_and_lists_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let root = manifest::normalize(tmp.path()).unwrap();
        package(&root, ".", "solo", "");
        write_file(&root, "src/lib.rs", "//! Solo.\n");
        let written = write(&root.join("src")).unwrap().unwrap();
        assert_eq!(written.path, root.join("vibelight.toml"));
        assert_eq!(written.left_out, 0);
        let text = fs::read_to_string(&written.path).unwrap();
        assert!(
            text.ends_with(
                "crates = [\".\"]\nfollow = [\"**\"]\nexclude = []\nout = \".vibelight\"\n"
            ),
            "{text}"
        );
    }

    #[test]
    fn a_folder_with_no_manifest_above_it_gets_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(write(tmp.path()).unwrap(), None);
        assert!(!tmp.path().join("vibelight.toml").exists());
    }

    #[test]
    fn alias_words_name_packages_and_manifests() {
        let words: Vec<String> =
            "run -pone --package two --package=three -p four --manifest-path tools/x/Cargo.toml"
                .split_whitespace()
                .map(str::to_owned)
                .collect();
        let names: Vec<String> = alias_targets(&words)
            .into_iter()
            .map(|target| match target {
                Target::Package(name) => name,
                Target::Manifest(path) => format!("manifest {path}"),
            })
            .collect();
        assert_eq!(
            names,
            ["one", "two", "three", "four", "manifest tools/x/Cargo.toml"]
        );
    }
}
