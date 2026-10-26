//! Runs with `vibelight.toml`: the config chooses crates and folders, crate
//! arguments replace only its crates, each format flag wins over the flag
//! for both formats and over the config, and config errors name the key.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use crate::support::{copy_dir, fixtures, has_ext, read_outputs, strip_html_links, vibelight};

/// A copy of the garden workspace, with `config` as its `vibelight.toml`
/// when it is not empty.
pub(crate) fn garden(config: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let repo = tmp.path().join("repo");
    copy_dir(&fixtures().join("workspaces").join("garden"), &repo);
    if !config.is_empty() {
        fs::write(repo.join("vibelight.toml"), config).expect("the config can be written");
    }
    (tmp, repo)
}

/// The config of a repository that commits its markdown without links and
/// keeps its HTML, with links, in a folder below it.
pub(crate) const REPOSITORY: &str = "workspace = true\n\n\
    [markdown]\nout = \"vibelight\"\nsource_links = false\n\n\
    [html]\nout = \"vibelight/html\"\n";

/// Runs the binary in `cwd` with `args`.
pub(crate) fn vibelight_in(cwd: &Path, args: &[&str]) -> Output {
    vibelight()
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("the vibelight binary runs")
}

pub(crate) fn lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8(bytes.to_vec())
        .expect("the output is UTF-8")
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The goldens of one format, by file name.
fn goldens(ext: &str) -> Vec<(String, String)> {
    let all = read_outputs(
        &fixtures()
            .join("workspaces")
            .join("garden")
            .join("expected"),
    );
    all.into_iter()
        .filter(|(name, _)| has_ext(name, ext))
        .collect()
}

fn names(files: &[(String, String)]) -> Vec<&str> {
    files.iter().map(|(name, _)| name.as_str()).collect()
}

const NOTE: &str = "note: skipped testonly: it has only tests, examples, or benches";

#[test]
fn with_no_arguments_the_config_runs_from_a_subfolder_into_its_own_folders() {
    let (_tmp, repo) = garden(REPOSITORY);
    let result = vibelight_in(&repo.join("crates").join("beta"), &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(lines(&result.stderr), [NOTE]);
    let markdown = read_outputs(&repo.join("vibelight"));
    assert_eq!(names(&markdown), ["alpha.md", "beta.md", "facade.md"]);
    assert!(markdown == goldens("md"), "the markdown has source links");
    let html = read_outputs(&repo.join("vibelight").join("html"));
    assert_eq!(names(&html), ["alpha.html", "beta.html", "facade.html"]);
    let alpha = &html[0].1;
    assert!(alpha.contains("href=\"../../crates/alpha/src/lib.rs#L6\""));
    assert!(alpha.contains("href=\"../../crates/alpha/src/lib.rs#L18\""));
    assert!(
        html[2]
            .1
            .contains("<a class=\"cross\" href=\"alpha.html#type.alpha.Meter\">")
    );
    let stripped: Vec<(String, String)> = html
        .iter()
        .map(|(name, text)| (name.clone(), strip_html_links(text)))
        .collect();
    assert!(
        stripped == goldens("html"),
        "the HTML is not the golden plus links"
    );
    assert!(!repo.join("crates").join("beta").join("beta.md").exists());
}

#[test]
fn the_kill_list_key_turns_the_kill_list_off_and_its_switch_wins() {
    let config = format!("kill_list = \"off\"\n{REPOSITORY}");
    let (_tmp, repo) = garden(&config);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let alpha = fs::read_to_string(repo.join("vibelight").join("alpha.md")).unwrap();
    assert!(!alpha.contains("**MACROS:**"), "{alpha}");
    assert!(alpha.contains("and attribute macro.\n"));
    let page = fs::read_to_string(repo.join("vibelight").join("html").join("alpha.html")).unwrap();
    assert!(!page.contains("<p class=\"kill\">"));
    let result = vibelight_in(&repo, &["--kill-list", "full"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let alpha = fs::read_to_string(repo.join("vibelight").join("alpha.md")).unwrap();
    assert!(alpha.contains("**MACROS:** none\n"), "{alpha}");
}

#[test]
fn explicit_config_keys_beat_the_analysis_profile() {
    let config = "workspace = true\nprofile = \"analysis\"\norder = \"levels\"\n\n\
        [markdown]\nout = \"vibelight\"\nsource_links = true\n";
    let (_tmp, repo) = garden(config);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let alpha = fs::read_to_string(repo.join("vibelight").join("alpha.md")).unwrap();
    assert!(
        alpha.contains("\nSource: "),
        "the key keeps the links on:\n{alpha}"
    );
    assert!(
        alpha.contains("\n## Unconnected types\n"),
        "the key keeps the level order"
    );
    assert!(
        alpha.contains("\nCounts: "),
        "the profile still turns the facts on"
    );
    let result = vibelight_in(&repo, &["--order", "modules"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let alpha = fs::read_to_string(repo.join("vibelight").join("alpha.md")).unwrap();
    assert!(alpha.contains("\n## Levels\n"), "the switch beats the key");
}

#[test]
fn a_config_without_folders_writes_beside_itself_from_any_subfolder() {
    let (_tmp, repo) = garden("workspace = true\n");
    let alpha = repo.join("crates").join("alpha");
    let result = vibelight_in(&alpha, &["--no-source-links"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(repo.join("alpha.md").is_file() && repo.join("facade.html").is_file());
    assert!(!alpha.join("alpha.md").exists());
    let result = vibelight_in(&alpha, &["--no-source-links", "--check"]);
    assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stdout));
    assert_eq!(lines(&result.stdout), ["check passed: 3 files match"]);
}

#[test]
fn crate_arguments_choose_the_crates_and_the_config_still_places_the_files() {
    let (tmp, repo) = garden(
        "workspace = true\n\n[markdown]\nout = \"docs\"\nsource_links = false\n\n\
         [html]\nenabled = false\n",
    );
    let result = vibelight_in(&repo, &["crates/beta"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let stdout = lines(&result.stdout);
    assert_eq!(stdout.len(), 1, "{stdout:?}");
    assert!(
        Path::new(&stdout[0]).ends_with("docs/beta.md"),
        "{stdout:?}"
    );
    assert_eq!(names(&read_outputs(&repo.join("docs"))), ["beta.md"]);
    assert!(!repo.join("beta.html").exists());
    let alone = tmp.path().join("alone");
    let flags = vibelight_in(
        &repo,
        &[
            "crates/beta",
            "-o",
            alone.to_str().unwrap(),
            "--no-source-links",
        ],
    );
    assert!(flags.status.success());
    assert_eq!(
        fs::read(repo.join("docs").join("beta.md")).unwrap(),
        fs::read(alone.join("beta.md")).unwrap()
    );
}

#[test]
fn each_format_flag_wins_over_the_flag_for_both_and_over_the_config() {
    let (_tmp, repo) = garden(
        "workspace = true\n\n[markdown]\nout = \"md\"\nsource_links = false\n\n\
         [html]\nout = \"web\"\nsource_links = true\n",
    );
    let crates = repo.join("crates");
    let args = [
        "-o",
        "both",
        "--md-out",
        "mdflag",
        "--md-source-links",
        "on",
        "--html-source-links",
        "off",
    ];
    let result = vibelight_in(&crates, &args);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let markdown = read_outputs(&crates.join("mdflag"));
    assert_eq!(names(&markdown), ["alpha.md", "beta.md", "facade.md"]);
    assert!(
        markdown[0]
            .1
            .contains("\nSource: [src/lib.rs:6](../alpha/src/lib.rs#L6)\n"),
        "{}",
        markdown[0].1
    );
    assert!(
        read_outputs(&crates.join("both")) == goldens("html"),
        "the HTML has source links"
    );
    assert!(!repo.join("md").exists() && !repo.join("web").exists());
}

#[test]
fn the_switch_for_both_formats_loses_only_to_a_format_switch() {
    let (_tmp, repo) = garden("workspace = true\n");
    let args = [
        "-o",
        "out",
        "--no-source-links",
        "--html-source-links",
        "on",
    ];
    let result = vibelight_in(&repo, &args);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let files = read_outputs(&repo.join("out"));
    let (html, markdown): (Vec<_>, Vec<_>) = files
        .into_iter()
        .partition(|(name, _)| has_ext(name, "html"));
    assert!(markdown == goldens("md"), "the markdown has source links");
    assert!(html[0].1.contains("href=\"../crates/alpha/src/lib.rs#L6\""));
}

#[test]
fn a_format_the_config_turns_off_comes_back_only_for_its_own_folder_flag() {
    let (_tmp, repo) = garden("workspace = true\n\n[html]\nenabled = false\n");
    assert!(vibelight_in(&repo, &["-o", "a"]).status.success());
    assert_eq!(
        names(&read_outputs(&repo.join("a"))),
        ["alpha.md", "beta.md", "facade.md"]
    );
    assert!(
        vibelight_in(&repo, &["-o", "a", "--html-out", "b"])
            .status
            .success()
    );
    assert_eq!(
        names(&read_outputs(&repo.join("b"))),
        ["alpha.html", "beta.html", "facade.html"]
    );
}

#[test]
fn config_errors_name_the_key_and_exit_with_1() {
    let (_tmp, repo) = garden("workspace = true\n\n[markdown]\ndir = \"x\"\n");
    let result = vibelight_in(&repo, &[]);
    assert_eq!(result.status.code(), Some(1));
    let stderr = lines(&result.stderr);
    assert_eq!(stderr.len(), 1, "{stderr:?}");
    assert!(stderr[0].starts_with("error: "), "{stderr:?}");
    assert!(
        stderr[0].ends_with("vibelight.toml: unknown key `markdown.dir`"),
        "{stderr:?}"
    );
    fs::write(
        repo.join("vibelight.toml"),
        "workspace = true\n[markdown]\nenabled = false\n[html]\nenabled = false\n",
    )
    .unwrap();
    let result = vibelight_in(&repo, &[]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stderr),
        ["error: the config turns both formats off, so there is nothing to write"]
    );
}

#[test]
fn with_no_crate_named_anywhere_the_run_is_a_usage_error() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight_in(tmp.path(), &[]);
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains(
            "name a crate, pass --workspace, or add a vibelight.toml that names the crates"
        ),
        "{stderr}"
    );
    let (_tmp, repo) = garden("[markdown]\nout = \"x\"\n");
    assert_eq!(vibelight_in(&repo, &[]).status.code(), Some(2));
    let result = vibelight_in(&repo, &["crates/beta", "--md-source-links", "maybe"]);
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains("[possible values: on, off]"), "{stderr}");
}
