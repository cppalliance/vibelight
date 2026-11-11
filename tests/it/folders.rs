//! The folders of a run hold what the last run wrote: an outline no crate
//! of the run writes any more is removed with a note, in both formats and
//! in the markdown of an older release too, and so is each file that an
//! earlier release left where it wrote. Every other file in them is left
//! alone.

use std::fs;
use std::path::PathBuf;

use crate::config::{MEMBERS, NOTE, garden, lines, names, vibelight_in};
use crate::support::{markdown, read_run};

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

/// The one note for the files removed from where an earlier release wrote
/// them, as a run into `.vibelight` prints it.
fn moved(files: &str) -> String {
    format!(
        "note: removed {files} that an earlier release wrote: the HTML is now {} and the \
         markdown is in {}",
        shown(&[".vibelight", "index.html"]),
        shown(&[".vibelight", "md"])
    )
}

/// A markdown outline of records, of a crate no run of the garden writes.
const RECORDS: &str = "*Updated 2026-01-01T00:00:00Z*\n\
     {\"k\":\"vibelight-outline\",\"schema\":1,\"crate\":\"gone\",\"target\":\"lib\"}\n\
     {\"k\":\"modules\",\"items\":[]}\n";

/// How an HTML file of a run starts.
const PAGE: &str = "<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n<html>\n";

#[test]
fn a_run_removes_the_outlines_no_crate_writes_and_keeps_every_other_file() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let out = repo.join(".vibelight");
    let md = markdown(&out);
    let outline = "*Updated 2026-01-01T00:00:00Z*\n\n<a id=\"crate.old\"></a>\n# old\n";
    fs::write(md.join("gone.md"), RECORDS).unwrap();
    fs::write(md.join("old.md"), outline).unwrap();
    fs::write(md.join("README.md"), "# Notes\n").unwrap();
    fs::write(md.join("notes.txt"), outline).unwrap();
    fs::write(md.join("old.html"), PAGE).unwrap();
    fs::write(out.join("old.html"), PAGE).unwrap();
    fs::write(out.join("other.html"), "<!DOCTYPE html>\n").unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [
            NOTE.to_owned(),
            removed(&[".vibelight", "md", "gone.md"]),
            removed(&[".vibelight", "md", "old.md"]),
            removed(&[".vibelight", "old.html"]),
        ]
    );
    assert!(
        !md.join("gone.md").exists(),
        "a stale outline of records stays"
    );
    assert!(
        !md.join("old.md").exists(),
        "a stale outline of an older release stays"
    );
    assert!(!out.join("old.html").exists());
    for kept in [
        "md/README.md",
        "md/notes.txt",
        "md/old.html",
        "md/index.md",
        "other.html",
        ".gitignore",
        "index.html",
    ] {
        assert!(out.join(kept).is_file(), "{kept} was removed");
    }
}

#[test]
fn a_run_removes_what_an_earlier_release_left_where_it_wrote() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let out = repo.join(".vibelight");
    let md = markdown(&out);
    // An earlier release wrote the markdown and its index in the folder
    // itself, and the HTML file, with script files before that, in `html`.
    let old_html = out.join("html");
    fs::create_dir(&old_html).unwrap();
    fs::copy(md.join("alpha.md"), out.join("alpha.md")).unwrap();
    fs::copy(md.join("index.md"), out.join("index.md")).unwrap();
    fs::copy(out.join("index.html"), old_html.join("index.html")).unwrap();
    fs::write(
        old_html.join("search.js"),
        "vibelightSearch({\"pages\":[]});\n",
    )
    .unwrap();
    fs::write(out.join("README.md"), "# Notes\n").unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [NOTE.to_owned(), moved("4 files")],
        "one note for the outline, the index, the page, and the script file, which says \
         where the files are now"
    );
    for gone in ["alpha.md", "index.md"] {
        assert!(!out.join(gone).exists(), "{gone} is still there");
    }
    assert!(!old_html.exists(), "the run emptied the folder, so it went");
    for kept in [
        "README.md",
        "index.html",
        "md/alpha.md",
        "md/index.md",
        ".gitignore",
    ] {
        assert!(out.join(kept).is_file(), "{kept} was removed");
    }
    let again = vibelight_in(&repo, &[]);
    assert_eq!(lines(&again.stderr), [NOTE], "nothing is left to remove");
}

#[test]
fn the_html_folder_of_an_earlier_release_stays_while_it_holds_another_file() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let out = repo.join(".vibelight");
    let old_html = out.join("html");
    fs::create_dir(&old_html).unwrap();
    fs::write(old_html.join("index.html"), PAGE).unwrap();
    fs::write(old_html.join("notes.txt"), "Kept by hand.\n").unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert_eq!(lines(&result.stderr), [NOTE.to_owned(), moved("1 file")]);
    assert!(!old_html.join("index.html").exists());
    assert!(old_html.join("notes.txt").is_file());
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
        [NOTE.to_owned(), removed(&[".vibelight", "md", "facade.md"])]
    );
    assert_eq!(
        names(&read_run(&repo.join(".vibelight"))),
        ["alpha.html", "alpha.md", "beta.html", "beta.md"],
        "the folder's one file holds no page for the crate"
    );
}

#[test]
fn a_run_of_one_named_crate_leaves_only_that_crate_in_the_folder() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let result = vibelight_in(&repo, &["crates/beta"]);
    assert!(result.status.success());
    assert_eq!(
        lines(&result.stderr),
        [
            removed(&[".vibelight", "md", "alpha.md"]),
            removed(&[".vibelight", "md", "facade.md"])
        ]
    );
    assert_eq!(
        names(&read_run(&repo.join(".vibelight"))),
        ["beta.html", "beta.md"]
    );
    let index = fs::read_to_string(markdown(&repo.join(".vibelight")).join("index.md")).unwrap();
    assert!(!index.contains("alpha"), "{index}");
}
