//! Leaves crates out of a run: the globs that name them, and the filter
//! that tells which crate folders they match and which globs matched none.

use std::path::{Path, PathBuf};

use crate::glob;
use crate::inputs::relative;

/// The globs that leave crates out of a run, with the folder they are
/// measured from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Exclude {
    pub(crate) globs: Vec<String>,
    /// The folder the globs are measured from: the config's folder for the
    /// config's globs. `None` measures them from the base of the run, the
    /// way follow globs are.
    pub(crate) from: Option<PathBuf>,
}

/// An `Exclude` at work on one run. It remembers which globs matched a
/// crate folder.
pub(crate) struct Filter<'a> {
    exclude: &'a Exclude,
    base: &'a Path,
    matched: Vec<bool>,
}

impl<'a> Filter<'a> {
    /// Starts a filter. `base` is the directory of the run that globs
    /// without a folder of their own are measured from.
    pub(crate) fn new(exclude: &'a Exclude, base: &'a Path) -> Self {
        Self {
            exclude,
            base,
            matched: vec![false; exclude.globs.len()],
        }
    }

    /// Tells whether any glob matches the crate folder `dir`. Every glob
    /// that matches it counts as used, not only the first.
    pub(crate) fn excludes(&mut self, dir: &Path) -> bool {
        let from = self.exclude.from.as_deref().unwrap_or(self.base);
        let Some(rel) = relative(from, dir) else {
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

    fn globs(list: &[&str], from: Option<&str>) -> Exclude {
        Exclude {
            globs: list.iter().map(|g| (*g).to_owned()).collect(),
            from: from.map(PathBuf::from),
        }
    }

    #[test]
    fn globs_match_crate_folders_below_the_base_of_the_run() {
        let exclude = globs(
            &["crates/build-*", "crates/harness-internal/test-plugin"],
            None,
        );
        let base = Path::new("/repo");
        let mut filter = Filter::new(&exclude, base);
        assert!(filter.excludes(Path::new("/repo/crates/build-ui")));
        assert!(filter.excludes(Path::new("/repo/crates/harness-internal/test-plugin")));
        assert!(!filter.excludes(Path::new("/repo/crates/harness-internal/runner")));
        assert!(!filter.excludes(Path::new("/repo/crates/build-ui/extra")));
        assert!(!filter.excludes(Path::new("/elsewhere/crates/build-ui")));
        assert_eq!(filter.unmatched(), Vec::<String>::new());
    }

    #[test]
    fn the_globs_of_a_config_are_measured_from_the_folder_of_the_config() {
        let exclude = globs(&["beta", "crates/beta"], Some("/repo/crates"));
        let mut filter = Filter::new(&exclude, Path::new("/repo"));
        assert!(filter.excludes(Path::new("/repo/crates/beta")));
        assert!(!filter.excludes(Path::new("/repo/beta")));
        assert_eq!(
            filter.unmatched(),
            ["exclude pattern crates/beta matches no crate"]
        );
    }

    #[test]
    fn a_crate_that_two_globs_match_counts_for_both() {
        let exclude = globs(&["crates/*", "**/beta", "crates/zeta"], None);
        let mut filter = Filter::new(&exclude, Path::new("/repo"));
        assert!(filter.excludes(Path::new("/repo/crates/beta")));
        assert_eq!(
            filter.unmatched(),
            ["exclude pattern crates/zeta matches no crate"]
        );
    }

    #[test]
    fn without_globs_nothing_is_excluded_and_nothing_is_unmatched() {
        let exclude = Exclude::default();
        let mut filter = Filter::new(&exclude, Path::new("/repo"));
        assert!(!filter.excludes(Path::new("/repo/crates/beta")));
        assert_eq!(filter.unmatched(), Vec::<String>::new());
    }
}
