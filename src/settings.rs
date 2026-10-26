//! Settles what a run covers, where each format goes, and the options that
//! shape every file, from the command line and `vibelight.toml`: a flag
//! wins over the config, and the config wins over the defaults.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::config::{Config, Crates, Format};
use crate::exclude::Exclude;
use crate::inputs::Selection;

/// The command-line values that settle a run.
#[derive(Debug, Default, Clone)]
pub(crate) struct Flags {
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) workspace: bool,
    pub(crate) follow: Vec<String>,
    pub(crate) exclude: Vec<String>,
    /// The folder for both formats.
    pub(crate) out: Option<PathBuf>,
    pub(crate) md_out: Option<PathBuf>,
    pub(crate) html_out: Option<PathBuf>,
    /// True turns the source links of both formats off.
    pub(crate) no_source_links: bool,
    pub(crate) md_source_links: Option<bool>,
    pub(crate) html_source_links: Option<bool>,
    pub(crate) kill_list: Option<KillList>,
    /// True turns the facts on.
    pub(crate) facts: bool,
    pub(crate) order: Option<OrderBy>,
    pub(crate) members: Option<MembersBy>,
    pub(crate) profile: Option<Profile>,
}

/// How a type's block orders its functions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum MembersBy {
    /// By name, whatever impl holds them.
    #[default]
    Name,
    /// By inherent impl, in source order, so the output follows the files.
    Impl,
}

/// A set of defaults for one kind of reader; a switch or a config key
/// still wins over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum Profile {
    /// For analyzing an API: source links off, only the kill list's counts,
    /// the facts on, and the module order.
    Analysis,
}

/// How a file orders its declarations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum OrderBy {
    /// By level, then a Modules section.
    #[default]
    Levels,
    /// In one section per module, with an index by level.
    Modules,
}

/// How much of the macro kill list a file shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum KillList {
    /// Every entry, after the counts.
    #[default]
    Full,
    /// Only the counts.
    Counts,
    /// Neither the counts nor the entries.
    Off,
}

/// The settings that shape every file of a run, beside each format's
/// folder and source links.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Options {
    pub(crate) kill_list: KillList,
    /// The `Counts:` and `Used by:` lines and the Same shapes section.
    pub(crate) facts: bool,
    pub(crate) order: OrderBy,
    pub(crate) members: MembersBy,
}

/// Where one format is written, and whether it carries source links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) dir: PathBuf,
    pub(crate) links: bool,
}

/// What a run covers and writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    /// `None` when neither the command line nor the config names a crate.
    pub(crate) selection: Option<Selection>,
    pub(crate) follow: Vec<String>,
    pub(crate) exclude: Exclude,
    /// `None` for a format the config turns off.
    pub(crate) markdown: Option<Output>,
    pub(crate) html: Option<Output>,
    pub(crate) options: Options,
}

/// Settles a run. Crate paths or the workspace switch choose the crates,
/// and the config's `workspace`, `crates`, `follow`, and `exclude` are then
/// ignored. Otherwise the config chooses the crates, and follow globs or
/// exclude globs on the command line replace the config's. Each format's
/// folder and source links come
/// from that format's own flag, then the flag for both formats, then the
/// config, then the defaults: the config's folder, or the current
/// directory without a config, with source links on. A format the config
/// turns off is not written, unless its own folder flag names a folder for
/// it.
///
/// # Errors
/// Returns an error when the workspace switch has more than one path, or
/// when the config turns both formats off.
pub(crate) fn plan(flags: &Flags, config: Option<&Config>) -> Result<Plan> {
    let (selection, follow, exclude) = if !flags.paths.is_empty() || flags.workspace {
        let selection = if flags.workspace {
            if flags.paths.len() > 1 {
                bail!("the workspace switch takes at most one path, the directory to start from");
            }
            let start = flags.paths.first().cloned();
            Selection::Workspace(start.unwrap_or_else(|| PathBuf::from(".")))
        } else {
            Selection::Paths {
                paths: flags.paths.clone(),
                globbed: Vec::new(),
            }
        };
        let exclude = Exclude {
            globs: flags.exclude.clone(),
            from: None,
        };
        (Some(selection), flags.follow.clone(), exclude)
    } else {
        let selection = config.and_then(|config| match &config.crates {
            Crates::Unset => None,
            Crates::Workspace(dir) => Some(Selection::Workspace(dir.clone())),
            Crates::Listed { paths, globbed } => Some(Selection::Paths {
                paths: paths.clone(),
                globbed: globbed.clone(),
            }),
        });
        let follow = if flags.follow.is_empty() {
            config.map(|c| c.follow.clone()).unwrap_or_default()
        } else {
            flags.follow.clone()
        };
        let exclude = match config {
            Some(config) if flags.exclude.is_empty() => Exclude {
                globs: config.exclude.clone(),
                from: Some(config.dir.clone()),
            },
            _ => Exclude {
                globs: flags.exclude.clone(),
                from: None,
            },
        };
        (selection, follow, exclude)
    };
    let analysis = flags.profile.or(config.and_then(|c| c.profile)) == Some(Profile::Analysis);
    let unset = Format::default();
    let home = config.map(|c| c.dir.as_path());
    let markdown = output(
        config.map_or(&unset, |c| &c.markdown),
        home,
        (flags.md_out.as_ref(), flags.md_source_links),
        flags,
        !analysis,
    );
    let html = output(
        config.map_or(&unset, |c| &c.html),
        home,
        (flags.html_out.as_ref(), flags.html_source_links),
        flags,
        !analysis,
    );
    if markdown.is_none() && html.is_none() {
        bail!("the config turns both formats off, so there is nothing to write");
    }
    let (kill_list, order) = if analysis {
        (KillList::Counts, OrderBy::Modules)
    } else {
        (KillList::Full, OrderBy::Levels)
    };
    let options = Options {
        kill_list: flags
            .kill_list
            .or(config.and_then(|c| c.kill_list))
            .unwrap_or(kill_list),
        facts: flags.facts || config.and_then(|c| c.facts).unwrap_or(analysis),
        order: flags
            .order
            .or(config.and_then(|c| c.order))
            .unwrap_or(order),
        members: flags
            .members
            .or(config.and_then(|c| c.members))
            .unwrap_or_default(),
    };
    Ok(Plan {
        selection,
        follow,
        exclude,
        markdown,
        html,
        options,
    })
}

/// Settles one format from its own folder and link flags, the flags for
/// both formats, the config, and `links_by_default`, which a profile can
/// turn off.
fn output(
    config: &Format,
    home: Option<&Path>,
    (out, links): (Option<&PathBuf>, Option<bool>),
    flags: &Flags,
    links_by_default: bool,
) -> Option<Output> {
    if config.enabled == Some(false) && out.is_none() {
        return None;
    }
    let dir = out
        .or(flags.out.as_ref())
        .or(config.out.as_ref())
        .cloned()
        .or_else(|| home.map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let links = links
        .or(flags.no_source_links.then_some(false))
        .or(config.source_links)
        .unwrap_or(links_by_default);
    Some(Output { dir, links })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            dir: PathBuf::from("/repo"),
            crates: Crates::Workspace(PathBuf::from("/repo")),
            follow: vec!["vendor/*".to_owned()],
            exclude: vec!["crates/build-*".to_owned()],
            markdown: Format {
                out: Some(PathBuf::from("/repo/vibelight")),
                source_links: Some(false),
                enabled: None,
            },
            html: Format {
                out: Some(PathBuf::from("/repo/vibelight/html")),
                source_links: None,
                enabled: None,
            },
            kill_list: Some(KillList::Counts),
            facts: Some(true),
            order: Some(OrderBy::Modules),
            members: None,
            profile: None,
        }
    }

    fn output(dir: &str, links: bool) -> Output {
        Output {
            dir: PathBuf::from(dir),
            links,
        }
    }

    #[test]
    fn with_nothing_set_both_formats_go_to_the_current_directory_with_links() {
        let plan = plan(&Flags::default(), None).unwrap();
        assert_eq!(plan.selection, None);
        assert_eq!(plan.markdown, Some(output(".", true)));
        assert_eq!(plan.html, Some(output(".", true)));
    }

    #[test]
    fn the_config_chooses_crates_folders_and_links_when_no_flag_does() {
        let plan = plan(&Flags::default(), Some(&config())).unwrap();
        assert_eq!(
            plan.selection,
            Some(Selection::Workspace(PathBuf::from("/repo")))
        );
        assert_eq!(plan.follow, ["vendor/*"]);
        assert_eq!(plan.markdown, Some(output("/repo/vibelight", false)));
        assert_eq!(plan.html, Some(output("/repo/vibelight/html", true)));
    }

    #[test]
    fn the_kill_list_comes_from_its_switch_then_the_config_then_full() {
        assert_eq!(
            plan(&Flags::default(), None).unwrap().options.kill_list,
            KillList::Full
        );
        let from_config = plan(&Flags::default(), Some(&config())).unwrap();
        assert_eq!(from_config.options.kill_list, KillList::Counts);
        let flags = Flags {
            kill_list: Some(KillList::Off),
            ..Flags::default()
        };
        let from_switch = plan(&flags, Some(&config())).unwrap();
        assert_eq!(from_switch.options.kill_list, KillList::Off);
    }

    #[test]
    fn the_facts_come_from_their_switch_or_the_config_and_are_off_by_default() {
        assert!(!plan(&Flags::default(), None).unwrap().options.facts);
        assert!(
            plan(&Flags::default(), Some(&config()))
                .unwrap()
                .options
                .facts
        );
        let flags = Flags {
            facts: true,
            ..Flags::default()
        };
        assert!(plan(&flags, None).unwrap().options.facts);
    }

    #[test]
    fn the_order_comes_from_its_switch_then_the_config_then_levels() {
        let config = config();
        let order = |flags: &Flags, config| plan(flags, config).unwrap().options.order;
        assert_eq!(order(&Flags::default(), None), OrderBy::Levels);
        assert_eq!(order(&Flags::default(), Some(&config)), OrderBy::Modules);
        let flags = Flags {
            order: Some(OrderBy::Levels),
            ..Flags::default()
        };
        assert_eq!(order(&flags, Some(&config)), OrderBy::Levels);
    }

    #[test]
    fn the_analysis_profile_changes_only_the_defaults() {
        let flags = Flags {
            profile: Some(Profile::Analysis),
            ..Flags::default()
        };
        let plan = plan(&flags, None).unwrap();
        assert_eq!(plan.markdown, Some(output(".", false)));
        assert_eq!(plan.html, Some(output(".", false)));
        assert_eq!(
            plan.options,
            Options {
                kill_list: KillList::Counts,
                facts: true,
                order: OrderBy::Modules,
                members: MembersBy::Name,
            }
        );
        let mut config = config();
        config.profile = Some(Profile::Analysis);
        config.kill_list = None;
        config.order = None;
        config.facts = Some(false);
        let explicit = Flags {
            html_source_links: Some(true),
            order: Some(OrderBy::Levels),
            ..Flags::default()
        };
        let plan = super::plan(&explicit, Some(&config)).unwrap();
        assert_eq!(plan.markdown, Some(output("/repo/vibelight", false)));
        assert_eq!(plan.html, Some(output("/repo/vibelight/html", true)));
        assert_eq!(
            plan.options,
            Options {
                kill_list: KillList::Counts,
                facts: false,
                order: OrderBy::Levels,
                members: MembersBy::Name,
            }
        );
    }

    #[test]
    fn a_config_without_folders_writes_both_formats_beside_itself() {
        let mut config = config();
        config.markdown.out = None;
        config.html.out = None;
        let plan = plan(&Flags::default(), Some(&config)).unwrap();
        assert_eq!(plan.markdown, Some(output("/repo", false)));
        assert_eq!(plan.html, Some(output("/repo", true)));
    }

    #[test]
    fn crate_arguments_replace_the_config_crates_but_keep_its_outputs() {
        let flags = Flags {
            paths: vec![PathBuf::from("crates/a")],
            ..Flags::default()
        };
        let plan = plan(&flags, Some(&config())).unwrap();
        assert_eq!(
            plan.selection,
            Some(Selection::Paths {
                paths: vec![PathBuf::from("crates/a")],
                globbed: Vec::new(),
            })
        );
        assert!(plan.follow.is_empty(), "the config's follow globs go too");
        assert_eq!(plan.markdown, Some(output("/repo/vibelight", false)));
    }

    fn exclude(globs: &[&str], from: Option<&str>) -> Exclude {
        Exclude {
            globs: globs.iter().map(|g| (*g).to_owned()).collect(),
            from: from.map(PathBuf::from),
        }
    }

    #[test]
    fn the_exclude_globs_come_from_their_flag_then_the_config() {
        let from_config = plan(&Flags::default(), Some(&config())).unwrap();
        assert_eq!(
            from_config.exclude,
            exclude(&["crates/build-*"], Some("/repo"))
        );
        let flags = Flags {
            exclude: vec!["tools/*".to_owned()],
            ..Flags::default()
        };
        let from_flag = plan(&flags, Some(&config())).unwrap();
        assert_eq!(from_flag.exclude, exclude(&["tools/*"], None));
        assert_eq!(from_flag.selection, from_config.selection);
        let without_config = plan(&flags, None).unwrap();
        assert_eq!(without_config.exclude, exclude(&["tools/*"], None));
    }

    #[test]
    fn crate_arguments_drop_the_config_exclude_globs_unless_the_flag_is_given() {
        let paths = Flags {
            paths: vec![PathBuf::from("crates/a")],
            ..Flags::default()
        };
        assert_eq!(
            plan(&paths, Some(&config())).unwrap().exclude,
            Exclude::default()
        );
        let workspace = Flags {
            workspace: true,
            exclude: vec!["x".to_owned()],
            ..Flags::default()
        };
        assert_eq!(
            plan(&workspace, Some(&config())).unwrap().exclude,
            exclude(&["x"], None)
        );
    }

    #[test]
    fn format_flags_win_over_the_flags_for_both_and_those_win_over_the_config() {
        let flags = Flags {
            out: Some(PathBuf::from("both")),
            html_out: Some(PathBuf::from("pages")),
            no_source_links: true,
            md_source_links: Some(true),
            follow: vec!["extra/*".to_owned()],
            ..Flags::default()
        };
        let plan = plan(&flags, Some(&config())).unwrap();
        assert_eq!(plan.follow, ["extra/*"]);
        assert_eq!(plan.markdown, Some(output("both", true)));
        assert_eq!(plan.html, Some(output("pages", false)));
    }

    #[test]
    fn a_format_the_config_turns_off_is_written_only_when_its_own_folder_flag_names_one() {
        let mut config = config();
        config.html.enabled = Some(false);
        let both = Flags {
            out: Some(PathBuf::from("both")),
            ..Flags::default()
        };
        assert_eq!(plan(&both, Some(&config)).unwrap().html, None);
        let own = Flags {
            html_out: Some(PathBuf::from("pages")),
            ..Flags::default()
        };
        assert_eq!(
            plan(&own, Some(&config)).unwrap().html,
            Some(output("pages", true))
        );
        config.markdown.enabled = Some(false);
        let error = super::plan(&Flags::default(), Some(&config)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the config turns both formats off, so there is nothing to write"
        );
    }
}
