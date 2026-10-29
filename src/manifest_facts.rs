//! Reads what analysis needs from a package manifest beside the path
//! dependencies: the package's name, whether it may be published, its
//! features, every dependency entry, and where its build script, tests,
//! benches, and examples are.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Result, bail};
use toml::{Table, Value};

use crate::manifest;
use crate::manifest_roots::{build_script, roots};

/// What analysis reads from one package manifest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ManifestFacts {
    /// The package name; `None` with no manifest.
    pub(crate) name: Option<String>,
    /// Whether `publish` allows publishing the package. It never shows that
    /// the package is published.
    pub(crate) publish_allowed: bool,
    /// The `[features]` table and the implicit feature of each optional
    /// dependency no `dep:` entry names, sorted by name.
    pub(crate) features: Vec<Feature>,
    /// Every dependency entry, sorted by name, kind, and target table.
    pub(crate) deps: Vec<DepEntry>,
    /// The build script, crate-relative: `package.build`, or `build.rs`.
    pub(crate) build: Option<String>,
    /// The test roots, crate-relative, sorted: each `[[test]]` entry's path
    /// and the default `tests/*.rs` and `tests/*/main.rs`.
    pub(crate) tests: Vec<String>,
    /// The bench roots, found the way the test roots are.
    pub(crate) benches: Vec<String>,
    /// The example roots, found the way the test roots are.
    pub(crate) examples: Vec<String>,
}

/// One feature and what it enables, as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Feature {
    pub(crate) name: String,
    pub(crate) enables: Vec<String>,
    /// True for the feature Cargo makes for an optional dependency.
    pub(crate) implicit: bool,
}

/// The table a dependency entry is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DepKind {
    Normal,
    Dev,
    Build,
}

/// One dependency entry, keyed by its name, kind, and target table, which
/// lead its order.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DepEntry {
    /// The table key, which the source writes as the crate's name.
    pub(crate) name: String,
    pub(crate) kind: DepKind,
    /// The `[target.'...']` table it is under, as written.
    pub(crate) target: Option<String>,
    pub(crate) optional: bool,
    /// The package `package = "..."` names, here or in the workspace entry.
    pub(crate) package: Option<String>,
    /// Whether it is `workspace = true`.
    pub(crate) workspace: bool,
    /// Its features with the workspace entry's added, sorted, each once.
    pub(crate) features: Vec<String>,
}

impl DepEntry {
    /// Returns what an entry is keyed by: its name, kind, and target table.
    fn key(&self) -> (&str, DepKind, Option<&str>) {
        (&self.name, self.kind, self.target.as_deref())
    }
}

/// The dependency tables and the kind each holds, with the spellings Cargo
/// still reads.
const DEP_TABLES: [(&str, DepKind); 5] = [
    ("dependencies", DepKind::Normal),
    ("dev-dependencies", DepKind::Dev),
    ("dev_dependencies", DepKind::Dev),
    ("build-dependencies", DepKind::Build),
    ("build_dependencies", DepKind::Build),
];

/// Reads the facts of the package in `dir`; with no manifest there are
/// none.
///
/// # Errors
/// Returns an error when the manifest cannot be read or parsed.
pub(crate) fn read(dir: &Path) -> Result<ManifestFacts> {
    let path = dir.join("Cargo.toml");
    if !path.is_file() {
        return Ok(ManifestFacts::default());
    }
    let table = manifest::read(&path)?;
    let workspace = manifest::find_workspace(dir)?
        .and_then(|found| manifest::read(&found.root.join("Cargo.toml")).ok())
        .and_then(|root| root.get("workspace").and_then(Value::as_table).cloned());
    Ok(facts(dir, &table, workspace.as_ref()))
}

fn facts(dir: &Path, table: &Table, workspace: Option<&Table>) -> ManifestFacts {
    let package = table.get("package").and_then(Value::as_table);
    let name = package
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let flag = |key: &str| {
        package
            .and_then(|p| p.get(key))
            .and_then(Value::as_bool)
            .unwrap_or(true)
    };
    let deps = dep_entries(table, workspace);
    ManifestFacts {
        publish_allowed: name.is_some() && publish(package, workspace),
        name,
        features: features(table, &deps),
        deps,
        build: build_script(dir, package),
        tests: roots(dir, table, "test", "tests", flag("autotests")),
        benches: roots(dir, table, "bench", "benches", flag("autobenches")),
        examples: roots(dir, table, "example", "examples", flag("autoexamples")),
    }
}

/// Tells whether `publish` allows publishing: absent, `true`, or a list of
/// registries that is not empty, read from `[workspace.package]` for
/// `publish.workspace = true`.
fn publish(package: Option<&Table>, workspace: Option<&Table>) -> bool {
    let written = package.and_then(|p| p.get("publish"));
    let inherited = written
        .and_then(Value::as_table)
        .and_then(|t| t.get("workspace"))
        .and_then(Value::as_bool)
        == Some(true);
    let value = if inherited {
        workspace
            .and_then(|w| w.get("package"))
            .and_then(Value::as_table)
            .and_then(|p| p.get("publish"))
    } else {
        written
    };
    match value {
        Some(Value::Boolean(allowed)) => *allowed,
        Some(Value::Array(registries)) => !registries.is_empty(),
        _ => true,
    }
}

/// Returns every dependency entry in each kind's table and under each
/// `[target.'...']` table, sorted; the first entry with a key wins.
fn dep_entries(table: &Table, workspace: Option<&Table>) -> Vec<DepEntry> {
    let shared = workspace
        .and_then(|w| w.get("dependencies"))
        .and_then(Value::as_table);
    let mut holders: Vec<(Option<&str>, &Table)> = vec![(None, table)];
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        holders.extend(
            targets
                .iter()
                .filter_map(|(key, value)| Some((Some(key.as_str()), value.as_table()?))),
        );
    }
    let mut out: BTreeMap<(String, DepKind, Option<String>), DepEntry> = BTreeMap::new();
    for (target, holder) in holders {
        for (key, kind) in DEP_TABLES {
            let Some(deps) = holder.get(key).and_then(Value::as_table) else {
                continue;
            };
            for (name, value) in deps {
                let key = (name.clone(), kind, target.map(str::to_owned));
                out.entry(key)
                    .or_insert_with(|| entry(name, kind, target, value, shared));
            }
        }
    }
    out.into_values().collect()
}

fn entry(
    name: &str,
    kind: DepKind,
    target: Option<&str>,
    value: &Value,
    shared: Option<&Table>,
) -> DepEntry {
    let local = value.as_table();
    let get = |key: &str| local.and_then(|t| t.get(key));
    let workspace = get("workspace").and_then(Value::as_bool) == Some(true);
    let inherited = shared
        .filter(|_| workspace)
        .and_then(|s| s.get(name))
        .and_then(Value::as_table);
    let mut features = strings(get("features"));
    features.extend(strings(inherited.and_then(|t| t.get("features"))));
    features.sort();
    features.dedup();
    let package = get("package")
        .or_else(|| inherited.and_then(|t| t.get("package")))
        .and_then(Value::as_str)
        .map(str::to_owned);
    DepEntry {
        name: name.to_owned(),
        kind,
        target: target.map(str::to_owned),
        optional: get("optional").and_then(Value::as_bool) == Some(true),
        package,
        workspace,
        features,
    }
}

/// Returns the `[features]` table and the implicit feature of each optional
/// dependency that no feature names with `dep:`, sorted by name.
fn features(table: &Table, deps: &[DepEntry]) -> Vec<Feature> {
    let mut out: BTreeMap<String, Feature> = BTreeMap::new();
    let declared = table.get("features").and_then(Value::as_table);
    for (name, value) in declared.into_iter().flatten() {
        let feature = Feature {
            name: name.clone(),
            enables: strings(Some(value)),
            implicit: false,
        };
        out.insert(name.clone(), feature);
    }
    let named: BTreeSet<String> = out
        .values()
        .flat_map(|f| &f.enables)
        .filter_map(|e| e.strip_prefix("dep:"))
        .map(str::to_owned)
        .collect();
    for dep in deps.iter().filter(|d| d.optional) {
        if named.contains(&dep.name) || out.contains_key(&dep.name) {
            continue;
        }
        let feature = Feature {
            name: dep.name.clone(),
            enables: vec![format!("dep:{}", dep.name)],
            implicit: true,
        };
        out.insert(dep.name.clone(), feature);
    }
    out.into_values().collect()
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

impl ManifestFacts {
    /// Checks the facts' invariants: features and dependency entries sorted
    /// and unique, each implicit feature tied to an optional dependency,
    /// paths with forward slashes, and publishing allowed only for a named
    /// package.
    ///
    /// # Errors
    /// Returns an error naming the first invariant that fails.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.publish_allowed && self.name.is_none() {
            bail!("a package without a name allows publishing");
        }
        if !self.features.windows(2).all(|w| w[0].name < w[1].name) {
            bail!("features are not sorted by name");
        }
        for feature in self.features.iter().filter(|f| f.implicit) {
            let optional = self
                .deps
                .iter()
                .any(|d| d.optional && d.name == feature.name);
            if !optional || feature.enables != [format!("dep:{}", feature.name)] {
                bail!(
                    "implicit feature {} has no optional dependency",
                    feature.name
                );
            }
        }
        let sorted = self
            .deps
            .windows(2)
            .all(|w| w[0].key() < w[1].key() && w[0] < w[1]);
        if !sorted {
            bail!("dependency entries are not sorted, each key once");
        }
        for dep in &self.deps {
            if !dep.features.windows(2).all(|w| w[0] < w[1]) {
                bail!("the features of dependency {} are not sorted", dep.name);
            }
        }
        let lists = [&self.tests, &self.benches, &self.examples];
        let paths = lists.iter().flat_map(|list| list.iter()).chain(&self.build);
        for path in paths {
            if path.contains('\\') {
                bail!("target path {path} has a backslash");
            }
        }
        if lists
            .iter()
            .any(|list| !list.windows(2).all(|w| w[0] < w[1]))
        {
            bail!("target paths are not sorted and unique");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write(dir: &Path, rel: &str, text: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|i| (*i).to_owned()).collect()
    }

    #[test]
    fn facts_read_features_dependencies_and_target_roots() {
        let tmp = tempfile::tempdir().unwrap();
        let root = manifest::normalize(tmp.path()).unwrap();
        write(
            &root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"app\"]\n[workspace.package]\npublish = false\n\
             [workspace.dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\n\
             real = { package = \"real-name\", version = \"1\" }\n",
        );
        write(
            &root,
            "app/Cargo.toml",
            "[package]\nname = \"app\"\npublish.workspace = true\nautobenches = false\n\
             build = \"tools/build.rs\"\n\
             [features]\ndefault = [\"fast\"]\nfast = []\nwire = [\"dep:json\"]\n\
             [dependencies]\nserde = { workspace = true, features = [\"rc\"] }\n\
             json = { version = \"1\", optional = true }\nlog = { version = \"0.4\", optional = true }\n\
             real = { workspace = true }\n\
             [dev-dependencies]\nserde = \"1\"\n\
             [target.'cfg(unix)'.build-dependencies]\ncc = \"1\"\n\
             [[test]]\nname = \"it\"\npath = \"tests/it/main.rs\"\n\
             [[example]]\nname = \"demo\"\n",
        );
        for rel in [
            "tests/smoke.rs",
            "tests/it/main.rs",
            "benches/speed.rs",
            "examples/demo.rs",
            "examples/multi/main.rs",
        ] {
            write(&root.join("app"), rel, "");
        }
        let facts = read(&root.join("app")).unwrap();
        facts.validate().unwrap();
        assert_eq!(facts.name.as_deref(), Some("app"));
        assert!(!facts.publish_allowed, "the workspace forbids it");
        let feature = |name: &str, enables: &[&str], implicit| Feature {
            name: name.to_owned(),
            enables: owned(enables),
            implicit,
        };
        assert_eq!(
            facts.features,
            [
                feature("default", &["fast"], false),
                feature("fast", &[], false),
                feature("log", &["dep:log"], true),
                feature("wire", &["dep:json"], false),
            ]
        );
        let dep = |name: &str, kind| DepEntry {
            name: name.to_owned(),
            kind,
            target: None,
            optional: false,
            package: None,
            workspace: false,
            features: Vec::new(),
        };
        assert_eq!(
            facts.deps,
            [
                DepEntry {
                    target: Some("cfg(unix)".to_owned()),
                    ..dep("cc", DepKind::Build)
                },
                DepEntry {
                    optional: true,
                    ..dep("json", DepKind::Normal)
                },
                DepEntry {
                    optional: true,
                    ..dep("log", DepKind::Normal)
                },
                DepEntry {
                    package: Some("real-name".to_owned()),
                    workspace: true,
                    ..dep("real", DepKind::Normal)
                },
                DepEntry {
                    workspace: true,
                    features: owned(&["derive", "rc"]),
                    ..dep("serde", DepKind::Normal)
                },
                dep("serde", DepKind::Dev),
            ]
        );
        assert_eq!(facts.build.as_deref(), Some("tools/build.rs"));
        assert_eq!(facts.tests, ["tests/it/main.rs", "tests/smoke.rs"]);
        assert!(facts.benches.is_empty(), "autobenches is off");
        assert_eq!(
            facts.examples,
            ["examples/demo.rs", "examples/multi/main.rs"]
        );
    }

    #[test]
    fn publish_allows_unless_false_or_no_registry_and_defaults_apply() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = manifest::normalize(tmp.path()).unwrap();
        let publish = |text: &str| {
            write(
                &dir,
                "Cargo.toml",
                &format!("[package]\nname = \"p\"\n{text}"),
            );
            read(&dir).unwrap().publish_allowed
        };
        assert!(publish(""));
        assert!(publish("publish = [\"registry\"]\n"));
        assert!(!publish("publish = []\n"));
        assert!(!publish("publish = false\n"));
        write(&dir, "build.rs", "");
        let facts = read(&dir).unwrap();
        assert_eq!(facts.build.as_deref(), Some("build.rs"));
        let none = read(&dir.join("missing")).unwrap();
        assert_eq!(none, ManifestFacts::default());
        none.validate().unwrap();
    }

    #[test]
    fn validate_rejects_broken_facts() {
        let mut facts = ManifestFacts {
            name: None,
            publish_allowed: true,
            ..ManifestFacts::default()
        };
        assert!(facts.validate().is_err());
        facts.publish_allowed = false;
        facts.tests = owned(&["tests\\a.rs"]);
        assert!(facts.validate().is_err());
        facts.tests = owned(&["tests/b.rs", "tests/a.rs"]);
        assert!(facts.validate().is_err());
    }
}
