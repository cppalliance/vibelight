//! Runs the binary the ways a user does: targets, output folders, line
//! endings, exit codes, and repeat runs.

use std::fs;
use std::process::Command;

use crate::support::{
    copy_dir, fixture_names, fixtures, input_of, outputs, read_outputs, run, rust_files,
};

#[test]
fn a_second_run_gives_identical_bytes() {
    for links in [false, true] {
        let tmp = tempfile::tempdir().expect("a temporary folder can be made");
        let input = fixtures().join("prompt");
        let first = tmp.path().join("first");
        let second = tmp.path().join("second");
        assert!(run(&input, &first, links).status.success());
        assert!(run(&input, &second, links).status.success());
        let one = read_outputs(&first);
        let two = read_outputs(&second);
        assert!(!links || one.iter().any(|(_, t)| t.contains("Source: ")));
        assert_eq!(one, two);
    }
}

#[test]
fn crlf_copies_give_the_same_output() {
    for name in fixture_names() {
        let original = fixtures().join(&name);
        let tmp = tempfile::tempdir().expect("a temporary folder can be made");
        let copy = tmp.path().join(&name);
        copy_dir(&original, &copy);
        for path in rust_files(&copy) {
            let text = fs::read_to_string(&path).expect("a fixture file is readable");
            fs::write(&path, text.replace('\n', "\r\n")).expect("a fixture copy is writable");
        }
        assert_eq!(
            outputs(&input_of(&original), false),
            outputs(&input_of(&copy), false),
            "{name}"
        );
    }
}

#[test]
fn a_missing_output_folder_is_created_and_files_are_overwritten() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let out = tmp.path().join("a").join("b");
    let input = fixtures().join("structs");
    assert!(run(&input, &out, false).status.success());
    fs::write(out.join("structs.md"), "stale").expect("the output is writable");
    assert!(run(&input, &out, false).status.success());
    let text = fs::read_to_string(out.join("structs.md")).expect("the outline was written");
    assert!(text.starts_with("<a id=\"crate.structs\"></a>"));
}

#[test]
fn stdout_lists_the_written_paths_sorted() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = run(&fixtures().join("multi"), tmp.path(), false);
    let stdout = String::from_utf8(result.stdout).expect("stdout is UTF-8");
    let names: Vec<String> = stdout
        .lines()
        .map(|l| {
            l.rsplit(['/', '\\'])
                .next()
                .expect("a path has a name")
                .to_owned()
        })
        .collect();
    assert_eq!(
        names,
        ["multi-cli.html", "multi-cli.md", "multi.html", "multi.md"]
    );
}

#[test]
fn a_custom_lib_path_is_read_from_the_manifest() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let dir = tmp.path().join("custom");
    fs::create_dir_all(dir.join("lib")).expect("a folder can be made");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"custom\"\n[lib]\nname = \"core-x\"\npath = \"lib/top.rs\"\n",
    )
    .expect("a manifest can be written");
    fs::write(
        dir.join("lib").join("top.rs"),
        "//! Top.\n\n/// A thing.\npub struct Thing;\n",
    )
    .expect("a file can be written");
    let written = outputs(&dir, false);
    let names: Vec<&str> = written.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["core_x.html", "core_x.md"]);
    assert!(written[1].1.contains("### Thing"));
}

#[test]
fn unusable_input_exits_with_1() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let out = tmp.path().join("out");
    let empty = tmp.path().join("empty");
    fs::create_dir_all(&empty).expect("a folder can be made");
    let result = run(&empty, &out, false);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("not a crate"));
    let workspace = tmp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("a folder can be made");
    fs::write(workspace.join("Cargo.toml"), "[workspace]\nmembers = []\n")
        .expect("a manifest can be written");
    let result = run(&workspace, &out, false);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("workspace manifest"));
}

#[test]
fn usage_errors_keep_clap_exit_code() {
    let result = Command::new(env!("CARGO_BIN_EXE_vibelight"))
        .output()
        .expect("the binary runs");
    assert_eq!(result.status.code(), Some(2));
}

#[test]
fn warnings_name_file_and_line_and_end_with_a_count() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = run(&fixtures().join("unparsable"), tmp.path(), false);
    assert!(result.status.success());
    let stderr = String::from_utf8(result.stderr).expect("stderr is UTF-8");
    let lines: Vec<&str> = stderr.lines().collect();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("warning: src/broken.rs:1: cannot parse")),
        "{stderr}"
    );
    assert!(
        lines.contains(&"warning: src/lib.rs:9: unsupported verbatim impl item"),
        "{stderr}"
    );
    assert!(
        lines.contains(&"warning: src/lib.rs:15: unsupported verbatim item"),
        "{stderr}"
    );
    assert_eq!(lines.last(), Some(&"3 warnings"));
}

#[test]
fn spellings_add_one_warning_line_only_when_an_item_has_two_spellings() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = run(
        &fixtures().join("spellings"),
        &tmp.path().join("two"),
        false,
    );
    assert!(result.status.success());
    let stderr = String::from_utf8(result.stderr).expect("stderr is UTF-8");
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        [
            "warning: 5 items written more than one way; see Spellings",
            "1 warning"
        ]
    );
    let result = run(
        &fixtures().join("spellings-none"),
        &tmp.path().join("one"),
        false,
    );
    assert!(result.status.success());
    assert!(
        result.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn changing_a_declaration_changes_the_output() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let copy = tmp.path().join("structs");
    copy_dir(&fixtures().join("structs"), &copy);
    let lib = copy.join("src").join("lib.rs");
    let text = fs::read_to_string(&lib).expect("the fixture is readable");
    fs::write(&lib, text.replace("pub x: i32", "pub x: i64")).expect("the copy is writable");
    let before = outputs(&fixtures().join("structs"), false);
    let after = outputs(&copy, false);
    assert_ne!(before, after);
}
