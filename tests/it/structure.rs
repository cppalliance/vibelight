//! Tests for the structure rules in `build/structure.rs`, each on a small temporary package.

#[path = "../../build/structure.rs"]
mod rules;

use std::fs;
use std::path::Path;

use rules::Violation;
use tempfile::TempDir;

/// A package that passes every rule.
const CLEAN: [(&str, &str); 3] = [
    ("Cargo.toml", "[package]\nname = \"demo\"\n"),
    ("src/main.rs", "//! Demo binary.\n\nfn main() {}\n"),
    ("plans/2026-10-25-demo.plan.md", "# Demo plan\n"),
];

/// A temporary package holding the `CLEAN` files, then `files` written over them.
fn package(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("the system temp directory accepts a new directory");
    for (path, text) in CLEAN.iter().chain(files) {
        let path = dir.path().join(path);
        let parent = path.parent().expect("a joined path has a parent");
        fs::create_dir_all(parent).expect("the temp directory accepts new folders");
        fs::write(path, text).expect("the temp directory accepts new files");
    }
    dir
}

/// The path and rule of each violation, in order.
fn summary(violations: &[Violation]) -> Vec<(&str, &str)> {
    violations
        .iter()
        .map(|violation| (violation.path.as_str(), violation.rule))
        .collect()
}

/// Fails, listing every violation, when there are any.
fn assert_clean(violations: &[Violation]) {
    let messages: Vec<String> = violations.iter().map(ToString::to_string).collect();
    assert!(
        messages.is_empty(),
        "unexpected violations:\n{}",
        messages.join("\n")
    );
}

#[test]
fn second_manifest_is_a_violation() {
    let dir = package(&[("crates/helper/Cargo.toml", "[package]\n")]);
    let found = rules::extra_manifests(dir.path()).unwrap();
    assert_eq!(
        summary(&found),
        [("crates/helper/Cargo.toml", rules::SECOND_MANIFEST)]
    );
}

#[test]
fn root_manifest_alone_passes() {
    let dir = package(&[]);
    assert_clean(&rules::extra_manifests(dir.path()).unwrap());
}

#[test]
fn workspace_table_is_a_violation() {
    let dir = package(&[(
        "Cargo.toml",
        r#"[package]
name = "demo"

[workspace]
members = ["helper"]

[ workspace.lints.rust ]
unsafe_code = "forbid"
"#,
    )]);
    let found = rules::root_manifest(dir.path()).unwrap();
    assert_eq!(summary(&found), [("Cargo.toml", rules::WORKSPACE); 2]);
    assert_eq!(found[0].fix, "remove the [workspace] table at line 4");
    assert_eq!(
        found[1].fix,
        "remove the [workspace.lints.rust] table at line 7"
    );
}

#[test]
fn manifest_without_workspace_passes() {
    let dir = package(&[(
        "Cargo.toml",
        r#"[package]
name = "demo"
description = "has no [workspace] table"

[lints.rust]
unsafe_code = "forbid"
"#,
    )]);
    assert_clean(&rules::root_manifest(dir.path()).unwrap());
}

#[test]
fn path_dependency_is_a_violation() {
    let dir = package(&[(
        "Cargo.toml",
        r#"[package]
name = "demo"

[dependencies]
helper = { path = "../helper" }
pinned = { version = "1", path = "../pinned" }

[dev-dependencies]
fixture.path = "../fixture"

[build-dependencies.generator]
path = "../generator"

[target.'cfg(windows)'.dependencies]
shim = {path="../shim"}
"#,
    )]);
    let found = rules::root_manifest(dir.path()).unwrap();
    assert_eq!(summary(&found), [("Cargo.toml", rules::PATH_DEPENDENCY); 5]);
    for (violation, line) in found.iter().zip([5, 6, 9, 12, 15]) {
        assert!(
            violation.fix.contains(&format!("at line {line} ")),
            "{violation}"
        );
    }
}

#[test]
fn registry_and_git_dependencies_pass() {
    let dir = package(&[(
        "Cargo.toml",
        r#"[package]
name = "demo"

[dependencies]
pathdiff = "0.2"
typed-path = { version = "0.9", features = ["path"] }
remote = { git = "https://example.com/path.git" } # path = "../remote"
"#,
    )]);
    assert_clean(&rules::root_manifest(dir.path()).unwrap());
}

#[test]
fn test_target_path_is_not_a_dependency() {
    let dir = package(&[(
        "Cargo.toml",
        r#"[package]
name = "demo"
autotests = false

[dependencies]
anyhow = "1"

[[test]]
name = "it"
path = "tests/it/main.rs"
"#,
    )]);
    assert_clean(&rules::root_manifest(dir.path()).unwrap());
}

#[test]
fn lib_rs_is_a_violation() {
    let dir = package(&[("src/lib.rs", "//! A library.\n")]);
    let found = rules::lib_target(dir.path());
    assert_eq!(summary(&found), [("src/lib.rs", rules::LIB_TARGET)]);
}

#[test]
fn module_named_lib_passes() {
    let dir = package(&[("src/parse/lib.rs", "//! A module named lib.\n")]);
    assert_clean(&rules::lib_target(dir.path()));
}

#[test]
fn file_over_500_lines_is_a_violation() {
    let long = "//! Line.\n".repeat(501);
    let dir = package(&[
        ("build/big.rs", long.as_str()),
        ("src/big.rs", long.as_str()),
        ("tests/it/big.rs", long.as_str()),
    ]);
    let found = rules::long_files(dir.path()).unwrap();
    let expected = ["build/big.rs", "src/big.rs", "tests/it/big.rs"];
    assert_eq!(
        summary(&found),
        expected.map(|path| (path, rules::LONG_FILE))
    );
    assert_eq!(found[0].fix, "split its 501 lines into smaller modules");
}

#[test]
fn file_of_500_lines_passes() {
    let full = "//! Line.\n".repeat(500);
    let long = "//! Line.\n".repeat(501);
    let dir = package(&[
        ("src/full.rs", full.as_str()),
        ("src/notes.txt", long.as_str()),
        ("examples/long.rs", long.as_str()),
    ]);
    assert_clean(&rules::long_files(dir.path()).unwrap());
}

#[test]
fn mod_rs_is_a_violation() {
    let dir = package(&[
        ("src/parse/mod.rs", "//! Parser.\n"),
        ("tests/it/common/mod.rs", "//! Helpers.\n"),
    ]);
    let found = rules::mod_rs(dir.path()).unwrap();
    let expected = ["src/parse/mod.rs", "tests/it/common/mod.rs"];
    assert_eq!(summary(&found), expected.map(|path| (path, rules::MOD_RS)));
    assert_eq!(found[0].fix, "rename it to src/parse.rs");
}

#[test]
fn module_file_named_after_its_module_passes() {
    let dir = package(&[
        ("src/parse.rs", "//! Parser.\n"),
        ("src/parse/lexer.rs", "//! Lexer.\n"),
    ]);
    assert_clean(&rules::mod_rs(dir.path()).unwrap());
}

#[test]
fn file_without_module_doc_is_a_violation() {
    let dir = package(&[
        ("src/main.rs", "// A plain comment.\nfn main() {}\n"),
        ("src/empty.rs", ""),
        ("src/late.rs", "\n//! Not the first line.\n"),
    ]);
    let found = rules::module_docs(dir.path()).unwrap();
    let expected = ["src/empty.rs", "src/late.rs", "src/main.rs"];
    assert_eq!(
        summary(&found),
        expected.map(|path| (path, rules::MODULE_DOC))
    );
}

#[test]
fn files_starting_with_module_doc_pass() {
    let dir = package(&[
        ("src/crlf.rs", "//! CRLF.\r\nfn crlf() {}\r\n"),
        ("src/bom.rs", "\u{feff}//! Byte order mark.\n"),
        ("tests/it/main.rs", "fn outside_src() {}\n"),
    ]);
    assert_clean(&rules::module_docs(dir.path()).unwrap());
}

#[test]
fn lint_level_attribute_is_a_violation() {
    let dir = package(&[
        ("src/main.rs", "//! Demo.\n#![allow(dead_code)]\n"),
        (
            "src/parse.rs",
            "//! Parser.
    #![deny(missing_docs)]
#![expect(unused)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
",
        ),
    ]);
    let found = rules::lint_levels(dir.path()).unwrap();
    let mut expected = vec![("src/main.rs", rules::LINT_LEVEL)];
    expected.extend([("src/parse.rs", rules::LINT_LEVEL); 4]);
    assert_eq!(summary(&found), expected);
    assert!(found[0].fix.contains("`#![allow(dead_code)]` at line 2 "));
    assert!(found[1].fix.contains("`#![deny(missing_docs)]` at line 2 "));
}

#[test]
fn outer_lint_attribute_passes() {
    let dir = package(&[
        (
            "src/main.rs",
            r#"//! Demo.
#![doc(html_no_source)]

#[allow(dead_code, reason = "an example")]
fn unused() {}
"#,
        ),
        ("tests/it/main.rs", "#![allow(dead_code)]\n"),
    ]);
    assert_clean(&rules::lint_levels(dir.path()).unwrap());
}

#[test]
fn misnamed_plan_is_a_violation() {
    let dir = package(&[
        ("plans/notes.md", ""),
        ("plans/2026-10-25.plan.md", ""),
        ("plans/2026-10-25-one-two-three-four.plan.md", ""),
        ("plans/2026-10-5-short-day.plan.md", ""),
        ("plans/2026-10-25-Upper.plan.md", ""),
        ("plans/2026-10-25-double--hyphen.plan.md", ""),
        ("plans/2026-10-25-snake_case.plan.md", ""),
        ("plans/2026-10-25-tool.md", ""),
        ("plans/old/2026-10-25-demo.plan.md", ""),
    ]);
    let found = rules::plan_names(dir.path()).unwrap();
    let expected = [
        "plans/2026-10-25-Upper.plan.md",
        "plans/2026-10-25-double--hyphen.plan.md",
        "plans/2026-10-25-one-two-three-four.plan.md",
        "plans/2026-10-25-snake_case.plan.md",
        "plans/2026-10-25-tool.md",
        "plans/2026-10-25.plan.md",
        "plans/2026-10-5-short-day.plan.md",
        "plans/notes.md",
        "plans/old/",
    ];
    assert_eq!(
        summary(&found),
        expected.map(|path| (path, rules::PLAN_NAME))
    );
}

#[test]
fn plan_names_with_one_to_three_words_pass() {
    let dir = package(&[
        ("plans/2026-10-25-tool.plan.md", ""),
        ("plans/2026-10-25-one-two-three.plan.md", ""),
        ("plans/1999-01-01-v2-r10.plan.md", ""),
    ]);
    assert_clean(&rules::plan_names(dir.path()).unwrap());
}

#[test]
fn fixtures_are_exempt() {
    let long = "// Line.\n".repeat(600);
    let dir = package(&[
        ("tests/fixtures/sample/Cargo.toml", "[workspace]\n"),
        ("tests/fixtures/sample/src/lib.rs", "#![allow(unused)]\n"),
        ("tests/fixtures/sample/src/a/mod.rs", "struct A;\n"),
        ("tests/fixtures/sample/src/long.rs", long.as_str()),
    ]);
    assert_clean(&rules::check(dir.path()).unwrap());
}

#[test]
fn target_and_git_are_skipped() {
    let dir = package(&[
        ("target/package/demo-0.1.0/Cargo.toml", "[package]\n"),
        ("target/debug/build/mod.rs", ""),
        (".git/modules/helper/Cargo.toml", ""),
    ]);
    assert_clean(&rules::check(dir.path()).unwrap());
}

#[test]
fn published_package_is_not_checked() {
    let dir = package(&[
        ("Cargo.toml", "[workspace]\n"),
        ("src/lib.rs", "fn undocumented() {}\n"),
        ("src/parse/mod.rs", ""),
    ]);
    assert!(!rules::is_packaged(dir.path()));
    assert!(!rules::check(dir.path()).unwrap().is_empty());

    fs::write(dir.path().join("Cargo.toml.orig"), "[package]\n").unwrap();
    assert!(rules::is_packaged(dir.path()));
    assert_clean(&rules::check(dir.path()).unwrap());
}

#[test]
fn check_sorts_by_path_then_rule() {
    let dir = package(&[
        ("Cargo.toml", "[package]\nname = \"demo\"\n\n[workspace]\n"),
        ("crates/helper/Cargo.toml", ""),
        ("src/lib.rs", "#![allow(dead_code)]\n"),
    ]);
    let found = rules::check(dir.path()).unwrap();
    assert_eq!(found.len(), 5);
    let sorted = found.is_sorted_by(|a, b| (&a.path, a.rule) <= (&b.path, b.rule));
    assert!(sorted, "{:?}", summary(&found));
}

#[test]
fn this_package_passes() {
    assert_clean(&rules::check(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap());
}
