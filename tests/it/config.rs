//! Runs with `vibelight.toml`: the config chooses the crates and the
//! folder, crate arguments replace only its crates, its follow and exclude
//! globs apply either way, and config errors name the key.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use crate::support::{copy_dir, fixtures, has_ext, read_outputs, read_plain, read_run, vibelight};

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

/// The config that outlines the garden's members into `.vibelight`.
pub(crate) const MEMBERS: &str = "workspace = true\n";

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

/// The garden's goldens, by file name.
pub(crate) fn goldens() -> Vec<(String, String)> {
    read_outputs(
        &fixtures()
            .join("workspaces")
            .join("garden")
            .join("expected"),
    )
}

pub(crate) fn names(files: &[(String, String)]) -> Vec<&str> {
    files.iter().map(|(name, _)| name.as_str()).collect()
}

pub(crate) const NOTE: &str = "note: skipped testonly: it has only tests, examples, or benches";

#[test]
fn with_no_arguments_the_config_runs_from_a_subfolder_into_its_own_folders() {
    let (_tmp, repo) = garden(MEMBERS);
    let result = vibelight_in(&repo.join("crates").join("beta"), &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let stderr = lines(&result.stderr);
    assert_eq!(stderr[0], NOTE);
    assert!(
        stderr[1].starts_with("note: created ") && stderr[1].ends_with(".gitignore"),
        "{stderr:?}"
    );
    assert_eq!(stderr.len(), 2, "{stderr:?}");
    let out = repo.join(".vibelight");
    assert_eq!(
        fs::read_to_string(out.join(".gitignore")).unwrap(),
        "*.md\n*.html\n"
    );
    let files = read_run(&out);
    assert_eq!(
        names(&files),
        [
            "alpha.html",
            "alpha.md",
            "beta.html",
            "beta.md",
            "facade.html",
            "facade.md"
        ]
    );
    assert!(
        read_plain(&out) == goldens(),
        "the run is the goldens plus links"
    );
    let alpha_md = &files[1].1;
    assert!(alpha_md.contains(
        "{\"file\":\"src/lib.rs\",\"line\":6,\"url\":\"../crates/alpha/src/lib.rs#L6\"}"
    ));
    let alpha_page = &files[0].1;
    assert!(alpha_page.contains("href=\"../../crates/alpha/src/lib.rs#L6\""));
    assert!(alpha_page.contains("href=\"../../crates/alpha/src/lib.rs#L18\""));
    assert!(
        files[4]
            .1
            .contains("<a class=\"cross\" href=\"alpha.html#type.alpha.Meter\">")
    );
    let index = fs::read_to_string(out.join("index.md")).unwrap();
    assert!(
        index.contains("- [alpha.md](alpha.md): `crates/alpha`, selected, "),
        "{index}"
    );
    let again = vibelight_in(&repo, &[]);
    assert_eq!(lines(&again.stderr), [NOTE], "the .gitignore is made once");
}

#[test]
fn a_repository_without_a_config_gets_one_that_chooses_its_crates() {
    let (_tmp, repo) = garden("");
    let result = vibelight_in(&repo.join("crates").join("alpha"), &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let config = fs::read_to_string(repo.join("vibelight.toml")).unwrap();
    assert_eq!(
        config,
        "# Written by vibelight. It chooses the crates a plain `vibelight` outlines:\n\
         # edit `exclude` to change which of them are analyzed.\n\
         workspace = true\nfollow = [\"**\"]\nexclude = []\nout = \".vibelight\"\n"
    );
    let stderr = lines(&result.stderr);
    assert!(
        stderr[0].starts_with("note: created ")
            && stderr[0].ends_with("vibelight.toml; edit it to choose the crates"),
        "{stderr:?}"
    );
    let out = repo.join(".vibelight");
    let markdown: Vec<(String, String)> = read_run(&out)
        .into_iter()
        .filter(|(name, _)| has_ext(name, "md"))
        .collect();
    assert_eq!(
        names(&markdown),
        ["alpha.md", "beta.md", "delta.md", "facade.md", "gamma.md"]
    );
    let index = fs::read_to_string(out.join("index.md")).unwrap();
    for (stem, dir, how) in [
        ("alpha", "crates/alpha", "selected"),
        ("delta", "crates/delta", "followed"),
        ("gamma", "vendor/gamma", "followed"),
    ] {
        assert!(
            index.contains(&format!("- [{stem}.md]({stem}.md): `{dir}`, {how}, ")),
            "{index}"
        );
    }
    let second = vibelight_in(&repo, &[]);
    assert_eq!(lines(&second.stderr), [NOTE], "the config is written once");
    assert_eq!(
        fs::read_to_string(repo.join("vibelight.toml")).unwrap(),
        config
    );
}

#[test]
fn removed_keys_are_unknown_keys_that_exit_with_1() {
    for (text, key) in [
        ("kill_list = \"off\"\n", "kill_list"),
        ("profile = \"analysis\"\n", "profile"),
        ("[markdown]\nout = \"x\"\n", "markdown"),
    ] {
        let (_tmp, repo) = garden(&format!("{MEMBERS}{text}"));
        let result = vibelight_in(&repo, &[]);
        assert_eq!(result.status.code(), Some(1));
        let stderr = lines(&result.stderr);
        assert_eq!(stderr.len(), 1, "{stderr:?}");
        assert!(stderr[0].starts_with("error: "), "{stderr:?}");
        assert!(
            stderr[0].ends_with(&format!("vibelight.toml: unknown key `{key}`")),
            "{stderr:?}"
        );
        assert!(!repo.join(".vibelight").exists());
    }
}

#[test]
fn crate_arguments_choose_the_crates_and_the_config_still_places_the_files() {
    let (tmp, repo) = garden("workspace = true\nout = \"docs\"\n");
    let result = vibelight_in(&repo, &["crates/beta"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let stdout = lines(&result.stdout);
    assert_eq!(stdout.len(), 4, "{stdout:?}");
    assert!(
        Path::new(&stdout[0]).ends_with("docs/beta.md"),
        "{stdout:?}"
    );
    assert!(Path::new(&stdout[1]).ends_with("docs/html/beta.html"));
    assert!(Path::new(&stdout[2]).ends_with("docs/html/index.html"));
    assert!(Path::new(&stdout[3]).ends_with("docs/index.md"));
    assert!(repo.join("docs").join(".gitignore").is_file());
    let alone = tmp.path().join("alone");
    let result = vibelight_in(&repo, &["crates/beta", "-o", alone.to_str().unwrap()]);
    assert!(result.status.success());
    assert!(
        read_plain(&repo.join("docs")) == read_plain(&alone),
        "the folder changed what the run writes"
    );
    assert!(!alone.join(".gitignore").exists());
}

#[test]
fn crate_arguments_keep_the_follow_and_exclude_globs_of_the_config() {
    let (_tmp, repo) = garden(
        "workspace = true\nfollow = [\"crates/d*\", \"vendor/*\"]\n\
         exclude = [\"vendor/gamma\", \"crates/zeta\"]\n",
    );
    let result = vibelight_in(&repo, &["crates/alpha", "-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(
        result.stderr.is_empty(),
        "a named run reports no unmatched glob: {:?}",
        lines(&result.stderr)
    );
    assert_eq!(
        names(&read_run(&repo.join("out"))),
        ["alpha.html", "alpha.md", "delta.html", "delta.md"]
    );
}

#[test]
fn a_crate_named_on_the_command_line_is_outlined_even_when_excluded() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/beta\"]\n");
    let result = vibelight_in(&repo, &["crates/beta", "-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(result.stderr.is_empty(), "{:?}", lines(&result.stderr));
    assert_eq!(
        names(&read_run(&repo.join("out"))),
        ["beta.html", "beta.md"]
    );
}

#[test]
fn a_single_file_ignores_the_follow_and_exclude_globs_of_the_config() {
    let (tmp, repo) = garden("workspace = true\nfollow = [\"**\"]\nexclude = [\"x\"]\n");
    let file = fixtures().join("single").join("tool.rs");
    let out = tmp.path().join("out");
    let result = vibelight_in(
        &repo,
        &[file.to_str().unwrap(), "-o", out.to_str().unwrap()],
    );
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(names(&read_run(&out)), ["tool.html", "tool.md"]);
}

#[test]
fn with_no_crate_named_anywhere_the_run_is_a_usage_error() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight_in(tmp.path(), &[]);
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("name a crate, or run vibelight inside a Rust repository"),
        "{stderr}"
    );
    assert!(!tmp.path().join("vibelight.toml").exists());
    let (_tmp, repo) = garden("out = \"x\"\n");
    let result = vibelight_in(&repo, &[]);
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("name a crate, or set `workspace` or `crates` in vibelight.toml"),
        "{stderr}"
    );
}
