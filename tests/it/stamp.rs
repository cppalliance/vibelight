//! The stamp on the first line: it records when a file's content last
//! changed, and a run leaves a file whose content is the same untouched. A
//! crate's HTML page depends on its own crate alone, but its markdown is
//! contextual to the run, so a change in one crate can restamp the
//! markdown of another.

use std::fs;
use std::path::Path;
use std::process::Output;

use crate::config::{MEMBERS, garden, lines};
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
    let (_tmp, repo) = garden(MEMBERS);
    let first = run_at(&repo, crate::support::EPOCH, &[]);
    assert!(first.status.success(), "{:?}", lines(&first.stderr));
    let folder = repo.join(".vibelight");
    let before = read_outputs(&folder);
    let html_before = read_outputs(&folder.join("html"));
    let second = run_at(&repo, LATER, &[]);
    assert!(second.status.success(), "{:?}", lines(&second.stderr));
    assert_eq!(lines(&second.stdout), lines(&first.stdout));
    assert_eq!(lines(&second.stdout).len(), 8);
    assert_eq!(read_outputs(&folder), before);
    assert_eq!(read_outputs(&folder.join("html")), html_before);
    assert_eq!(first_line(&folder.join("alpha.md")), FIRST_STAMP);
}

/// Tells, for each outline of the garden, whether a run restamped it.
fn restamped(folder: &Path) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for stem in ["alpha", "beta", "facade"] {
        let md = first_line(&folder.join(format!("{stem}.md")));
        let page = first_line(&folder.join("html").join(format!("{stem}.html")));
        out.push((format!("{stem}.md"), md == LATER_STAMP));
        out.push((format!("{stem}.html"), page.contains("2027-01-01")));
    }
    out
}

fn expect(flags: [(&str, bool); 6]) -> Vec<(String, bool)> {
    flags.map(|(name, set)| (name.to_owned(), set)).to_vec()
}

/// Runs the garden, edits alpha's source with `edit`, and runs it again
/// later, returning the folder.
fn edited(repo: &Path, edit: impl FnOnce(String) -> String) -> std::path::PathBuf {
    assert!(run_at(repo, crate::support::EPOCH, &[]).status.success());
    let source = repo.join("crates").join("alpha").join("src").join("lib.rs");
    let text = fs::read_to_string(&source).expect("the source is readable");
    let changed = edit(text.clone());
    assert_ne!(changed, text, "the edit changes the source");
    fs::write(&source, changed).expect("the source is writable");
    assert!(run_at(repo, LATER, &[]).status.success());
    repo.join(".vibelight")
}

#[test]
fn a_changed_declaration_restamps_only_its_files() {
    let (_tmp, repo) = garden(MEMBERS);
    let folder = edited(&repo, |text| {
        text.replace("fn read(&self) -> u32;", "fn read(&self) -> u64;")
    });
    assert_eq!(
        restamped(&folder),
        expect([
            ("alpha.md", true),
            ("alpha.html", true),
            ("beta.md", false),
            ("beta.html", false),
            ("facade.md", false),
            ("facade.html", false),
        ])
    );
    let page = fs::read_to_string(folder.join("html").join("alpha.html")).unwrap();
    assert!(page.starts_with("<!-- Updated 2027-01-01T00:00:00Z -->\n<!DOCTYPE html>\n"));
    assert!(page.contains(
        "<main>\n<p class=\"stamp\">Updated \
         <time datetime=\"2027-01-01T00:00:00Z\">2027-01-01T00:00:00Z</time></p>\n<h1 "
    ));
}

#[test]
fn a_changed_doc_restamps_the_markdown_that_quotes_it_and_no_other_page() {
    let (_tmp, repo) = garden(MEMBERS);
    let folder = edited(&repo, |text| {
        text.replace(
            "/// A meter that reads a gauge of beta.",
            "/// A meter that reads one gauge.",
        )
    });
    assert_eq!(
        restamped(&folder),
        expect([
            ("alpha.md", true),
            ("alpha.html", true),
            ("beta.md", false),
            ("beta.html", false),
            ("facade.md", true),
            ("facade.html", false),
        ]),
        "facade's markdown quotes alpha's doc summary; its page is target-local"
    );
    let facade = fs::read_to_string(folder.join("facade.md")).unwrap();
    assert!(facade.contains("\"summary\":\"A meter that reads one gauge.\""));
}

#[test]
fn a_malformed_epoch_stops_a_run_before_it_writes() {
    let (_tmp, repo) = garden("");
    let result = run_at(&repo, "soon", &[]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stderr),
        [
            "error: SOURCE_DATE_EPOCH must be a whole number of seconds from 0 to \
             253402300799, but it is `soon`"
        ]
    );
    assert!(!repo.join(".vibelight").exists(), "the run wrote");
    assert!(
        !repo.join("vibelight.toml").exists(),
        "the run wrote a config"
    );
}
