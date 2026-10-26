//! The exclude setting: a config or a flag takes crates out of whatever the
//! run selects, an excluded crate is never read and writes nothing, and a
//! glob that matches no crate is a note, so a typo cannot go unseen.

use std::fs;

use crate::config::{garden, lines, vibelight_in};
use crate::support::{fixtures, read_outputs};

const NOTE: &str = "note: skipped testonly: it has only tests, examples, or benches";
const FLAT: &str = "--no-source-links";

fn names(files: &[(String, String)]) -> Vec<&str> {
    files.iter().map(|(name, _)| name.as_str()).collect()
}

/// The garden's goldens, less the files of the crates named.
fn goldens_without(stems: &[&str]) -> Vec<(String, String)> {
    let goldens = fixtures()
        .join("workspaces")
        .join("garden")
        .join("expected");
    read_outputs(&goldens)
        .into_iter()
        .filter(|(name, _)| {
            !stems
                .iter()
                .any(|stem| name.starts_with(&format!("{stem}.")))
        })
        .collect()
}

#[test]
fn a_workspace_run_without_one_member_changes_no_other_file() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facade\"]\n");
    let result = vibelight_in(&repo, &["-o", "less", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(lines(&result.stderr), [NOTE]);
    assert_eq!(lines(&result.stdout).len(), 4, "{:?}", result.stdout);
    let less = read_outputs(&repo.join("less"));
    assert_eq!(
        names(&less),
        ["alpha.html", "alpha.md", "beta.html", "beta.md"]
    );
    assert!(less == goldens_without(&["facade"]), "a kept file changed");
    let all = vibelight_in(&repo, &["--workspace", "-o", "all", FLAT]);
    assert!(all.status.success(), "{:?}", lines(&all.stderr));
    let all = read_outputs(&repo.join("all"));
    assert_eq!(all.len(), 6);
    let kept: Vec<(String, String)> = all
        .into_iter()
        .filter(|(name, _)| !name.starts_with("facade."))
        .collect();
    assert!(kept == less, "the exclude changed a file it kept");
}

#[test]
fn an_excluded_crate_is_never_read() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facade\"]\n");
    fs::write(
        repo.join("crates").join("facade").join("Cargo.toml"),
        "[package\nname = broken",
    )
    .unwrap();
    let result = vibelight_in(&repo, &["-o", "less", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(read_outputs(&repo.join("less")).len(), 4);
    let result = vibelight_in(&repo, &["--workspace", "-o", "all", FLAT]);
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn an_exclude_removes_a_followed_dependency_and_leaves_it_plain() {
    let (_tmp, repo) = garden(
        "crates = [\"crates/alpha\"]\nfollow = [\"crates/d*\", \"vendor/*\"]\n\
         exclude = [\"vendor/gamma\"]\n",
    );
    let result = vibelight_in(&repo, &["-o", "out", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(result.stderr.is_empty(), "{:?}", lines(&result.stderr));
    let out = read_outputs(&repo.join("out"));
    assert_eq!(
        names(&out),
        ["alpha.html", "alpha.md", "delta.html", "delta.md"]
    );
    let by_hand = ["crates/alpha", "--follow", "crates/d*", "-o", "base", FLAT];
    assert!(vibelight_in(&repo, &by_hand).status.success());
    assert!(
        read_outputs(&repo.join("base")) == out,
        "excluding gamma differs from never following it"
    );
    let alpha = &out[0].1;
    assert!(
        alpha.contains("<a class=\"cross\" href=\"delta.html#type.delta.Stray\">delta::Stray</a>"),
        "{alpha}"
    );
    assert!(!alpha.contains("gamma.html"), "{alpha}");
    assert!(alpha.contains("gamma::Vendored,"), "gamma stays plain text");
}

#[test]
fn an_excluded_crate_is_not_followed_through() {
    let (_tmp, repo) = garden(
        "crates = [\"crates/facade\"]\nfollow = [\"crates/*\"]\nexclude = [\"crates/alpha\"]\n",
    );
    let result = vibelight_in(&repo, &["-o", "out", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(result.stderr.is_empty(), "{:?}", lines(&result.stderr));
    assert_eq!(
        names(&read_outputs(&repo.join("out"))),
        ["beta.html", "beta.md", "facade.html", "facade.md"]
    );
    let by_hand = ["crates/facade", "--follow", "crates/*", "-o", "base", FLAT];
    assert!(vibelight_in(&repo, &by_hand).status.success());
    assert_eq!(
        names(&read_outputs(&repo.join("base"))).len(),
        8,
        "alpha, beta, delta, and facade without the exclude"
    );
}

#[test]
fn a_named_crate_that_is_also_excluded_is_left_out_with_a_note() {
    let (_tmp, repo) = garden("");
    let args = [
        "crates/alpha",
        "crates/beta",
        "--exclude",
        "crates/beta",
        "-o",
        "out",
        FLAT,
    ];
    let result = vibelight_in(&repo, &args);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        ["note: crates/beta is named as a crate and also excluded, so it is left out"]
    );
    assert_eq!(
        names(&read_outputs(&repo.join("out"))),
        ["alpha.html", "alpha.md"]
    );
}

#[test]
fn a_config_crates_list_is_cut_by_the_exclude_whether_it_names_or_globs() {
    let (_tmp, repo) = garden(
        "crates = [\"crates/alpha\", \"crates/beta\", \"crates/f*\"]\n\
         exclude = [\"crates/beta\", \"crates/facade\"]\n",
    );
    let result = vibelight_in(&repo, &["-o", "out", FLAT]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        ["note: crates/beta is named as a crate and also excluded, so it is left out"]
    );
    assert_eq!(
        names(&read_outputs(&repo.join("out"))),
        ["alpha.html", "alpha.md"]
    );
}
