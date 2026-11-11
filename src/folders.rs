//! Writes a run's files into its folders: the HTML file into the base
//! folder, and the markdown outlines and their index into its `md` folder.
//! A file whose content is the same is left alone, so its stamp keeps the
//! time its content last changed. The folders hold only what the last run
//! wrote: an outline that no crate of the run writes any more is removed,
//! and so is each file that an earlier release left where it wrote, the
//! markdown in the base folder and the HTML in its `html` folder. Every
//! other file is left alone. Vibelight's own folder also gets a
//! `.gitignore`, so no outline is ever committed.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::index;
use crate::settings::Plan;
use crate::stamp;

/// The `.gitignore` of vibelight's own folder: the outlines and the index,
/// in any folder below it, the HTML file, and the script files that
/// earlier releases wrote beside it.
const GITIGNORE: &str = "*.md\n*.html\n*.js\n";

/// The folder, below the base folder, that earlier releases wrote the HTML
/// into, when the markdown went into the base folder itself.
const OLD_HTML: &str = "html";

/// The `.gitignore` that earlier releases wrote. A run replaces a file that
/// holds exactly this and leaves any other `.gitignore` alone.
const OLD_GITIGNORE: &str = "*.md\n*.html\n";

/// What writing a run did beside writing its files.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Written {
    /// The `.gitignore` the run created.
    pub(crate) gitignore: Option<PathBuf>,
    /// The `.gitignore` of an earlier release that the run replaced.
    pub(crate) upgraded: Option<PathBuf>,
    /// The outlines the run removed, sorted.
    pub(crate) removed: Vec<PathBuf>,
    /// The files the run removed from where an earlier release wrote them,
    /// sorted: its outlines and its index in the base folder, and its page
    /// and script files in the `html` folder.
    pub(crate) moved: Vec<PathBuf>,
}

/// Creates the folders, writes every file whose content changed, gives
/// vibelight's own folder its `.gitignore` when it has none, and removes
/// the outlines no crate of the run writes, with what an earlier release
/// left where it wrote. That release's `html` folder goes too when the run
/// empties it.
///
/// # Errors
/// Returns an error when a folder cannot be created or a file cannot be
/// written or removed.
pub(crate) fn write(files: &[(PathBuf, String)], plan: &Plan) -> Result<Written> {
    let base = plan.base();
    let markdown = plan.markdown_dir();
    for dir in [base, markdown.as_path()] {
        fs::create_dir_all(dir)
            .with_context(|| format!("cannot create output directory {}", dir.display()))?;
    }
    let mut written = Written::default();
    let ignore = base.join(".gitignore");
    if plan.owned {
        if !ignore.exists() {
            fs::write(&ignore, GITIGNORE)
                .with_context(|| format!("cannot write {}", ignore.display()))?;
            written.gitignore = Some(ignore);
        } else if fs::read_to_string(&ignore).is_ok_and(|text| text == OLD_GITIGNORE) {
            fs::write(&ignore, GITIGNORE)
                .with_context(|| format!("cannot write {}", ignore.display()))?;
            written.upgraded = Some(ignore);
        }
    }
    for (path, text) in files {
        if changed(path, text) {
            fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))?;
        }
    }
    let kept: BTreeSet<&Path> = files.iter().map(|(path, _)| path.as_path()).collect();
    written.removed = stale(&markdown, "md", &kept);
    written.removed.extend(stale(base, "html", &kept));
    written.removed.sort();
    let old_html = base.join(OLD_HTML);
    written.moved = stale(base, "md", &kept);
    written.moved.extend(stale(&old_html, "html", &kept));
    written.moved.extend(stale(&old_html, "js", &kept));
    written.moved.sort();
    for path in written.removed.iter().chain(&written.moved) {
        fs::remove_file(path).with_context(|| format!("cannot remove {}", path.display()))?;
    }
    if written.moved.iter().any(|path| path.starts_with(&old_html)) {
        // The folder goes only when the run emptied it: removing a folder
        // that still holds a file fails, and that file's owner keeps it.
        let _ = fs::remove_dir(&old_html);
    }
    Ok(written)
}

/// Tells whether the text differs from the file on disk. A CRLF line
/// ending on disk counts as LF, and the stamps are masked, so a file that
/// differs only in its stamp is the same.
fn changed(path: &Path, text: &str) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return true;
    };
    stamp::mask(&String::from_utf8_lossy(&bytes).replace("\r\n", "\n")) != stamp::mask(text)
}

/// Returns the outlines with extension `ext` directly in `dir` that `kept`
/// does not hold, sorted. A file counts only when it starts the way every
/// outline of that format does, so other files that share the folder never
/// count.
fn stale(dir: &Path, ext: &str, kept: &BTreeSet<&Path>) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|e| e == ext)
                && !kept.contains(path.as_path())
                && is_outline(path)
        })
        .collect();
    found.sort();
    found
}

/// Tells whether a file starts like an outline: a markdown outline with a
/// stamp line and then the outline header record; an HTML page with its
/// stamp comment and then the doctype; one of the script files an earlier
/// release wrote beside its page. A markdown outline of an older release
/// counts too, so a stale one is still cleaned up: a stamp line, a blank
/// line, and a crate anchor, or the anchor first. So does the index of a
/// run, which an earlier release wrote in the base folder: its heading, a
/// blank line, and its opening words.
fn is_outline(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    let mut reader = BufReader::new(file);
    let mut head = Vec::new();
    for _ in 0..3 {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return false;
        }
        head.push(line.trim_end().to_owned());
    }
    let stamped = stamp::is_line(&head[0]);
    (stamped && head[0].starts_with('*') && head[1].starts_with(HEADER))
        || (stamped && head[0].starts_with("<!--") && head[1] == "<!DOCTYPE html>")
        || OLD_SCRIPTS.iter().any(|start| head[0].starts_with(start))
        || is_anchor(&head[0])
        || (stamped && head[1].is_empty() && is_anchor(&head[2]))
        || (head[0] == index::HEADING && head[1].is_empty() && head[2].starts_with(index::OPENING))
}

/// How the header record on a markdown outline's second line starts.
const HEADER: &str = "{\"k\":\"vibelight-outline\",";

/// How the script files of earlier releases start, which the folder's one
/// file now holds: a page's data file, the search index, and the app.
const OLD_SCRIPTS: [&str; 3] = ["vibelightPage(", "vibelightSearch(", "// vibelight: "];

fn is_anchor(line: &str) -> bool {
    line.starts_with("<a id=\"crate.") && line.ends_with("\"></a>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_on_disk_and_another_stamp_count_as_the_same() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a.md");
        fs::write(&path, "one\r\ntwo\r\n").unwrap();
        assert!(!changed(&path, "one\ntwo\n"));
        assert!(changed(&path, "one\nthree\n"));
        assert!(changed(&tmp.path().join("gone.md"), "x"));
        let stamped = |time: &str| format!("{}\n\nbody\n", stamp::markdown(time));
        fs::write(&path, stamped("2026-01-01T00:00:00Z")).unwrap();
        assert!(!changed(&path, &stamped("2027-06-30T12:00:00Z")));
        assert!(changed(
            &path,
            "*Updated 2027-06-30T12:00:00Z*\n\nchanged\n"
        ));
    }

    #[test]
    fn only_outlines_that_the_run_does_not_write_are_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let outline = "<a id=\"crate.old\"></a>\n# old\n";
        let stamped = "*Updated 2026-01-01T00:00:00Z*\n\n<a id=\"crate.new\"></a>\n# new\n";
        let page = "<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n<html>\n";
        fs::write(dir.join("kept.md"), "<a id=\"crate.kept\"></a>\n").unwrap();
        fs::write(dir.join("old.md"), outline).unwrap();
        fs::write(dir.join("new.md"), stamped).unwrap();
        fs::write(dir.join("README.md"), "# Notes\n").unwrap();
        fs::write(dir.join("notes.md"), "*Updated soon*\n\n# Notes\n").unwrap();
        fs::write(dir.join("index.md"), "# Outlines\n").unwrap();
        fs::write(dir.join("old.html"), page).unwrap();
        fs::write(dir.join("other.html"), "<!DOCTYPE html>\n").unwrap();
        fs::create_dir(dir.join("html")).unwrap();
        fs::write(dir.join("html").join("deep.md"), outline).unwrap();
        let kept_path = dir.join("kept.md");
        let kept: BTreeSet<&Path> = [kept_path.as_path()].into();
        assert_eq!(
            stale(dir, "md", &kept),
            [dir.join("new.md"), dir.join("old.md")]
        );
        assert_eq!(stale(dir, "html", &kept), [dir.join("old.html")]);
    }

    #[test]
    fn the_index_of_a_run_counts_and_another_file_of_its_name_does_not() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("index.md");
        fs::write(&path, index::render(&[])).unwrap();
        assert!(
            is_outline(&path),
            "an earlier release wrote its index in the base folder"
        );
        fs::write(&path, "# Outlines\n\nKept by hand.\n").unwrap();
        assert!(!is_outline(&path), "a person wrote this one");
    }

    #[test]
    fn the_script_files_of_an_earlier_release_are_stale_and_other_scripts_are_not() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let data = "vibelightPage(\"old\", \"old\", `\n<main>\n</main>\n`);\n";
        fs::write(dir.join("old.page.js"), data).unwrap();
        fs::write(dir.join("search.js"), "vibelightSearch({\"pages\":[]});\n").unwrap();
        fs::write(
            dir.join("vibelight.js"),
            "// vibelight: the outlines as one page that works from file:.\n",
        )
        .unwrap();
        fs::write(dir.join("app.js"), "console.log(1);\n").unwrap();
        let kept = BTreeSet::new();
        assert_eq!(
            stale(dir, "js", &kept),
            [
                dir.join("old.page.js"),
                dir.join("search.js"),
                dir.join("vibelight.js")
            ]
        );
    }

    #[test]
    fn new_and_old_outlines_count_and_malformed_unstamped_and_other_files_do_not() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let header =
            "{\"k\":\"vibelight-outline\",\"schema\":1,\"crate\":\"a\",\"target\":\"lib\"}";
        let cases = [
            (
                "new.md",
                format!("*Updated 2026-01-01T00:00:00Z*\n{header}\n"),
                true,
            ),
            (
                "old.md",
                "*Updated 2026-01-01T00:00:00Z*\n\n<a id=\"crate.a\"></a>\n# a\n".to_owned(),
                true,
            ),
            (
                "oldest.md",
                "<a id=\"crate.a\"></a>\n# a\n".to_owned(),
                true,
            ),
            (
                "malformed.md",
                "*Updated 2026-01-01T00:00:00Z*\n{\"k\":\"lead\"}\n".to_owned(),
                false,
            ),
            ("unstamped.md", format!("{header}\n"), false),
            ("late.md", format!("*Updated soon*\n\n{header}\n"), false),
            ("notes.md", "# Notes\n\nKept by hand.\n".to_owned(), false),
            ("empty.md", String::new(), false),
        ];
        for (name, text, outline) in &cases {
            let path = dir.join(name);
            fs::write(&path, text).unwrap();
            assert_eq!(is_outline(&path), *outline, "{name}");
        }
        let kept_path = dir.join("new.md");
        let kept: BTreeSet<&Path> = [kept_path.as_path()].into();
        assert_eq!(
            stale(dir, "md", &kept),
            [dir.join("old.md"), dir.join("oldest.md")]
        );
    }
}
