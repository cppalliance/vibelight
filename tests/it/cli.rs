//! Runs the binary the ways a user does: targets, output folders, line
//! endings, exit codes, and repeat runs.

use std::fs;

use crate::support::{
    copy_dir, fixture_names, fixtures, input_of, outputs, read_run, run, rust_files, vibelight,
};

#[test]
fn a_second_run_gives_identical_bytes() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let input = fixtures().join("prompt");
    let first = tmp.path().join("first");
    let second = tmp.path().join("second");
    assert!(run(&input, &first).status.success());
    assert!(run(&input, &second).status.success());
    let one = read_run(&first);
    assert!(
        one.iter()
            .all(|(_, text)| text.contains("\"root\":\"") || text.contains("Source: "))
    );
    assert_eq!(one, read_run(&second));
    assert_eq!(
        fs::read(first.join("index.md")).unwrap(),
        fs::read(second.join("index.md")).unwrap()
    );
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
            outputs(&input_of(&original)),
            outputs(&input_of(&copy)),
            "{name}"
        );
    }
}

#[test]
fn a_missing_output_folder_is_created_and_files_are_overwritten() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let out = tmp.path().join("a").join("b");
    let input = fixtures().join("structs");
    assert!(run(&input, &out).status.success());
    fs::write(out.join("structs.md"), "stale").expect("the output is writable");
    assert!(run(&input, &out).status.success());
    let text = fs::read_to_string(out.join("structs.md")).expect("the outline was written");
    assert!(text.starts_with(
        "*Updated 2026-01-01T00:00:00Z*\n\
         {\"k\":\"vibelight-outline\",\"schema\":1,\"crate\":\"structs\",\"target\":\"lib\",\
         \"root\":\""
    ));
    assert!(out.join("html").join("structs.html").is_file());
    assert!(
        !out.join(".gitignore").exists(),
        "a folder -o names is not vibelight's own"
    );
}

#[test]
fn stdout_lists_the_written_paths_sorted() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = run(&fixtures().join("multi"), &tmp.path().join("out"));
    let stdout = String::from_utf8(result.stdout).expect("stdout is UTF-8");
    let names: Vec<String> = stdout
        .lines()
        .map(|l| {
            let mut parts = l.rsplit(['/', '\\']);
            let name = parts.next().expect("a path has a name");
            let folder = parts.next().expect("a path has a folder");
            format!("{folder}/{name}")
        })
        .collect();
    assert_eq!(
        names,
        [
            "html/index.html",
            "html/multi-cli.html",
            "html/multi.html",
            "out/index.md",
            "out/multi-cli.md",
            "out/multi.md"
        ]
    );
}

#[test]
fn both_indexes_report_the_size_of_each_markdown_outline_as_written() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let out = tmp.path().join("out");
    assert!(run(&fixtures().join("multi"), &out).status.success());
    let index = fs::read_to_string(out.join("index.md")).expect("the index was written");
    let page = fs::read_to_string(out.join("html").join("index.html")).expect("a page");
    for stem in ["multi", "multi-cli"] {
        let bytes = fs::read(out.join(format!("{stem}.md")))
            .expect("the outline was written")
            .len();
        let kb = bytes.div_ceil(1024);
        assert!(
            index.contains(&format!(
                "- [{stem}.md]({stem}.md): `.`, selected, {kb} KB\n"
            )),
            "{index}"
        );
        assert!(
            page.contains(&format!(
                "<a href=\"{stem}.html\">{stem}</a> <code>.</code>, selected, {kb} KB of markdown"
            )),
            "{page}"
        );
    }
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
    let written = outputs(&dir);
    let names: Vec<&str> = written.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["core_x.html", "core_x.md"]);
    assert!(
        written[1]
            .1
            .contains("{\"k\":\"struct\",\"path\":\"Thing\",\"rust\":\"pub struct Thing\",")
    );
}

#[test]
fn unusable_input_exits_with_1() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let out = tmp.path().join("out");
    let empty = tmp.path().join("empty");
    fs::create_dir_all(&empty).expect("a folder can be made");
    let result = run(&empty, &out);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("not a crate"));
    let workspace = tmp.path().join("workspace");
    fs::create_dir_all(&workspace).expect("a folder can be made");
    fs::write(workspace.join("Cargo.toml"), "[workspace]\nmembers = []\n")
        .expect("a manifest can be written");
    let result = run(&workspace, &out);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("workspace manifest"));
}

#[test]
fn usage_errors_keep_clap_exit_code() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = vibelight()
        .current_dir(tmp.path())
        .output()
        .expect("the binary runs");
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("name a crate, or run vibelight inside a Rust repository"),
        "{stderr}"
    );
    for removed in ["--facts", "--workspace", "--check", "--no-source-links"] {
        let result = vibelight()
            .args([".", removed])
            .current_dir(tmp.path())
            .output()
            .expect("the binary runs");
        assert_eq!(result.status.code(), Some(2), "{removed}");
    }
}

#[test]
fn warnings_name_file_and_line_and_end_with_a_count() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = run(&fixtures().join("unparsable"), tmp.path());
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
    let result = run(&fixtures().join("spellings"), &tmp.path().join("two"));
    assert!(result.status.success());
    let stderr = String::from_utf8(result.stderr).expect("stderr is UTF-8");
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        [
            "warning: 6 items written more than one way; see spelling records",
            "1 warning"
        ]
    );
    let result = run(&fixtures().join("spellings-none"), &tmp.path().join("one"));
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
    let before = outputs(&fixtures().join("structs"));
    let after = outputs(&copy);
    assert_ne!(before, after);
}
