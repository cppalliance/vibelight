//! Chooses the crates of a run: crate paths, the members of a workspace,
//! and the path dependencies that follow globs allow.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use toml::Value;

use crate::glob;
use crate::manifest::{self, Dep, Workspace};
use crate::root::{self, Input};

/// One crate of a run.
#[derive(Debug)]
pub(crate) struct Package {
    /// The crate directory, absolute and normalized, which the path
    /// dependencies of other crates point at.
    pub(crate) dir: PathBuf,
    /// The crate's targets, read from the path as it was given.
    pub(crate) input: Input,
    /// The path dependencies its manifest names.
    pub(crate) deps: Vec<Dep>,
}

/// The crates a run starts from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Selection {
    /// Crate paths, each of which must be a crate, and crate folders that a
    /// glob found, which are skipped with a note when they have nothing to
    /// outline.
    Paths {
        paths: Vec<PathBuf>,
        globbed: Vec<PathBuf>,
    },
    /// The members of the workspace that holds this directory.
    Workspace(PathBuf),
}

/// The crates of a run.
#[derive(Debug)]
pub(crate) struct Chosen {
    /// The directory follow globs and the file paths of multi-crate
    /// warnings are measured from: the workspace root, or else the first
    /// crate.
    pub(crate) base: PathBuf,
    /// Sorted by directory.
    pub(crate) packages: Vec<Package>,
    /// One line for each crate skipped because it has nothing to outline.
    pub(crate) notes: Vec<String>,
}

/// Chooses the crates of a run from the selection and the follow globs.
///
/// # Errors
/// Returns an error for a single `.rs` file given with other crates, a
/// workspace selection with no workspace, or a crate path that cannot be
/// read or has nothing to outline.
pub(crate) fn choose(selection: &Selection, follow: &[String]) -> Result<Chosen> {
    let (paths, globbed, workspace): (&[PathBuf], &[PathBuf], Option<&PathBuf>) = match selection {
        Selection::Paths { paths, globbed } => (paths, globbed, None),
        Selection::Workspace(start) => (&[], &[], Some(start)),
    };
    if let Some(file) = paths.iter().find(|p| p.is_file() && !is_manifest(p)) {
        if paths.len() > 1 || !globbed.is_empty() || !follow.is_empty() {
            bail!(
                "{} is a single file, which cannot be outlined together with other crates",
                file.display()
            );
        }
        let input = root::find(file)?;
        let dir = manifest::normalize(&input.dir)?;
        return Ok(Chosen {
            base: dir.clone(),
            packages: vec![Package {
                dir,
                input,
                deps: Vec::new(),
            }],
            notes: Vec::new(),
        });
    }
    let first = workspace
        .cloned()
        .or_else(|| paths.first().map(|p| crate_dir(p)))
        .or_else(|| globbed.first().cloned())
        .unwrap_or_else(|| PathBuf::from("."));
    let found = manifest::find_workspace(&first)?;
    let base = match &found {
        Some(w) => w.root.clone(),
        None => manifest::normalize(&first)?,
    };
    let mut chosen: BTreeMap<PathBuf, Package> = BTreeMap::new();
    let mut notes = Vec::new();
    if workspace.is_some() {
        let Some(w) = &found else {
            bail!("no workspace manifest at {} or above it", first.display());
        };
        for dir in manifest::members(w) {
            if let Some(package) = optional(&dir, &mut notes)? {
                chosen.insert(package.dir.clone(), package);
            }
        }
    } else {
        for path in paths {
            let package = package(root::find(&crate_dir(path))?)?;
            chosen.insert(package.dir.clone(), package);
        }
        for dir in globbed {
            if let Some(package) = optional(dir, &mut notes)? {
                chosen.insert(package.dir.clone(), package);
            }
        }
    }
    if !follow.is_empty() {
        let mut seen: BTreeSet<PathBuf> = chosen.keys().cloned().collect();
        let mut queue: Vec<PathBuf> = chosen
            .values()
            .flat_map(|p| p.deps.iter().map(|d| d.dir.clone()))
            .collect();
        while let Some(dir) = queue.pop() {
            if !seen.insert(dir.clone()) {
                continue;
            }
            let Some(rel) = relative(&base, &dir) else {
                continue;
            };
            if !follow.iter().any(|g| glob::matches(g, &rel)) {
                continue;
            }
            if let Some(package) = optional(&dir, &mut notes)? {
                queue.extend(package.deps.iter().map(|d| d.dir.clone()));
                chosen.insert(package.dir.clone(), package);
            }
        }
    }
    notes.sort();
    notes.dedup();
    if chosen.is_empty() {
        bail!("no crate of the run has anything to outline");
    }
    Ok(Chosen {
        base,
        packages: chosen.into_values().collect(),
        notes,
    })
}

fn is_manifest(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n == "Cargo.toml")
}

/// Returns the crate directory a path names: the path itself, or the
/// directory of a `Cargo.toml`.
fn crate_dir(path: &Path) -> PathBuf {
    if is_manifest(path) {
        return path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    }
    path.to_path_buf()
}

/// Reads a crate's dependencies beside the targets already found.
fn package(input: Input) -> Result<Package> {
    let dir = manifest::normalize(&input.dir)?;
    let deps = dependencies(&dir)?;
    Ok(Package { dir, input, deps })
}

fn dependencies(dir: &Path) -> Result<Vec<Dep>> {
    let path = dir.join("Cargo.toml");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let table = manifest::read(&path)?;
    let workspace: Option<Workspace> = manifest::find_workspace(dir)?;
    Ok(manifest::path_deps(dir, &table, workspace.as_ref()))
}

/// Reads a crate that the run pulled in, or notes why it is skipped when
/// it has nothing to outline.
fn optional(dir: &Path, notes: &mut Vec<String>) -> Result<Option<Package>> {
    let targets = root::dir_targets(dir)?;
    if targets.is_empty() {
        notes.push(format!(
            "skipped {}: it has {}",
            package_name(dir),
            empty_reason(dir)
        ));
        return Ok(None);
    }
    let input = Input {
        dir: dir.to_path_buf(),
        targets,
    };
    package(input).map(Some)
}

fn package_name(dir: &Path) -> String {
    manifest::read(&dir.join("Cargo.toml"))
        .ok()
        .and_then(|table| {
            let package = table.get("package")?.as_table()?;
            package
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .or_else(|| dir.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

fn empty_reason(dir: &Path) -> &'static str {
    if ["tests", "examples", "benches"]
        .iter()
        .any(|d| dir.join(d).is_dir())
    {
        "only tests, examples, or benches"
    } else if dir.join("build.rs").is_file() {
        "only a build script"
    } else {
        "no library or binary source"
    }
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
