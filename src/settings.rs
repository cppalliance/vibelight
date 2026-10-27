//! Settles what a run covers and where it writes, from the command line
//! and `vibelight.toml`, and fixes the options that shape each format: the
//! markdown is for a model analyzing the code, and the HTML is for a person
//! browsing it.

use std::path::{Path, PathBuf};

use crate::config::{self, Config, Crates};
use crate::exclude::Exclude;
use crate::inputs::Selection;

/// How a type's block orders its functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MembersBy {
    /// By name, whatever impl holds them.
    Name,
    /// By inherent impl, in source order, so the output follows the files.
    Impl,
}

/// How a file orders its declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderBy {
    /// In groups of types, each named for a module, then a Modules section.
    Groups,
    /// In one section per module, the types of each in dependency order.
    Modules,
}

/// How much of the macro kill list a file shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KillList {
    /// Every entry, after the counts.
    Full,
    /// Only the counts.
    Counts,
}

/// How a file spells its declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Notation {
    /// As a C++ header, for a person who reads C++.
    Cpp,
    /// As Rust with the bodies left out, for a model.
    Rust,
}

/// The settings that shape the files of one format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Options {
    pub(crate) notation: Notation,
    pub(crate) kill_list: KillList,
    /// The `Counts:`, `Used by:`, and `Fields:` lines and the External
    /// items and Same shapes sections.
    pub(crate) facts: bool,
    pub(crate) order: OrderBy,
    pub(crate) members: MembersBy,
    /// The Legend section, which explains the notation.
    pub(crate) legend: bool,
}

/// The markdown holds everything a model can find slop with.
pub(crate) const MARKDOWN: Options = Options {
    notation: Notation::Rust,
    kill_list: KillList::Counts,
    facts: true,
    order: OrderBy::Modules,
    members: MembersBy::Impl,
    legend: true,
};

/// The HTML holds what a person browses: the declarations, linked.
pub(crate) const HTML: Options = Options {
    notation: Notation::Cpp,
    kill_list: KillList::Full,
    facts: false,
    order: OrderBy::Groups,
    members: MembersBy::Name,
    legend: false,
};

/// Where one format is written, and the options that shape it.
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
}

impl Plan {
    pub(crate) fn markdown(&self) -> Output {
        Output {
            dir: self.out.clone(),
            options: MARKDOWN,
        }
    }

    pub(crate) fn html(&self) -> Output {
        Output {
            dir: self.out.join("html"),
            options: HTML,
        }
    }
}

/// Settles a run. Crate paths replace the config's crate choice, and the
/// config's follow and exclude globs apply either way. The folder is the
/// one `out` names, then the config's, then `.vibelight` in the current
/// directory.
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
    }

    #[test]
    fn crate_paths_replace_only_the_crates_of_the_config() {
        let paths = [PathBuf::from("crates/a")];
        let plan = plan(&paths, None, Some(&config()));
        assert_eq!(plan.selection, Some(Selection::Named(paths.to_vec())));
        assert_eq!(plan.follow, ["**"]);
        assert_eq!(plan.exclude.globs, ["crates/build-*"]);
        assert_eq!(plan.out, PathBuf::from("/repo/.vibelight"));
    }

    #[test]
    fn a_folder_named_on_the_command_line_wins_and_is_not_vibelights_own() {
        let plan = plan(&[], Some(Path::new("elsewhere")), Some(&config()));
        assert_eq!(plan.out, PathBuf::from("elsewhere"));
        assert!(!plan.owned);
        assert_eq!(plan.markdown().options, MARKDOWN);
        assert_eq!(plan.html().options, HTML);
    }
}
