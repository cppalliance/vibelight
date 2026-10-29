//! Finds where a package's build script, tests, benches, and examples are,
//! from its manifest and the default places Cargo looks.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::{Table, Value};

/// Returns the build script: `package.build` when it names a file, or else
/// `build.rs` when it exists and `build` is not `false`.
pub(crate) fn build_script(dir: &Path, package: Option<&Table>) -> Option<String> {
    match package.and_then(|p| p.get("build")) {
        Some(Value::String(path)) => Some(path.replace('\\', "/")),
        Some(Value::Boolean(false)) => None,
        _ => dir
            .join("build.rs")
            .is_file()
            .then(|| "build.rs".to_owned()),
    }
}

/// Returns the roots of one kind of target, crate-relative, sorted, each
/// once: each `[[kind]]` entry's `path`, or its default place, and, with
/// `auto`, each `folder/*.rs` and `folder/*/main.rs`.
pub(crate) fn roots(
    dir: &Path,
    table: &Table,
    kind: &str,
    folder: &str,
    auto: bool,
) -> Vec<String> {
    let mut out = BTreeSet::new();
    let entries = table.get(kind).and_then(Value::as_array);
    for entry in entries.into_iter().flatten().filter_map(Value::as_table) {
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .map(|path| path.replace('\\', "/"));
        let named = entry
            .get("name")
            .and_then(Value::as_str)
            .map(|name| default_root(dir, folder, name));
        out.extend(path.or(named));
    }
    if auto {
        out.extend(discover(dir, folder));
    }
    out.into_iter().collect()
}

/// Returns where a named target lives by default: `folder/name/main.rs`
/// when that exists, or else `folder/name.rs`.
fn default_root(dir: &Path, folder: &str, name: &str) -> String {
    let nested = format!("{folder}/{name}/main.rs");
    if dir.join(&nested).is_file() {
        nested
    } else {
        format!("{folder}/{name}.rs")
    }
}

/// Finds `folder/*.rs` and `folder/*/main.rs`.
fn discover(dir: &Path, folder: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir.join(folder)) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            found.push(format!("{folder}/{name}"));
        } else if path.join("main.rs").is_file() {
            found.push(format!("{folder}/{name}/main.rs"));
        }
    }
    found
}
