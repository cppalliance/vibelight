//! Leaves crates out of a run: the globs that name them, and the filter
//! that tells which crate folders they match and which globs matched none.

use std::path::{Path, PathBuf};

use crate::glob;
use crate::manifest::relative;

/// The globs that leave crates out of a run, with the folder they are
/// measured from: the config's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Exclude {
    pub(crate) globs: Vec<String>,
    pub(crate) from: PathBuf,
}

/// An `Exclude` at work on one run. It remembers which globs matched a
/// crate folder.
pub(crate) struct Filter<'a> {
    exclude: &'a Exclude,
    matched: Vec<bool>,
}

impl<'a> Filter<'a> {
    pub(crate) fn new(exclude: &'a Exclude) -> Self {
        Self {
            exclude,
            matched: vec![false; exclude.globs.len()],
        }
    }

    /// Tells whether any glob matches the crate folder `dir`. Every glob
    /// that matches it counts as used, not only the first.
    pub(crate) fn excludes(&mut self, dir: &Path) -> bool {
        let Some(rel) = relative(&self.exclude.from, dir) else {
            return false;
        };
        let mut found = false;
        for (glob, used) in self.exclude.globs.iter().zip(&mut self.matched) {
            if glob::matches(glob, &rel) {
                *used = true;
                found = true;
            }
        }
        found
    }

    /// One note for each glob that matched no crate folder, in the order
    /// the globs were given.
    pub(crate) fn unmatched(&self) -> Vec<String> {
        self.exclude
            .globs
            .iter()
            .zip(&self.matched)
            .filter(|(_, used)| !**used)
            .map(|(glob, _)| format!("exclude pattern {glob} matches no crate"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn globs(list: &[&str], from: &str) -> Exclude {
        Exclude {
            globs: list.iter().map(|g| (*g).to_owned()).collect(),
            from: PathBuf::from(from),
        }
    }

    #[test]
    fn globs_match_crate_folders_below_the_folder_of_the_config() {
        let exclude = globs(
            &["crates/build-*", "crates/harness-internal/test-plugin"],
            "/repo",
        );
        let mut filter = Filter::new(&exclude);
        assert!(filter.excludes(Path::new("/repo/crates/build-ui")));
        assert!(filter.excludes(Path::new("/repo/crates/harness-internal/test-plugin")));
        assert!(!filter.excludes(Path::new("/repo/crates/harness-internal/runner")));
        assert!(!filter.excludes(Path::new("/repo/crates/build-ui/extra")));
        assert!(!filter.excludes(Path::new("/elsewhere/crates/build-ui")));
        assert_eq!(filter.unmatched(), Vec::<String>::new());
    }

    #[test]
    fn a_config_below_the_root_measures_from_its_own_folder() {
        let exclude = globs(&["beta", "crates/beta"], "/repo/crates");
        let mut filter = Filter::new(&exclude);
        assert!(filter.excludes(Path::new("/repo/crates/beta")));
        assert!(!filter.excludes(Path::new("/repo/beta")));
        assert_eq!(
            filter.unmatched(),
            ["exclude pattern crates/beta matches no crate"]
        );
    }

    #[test]
    fn a_crate_that_two_globs_match_counts_for_both() {
        let exclude = globs(&["crates/*", "**/beta", "crates/zeta"], "/repo");
        let mut filter = Filter::new(&exclude);
        assert!(filter.excludes(Path::new("/repo/crates/beta")));
        assert_eq!(
            filter.unmatched(),
            ["exclude pattern crates/zeta matches no crate"]
        );
    }

    #[test]
    fn without_globs_nothing_is_excluded_and_nothing_is_unmatched() {
        let exclude = Exclude::default();
        let mut filter = Filter::new(&exclude);
        assert!(!filter.excludes(Path::new("/repo/crates/beta")));
        assert_eq!(filter.unmatched(), Vec::<String>::new());
    }
}
