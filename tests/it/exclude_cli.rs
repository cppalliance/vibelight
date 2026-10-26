//! The exclude setting from the outside: the flag beside the config, the
//! folder a config's globs are measured from, globs that match nothing,
//! config errors, and `--check` reporting what an excluded crate wrote.

use std::fs;
use std::path::PathBuf;

use crate::config::{REPOSITORY, garden, lines, vibelight_in};
use crate::support::{fixtures, read_outputs};

const NOTE: &str = "note: skipped testonly: it has only tests, examples, or benches";
const FLAT: &str = "--no-source-links";

fn names(files: &[(String, String)]) -> Vec<&str> {
    files.iter().map(|(name, _)| name.as_str()).collect()
}

/// A path below the repository, written the way the check prints it.
fn shown(parts: &[&str]) -> String {
    parts.iter().collect::<PathBuf>().display().to_string()
}

#[test]
fn a_glob_that_matches_no_crate_is_a_note_and_removes_nothing() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facad\", \"crates/zeta\"]\n");
    let result = vibelight_in(&repo, &["-o", "out", FLAT]);
    assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        [
            "note: exclude pattern crates/facad matches no crate",
            "note: exclude pattern crates/zeta matches no crate",
            NOTE,
        ]
    );
    let goldens = fixtures()
        .join("workspaces")
        .join("garden")
        .join("expected");
    assert!(
        read_outputs(&repo.join("out")) == read_outputs(&goldens),
        "a mistyped glob changed a file"
    );
}

#[test]
fn the_flag_replaces_the_config_list_and_crate_arguments_drop_it() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facade\"]\n");
    let result = vibelight_in(&repo, &["--exclude", "crates/beta", "-o", "out", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(lines(&result.stderr), [NOTE]);
    assert_eq!(
        names(&read_outputs(&repo.join("out"))),
        ["alpha.html", "alpha.md", "facade.html", "facade.md"]
    );
    let result = vibelight_in(&repo, &["crates/facade", "-o", "one", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(result.stderr.is_empty(), "{:?}", lines(&result.stderr));
    assert_eq!(
        names(&read_outputs(&repo.join("one"))),
        ["facade.html", "facade.md"]
    );
    let result = vibelight_in(&repo, &["--workspace", "-o", "all", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(read_outputs(&repo.join("all")).len(), 6);
}

#[test]
fn the_flag_repeats_takes_every_glob_character_and_works_with_the_workspace_switch() {
    let (_tmp, repo) = garden("");
    let args = [
        "--workspace",
        "--exclude",
        "crates/?eta",
        "--exclude",
        "**/f*",
        "-o",
        "out",
        FLAT,
    ];
    let result = vibelight_in(&repo, &args);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(lines(&result.stderr), [NOTE]);
    assert_eq!(
        names(&read_outputs(&repo.join("out"))),
        ["alpha.html", "alpha.md"]
    );
}

#[test]
fn excluding_every_crate_is_the_empty_run_error() {
    let (_tmp, repo) = garden("");
    let result = vibelight_in(
        &repo,
        &["--workspace", "--exclude", "crates/*", "-o", "out"],
    );
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stderr),
        ["error: no crate of the run has anything to outline"]
    );
    assert!(!repo.join("out").exists());
}

#[test]
fn a_single_rs_file_has_no_crate_folder_to_exclude_from() {
    let (tmp, repo) = garden("");
    let file = fixtures().join("single").join("tool.rs");
    let out = tmp.path().join("out");
    let args = [file.to_str().unwrap(), "--exclude", "x", "-o"];
    let result = vibelight_in(&repo, &[&args[..], &[out.to_str().unwrap()]].concat());
    assert_eq!(result.status.code(), Some(1));
    let stderr = lines(&result.stderr).join("\n");
    assert!(
        stderr.contains("is a single file, which has no crate folder for an exclude glob to match"),
        "{stderr}"
    );
}

#[test]
fn the_globs_of_a_config_are_measured_from_its_own_folder() {
    let (_tmp, repo) = garden("");
    let crates = repo.join("crates");
    fs::write(
        crates.join("vibelight.toml"),
        "workspace = \"..\"\nexclude = [\"beta\", \"crates/alpha\"]\n",
    )
    .unwrap();
    let out = repo.join("out");
    let result = vibelight_in(&crates, &["-o", out.to_str().unwrap(), FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        ["note: exclude pattern crates/alpha matches no crate", NOTE]
    );
    assert_eq!(
        names(&read_outputs(&out)),
        ["alpha.html", "alpha.md", "facade.html", "facade.md"]
    );
}

#[test]
fn a_config_that_gives_exclude_the_wrong_type_is_an_error_that_names_the_key() {
    let (_tmp, repo) = garden("workspace = true\nexclude = \"crates/*\"\n");
    let result = vibelight_in(&repo, &[]);
    assert_eq!(result.status.code(), Some(1));
    let stderr = lines(&result.stderr);
    assert_eq!(stderr.len(), 1, "{stderr:?}");
    assert!(
        stderr[0].ends_with("vibelight.toml: `exclude` is a list of strings"),
        "{stderr:?}"
    );
}

#[test]
fn the_check_reports_the_markdown_of_an_excluded_crate_as_stale() {
    let (_tmp, repo) = garden(REPOSITORY);
    assert!(vibelight_in(&repo, &[]).status.success());
    let stale = [
        format!("stale: {}", shown(&["vibelight", "facade.md"])),
        "check failed: 1 problem".to_owned(),
    ];
    let result = vibelight_in(&repo, &["--exclude", "crates/facade", "--check"]);
    assert_eq!(result.status.code(), Some(1), "{:?}", lines(&result.stdout));
    assert_eq!(lines(&result.stdout), stale);
    fs::write(
        repo.join("vibelight.toml"),
        format!("exclude = [\"crates/facade\"]\n{REPOSITORY}"),
    )
    .unwrap();
    let result = vibelight_in(&repo, &["--check"]);
    assert_eq!(result.status.code(), Some(1), "{:?}", lines(&result.stdout));
    assert_eq!(lines(&result.stdout), stale);
    assert_eq!(lines(&result.stderr), [NOTE]);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [
            NOTE.to_owned(),
            format!(
                "warning: {} is stale: no crate of this run writes it",
                shown(&["vibelight", "facade.md"])
            ),
            "1 warning".to_owned(),
        ]
    );
    assert!(repo.join("vibelight").join("facade.md").is_file());
    assert!(
        repo.join("vibelight")
            .join("html")
            .join("facade.html")
            .is_file()
    );
}
