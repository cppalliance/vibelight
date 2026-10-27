//! Reads `vibelight.toml`: which crates a run covers and where it writes.
//! It is found in the current directory or the first parent that holds
//! one, and its relative paths are measured from its own directory.

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use toml::{Table, Value};

use crate::{glob, manifest};

/// The file looked up in the current directory and its parents.
pub(crate) const FILE: &str = "vibelight.toml";

/// The folder a run writes into when nothing names another one, beside the
/// config, or in the current directory without one.
pub(crate) const OUT: &str = ".vibelight";

/// The crates a config chooses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Crates {
    /// Neither `workspace` nor `crates` is set.
    Unset,
    /// The members of the workspace that holds this directory.
    Workspace(PathBuf),
    /// Each plain entry of `crates`, and each crate directory that a glob
    /// entry matches.
    Listed {
        paths: Vec<PathBuf>,
        globbed: Vec<PathBuf>,
    },
}

/// A parsed `vibelight.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    /// The folder that holds the file.
    pub(crate) dir: PathBuf,
    pub(crate) crates: Crates,
    pub(crate) follow: Vec<String>,
    pub(crate) exclude: Vec<String>,
    /// The folder to write into, from `out`, or `.vibelight` beside the file.
    pub(crate) out: PathBuf,
}

/// The keys a config may hold.
const KEYS: &[&str] = &["workspace", "crates", "follow", "exclude", "out"];

/// Finds and reads the first `vibelight.toml` in `start` or above it.
///
/// # Errors
/// Returns an error when the file cannot be read or holds a key or value
/// it should not.
pub(crate) fn find(start: &Path) -> Result<Option<Config>> {
    let start = manifest::normalize(start)?;
    for dir in start.ancestors() {
        let path = dir.join(FILE);
        if path.is_file() {
            return read(&path).map(Some);
        }
    }
    Ok(None)
}

/// Reads the config file at `path`, an absolute path.
///
/// # Errors
/// Returns an error when the file cannot be read or does not parse.
pub(crate) fn read(path: &Path) -> Result<Config> {
    let text =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    parse(path, &text)
}

/// Parses the text of the config file at `path`, an absolute path.
///
/// # Errors
/// Returns an error naming the key that is unknown or has the wrong type.
pub(crate) fn parse(path: &Path, text: &str) -> Result<Config> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let name = path.display();
    let table: Table = text
        .parse()
        .with_context(|| format!("cannot parse {name}"))?;
    if let Some(key) = table.keys().find(|key| !KEYS.contains(&key.as_str())) {
        bail!("{name}: unknown key `{key}`");
    }
    let crates = match (table.get("workspace"), table.get("crates")) {
        (Some(_), Some(_)) => bail!("{name}: set `workspace` or `crates`, not both"),
        (Some(Value::Boolean(true)), None) => Crates::Workspace(dir.to_path_buf()),
        (Some(Value::Boolean(false)) | None, None) => Crates::Unset,
        (Some(Value::String(start)), None) => Crates::Workspace(manifest::join(dir, start)),
        (Some(_), None) => bail!("{name}: `workspace` is true or a directory"),
        (None, Some(value)) => listed(dir, &strings(&name, "crates", value)?),
    };
    let follow = match table.get("follow") {
        Some(value) => strings(&name, "follow", value)?,
        None => Vec::new(),
    };
    let exclude = match table.get("exclude") {
        Some(value) => strings(&name, "exclude", value)?,
        None => Vec::new(),
    };
    let out = match table.get("out") {
        Some(value) => {
            let out = value
                .as_str()
                .with_context(|| format!("{name}: `out` is a directory"))?;
            manifest::join(dir, out)
        }
        None => dir.join(OUT),
    };
    Ok(Config {
        dir: dir.to_path_buf(),
        crates,
        follow,
        exclude,
        out,
    })
}

fn strings(name: &impl Display, key: &str, value: &Value) -> Result<Vec<String>> {
    let wrong = || format!("{name}: `{key}` is a list of strings");
    value
        .as_array()
        .with_context(wrong)?
        .iter()
        .map(|item| item.as_str().map(str::to_owned).with_context(wrong))
        .collect()
}

fn listed(dir: &Path, entries: &[String]) -> Crates {
    let mut paths = Vec::new();
    let mut globbed = Vec::new();
    for entry in entries {
        if glob::is_glob(entry) {
            globbed.extend(manifest::crate_dirs(dir, entry));
        } else {
            paths.push(manifest::join(dir, entry));
        }
    }
    Crates::Listed { paths, globbed }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(text: &str) -> (PathBuf, Result<Config>) {
        let dir = manifest::normalize(Path::new("repo")).unwrap();
        let parsed = parse(&dir.join(FILE), text);
        (dir, parsed)
    }

    #[test]
    fn every_key_is_read_and_paths_are_measured_from_the_file() {
        let (dir, parsed) = config(
            "workspace = true\nfollow = [\"vendor/*\"]\n\
             exclude = [\"crates/build-*\", \"crates/harness-internal/test-plugin\"]\n\
             out = \"../outlines\"\n",
        );
        let config = parsed.unwrap();
        assert_eq!(config.crates, Crates::Workspace(dir.clone()));
        assert_eq!(config.follow, ["vendor/*"]);
        assert_eq!(
            config.exclude,
            ["crates/build-*", "crates/harness-internal/test-plugin"]
        );
        assert_eq!(config.out, dir.parent().unwrap().join("outlines"));
    }

    #[test]
    fn missing_keys_are_unset_and_the_folder_is_vibelights_own() {
        let (dir, parsed) = config("");
        let config = parsed.unwrap();
        assert_eq!(config.dir, dir);
        assert_eq!(config.crates, Crates::Unset);
        assert_eq!(config.follow, Vec::<String>::new());
        assert_eq!(config.exclude, Vec::<String>::new());
        assert_eq!(config.out, dir.join(".vibelight"));
    }

    #[test]
    fn workspace_can_name_a_directory_and_crates_can_list_paths() {
        let (dir, parsed) = config("workspace = \"../other\"\n");
        let other = dir.parent().unwrap().join("other");
        assert_eq!(parsed.unwrap().crates, Crates::Workspace(other));
        let (dir, parsed) = config("crates = [\"crates/a\", \"./b\"]\n");
        assert_eq!(
            parsed.unwrap().crates,
            Crates::Listed {
                paths: vec![dir.join("crates").join("a"), dir.join("b")],
                globbed: Vec::new(),
            }
        );
    }

    #[test]
    fn a_crates_glob_lists_the_crate_folders_it_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = manifest::normalize(tmp.path()).unwrap();
        for name in ["a", "b"] {
            let crate_dir = dir.join("crates").join(name);
            fs::create_dir_all(&crate_dir).unwrap();
            fs::write(crate_dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        }
        fs::create_dir_all(dir.join("crates").join("notes")).unwrap();
        let parsed = parse(&dir.join(FILE), "crates = [\"crates/*\"]\n").unwrap();
        assert_eq!(
            parsed.crates,
            Crates::Listed {
                paths: Vec::new(),
                globbed: vec![dir.join("crates").join("a"), dir.join("crates").join("b")],
            }
        );
    }

    #[test]
    fn unknown_keys_and_wrong_types_are_errors_that_name_the_key() {
        let error = |text: &str| config(text).1.unwrap_err().to_string();
        assert!(error("workspac = true\n").contains("unknown key `workspac`"));
        assert!(error("[markdown]\nout = \"x\"\n").contains("unknown key `markdown`"));
        assert!(error("facts = true\n").contains("unknown key `facts`"));
        assert!(error("workspace = 3\n").contains("`workspace` is true or a directory"));
        assert!(error("crates = \"a\"\n").contains("`crates` is a list of strings"));
        assert!(error("exclude = \"crates/*\"\n").contains("`exclude` is a list of strings"));
        assert!(error("exclude = [\"a\", 2]\n").contains("`exclude` is a list of strings"));
        assert!(error("out = 1\n").contains("`out` is a directory"));
        assert!(error("workspace = true\ncrates = []\n").contains("not both"));
    }
}
