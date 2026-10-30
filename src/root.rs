//! Finds the targets of a crate directory or a single `.rs` file.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use toml::Value;

use crate::manifest;

/// One outline to write: a library, a binary, or a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Target {
    /// The output file stem, such as `runvil_parser` or `foo.bin`.
    pub(crate) stem: String,
    /// The target name the H1 shows.
    pub(crate) title: String,
    /// The crate name ids use: the target name with hyphens turned into
    /// underscores.
    pub(crate) crate_name: String,
    /// The root file, relative to the crate directory, with forward slashes.
    pub(crate) root: String,
    /// What the target is; `TargetKind::Lib` marks the library target, the one
    /// dependencies name.
    pub(crate) kind: TargetKind,
}

/// What a target is, set where the target is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetKind {
    /// A package's library target.
    Lib,
    /// A package's binary target.
    Bin,
    /// A single `.rs` file outlined on its own.
    SingleFile,
}

/// The crate being outlined: its directory and its targets, sorted by stem.
#[derive(Debug)]
pub(crate) struct Input {
    /// The directory every crate-relative path is measured from.
    pub(crate) dir: PathBuf,
    /// The targets, sorted by output stem.
    pub(crate) targets: Vec<Target>,
}

/// Reads a crate directory or a single `.rs` file into its targets.
///
/// # Errors
/// Returns an error when the path is unreadable, is neither a directory nor a
/// `.rs` file, holds a workspace-only manifest, or has no targets.
pub(crate) fn find(path: &Path) -> Result<Input> {
    let meta = fs::metadata(path).with_context(|| format!("cannot read {}", path.display()))?;
    if meta.is_file() {
        if path.file_name().is_some_and(|n| n == "Cargo.toml") {
            let dir = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            return find(dir);
        }
        return single_file(path);
    }
    let targets = dir_targets(path)?;
    if targets.is_empty() {
        bail!(
            "{} is not a crate: it has no library or binary target",
            path.display()
        );
    }
    Ok(Input {
        dir: path.to_path_buf(),
        targets,
    })
}

/// Reads the targets of a crate directory, sorted by output stem; the list
/// is empty for a crate with nothing to outline.
///
/// # Errors
/// Returns an error when the manifest is unreadable, holds no package, or
/// is a workspace-only manifest.
pub(crate) fn dir_targets(dir: &Path) -> Result<Vec<Target>> {
    let manifest = dir.join("Cargo.toml");
    let mut targets = if manifest.is_file() {
        from_manifest(dir, &manifest)?
    } else {
        without_manifest(dir)?
    };
    targets.sort_by(|a, b| a.stem.cmp(&b.stem));
    Ok(targets)
}

fn single_file(path: &Path) -> Result<Input> {
    if path.extension().and_then(|e| e.to_str()) != Some("rs") {
        bail!(
            "{} is neither a crate directory nor a .rs file",
            path.display()
        );
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .with_context(|| format!("{} has no usable file name", path.display()))?;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("{} has no usable file name", path.display()))?;
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    Ok(Input {
        dir,
        targets: vec![Target {
            stem: stem.to_owned(),
            title: stem.to_owned(),
            crate_name: stem.replace('-', "_"),
            root: name.to_owned(),
            kind: TargetKind::SingleFile,
        }],
    })
}

fn from_manifest(dir: &Path, manifest: &Path) -> Result<Vec<Target>> {
    let table = manifest::read(manifest)?;
    let Some(package) = table.get("package").and_then(Value::as_table) else {
        if table.contains_key("workspace") {
            bail!(
                "{} is a workspace manifest; point vibelight at a member crate's directory, or set `workspace = true` in vibelight.toml",
                manifest.display()
            );
        }
        bail!("{} has no [package] table", manifest.display());
    };
    let package_name = package
        .get("name")
        .and_then(Value::as_str)
        .with_context(|| format!("{} has no package name", manifest.display()))?;
    let flag = |key: &str| package.get(key).and_then(Value::as_bool).unwrap_or(true);

    let lib_table = table.get("lib").and_then(Value::as_table);
    let lib_name = lib_table
        .and_then(|t| t.get("name"))
        .and_then(Value::as_str)
        .unwrap_or(package_name)
        .replace('-', "_");
    let lib_path = match lib_table
        .and_then(|t| t.get("path"))
        .and_then(Value::as_str)
    {
        Some(path) => Some(slashes(path)),
        None if (lib_table.is_some() || flag("autolib")) && dir.join("src/lib.rs").is_file() => {
            Some("src/lib.rs".to_owned())
        }
        None => None,
    };

    let mut bins: BTreeMap<String, String> = BTreeMap::new();
    if let Some(entries) = table.get("bin").and_then(Value::as_array) {
        for entry in entries.iter().filter_map(Value::as_table) {
            let Some(name) = entry.get("name").and_then(Value::as_str) else {
                continue;
            };
            let path = entry
                .get("path")
                .and_then(Value::as_str)
                .map_or_else(|| default_bin_path(dir, name), slashes);
            bins.insert(name.to_owned(), path);
        }
    }
    if flag("autobins") {
        let mut found = discover_bins(dir);
        if dir.join("src/main.rs").is_file() {
            found.insert(package_name.to_owned(), "src/main.rs".to_owned());
        }
        for (name, path) in found {
            if !bins.contains_key(&name) && !bins.values().any(|p| *p == path) {
                bins.insert(name, path);
            }
        }
    }
    Ok(assemble(lib_path.map(|p| (lib_name, p)), bins))
}

fn without_manifest(dir: &Path) -> Result<Vec<Target>> {
    let absolute =
        std::path::absolute(dir).with_context(|| format!("cannot resolve {}", dir.display()))?;
    let dir_name = absolute
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("{} has no usable directory name", dir.display()))?;
    let lib = dir
        .join("src/lib.rs")
        .is_file()
        .then(|| (dir_name.replace('-', "_"), "src/lib.rs".to_owned()));
    let mut bins = discover_bins(dir);
    if dir.join("src/main.rs").is_file() {
        bins.insert(dir_name.to_owned(), "src/main.rs".to_owned());
    }
    Ok(assemble(lib, bins))
}

/// Turns a library and binaries into targets, giving a binary that shares
/// the library's name a `.bin` stem.
fn assemble(lib: Option<(String, String)>, bins: BTreeMap<String, String>) -> Vec<Target> {
    let mut targets = Vec::new();
    let lib_stem = lib.as_ref().map(|(name, _)| name.clone());
    if let Some((name, root)) = lib {
        targets.push(Target {
            stem: name.clone(),
            title: name.clone(),
            crate_name: name,
            root,
            kind: TargetKind::Lib,
        });
    }
    for (name, root) in bins {
        let stem = if lib_stem.as_deref() == Some(name.as_str()) {
            format!("{name}.bin")
        } else {
            name.clone()
        };
        targets.push(Target {
            stem,
            crate_name: name.replace('-', "_"),
            title: name,
            root,
            kind: TargetKind::Bin,
        });
    }
    targets
}

/// Finds `src/bin/*.rs` and `src/bin/*/main.rs`, keyed by binary name.
fn discover_bins(dir: &Path) -> BTreeMap<String, String> {
    let mut bins = BTreeMap::new();
    let Ok(entries) = fs::read_dir(dir.join("src/bin")) else {
        return bins;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.is_file() {
            if let Some(stem) = name.strip_suffix(".rs") {
                bins.insert(stem.to_owned(), format!("src/bin/{name}"));
            }
        } else if path.join("main.rs").is_file() {
            bins.insert(name.to_owned(), format!("src/bin/{name}/main.rs"));
        }
    }
    bins
}

fn default_bin_path(dir: &Path, name: &str) -> String {
    let file = format!("src/bin/{name}.rs");
    if dir.join(&file).is_file() {
        return file;
    }
    let nested = format!("src/bin/{name}/main.rs");
    if dir.join(&nested).is_file() {
        return nested;
    }
    "src/main.rs".to_owned()
}

fn slashes(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn stems(input: &Input) -> Vec<(String, String, String)> {
        input
            .targets
            .iter()
            .map(|t| (t.stem.clone(), t.crate_name.clone(), t.root.clone()))
            .collect()
    }

    #[test]
    fn lib_and_bin_get_two_targets() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            "[package]\nname = \"cloud-providers\"\n",
        );
        write(tmp.path(), "src/lib.rs", "");
        write(tmp.path(), "src/bin/serve.rs", "");
        let input = find(tmp.path()).unwrap();
        assert_eq!(
            stems(&input),
            [
                (
                    "cloud_providers".into(),
                    "cloud_providers".into(),
                    "src/lib.rs".into()
                ),
                ("serve".into(), "serve".into(), "src/bin/serve.rs".into()),
            ]
        );
    }

    #[test]
    fn bin_named_like_the_lib_gets_a_bin_stem() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "Cargo.toml", "[package]\nname = \"foo\"\n");
        write(tmp.path(), "src/lib.rs", "");
        write(tmp.path(), "src/main.rs", "");
        let input = find(tmp.path()).unwrap();
        let names: Vec<_> = input.targets.iter().map(|t| t.stem.as_str()).collect();
        assert_eq!(names, ["foo", "foo.bin"]);
        assert_eq!(input.targets[1].title, "foo");
    }

    #[test]
    fn custom_lib_path_and_declared_bins() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            tmp.path(),
            "Cargo.toml",
            "[package]\nname = \"x\"\n[lib]\nname = \"core-x\"\npath = \"lib/top.rs\"\n\
             [[bin]]\nname = \"tool\"\npath = \"src/main.rs\"\n",
        );
        write(tmp.path(), "lib/top.rs", "");
        write(tmp.path(), "src/main.rs", "");
        let input = find(tmp.path()).unwrap();
        assert_eq!(
            stems(&input),
            [
                ("core_x".into(), "core_x".into(), "lib/top.rs".into()),
                ("tool".into(), "tool".into(), "src/main.rs".into()),
            ]
        );
    }

    #[test]
    fn no_manifest_uses_the_directory_name() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("my-crate");
        write(&dir, "src/lib.rs", "");
        write(&dir, "src/main.rs", "");
        write(&dir, "src/bin/extra.rs", "");
        let input = find(&dir).unwrap();
        let names: Vec<_> = input.targets.iter().map(|t| t.stem.as_str()).collect();
        assert_eq!(names, ["extra", "my-crate", "my_crate"]);
    }

    #[test]
    fn single_file_uses_its_stem() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "demo.rs", "");
        let input = find(&tmp.path().join("demo.rs")).unwrap();
        assert_eq!(
            stems(&input),
            [("demo".into(), "demo".into(), "demo.rs".into())]
        );
        assert_eq!(input.dir, tmp.path());
        assert_eq!(input.targets[0].kind, TargetKind::SingleFile);
    }

    #[test]
    fn each_target_records_its_kind() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "Cargo.toml", "[package]\nname = \"foo\"\n");
        write(tmp.path(), "src/lib.rs", "");
        write(tmp.path(), "src/main.rs", "");
        let input = find(tmp.path()).unwrap();
        let kinds: Vec<TargetKind> = input.targets.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, [TargetKind::Lib, TargetKind::Bin]);
    }

    #[test]
    fn workspace_only_manifest_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = [\"a\"]\n");
        let error = find(tmp.path()).unwrap_err().to_string();
        assert!(error.contains("workspace manifest"), "{error}");
    }

    #[test]
    fn a_directory_without_targets_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "notes.txt", "");
        let error = find(tmp.path()).unwrap_err().to_string();
        assert!(error.contains("not a crate"), "{error}");
    }
}
