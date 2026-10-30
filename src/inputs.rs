//! Chooses the crates of a run: crate paths, the members of a workspace,
//! and the path dependencies that follow globs allow, less the crates that
//! exclude globs leave out. A crate named on the command line is never
//! left out.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use toml::Value;

use crate::exclude::{Exclude, Filter};
use crate::glob;
use crate::manifest::{self, Dep, Workspace};
use crate::manifest_facts::{self, ManifestFacts};
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
    /// True when the run reached it by following a path dependency rather
    /// than choosing it.
    pub(crate) followed: bool,
    /// What analysis reads from its manifest.
    pub(crate) facts: ManifestFacts,
}

/// The crates a run starts from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Selection {
    /// Crate paths from the command line, each of which must be a crate, or
    /// one `.rs` file. No exclude glob leaves one out.
    Named(Vec<PathBuf>),
    /// The config's `crates`: paths, each of which must be a crate, and
    /// crate folders that a glob found, which are skipped with a note when
    /// they have nothing to outline.
    Listed {
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

/// Chooses the crates of a run from the selection and the follow globs,
/// and then leaves out each crate that an exclude glob matches. An
/// excluded crate is never read, and it is not followed into. A crate the
/// config lists and also excludes gets a note, and so does an exclude glob
/// that matches no crate the config's selection considered. A single `.rs`
/// file has no folder to follow from or exclude, so the globs pass it by.
///
/// # Errors
/// Returns an error for a single `.rs` file given with other crates, a
/// workspace selection with no workspace, a crate path that cannot be read
/// or has nothing to outline, or no crate left.
pub(crate) fn choose(
    selection: &Selection,
    follow: &[String],
    exclude: &Exclude,
) -> Result<Chosen> {
    let none: &[PathBuf] = &[];
    let (named, paths, globbed, workspace) = match selection {
        Selection::Named(named) => (named.as_slice(), none, none, None),
        Selection::Listed { paths, globbed } => (none, paths.as_slice(), globbed.as_slice(), None),
        Selection::Workspace(start) => (none, none, none, Some(start)),
    };
    if let Some(file) = named.iter().find(|p| p.is_file() && !is_manifest(p)) {
        if named.len() > 1 {
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
                followed: false,
                facts: ManifestFacts::default(),
            }],
            notes: Vec::new(),
        });
    }
    let first = workspace
        .cloned()
        .or_else(|| named.first().map(|p| crate_dir(p)))
        .or_else(|| paths.first().map(|p| crate_dir(p)))
        .or_else(|| globbed.first().cloned())
        .unwrap_or_else(|| PathBuf::from("."));
    let found = manifest::find_workspace(&first)?;
    let base = match &found {
        Some(w) => w.root.clone(),
        None => manifest::normalize(&first)?,
    };
    let mut filter = Filter::new(exclude);
    let mut chosen: BTreeMap<PathBuf, Package> = BTreeMap::new();
    let mut notes = Vec::new();
    if workspace.is_some() {
        let Some(w) = &found else {
            bail!("no workspace manifest at {} or above it", first.display());
        };
        for dir in manifest::members(w) {
            if filter.excludes(&dir) {
                continue;
            }
            if let Some(package) = optional(&dir, &mut notes)? {
                chosen.insert(package.dir.clone(), package);
            }
        }
    } else {
        for path in named {
            let package = package(root::find(&crate_dir(path))?)?;
            chosen.insert(package.dir.clone(), package);
        }
        for path in paths {
            let dir = manifest::normalize(&crate_dir(path))?;
            if filter.excludes(&dir) {
                notes.push(format!(
                    "{} is named as a crate and also excluded, so it is left out",
                    shown(&base, &dir)
                ));
                continue;
            }
            let package = package(root::find(&crate_dir(path))?)?;
            chosen.insert(package.dir.clone(), package);
        }
        for dir in globbed {
            if filter.excludes(dir) {
                continue;
            }
            if let Some(package) = optional(dir, &mut notes)? {
                chosen.insert(package.dir.clone(), package);
            }
        }
    }
    if !follow.is_empty() {
        pull_in(&mut chosen, &base, follow, &mut filter, &mut notes)?;
    }
    if named.is_empty() {
        notes.extend(filter.unmatched());
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

/// Adds the path dependencies of the chosen crates that a follow glob
/// allows, and then theirs, except those the filter excludes.
fn pull_in(
    chosen: &mut BTreeMap<PathBuf, Package>,
    base: &Path,
    follow: &[String],
    filter: &mut Filter<'_>,
    notes: &mut Vec<String>,
) -> Result<()> {
    let mut seen: BTreeSet<PathBuf> = chosen.keys().cloned().collect();
    let mut queue: Vec<PathBuf> = chosen
        .values()
        .flat_map(|p| p.deps.iter().map(|d| d.dir.clone()))
        .collect();
    while let Some(dir) = queue.pop() {
        if !seen.insert(dir.clone()) {
            continue;
        }
        let Some(rel) = manifest::relative(base, &dir) else {
            continue;
        };
        if !follow.iter().any(|g| glob::matches(g, &rel)) || filter.excludes(&dir) {
            continue;
        }
        if let Some(mut package) = optional(&dir, notes)? {
            package.followed = true;
            queue.extend(package.deps.iter().map(|d| d.dir.clone()));
            chosen.insert(package.dir.clone(), package);
        }
    }
    Ok(())
}

/// Shows a crate folder below `base` with forward slashes, or else in full.
pub(crate) fn shown(base: &Path, dir: &Path) -> String {
    manifest::relative(base, dir)
        .filter(|rel| !rel.is_empty())
        .unwrap_or_else(|| dir.display().to_string())
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
    let facts = manifest_facts::read(&dir)?;
    Ok(Package {
        dir,
        input,
        deps,
        followed: false,
        facts,
    })
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
