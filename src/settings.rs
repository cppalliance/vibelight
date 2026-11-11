//! Settles what a run covers and where it writes, from the command line
//! and `vibelight.toml`. The markdown has no options; only the config's
//! boilerplate and quiet entries tune its leads.

use std::path::{Path, PathBuf};

use crate::config::{self, Config, Crates};
use crate::exclude::Exclude;
use crate::inputs::Selection;
use crate::roles::Chosen;

/// Where the HTML is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) dir: PathBuf,
}

/// The folder of the markdown outlines and their index, below the base
/// folder.
const MARKDOWN: &str = "md";

/// What a run covers and where it writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    /// `None` when neither the command line nor the config names a crate.
    pub(crate) selection: Option<Selection>,
    pub(crate) follow: Vec<String>,
    pub(crate) exclude: Exclude,
    /// The base folder: the HTML file goes in it, and the markdown outlines
    /// and their index go in its `md` folder.
    out: PathBuf,
    /// True when the folder is vibelight's own rather than one `-o` names,
    /// so the run keeps a `.gitignore` in it.
    pub(crate) owned: bool,
    /// The config's `boilerplate` texts, whose doc passages the doc
    /// detectors leave unread.
    pub(crate) boilerplate: Vec<String>,
    /// The config's `quiet` entries, whose leads the markdown leaves out.
    pub(crate) quiet: Vec<String>,
    /// The config's `facades` globs, which choose the HTML's facades when
    /// set.
    pub(crate) facades: Option<Chosen>,
}

impl Plan {
    /// The base folder, which holds the HTML file, the folder of the
    /// markdown, and, when it is vibelight's own, its `.gitignore`.
    pub(crate) fn base(&self) -> &Path {
        &self.out
    }

    /// The folder of the markdown outlines and their index: `md` in the
    /// base folder.
    pub(crate) fn markdown_dir(&self) -> PathBuf {
        self.out.join(MARKDOWN)
    }

    /// Where the HTML file goes: the base folder itself.
    pub(crate) fn html(&self) -> Output {
        Output {
            dir: self.out.clone(),
        }
    }
}

/// Settles a run. Crate paths replace the config's crate choice, and the
/// config's follow and exclude globs, boilerplate, and quiet entries apply
/// either way. The folder is the one `out` names, then the config's, then
/// `.vibelight` in the current directory.
pub(crate) fn plan(paths: &[PathBuf], out: Option<&Path>, config: Option<&Config>) -> Plan {
    let selection = if paths.is_empty() {
        config.and_then(|config| match &config.crates {
            Crates::Unset => None,
            Crates::Workspace(dir) => Some(Selection::Workspace(dir.clone())),
            Crates::Listed { paths, globbed } => Some(Selection::Listed {
                paths: paths.clone(),
                globbed: globbed.clone(),
            }),
        })
    } else {
        Some(Selection::Named(paths.to_vec()))
    };
    let exclude = config.map_or_else(Exclude::default, |config| Exclude {
        globs: config.exclude.clone(),
        from: config.dir.clone(),
    });
    let (out, owned) = match (out, config) {
        (Some(out), _) => (out.to_path_buf(), false),
        (None, Some(config)) => (config.out.clone(), true),
        (None, None) => (PathBuf::from(config::OUT), true),
    };
    Plan {
        selection,
        follow: config.map(|c| c.follow.clone()).unwrap_or_default(),
        exclude,
        out,
        owned,
        boilerplate: config.map(|c| c.boilerplate.clone()).unwrap_or_default(),
        quiet: config.map(|c| c.quiet.clone()).unwrap_or_default(),
        facades: config.and_then(|c| {
            c.facades.clone().map(|globs| Chosen {
                globs,
                from: c.dir.clone(),
            })
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            dir: PathBuf::from("/repo"),
            crates: Crates::Workspace(PathBuf::from("/repo")),
            follow: vec!["**".to_owned()],
            exclude: vec!["crates/build-*".to_owned()],
            out: PathBuf::from("/repo/.vibelight"),
            boilerplate: vec!["Enforced by xtask.".to_owned()],
            quiet: vec!["forwarder".to_owned()],
            facades: None,
        }
    }

    #[test]
    fn with_nothing_named_there_is_no_selection_and_the_folder_is_vibelights_own() {
        let plan = plan(&[], None, None);
        assert_eq!(plan.selection, None);
        assert_eq!(plan.out, PathBuf::from(".vibelight"));
        assert!(plan.owned);
        assert_eq!(plan.html().dir, Path::new(".vibelight"));
        assert_eq!(plan.markdown_dir(), Path::new(".vibelight").join("md"));
    }

    #[test]
    fn the_config_chooses_the_crates_the_globs_and_the_folder() {
        let plan = plan(&[], None, Some(&config()));
        assert_eq!(
            plan.selection,
            Some(Selection::Workspace(PathBuf::from("/repo")))
        );
        assert_eq!(plan.follow, ["**"]);
        assert_eq!(
            plan.exclude,
            Exclude {
                globs: vec!["crates/build-*".to_owned()],
                from: PathBuf::from("/repo"),
            }
        );
        assert_eq!(plan.out, PathBuf::from("/repo/.vibelight"));
        assert!(plan.owned);
        assert_eq!(plan.boilerplate, ["Enforced by xtask."]);
        assert_eq!(plan.quiet, ["forwarder"]);
    }

    #[test]
    fn crate_paths_replace_only_the_crates_of_the_config() {
        let paths = [PathBuf::from("crates/a")];
        let plan = plan(&paths, None, Some(&config()));
        assert_eq!(plan.selection, Some(Selection::Named(paths.to_vec())));
        assert_eq!(plan.follow, ["**"]);
        assert_eq!(plan.exclude.globs, ["crates/build-*"]);
        assert_eq!(plan.out, PathBuf::from("/repo/.vibelight"));
        assert_eq!(plan.quiet, ["forwarder"]);
    }

    #[test]
    fn a_folder_named_on_the_command_line_wins_and_is_not_vibelights_own() {
        let plan = plan(&[], Some(Path::new("elsewhere")), Some(&config()));
        assert_eq!(plan.out, PathBuf::from("elsewhere"));
        assert!(!plan.owned);
        assert_eq!(plan.base(), Path::new("elsewhere"));
        assert_eq!(plan.markdown_dir(), Path::new("elsewhere").join("md"));
    }
}
