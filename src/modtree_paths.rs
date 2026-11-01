//! The paths a module tree walk tries: the directories a module's `mod`
//! items resolve against, the files a `mod name;` item may load, and
//! crate-relative path joins.

/// The directories a module's `mod` items resolve against: `child_dir` for
/// `mod name;` files, `path_base` for `#[path]` values.
pub(crate) struct Dirs {
    pub(crate) child_dir: String,
    pub(crate) path_base: String,
}

impl Dirs {
    /// Returns the directories of a module held in the file `rel`: its
    /// children live in the file's folder when the file is `mod.rs`-like,
    /// else in a folder named for the module.
    pub(crate) fn of_file(rel: &str, name: &str, mod_rs: bool) -> Self {
        let path_base = dir_of(rel);
        let child_dir = join(&path_base, if mod_rs { "" } else { name });
        Self {
            child_dir,
            path_base,
        }
    }

    /// Returns the directories of an inline module, which uses one
    /// directory for both: its `#[path]` value, or its name below the
    /// `child_dir` of the module that declares it.
    pub(crate) fn inline(&self, path_value: Option<&str>, name: &str) -> Self {
        let dir = match path_value {
            Some(value) => join(&self.path_base, value),
            None => join(&self.child_dir, name),
        };
        Self {
            child_dir: dir.clone(),
            path_base: dir,
        }
    }
}

/// A file a `mod name;` item may load, and whether it holds its children
/// the way a `mod.rs` does.
#[derive(Clone)]
pub(crate) struct Candidate {
    pub(crate) rel: String,
    pub(crate) mod_rs: bool,
}

/// Returns the files a `mod name;` item may load: the `#[path]` value, or
/// `name.rs`, then `name/mod.rs`.
pub(crate) fn candidates(path_value: Option<&str>, name: &str, dirs: &Dirs) -> Vec<Candidate> {
    let candidate = |rel, mod_rs| Candidate { rel, mod_rs };
    match path_value {
        Some(value) => vec![candidate(join(&dirs.path_base, value), true)],
        None => vec![
            candidate(join(&dirs.child_dir, &format!("{name}.rs")), false),
            candidate(join(&dirs.child_dir, &format!("{name}/mod.rs")), true),
        ],
    }
}

/// Returns the directory part of a crate-relative path, or `""`.
fn dir_of(rel: &str) -> String {
    rel.rfind('/')
        .map_or_else(String::new, |i| rel[..i].to_owned())
}

/// Joins crate-relative paths with forward slashes, resolving `.` and `..`
/// lexically.
fn join(base: &str, rel: &str) -> String {
    let mut parts: Vec<&str> = base.split('/').filter(|p| !p.is_empty()).collect();
    for part in rel.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_resolves_dots() {
        assert_eq!(join("src/a", "../b.rs"), "src/b.rs");
        assert_eq!(join("", "./x/y.rs"), "x/y.rs");
        assert_eq!(join("src", "..\\..\\up.rs"), "../up.rs");
    }
}
