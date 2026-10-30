//! Settles what a run covers and where it writes, from the command line
//! and `vibelight.toml`, and fixes the options that shape the HTML, which
//! is for a person browsing the code. The markdown, for a model analyzing
//! it, has no options; only the config's boilerplate and quiet entries
//! tune its leads.

use std::path::{Path, PathBuf};

use crate::config::{self, Config, Crates};
use crate::exclude::Exclude;
use crate::inputs::Selection;

/// How a type's block orders its functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MembersBy {
    /// By name, whatever impl holds them.
    Name,
}

/// How a file orders its declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderBy {
    /// In groups of types, each named for a module, then a Modules section.
    Groups,
}

/// How much of the macro kill list a file shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KillList {
    /// Every entry, after the counts.
    Full,
}

/// How a file spells its declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Notation {
    /// As a C++ header, for a person who reads C++.
    Cpp,
}

/// The settings that shape the HTML files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Options {
    pub(crate) notation: Notation,
    pub(crate) kill_list: KillList,
    pub(crate) order: OrderBy,
    pub(crate) members: MembersBy,
    /// The Legend section, which explains the notation.
    pub(crate) legend: bool,
}

/// The HTML holds what a person browses: the declarations, linked.
pub(crate) const HTML: Options = Options {
    notation: Notation::Cpp,
    kill_list: KillList::Full,
    order: OrderBy::Groups,
    members: MembersBy::Name,
    legend: true,
};

/// Where the HTML is written, and the options that shape it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) dir: PathBuf,
    pub(crate) options: Options,
}

/// What a run covers and where it writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    /// `None` when neither the command line nor the config names a crate.
    pub(crate) selection: Option<Selection>,
    pub(crate) follow: Vec<String>,
    pub(crate) exclude: Exclude,
    /// The folder of the markdown files and the index; the HTML files go in
    /// its `html` folder.
    pub(crate) out: PathBuf,
    /// True when the folder is vibelight's own rather than one `-o` names,
    /// so the run keeps a `.gitignore` in it.
    pub(crate) owned: bool,
    /// The config's `boilerplate` texts, whose doc passages the doc
    /// detectors leave unread.
    pub(crate) boilerplate: Vec<String>,
    /// The config's `quiet` entries, whose leads the markdown leaves out.
    pub(crate) quiet: Vec<String>,
}

impl Plan {
    /// The folder of the markdown outlines and the index.
    pub(crate) fn markdown_dir(&self) -> &Path {
        &self.out
    }

    pub(crate) fn html(&self) -> Output {
        Output {
            dir: self.out.join("html"),
            options: HTML,
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
        }
    }

    #[test]
    fn with_nothing_named_there_is_no_selection_and_the_folder_is_vibelights_own() {
        let plan = plan(&[], None, None);
        assert_eq!(plan.selection, None);
        assert_eq!(plan.out, PathBuf::from(".vibelight"));
        assert!(plan.owned);
        assert_eq!(plan.html().dir, Path::new(".vibelight").join("html"));
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
        assert_eq!(plan.markdown_dir(), Path::new("elsewhere"));
        assert_eq!(plan.html().options, HTML);
    }
}
