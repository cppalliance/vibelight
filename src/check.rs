//! Compares the files a run would write with the files on disk, for the
//! check switches, and finds the outline files that no crate of a run
//! writes any more.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::stamp;

/// One way the files on disk differ from what a run would write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Problem {
    /// The file differs; the number is the 1-based line of the first
    /// difference.
    Differs(PathBuf, usize),
    Missing(PathBuf),
    /// An outline file that no crate of the run writes.
    Stale(PathBuf),
}

/// Compares the text a run would write with the file on disk. A CRLF line
/// ending on disk counts as LF, since a checkout on Windows can write one,
/// and the stamps are masked, so a file that differs only in its stamp
/// matches.
pub(crate) fn compare(path: &Path, expected: &str) -> Option<Problem> {
    let Ok(bytes) = fs::read(path) else {
        return Some(Problem::Missing(path.to_path_buf()));
    };
    let actual = stamp::mask(&String::from_utf8_lossy(&bytes).replace("\r\n", "\n"));
    let expected = stamp::mask(expected);
    (actual != expected)
        .then(|| Problem::Differs(path.to_path_buf(), first_difference(&expected, &actual)))
}

/// Returns the 1-based number of the first line where two texts differ.
pub(crate) fn first_difference(expected: &str, actual: &str) -> usize {
    let mut expected = expected.split('\n');
    let mut actual = actual.split('\n');
    let mut line = 1;
    loop {
        match (expected.next(), actual.next()) {
            (Some(a), Some(b)) if a == b => line += 1,
            _ => return line,
        }
    }
}

/// Returns the markdown outlines directly in `dir` that `written` does not
/// hold, sorted. A file counts only when it starts the way every markdown
/// file vibelight writes does, with a stamp line, a blank line, and a crate
/// anchor, or the way older releases wrote them, with the anchor first, so
/// other markdown files that share the folder never count.
pub(crate) fn stale(dir: &Path, written: &BTreeSet<PathBuf>) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|e| e == "md")
                && !written.contains(path)
                && is_outline(path)
        })
        .collect();
    found.sort();
    found
}

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
    is_anchor(&head[0]) || (stamp::is_line(&head[0]) && head[1].is_empty() && is_anchor(&head[2]))
}

fn is_anchor(line: &str) -> bool {
    line.starts_with("<a id=\"crate.") && line.ends_with("\"></a>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_difference_is_the_first_unequal_line() {
        assert_eq!(first_difference("a\nb\nc\n", "a\nx\nc\n"), 2);
        assert_eq!(first_difference("a\nb\n", "a\nb\nc\n"), 3);
        assert_eq!(first_difference("a\n", ""), 1);
    }

    #[test]
    fn crlf_on_disk_matches_and_a_missing_file_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a.md");
        fs::write(&path, "one\r\ntwo\r\n").unwrap();
        assert_eq!(compare(&path, "one\ntwo\n"), None);
        assert_eq!(
            compare(&path, "one\nthree\n"),
            Some(Problem::Differs(path.clone(), 2))
        );
        let gone = tmp.path().join("gone.md");
        assert_eq!(compare(&gone, "x"), Some(Problem::Missing(gone.clone())));
    }

    #[test]
    fn a_file_that_differs_only_in_its_stamp_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a.md");
        let stamped = |time: &str| format!("{}\n\nbody\n", stamp::markdown(time));
        fs::write(&path, stamped("2026-01-01T00:00:00Z")).unwrap();
        assert_eq!(compare(&path, &stamped("2027-06-30T12:00:00Z")), None);
        assert_eq!(compare(&path, &stamped(stamp::PLACEHOLDER)), None);
        assert_eq!(
            compare(&path, "*Updated 2027-06-30T12:00:00Z*\n\nchanged\n"),
            Some(Problem::Differs(path.clone(), 3))
        );
    }

    #[test]
    fn only_outline_files_that_the_run_does_not_write_are_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let outline = "<a id=\"crate.old\"></a>\n# old\n";
        let stamped = "*Updated 2026-01-01T00:00:00Z*\n\n<a id=\"crate.new\"></a>\n# new\n";
        fs::write(dir.join("kept.md"), "<a id=\"crate.kept\"></a>\n").unwrap();
        fs::write(dir.join("old.md"), outline).unwrap();
        fs::write(dir.join("new.md"), stamped).unwrap();
        fs::write(dir.join("README.md"), "# Notes\n").unwrap();
        fs::write(dir.join("notes.md"), "*Updated soon*\n\n# Notes\n").unwrap();
        fs::write(dir.join("old.html"), outline).unwrap();
        fs::create_dir(dir.join("html")).unwrap();
        fs::write(dir.join("html").join("deep.md"), outline).unwrap();
        let written: BTreeSet<PathBuf> = [dir.join("kept.md")].into();
        assert_eq!(
            stale(dir, &written),
            [dir.join("new.md"), dir.join("old.md")]
        );
    }
}
