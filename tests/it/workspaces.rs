//! Runs several crates at once: the goldens of every crate, the same files
//! whatever order the crates are named in, follow globs, and links that all
//! resolve, same-page and cross-page.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use crate::config::{MEMBERS, lines, vibelight_in};
use crate::support::{self, copy_dir, fixtures, has_ext, read_outputs, read_plain};

fn workspaces() -> PathBuf {
    fixtures().join("workspaces")
}

fn text(path: &Path) -> &str {
    path.to_str().expect("a fixture path is UTF-8")
}

/// Runs the binary with `args`, writing into `out`.
fn vibelight(args: &[&str], out: &Path) -> Output {
    support::vibelight()
        .args(args)
        .arg("-o")
        .arg(out)
        .output()
        .expect("the vibelight binary runs")
}

/// A copy of a workspace fixture whose config outlines its members.
fn copy(name: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let repo = tmp.path().join(name);
    copy_dir(&workspaces().join(name), &repo);
    fs::write(repo.join("vibelight.toml"), MEMBERS).expect("the config can be written");
    (tmp, repo)
}

fn goldens(name: &str) -> Vec<(String, String)> {
    read_outputs(&workspaces().join(name).join("expected"))
}

#[test]
fn a_workspace_run_matches_the_goldens_of_every_crate() {
    let (_tmp, repo) = copy("garden");
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert_eq!(
        lines(&result.stderr),
        ["note: skipped testonly: it has only tests, examples, or benches"]
    );
    let written = read_plain(&repo.join("out"));
    let expected = goldens("garden");
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
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let (first, second) = (tmp.path().join("first"), tmp.path().join("second"));
    assert!(vibelight(&forward, &first).status.success());
    assert!(vibelight(&reversed, &second).status.success());
    let files = read_plain(&first);
    assert!(files == read_plain(&second), "the order changed a file");
    assert!(
        files == goldens("garden"),
        "named crates differ from the workspace run"
    );
}

#[test]
fn follow_globs_pull_in_only_the_matching_path_dependencies() {
    let (_tmp, repo) = copy("garden");
    fs::write(repo.join("vibelight.toml"), "follow = [\"crates/b*\"]\n").unwrap();
    let result = vibelight_in(&repo, &["crates/alpha", "-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let pages = |files: Vec<(String, String)>| -> Vec<(String, String)> {
        files
            .into_iter()
            .filter(|(name, _)| has_ext(name, "html"))
            .collect()
    };
    let written = read_plain(&repo.join("out"));
    let names: Vec<&str> = written.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["alpha.html", "alpha.md", "beta.html", "beta.md"]);
    let expected: Vec<(String, String)> = pages(goldens("garden"))
        .into_iter()
        .filter(|(name, _)| name.starts_with("alpha.") || name.starts_with("beta."))
        .collect();
    assert!(
        pages(written) == expected,
        "alpha and beta only, and the same pages as in the larger run; the markdown is \
         contextual to the run"
    );
}

/// Returns each link target in an HTML page: its `href` values.
fn hrefs(page: &str) -> Vec<String> {
    let open = "href=\"";
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(at) = rest.find(open) {
        let after = &rest[at + open.len()..];
        let end = after.find('"').expect("a link closes");
        out.push(after[..end].replace("&amp;", "&"));
        rest = &after[end..];
    }
    out
}

/// Returns the source file each location of a markdown outline names, as
/// a path from the outline's folder: the `root` of the outline of the
/// target that holds the file, joined with the file.
fn sources(page: &str, pages: &[(String, PathBuf, String)]) -> Vec<PathBuf> {
    let root_of = |stem: Option<&str>| match stem {
        None => support::root_of(page),
        Some(stem) => {
            let file = format!("{stem}.md");
            let (_, _, other) = (pages.iter().find(|(name, _, _)| *name == file))
                .unwrap_or_else(|| panic!("no {file} for a location"));
            support::root_of(other)
        }
    };
    (support::locations(page).iter())
        .map(|loc| {
            let (stem, file) = support::split_location(loc);
            Path::new(&root_of(stem)).join(file)
        })
        .collect()
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

/// How many links of each kind a run's pages hold.
struct Links {
    same: usize,
    cross: usize,
    source: usize,
    /// Links, same-page or cross-page, to a function, constant, or static.
    values: usize,
}

/// Runs a copy of a workspace fixture and checks that every link of every
/// outline reaches a file and an id that exist, from the folder the outline
/// sits in.
fn every_link_lands(name: &str) -> Links {
    let (_tmp, repo) = copy(name);
    let out = repo.join("out");
    assert!(vibelight_in(&repo, &["-o", "out"]).status.success());
    let mut pages: Vec<(String, PathBuf, String)> = Vec::new();
    for folder in [out.clone(), out.join("html")] {
        for (file, page) in read_outputs(&folder) {
            if file != "index.md" {
                pages.push((file, folder.clone(), page));
            }
        }
    }
    let mut counts = Links {
        same: 0,
        cross: 0,
        source: 0,
        values: 0,
    };
    for (file, folder, page) in &pages {
        if has_ext(file, "md") {
            for source in sources(page, &pages) {
                assert!(
                    folder.join(&source).is_file(),
                    "{file}: {} names no file",
                    source.display()
                );
                counts.source += 1;
            }
            continue;
        }
        for href in hrefs(page) {
            let (target, fragment) = href.split_once('#').unwrap_or((href.as_str(), ""));
            if ["fn.", "const.", "static."]
                .iter()
                .any(|kind| fragment.starts_with(kind))
            {
                counts.values += 1;
            }
            if target.is_empty() {
                assert!(
                    ids(page).contains(fragment),
                    "{file}: #{fragment} is missing"
                );
                counts.same += 1;
            } else if let Some((_, _, other)) = pages.iter().find(|(name, _, _)| name == target) {
                // A link with no fragment names a whole page, as the index's
                // link to each outline does.
                assert!(
                    fragment.is_empty() || ids(other).contains(fragment),
                    "{file}: {href} is missing"
                );
                counts.cross += 1;
            } else {
                assert!(
                    folder.join(target).is_file(),
                    "{file}: {href} names no file"
                );
                counts.source += 1;
            }
        }
    }
    counts
}

#[test]
fn every_link_of_a_multi_crate_run_reaches_a_file_and_id_that_exist() {
    let found = every_link_lands("garden");
    assert!(
        found.same > 20 && found.cross >= 8 && found.source > 20,
        "{} {} {}",
        found.same,
        found.cross,
        found.source
    );
    assert_eq!(found.values, 0, "the garden links no function");
}

#[test]
fn a_function_re_exported_twice_links_to_the_crate_that_defines_it() {
    let (_tmp, repo) = copy("values");
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(
        result.stderr.is_empty(),
        "no name is unresolved or ambiguous"
    );
    let written = read_plain(&repo.join("out"));
    let expected = goldens("values");
    assert_eq!(written.len(), 6);
    for ((file, got), (_, want)) in written.iter().zip(&expected) {
        assert!(
            got == want,
            "{file} differs from its golden\n--- expected\n{want}\n--- actual\n{got}"
        );
    }
    let front = &written
        .iter()
        .find(|(name, _)| name == "front.html")
        .expect("front.html is written")
        .1;
    assert!(
        front.contains(
            "<a class=\"cross\" href=\"origin.html#fn.origin.gauge.measure\">relay::measure</a>"
        ),
        "the link goes past relay to the crate that defines the function"
    );
}

#[test]
fn every_value_link_of_the_values_run_lands_on_an_id_of_the_defining_page() {
    let found = every_link_lands("values");
    assert_eq!(found.values, 8, "four in relay and four in front");
    assert_eq!(
        found.cross, 11,
        "the eight value links, and the index's one link to each of the three pages"
    );
}

#[test]
fn naming_the_value_crates_in_reverse_gives_the_same_files() {
    let crates = workspaces().join("values").join("crates");
    let forward: Vec<String> = ["origin", "relay", "front"]
        .iter()
        .map(|name| text(&crates.join(name)).to_owned())
        .collect();
    let reversed: Vec<&str> = forward.iter().rev().map(String::as_str).collect();
    let forward: Vec<&str> = forward.iter().map(String::as_str).collect();
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let (first, second) = (tmp.path().join("first"), tmp.path().join("second"));
    assert!(vibelight(&forward, &first).status.success());
    assert!(vibelight(&reversed, &second).status.success());
    let files = read_plain(&first);
    assert!(files == read_plain(&second), "the order changed a file");
    assert!(
        files == goldens("values"),
        "named crates differ from the workspace run"
    );
}

#[test]
fn two_crates_that_would_write_one_file_name_are_an_error() {
    let (_tmp, repo) = copy("clash");
    let result = vibelight_in(&repo, &["-o", "out"]);
    assert_eq!(result.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("one and two would both write same.md and same.html"),
        "{stderr}"
    );
}

#[test]
fn external_records_show_the_doc_summary_of_an_item_from_another_crate_of_the_run() {
    let (_tmp, repo) = copy("garden");
    assert!(vibelight_in(&repo, &["-o", "out"]).status.success());
    let facade = fs::read_to_string(repo.join("out").join("facade.md")).unwrap();
    assert!(
        facade.contains(
            "{\"k\":\"external\",\"path\":\"alpha::Meter\",\"uses\":1,\"named_by\":[\"Panel\"],\
             \"summary\":\"A meter that reads a gauge of beta.\"}\n\
             {\"k\":\"external\",\"path\":\"alpha::Reads\",\"uses\":2,\
             \"named_by\":[\"crate\",\"Panel\"],\"summary\":\"A trait other crates implement.\"}\n"
        ),
        "{facade}"
    );
    let alpha = fs::read_to_string(repo.join("out").join("alpha.md")).unwrap();
    assert!(
        alpha.contains(
            "{\"k\":\"external\",\"path\":\"delta::Stray\",\"uses\":1,\"named_by\":[\"Meter\"]}\n"
        ),
        "a crate outside the run has no summary:\n{alpha}"
    );
}

#[test]
fn one_named_crate_with_nothing_to_outline_still_fails() {
    let testonly = workspaces().join("garden").join("crates").join("testonly");
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight(&[text(&testonly)], tmp.path());
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("not a crate"));
}

const ROOTED: &str = r#"*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"rooted","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":6,"to":9}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Root"],"evidence":"no hit of `Root` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Root","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1}
{"k":"doc","path":"crate","text":"The root package."}
{"k":"struct","path":"Root","rust":"pub struct Root","loc":"src/lib.rs:0"}
{"k":"doc","path":"Root","text":"A root type."}
"#;

#[test]
fn a_root_manifest_that_is_also_a_package_is_outlined_with_its_members() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let root = tmp.path().join("rooted");
    let package = |dir: &Path, name: &str, lib: &str, extra: &str| {
        fs::create_dir_all(dir.join("src")).expect("a crate folder can be made");
        let manifest = format!("[package]\nname = \"{name}\"\nedition = \"2024\"\n{extra}");
        fs::write(dir.join("Cargo.toml"), manifest).expect("a manifest can be written");
        fs::write(dir.join("src").join("lib.rs"), lib).expect("a source can be written");
    };
    let root_lib = "//! The root package.\n\n/// A root type.\npub struct Root;\n";
    let member_lib = "//! A member.\n\n/// A member type.\npub struct Leaf;\n";
    package(
        &root,
        "rooted",
        root_lib,
        "\n[workspace]\nmembers = [\"sub\"]\n",
    );
    package(&root.join("sub"), "sub", member_lib, "");
    let result = vibelight_in(&root.join("sub"), &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    assert!(
        root.join("vibelight.toml").is_file(),
        "the root got the config"
    );
    let files = read_plain(&root.join(".vibelight"));
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["rooted.html", "rooted.md", "sub.html", "sub.md"]);
    assert!(files[1].1 == ROOTED, "rooted.md differs:\n{}", files[1].1);
}
