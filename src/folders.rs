//! Writes a run's files into its folders. A file whose content is the same
//! is left alone, so its stamp keeps the time its content last changed.
//! The folders hold only what the last run wrote: an outline that no crate
//! of the run writes any more is removed, and every other file is left
//! alone. Vibelight's own folder also gets a `.gitignore`, so no outline is
//! ever committed.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::settings::Plan;
use crate::stamp;

/// The `.gitignore` of vibelight's own folder: the outlines, the HTML
/// pages, and the index.
const GITIGNORE: &str = "*.md\n*.html\n";

/// What writing a run did beside writing its files.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Written {
    /// The `.gitignore` the run created.
    pub(crate) gitignore: Option<PathBuf>,
    /// The outlines the run removed, sorted.
    pub(crate) removed: Vec<PathBuf>,
}

/// Creates the folders, writes every file whose content changed, gives
/// vibelight's own folder its `.gitignore` when it has none, and removes
/// the outlines no crate of the run writes.
///
/// # Errors
/// Returns an error when a folder cannot be created or a file cannot be
/// written or removed.
pub(crate) fn write(files: &[(PathBuf, String)], plan: &Plan) -> Result<Written> {
    let markdown = plan.markdown_dir().to_path_buf();
    let html = plan.html().dir;
    for dir in [&markdown, &html] {
        fs::create_dir_all(dir)
            .with_context(|| format!("cannot create output directory {}", dir.display()))?;
    }
    let mut written = Written::default();
    let ignore = markdown.join(".gitignore");
    if plan.owned && !ignore.exists() {
        fs::write(&ignore, GITIGNORE)
            .with_context(|| format!("cannot write {}", ignore.display()))?;
        written.gitignore = Some(ignore);
    }
    for (path, text) in files {
        if changed(path, text) {
            fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))?;
        }
    }
    let kept: BTreeSet<&Path> = files.iter().map(|(path, _)| path.as_path()).collect();
    written.removed = stale(&markdown, "md", &kept);
    written.removed.extend(stale(&html, "html", &kept));
    written.removed.sort();
    for path in &written.removed {
        fs::remove_file(path).with_context(|| format!("cannot remove {}", path.display()))?;
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
/// stamp comment and then the doctype. A markdown outline of an older
/// release counts too, so a stale one is still cleaned up: a stamp line, a
/// blank line, and a crate anchor, or the anchor first.
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
        || is_anchor(&head[0])
        || (stamped && head[1].is_empty() && is_anchor(&head[2]))
}

/// How the header record on a markdown outline's second line starts.
const HEADER: &str = "{\"k\":\"vibelight-outline\",";

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
