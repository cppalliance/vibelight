//! Runs several crates at once: the goldens of every crate, the same files
//! whatever order the crates are named in, follow globs, and links that all
//! resolve, same-page and cross-page.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::support::{copy_dir, fixtures, has_ext, read_outputs};

fn workspaces() -> PathBuf {
    fixtures().join("workspaces")
}

fn text(path: &Path) -> &str {
    path.to_str().expect("a fixture path is UTF-8")
}

/// Runs the binary with `args`, writing into `out`.
fn vibelight(args: &[&str], out: &Path, links: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vibelight"));
    command.args(args).arg("-o").arg(out);
    if !links {
        command.arg("--no-source-links");
    }
    command.output().expect("the vibelight binary runs")
}

fn goldens() -> Vec<(String, String)> {
    read_outputs(&workspaces().join("garden").join("expected"))
}

#[test]
fn a_workspace_run_matches_the_goldens_of_every_crate() {
    let garden = workspaces().join("garden");
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight(&["--workspace", text(&garden)], tmp.path(), false);
    let stderr = String::from_utf8(result.stderr).expect("stderr is UTF-8");
    assert!(result.status.success(), "{stderr}");
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        ["note: skipped testonly: it has only tests, examples, or benches"]
    );
    let written = read_outputs(tmp.path());
    let expected = goldens();
    let names = |files: &[(String, String)]| -> Vec<String> {
        files.iter().map(|(name, _)| name.clone()).collect()
    };
    assert_eq!(names(&written), names(&expected));
    for ((file, got), (_, want)) in written.iter().zip(&expected) {
        assert!(
            got == want,
            "{file} differs from its golden\n--- expected\n{want}\n--- actual\n{got}"
        );
    }
}

#[test]
fn naming_the_crates_in_reverse_gives_the_same_files() {
    let crates = workspaces().join("garden").join("crates");
    let dirs: Vec<PathBuf> = ["alpha", "beta", "facade"]
        .iter()
        .map(|name| crates.join(name))
        .collect();
    let forward: Vec<&str> = dirs.iter().map(|dir| text(dir)).collect();
    let reversed: Vec<&str> = forward.iter().rev().copied().collect();
    let first = tempfile::tempdir().expect("a temporary folder can be made");
    let second = tempfile::tempdir().expect("a temporary folder can be made");
    assert!(vibelight(&forward, first.path(), false).status.success());
    assert!(vibelight(&reversed, second.path(), false).status.success());
    let files = read_outputs(first.path());
    assert!(
        files == read_outputs(second.path()),
        "the order changed a file"
    );
    assert!(
        files == goldens(),
        "named crates differ from the workspace run"
    );
}

#[test]
fn follow_globs_pull_in_only_the_matching_path_dependencies() {
    let alpha = workspaces().join("garden").join("crates").join("alpha");
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight(&[text(&alpha), "--follow", "crates/b*"], tmp.path(), false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let expected: Vec<(String, String)> = goldens()
        .into_iter()
        .filter(|(name, _)| name.starts_with("alpha.") || name.starts_with("beta."))
        .collect();
    assert!(
        read_outputs(tmp.path()) == expected,
        "alpha and beta only, and the same files as in the larger run"
    );
}

/// Returns each link target in an outline: `href` values in HTML, and the
/// targets of `[text](target)` links in markdown.
fn hrefs(file: &str, page: &str) -> Vec<String> {
    let (open, close) = if has_ext(file, "html") {
        ("href=\"", '"')
    } else {
        ("](", ')')
    };
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(at) = rest.find(open) {
        let after = &rest[at + open.len()..];
        let end = after.find(close).expect("a link closes");
        out.push(after[..end].replace("&amp;", "&"));
        rest = &after[end..];
    }
    out
}

/// Returns every anchor id a page defines.
fn ids(page: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = page;
    while let Some(at) = rest.find(" id=\"") {
        let after = &rest[at + 5..];
        let end = after.find('"').expect("an id closes");
        out.insert(after[..end].to_owned());
        rest = &after[end..];
    }
    out
}

#[test]
fn every_link_of_a_multi_crate_run_reaches_a_file_and_id_that_exist() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let copy = tmp.path().join("garden");
    copy_dir(&workspaces().join("garden"), &copy);
    let out = copy.join("out");
    assert!(
        vibelight(&["--workspace", text(&copy)], &out, true)
            .status
            .success()
    );
    let pages = read_outputs(&out);
    let (mut same, mut cross, mut source) = (0, 0, 0);
    for (file, page) in &pages {
        for href in hrefs(file, page) {
            let (target, fragment) = href.split_once('#').unwrap_or((href.as_str(), ""));
            if target.is_empty() {
                assert!(
                    ids(page).contains(fragment),
                    "{file}: #{fragment} is missing"
                );
                same += 1;
            } else if let Some((_, other)) = pages.iter().find(|(name, _)| name == target) {
                assert!(ids(other).contains(fragment), "{file}: {href} is missing");
                cross += 1;
            } else {
                assert!(out.join(target).is_file(), "{file}: {href} names no file");
                source += 1;
            }
        }
    }
    assert!(
        same > 20 && cross >= 8 && source > 10,
        "{same} {cross} {source}"
    );
}

#[test]
fn two_crates_that_would_write_one_file_name_are_an_error() {
    let clash = workspaces().join("clash");
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight(&["--workspace", text(&clash)], tmp.path(), false);
    assert_eq!(result.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("one and two would both write same.md and same.html"),
        "{stderr}"
    );
}

#[test]
fn one_named_crate_with_nothing_to_outline_still_fails() {
    let testonly = workspaces().join("garden").join("crates").join("testonly");
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight(&[text(&testonly)], tmp.path(), false);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("not a crate"));
}
