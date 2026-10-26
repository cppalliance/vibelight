//! The stamp on the first line: it records when a file's content last
//! changed, a run leaves a file whose content is the same untouched, and the
//! check ignores it.

use std::fs;
use std::path::Path;
use std::process::Output;

use crate::config::{REPOSITORY, garden, lines};
use crate::support::{read_outputs, vibelight};

const LATER: &str = "1798761600";
const LATER_STAMP: &str = "*Updated 2027-01-01T00:00:00Z*";
const FIRST_STAMP: &str = "*Updated 2026-01-01T00:00:00Z*";

/// Runs the binary in `cwd` with `args` and `epoch` as `SOURCE_DATE_EPOCH`.
fn run_at(cwd: &Path, epoch: &str, args: &[&str]) -> Output {
    vibelight()
        .env("SOURCE_DATE_EPOCH", epoch)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("the vibelight binary runs")
}

fn first_line(path: &Path) -> String {
    fs::read_to_string(path)
        .expect("the outline was written")
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn a_second_run_at_another_time_rewrites_nothing_and_still_lists_every_file() {
    let (_tmp, repo) = garden(REPOSITORY);
    let first = run_at(&repo, crate::support::EPOCH, &[]);
    assert!(first.status.success(), "{:?}", lines(&first.stderr));
    let folder = repo.join("vibelight");
    let before = read_outputs(&folder);
    let html_before = read_outputs(&folder.join("html"));
    let second = run_at(&repo, LATER, &[]);
    assert!(second.status.success(), "{:?}", lines(&second.stderr));
    assert_eq!(lines(&second.stdout), lines(&first.stdout));
    assert_eq!(lines(&second.stdout).len(), 6);
    assert_eq!(read_outputs(&folder), before);
    assert_eq!(read_outputs(&folder.join("html")), html_before);
    assert_eq!(first_line(&folder.join("alpha.md")), FIRST_STAMP);
}

#[test]
fn a_changed_declaration_restamps_only_its_files() {
    let (_tmp, repo) = garden(REPOSITORY);
    assert!(run_at(&repo, crate::support::EPOCH, &[]).status.success());
    let source = repo.join("crates").join("alpha").join("src").join("lib.rs");
    let text = fs::read_to_string(&source).unwrap();
    fs::write(
        &source,
        text.replace("fn read(&self) -> u32;", "fn read(&self) -> u64;"),
    )
    .unwrap();
    assert!(run_at(&repo, LATER, &[]).status.success());
    let folder = repo.join("vibelight");
    assert_eq!(first_line(&folder.join("alpha.md")), LATER_STAMP);
    assert_eq!(first_line(&folder.join("beta.md")), FIRST_STAMP);
    let page = fs::read_to_string(folder.join("html").join("alpha.html")).unwrap();
    assert!(page.starts_with("<!-- Updated 2027-01-01T00:00:00Z -->\n<!DOCTYPE html>\n"));
    assert!(page.contains(
        "<main>\n<p class=\"stamp\">Updated \
         <time datetime=\"2027-01-01T00:00:00Z\">2027-01-01T00:00:00Z</time></p>\n<h1 "
    ));
}

#[test]
fn the_check_ignores_the_stamp_and_never_reads_the_epoch() {
    let (_tmp, repo) = garden(REPOSITORY);
    assert!(run_at(&repo, crate::support::EPOCH, &[]).status.success());
    for epoch in [LATER, "not a number"] {
        let result = run_at(&repo, epoch, &["--check-html"]);
        assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stdout));
        assert_eq!(lines(&result.stdout), ["check passed: 6 files match"]);
    }
}

#[test]
fn a_malformed_epoch_stops_a_run_that_writes() {
    let (_tmp, repo) = garden(REPOSITORY);
    let result = run_at(&repo, "soon", &[]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stderr),
        [
            "error: SOURCE_DATE_EPOCH must be a whole number of seconds from 0 to \
             253402300799, but it is `soon`"
        ]
    );
    assert!(!repo.join("vibelight").exists(), "the run wrote");
}
