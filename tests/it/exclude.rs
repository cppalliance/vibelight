//! The config's exclude globs: they take crates out of whatever the config
//! selects, an excluded crate is never read and writes nothing, the globs
//! are measured from the config's folder, and a glob that matches no crate
//! is a note, so a typo cannot go unseen. The markdown is contextual to the
//! run, so leaving a crate out changes the leads and facts of the crates
//! it used. A page depends on its own crate and the crates it links to, so
//! each kept crate's HTML stays the same in these cases, which leave out a
//! downstream or followed crate, but not in general.

use std::fs;

use crate::config::{MEMBERS, NOTE, garden, goldens, lines, names, vibelight_in};
use crate::support::{has_ext, read_plain, read_run};

/// The garden's goldens, less the files of the crates named.
fn goldens_without(stems: &[&str]) -> Vec<(String, String)> {
    goldens()
        .into_iter()
        .filter(|(name, _)| {
            !stems
                .iter()
                .any(|stem| name.starts_with(&format!("{stem}.")))
        })
        .collect()
}

/// The HTML pages among a run's files.
fn pages(files: Vec<(String, String)>) -> Vec<(String, String)> {
    files
        .into_iter()
        .filter(|(name, _)| has_ext(name, "html"))
        .collect()
}

/// Returns the text of one file of a run.
fn text<'f>(files: &'f [(String, String)], name: &str) -> &'f str {
    &files
        .iter()
        .find(|(file, _)| file == name)
        .unwrap_or_else(|| panic!("{name} is written"))
        .1
}

#[test]
fn a_workspace_run_without_one_member_changes_no_other_page_and_only_its_leads() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facade\"]\n");
    let result = vibelight_in(&repo, &["-o", "less"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(lines(&result.stderr), [NOTE]);
    assert_eq!(lines(&result.stdout).len(), 6, "{:?}", result.stdout);
    let less = read_plain(&repo.join("less"));
    assert_eq!(
        names(&less),
        ["alpha.html", "alpha.md", "beta.html", "beta.md"]
    );
    assert!(
        pages(less.clone()) == pages(goldens_without(&["facade"])),
        "a kept page changed"
    );
    fs::write(repo.join("vibelight.toml"), MEMBERS).unwrap();
    let all = vibelight_in(&repo, &["-o", "all"]);
    assert!(all.status.success(), "{:?}", lines(&all.stderr));
    let all = read_plain(&repo.join("all"));
    assert_eq!(all.len(), 6);
    let kept: Vec<(String, String)> = all
        .iter()
        .filter(|(name, _)| !name.starts_with("facade."))
        .cloned()
        .collect();
    assert!(
        pages(kept) == pages(less.clone()),
        "the exclude changed a page it kept"
    );
    let cross = "{\"k\":\"lead\",\"id\":\"one-observed-trait-impl\",\"mode\":\"resolved\",\
                 \"subjects\":[\"Reads\",\"::facade::Panel\"],";
    let with = text(&all, "alpha.md");
    assert!(with.contains(cross), "{with}");
    assert!(with.contains("\"reexported_by\":[\"::facade\"],\"crates\":[\"facade\"]"));
    let without = text(&less, "alpha.md");
    assert!(!without.contains("facade"), "{without}");
    assert!(
        without.contains("\"subjects\":[\"Meter\",\"Meter::gauge\",\"Meter::stray\","),
        "without facade, nothing names Meter: {without}"
    );
    assert!(text(&all, "beta.md").contains("\"crates\":[\"alpha\",\"facade\"]"));
    assert!(text(&less, "beta.md").contains("\"crates\":[\"alpha\"]"));
}

#[test]
fn an_excluded_crate_is_never_read() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facade\"]\n");
    fs::write(
        repo.join("crates").join("facade").join("Cargo.toml"),
        "[package\nname = broken",
    )
    .unwrap();
    let result = vibelight_in(&repo, &["-o", "less"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(read_run(&repo.join("less")).len(), 4);
    fs::write(repo.join("vibelight.toml"), MEMBERS).unwrap();
    let result = vibelight_in(&repo, &["-o", "all"]);
    assert_eq!(result.status.code(), Some(1));
}

#[test]
fn an_exclude_removes_a_followed_dependency_and_leaves_it_plain() {
    let (_tmp, repo) = garden(
        "crates = [\"crates/alpha\"]\nfollow = [\"crates/d*\", \"vendor/*\"]\n\
         exclude = [\"vendor/gamma\"]\n",
    );
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(result.stderr.is_empty(), "{:?}", lines(&result.stderr));
    let out = read_run(&repo.join("out"));
    assert_eq!(
        names(&out),
        ["alpha.html", "alpha.md", "delta.html", "delta.md"]
    );
    fs::write(
        repo.join("vibelight.toml"),
        "crates = [\"crates/alpha\"]\nfollow = [\"crates/d*\"]\n",
    )
    .unwrap();
    assert!(vibelight_in(&repo, &["-o", "base"]).status.success());
    assert!(
        read_plain(&repo.join("base")) == read_plain(&repo.join("out")),
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
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(result.stderr.is_empty(), "{:?}", lines(&result.stderr));
    assert_eq!(
        names(&read_run(&repo.join("out"))),
        ["beta.html", "beta.md", "facade.html", "facade.md"]
    );
    fs::write(
        repo.join("vibelight.toml"),
        "crates = [\"crates/facade\"]\nfollow = [\"crates/*\"]\n",
    )
    .unwrap();
    assert!(vibelight_in(&repo, &["-o", "base"]).status.success());
    assert_eq!(
        read_run(&repo.join("base")).len(),
        8,
        "alpha, beta, delta, and facade without the exclude"
    );
}

#[test]
fn a_config_crates_list_is_cut_by_the_exclude_whether_it_names_or_globs() {
    let (_tmp, repo) = garden(
        "crates = [\"crates/alpha\", \"crates/beta\", \"crates/f*\"]\n\
         exclude = [\"crates/beta\", \"crates/facade\"]\n",
    );
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        ["note: crates/beta is named as a crate and also excluded, so it is left out"]
    );
    assert_eq!(
        names(&read_run(&repo.join("out"))),
        ["alpha.html", "alpha.md"]
    );
}

#[test]
fn a_glob_that_matches_no_crate_is_a_note_and_removes_nothing() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/facad\", \"crates/zeta\"]\n");
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert_eq!(result.status.code(), Some(0), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        [
            "note: exclude pattern crates/facad matches no crate",
            "note: exclude pattern crates/zeta matches no crate",
            NOTE,
        ]
    );
    assert!(
        read_plain(&repo.join("out")) == goldens(),
        "a mistyped glob changed a file"
    );
}

#[test]
fn globs_take_every_glob_character() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/?eta\", \"**/f*\"]\n");
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(lines(&result.stderr), [NOTE]);
    assert_eq!(
        names(&read_run(&repo.join("out"))),
        ["alpha.html", "alpha.md"]
    );
}

#[test]
fn excluding_every_crate_is_the_empty_run_error() {
    let (_tmp, repo) = garden("workspace = true\nexclude = [\"crates/*\"]\n");
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        lines(&result.stderr),
        ["error: no crate of the run has anything to outline"]
    );
    assert!(!repo.join("out").exists());
}

#[test]
fn the_globs_of_a_config_are_measured_from_its_own_folder() {
    let (_tmp, repo) = garden("");
    let crates = repo.join("crates");
    fs::write(
        crates.join("vibelight.toml"),
        "workspace = \"..\"\nexclude = [\"beta\", \"crates/alpha\"]\n",
    )
    .unwrap();
    let out = repo.join("out");
    let result = vibelight_in(&crates, &["-o", out.to_str().unwrap()]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        ["note: exclude pattern crates/alpha matches no crate", NOTE]
    );
    assert_eq!(
        names(&read_run(&out)),
        ["alpha.html", "alpha.md", "facade.html", "facade.md"]
    );
}

#[test]
fn a_config_that_gives_exclude_the_wrong_type_is_an_error_that_names_the_key() {
    let (_tmp, repo) = garden("workspace = true\nexclude = \"crates/*\"\n");
    let result = vibelight_in(&repo, &[]);
    assert_eq!(result.status.code(), Some(1));
    let stderr = lines(&result.stderr);
    assert_eq!(stderr.len(), 1, "{stderr:?}");
    assert!(
        stderr[0].ends_with("vibelight.toml: `exclude` is a list of strings"),
        "{stderr:?}"
    );
}
