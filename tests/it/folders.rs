//! The folders of a run hold what the last run wrote: an outline no crate
//! of the run writes any more is removed with a note, in both formats, and
//! every other file in them is left alone.

use std::fs;
use std::path::PathBuf;

use crate::config::{MEMBERS, NOTE, garden, lines, names, vibelight_in};
use crate::support::read_run;

/// A path below the repository, written the way the notes print it.
fn shown(parts: &[&str]) -> String {
    parts.iter().collect::<PathBuf>().display().to_string()
}

fn removed(parts: &[&str]) -> String {
    format!(
        "note: removed {}: no crate of this run writes it",
        shown(parts)
    )
}

#[test]
fn a_run_removes_the_outlines_no_crate_writes_and_keeps_every_other_file() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let out = repo.join(".vibelight");
    let outline = "*Updated 2026-01-01T00:00:00Z*\n\n<a id=\"crate.old\"></a>\n# old\n";
    let page = "<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n<html>\n";
    fs::write(out.join("old.md"), outline).unwrap();
    fs::write(out.join("README.md"), "# Notes\n").unwrap();
    fs::write(out.join("notes.txt"), outline).unwrap();
    fs::write(out.join("html").join("old.html"), page).unwrap();
    fs::write(out.join("html").join("old.md"), outline).unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [
            NOTE.to_owned(),
            removed(&[".vibelight", "html", "old.html"]),
            removed(&[".vibelight", "old.md"]),
        ]
    );
    assert!(!out.join("old.md").exists() && !out.join("html").join("old.html").exists());
    for kept in [
        "README.md",
        "notes.txt",
        "html/old.md",
        ".gitignore",
        "index.md",
    ] {
        assert!(out.join(kept).is_file(), "{kept} was removed");
    }
}

#[test]
fn a_crate_the_exclude_now_leaves_out_loses_its_outlines() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    fs::write(
        repo.join("vibelight.toml"),
        format!("{MEMBERS}exclude = [\"crates/facade\"]\n"),
    )
    .unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [
            NOTE.to_owned(),
            removed(&[".vibelight", "facade.md"]),
            removed(&[".vibelight", "html", "facade.html"]),
        ]
    );
    assert_eq!(
        names(&read_run(&repo.join(".vibelight"))),
        ["alpha.html", "alpha.md", "beta.html", "beta.md"]
    );
}

#[test]
fn a_run_of_one_named_crate_leaves_only_that_crate_in_the_folder() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let result = vibelight_in(&repo, &["crates/beta"]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr).len(),
        4,
        "{:?}",
        lines(&result.stderr)
    );
    assert_eq!(
        names(&read_run(&repo.join(".vibelight"))),
        ["beta.html", "beta.md"]
    );
    let index = fs::read_to_string(repo.join(".vibelight").join("index.md")).unwrap();
    assert!(!index.contains("alpha"), "{index}");
}
