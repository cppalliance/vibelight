//! The check switches: a clean tree passes, and a changed declaration, a
//! missing file, and a stale outline each give one line and exit code 1.
//! Checking writes nothing, and a normal run only warns about stale files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{REPOSITORY, garden, lines, vibelight_in};

const NOTE: &str = "note: skipped testonly: it has only tests, examples, or benches";

/// The garden with the repository config, after one normal run.
fn written() -> (tempfile::TempDir, PathBuf) {
    let (tmp, repo) = garden(REPOSITORY);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    (tmp, repo)
}

/// A path below the repository, written the way the check prints it.
fn shown(parts: &[&str]) -> String {
    parts.iter().collect::<PathBuf>().display().to_string()
}

#[test]
fn a_clean_tree_passes_and_crlf_on_disk_still_matches() {
    let (_tmp, repo) = written();
    let alpha = repo.join("vibelight").join("alpha.md");
    let crlf = fs::read_to_string(&alpha).unwrap().replace('\n', "\r\n");
    fs::write(&alpha, &crlf).unwrap();
    let result = vibelight_in(&repo, &["--check"]);
    assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stdout));
    assert_eq!(lines(&result.stdout), ["check passed: 3 files match"]);
    assert_eq!(lines(&result.stderr), [NOTE]);
    let result = vibelight_in(&repo, &["--check-html"]);
    assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stdout));
    assert_eq!(lines(&result.stdout), ["check passed: 6 files match"]);
    assert_eq!(fs::read_to_string(&alpha).unwrap(), crlf, "the check wrote");
}

#[test]
fn a_changed_declaration_differs_at_its_first_changed_line() {
    let (_tmp, repo) = written();
    let page = repo.join("vibelight").join("html").join("alpha.html");
    let html_line = fs::read_to_string(&page)
        .unwrap()
        .lines()
        .position(|line| line.contains("u32"))
        .expect("the trait method returns u32")
        + 1;
    let source = repo.join("crates").join("alpha").join("src").join("lib.rs");
    let text = fs::read_to_string(&source).unwrap();
    fs::write(
        &source,
        text.replace("fn read(&self) -> u32;", "fn read(&self) -> u64;"),
    )
    .unwrap();
    let result = vibelight_in(&repo, &["--check"]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stdout),
        [
            format!(
                "differs: {} (first difference at line 76)",
                shown(&["vibelight", "alpha.md"])
            ),
            "check failed: 1 problem".to_owned(),
        ]
    );
    let result = vibelight_in(&repo, &["--check-html"]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stdout),
        [
            format!(
                "differs: {} (first difference at line 76)",
                shown(&["vibelight", "alpha.md"])
            ),
            format!(
                "differs: {} (first difference at line {html_line})",
                shown(&["vibelight", "html", "alpha.html"])
            ),
            "check failed: 2 problems".to_owned(),
        ]
    );
}

#[test]
fn a_missing_outline_is_reported() {
    let (_tmp, repo) = written();
    fs::remove_file(repo.join("vibelight").join("beta.md")).unwrap();
    let result = vibelight_in(&repo, &["--check"]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stdout),
        [
            format!("missing: {}", shown(&["vibelight", "beta.md"])),
            "check failed: 1 problem".to_owned(),
        ]
    );
}

#[test]
fn a_stale_outline_is_reported_and_a_normal_run_only_warns_about_it() {
    let (_tmp, repo) = written();
    let folder = repo.join("vibelight");
    let outline = "<a id=\"crate.old\"></a>\n# old\n";
    fs::write(folder.join("old.md"), outline).unwrap();
    fs::write(folder.join("README.md"), "# Notes\n").unwrap();
    fs::write(folder.join("notes.txt"), outline).unwrap();
    fs::write(folder.join("html").join("old.md"), outline).unwrap();
    let result = vibelight_in(&repo, &["--check"]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stdout),
        [
            format!("stale: {}", shown(&["vibelight", "old.md"])),
            "check failed: 1 problem".to_owned(),
        ]
    );
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [
            NOTE.to_owned(),
            format!(
                "warning: {} is stale: no crate of this run writes it",
                shown(&["vibelight", "old.md"])
            ),
            "1 warning".to_owned(),
        ]
    );
    assert_eq!(fs::read_to_string(folder.join("old.md")).unwrap(), outline);
}

#[test]
fn explicit_crates_are_checked_against_the_folders_of_the_config() {
    let (_tmp, repo) = written();
    let all = ["crates/alpha", "crates/beta", "crates/facade", "--check"];
    let result = vibelight_in(&repo, &all);
    assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stdout));
    assert_eq!(lines(&result.stdout), ["check passed: 3 files match"]);
    let result = vibelight_in(&repo, &["crates/beta", "--check"]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stdout),
        [
            format!("stale: {}", shown(&["vibelight", "alpha.md"])),
            format!("stale: {}", shown(&["vibelight", "facade.md"])),
            "check failed: 2 problems".to_owned(),
        ]
    );
    let elsewhere = [&all[..3], &["-o", "elsewhere", "--check"]].concat();
    let result = vibelight_in(&repo, &elsewhere);
    assert_eq!(result.status.code(), Some(1));
    let missing = |name: &str| format!("missing: {}", shown(&["elsewhere", name]));
    assert_eq!(
        lines(&result.stdout),
        [
            missing("alpha.md"),
            missing("beta.md"),
            missing("facade.md"),
            "check failed: 3 problems".to_owned(),
        ]
    );
    assert!(
        !Path::new(&repo.join("elsewhere")).exists(),
        "the check wrote"
    );
}
