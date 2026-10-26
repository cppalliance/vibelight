//! Runs several crates at once: the goldens of every crate, the same files
//! whatever order the crates are named in, follow globs, and links that all
//! resolve, same-page and cross-page.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use crate::support::{self, copy_dir, fixtures, has_ext, read_outputs};

fn workspaces() -> PathBuf {
    fixtures().join("workspaces")
}

fn text(path: &Path) -> &str {
    path.to_str().expect("a fixture path is UTF-8")
}

/// Runs the binary with `args`, writing into `out`.
fn vibelight(args: &[&str], out: &Path, links: bool) -> Output {
    let mut command = support::vibelight();
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

/// How many links of each kind a run's pages hold.
struct Links {
    same: usize,
    cross: usize,
    source: usize,
    /// Links, same-page or cross-page, to a function, constant, or static.
    values: usize,
}

/// Runs a copy of a workspace fixture with source links on and checks that
/// every link of every page reaches a file and an id that exist.
fn every_link_lands(name: &str) -> Links {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let copy = tmp.path().join(name);
    copy_dir(&workspaces().join(name), &copy);
    let out = copy.join("out");
    assert!(
        vibelight(&["--workspace", text(&copy)], &out, true)
            .status
            .success()
    );
    let pages = read_outputs(&out);
    let mut counts = Links {
        same: 0,
        cross: 0,
        source: 0,
        values: 0,
    };
    for (file, page) in &pages {
        for href in hrefs(file, page) {
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
            } else if let Some((_, other)) = pages.iter().find(|(name, _)| name == target) {
                assert!(ids(other).contains(fragment), "{file}: {href} is missing");
                counts.cross += 1;
            } else {
                assert!(out.join(target).is_file(), "{file}: {href} names no file");
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
        found.same > 20 && found.cross >= 8 && found.source > 10,
        "{} {} {}",
        found.same,
        found.cross,
        found.source
    );
    assert_eq!(found.values, 0, "the garden links no function");
}

#[test]
fn a_function_re_exported_twice_links_to_the_crate_that_defines_it() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let values = workspaces().join("values");
    let result = vibelight(&["--workspace", text(&values)], tmp.path(), false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).is_empty(),
        "no name is unresolved or ambiguous"
    );
    let written = read_outputs(tmp.path());
    let expected = read_outputs(&values.join("expected"));
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
    assert_eq!(found.cross, 8, "every value link here crosses to origin");
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
        files == read_outputs(&workspaces().join("values").join("expected")),
        "named crates differ from the workspace run"
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
fn external_items_show_the_doc_line_of_an_item_from_another_crate_of_the_run() {
    let garden = workspaces().join("garden");
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight(
        &["--workspace", text(&garden), "--facts"],
        tmp.path(),
        false,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let facade = fs::read_to_string(tmp.path().join("facade.md")).expect("facade.md was written");
    assert!(
        facade.contains(
            "## External items\n\n\
             - `alpha`\n  \
             - `alpha::Meter`, 1 use by [Panel](#type.facade.Panel)\n    \
             - A meter that reads a gauge of beta.\n  \
             - `alpha::Reads`, 2 uses by [Panel](#type.facade.Panel), [facade](#mod.facade)\n    \
             - A trait other crates implement.\n"
        ),
        "{facade}"
    );
    let alpha = fs::read_to_string(tmp.path().join("alpha.md")).expect("alpha.md was written");
    assert!(
        alpha.contains("- `delta::Stray`, 1 use by [Meter](#type.alpha.Meter)\n- `gamma`"),
        "a crate outside the run has no doc line:\n{alpha}"
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

const ROOTED: &str = "*Updated 2026-01-01T00:00:00Z*

<a id=\"crate.rooted\"></a>
# rooted

```
//! The root package.
```

<a id=\"legend\"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- A type's block holds the members of all its inherent impls, wherever they are written, so a module that holds only impls shows nothing but its docs.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A `// internal` or `// private` line above a type's header gives the type's own visibility; a public type has none.
- A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.
- References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.
- `constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.
- `Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.
- A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top gathers them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Root](#type.rooted.Root)
- [Modules](#modules)
  - [rooted](#mod.rooted)

**MACROS:** none

<a id=\"surface\"></a>
## Public surface

- [Root](#type.rooted.Root)

<a id=\"unconnected\"></a>
## Unconnected types

<a id=\"type.rooted.Root\"></a>
### Root

```
/// A root type.
struct Root;
```

<a id=\"modules\"></a>
## Modules

<a id=\"mod.rooted\"></a>
### rooted

- [Root](#type.rooted.Root)
";

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
    let out = tmp.path().join("out");
    let result = vibelight(&["--workspace", text(&root)], &out, false);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files = read_outputs(&out);
    let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["rooted.html", "rooted.md", "sub.html", "sub.md"]);
    assert!(files[1].1 == ROOTED, "rooted.md differs:\n{}", files[1].1);
}
