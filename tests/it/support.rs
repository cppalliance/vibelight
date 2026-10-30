//! Shared helpers: running the binary, copying fixtures, and reading outputs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The folder holding every fixture.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// The fixtures the golden checks run over: every folder but `links`,
/// whose golden keeps its source links, and `workspaces`, which holds
/// multi-crate runs.
pub(crate) fn fixture_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(fixtures())
        .expect("the fixtures folder is readable")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry
                .path()
                .is_dir()
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .filter(|name| name != "links" && name != "workspaces")
        .collect();
    names.sort();
    names
}

/// Returns what the binary takes for a fixture folder: the folder, or the
/// one `.rs` file in it when it is not a crate.
pub(crate) fn input_of(dir: &Path) -> PathBuf {
    if dir.join("src").is_dir() || dir.join("Cargo.toml").is_file() {
        return dir.to_path_buf();
    }
    fs::read_dir(dir)
        .expect("the fixture folder is readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|x| x == "rs"))
        .expect("a fixture without src holds one .rs file")
}

/// Tells whether a file name has the given extension.
pub(crate) fn has_ext(name: &str, ext: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

/// The `SOURCE_DATE_EPOCH` every run of the tests sets: 2026-01-01T00:00:00Z,
/// the stamp every golden starts with.
pub(crate) const EPOCH: &str = "1767225600";

/// The binary, with the stamp pinned to `EPOCH`.
pub(crate) fn vibelight() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vibelight"));
    command.env("SOURCE_DATE_EPOCH", EPOCH);
    command
}

/// Runs the binary on `input`, writing into `out`.
pub(crate) fn run(input: &Path, out: &Path) -> Output {
    vibelight()
        .arg(input)
        .arg("-o")
        .arg(out)
        .output()
        .expect("the vibelight binary runs")
}

/// Runs the binary into a fresh folder and returns each outline's name and
/// text without its source links, sorted by name.
pub(crate) fn outputs(input: &Path) -> Vec<(String, String)> {
    outputs_with_streams(input).0
}

/// Runs the binary into a fresh folder and returns each outline's name and
/// text without its source links, sorted by name, with the run's stdout
/// and stderr lines.
pub(crate) fn outputs_with_streams(
    input: &Path,
) -> (Vec<(String, String)>, Vec<String>, Vec<String>) {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let out = tmp.path().join("out");
    let result = run(input, &out);
    assert!(
        result.status.success(),
        "vibelight failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let lines = |bytes: &[u8]| -> Vec<String> {
        String::from_utf8_lossy(bytes)
            .lines()
            .map(str::to_owned)
            .collect()
    };
    (
        read_plain(&out),
        lines(&result.stdout),
        lines(&result.stderr),
    )
}

/// Reads the outlines a run wrote into `out`: the markdown files in it,
/// less the index, and the HTML files in its `html` folder, less the index,
/// sorted by name.
pub(crate) fn read_run(out: &Path) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = read_outputs(out)
        .into_iter()
        .filter(|(name, _)| has_ext(name, "md") && name != "index.md")
        .chain(
            read_outputs(&out.join("html"))
                .into_iter()
                .filter(|(name, _)| has_ext(name, "html") && name != "index.html"),
        )
        .collect();
    files.sort();
    files
}

/// Like `read_run`, with the source links removed, so runs from different
/// folders compare equal, and with every source line of the markdown made
/// 0, so a change of layout that moves no declaration changes no golden.
pub(crate) fn read_plain(out: &Path) -> Vec<(String, String)> {
    read_run(out)
        .into_iter()
        .map(|(name, text)| {
            let plain = if has_ext(&name, "md") {
                normalize_outline(&text)
            } else {
                strip_html_links(&text)
            };
            (name, plain)
        })
        .collect()
}

/// Reads every file in a folder, sorted by name; nothing when the folder
/// is missing.
pub(crate) fn read_outputs(dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(String, String)> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .map(|p| {
            let name = p
                .file_name()
                .expect("a file has a name")
                .to_string_lossy()
                .into_owned();
            let text = fs::read_to_string(&p).expect("an output file is UTF-8");
            (name, text)
        })
        .collect();
    files.sort();
    files
}

/// Copies a folder tree.
pub(crate) fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("a copy folder can be made");
    for entry in fs::read_dir(from).expect("a fixture folder is readable") {
        let path = entry.expect("a folder entry is readable").path();
        let target = to.join(path.file_name().expect("an entry has a name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target).expect("a fixture file can be copied");
        }
    }
}

/// Returns every `.rs` file under a folder, sorted.
pub(crate) fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(folder) = stack.pop() {
        for entry in fs::read_dir(&folder).expect("a folder is readable") {
            let path = entry.expect("a folder entry is readable").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Normalizes a markdown outline for its golden. The stamp line stays, and
/// on every record line after it each `url` key goes with its value and
/// each `line` value becomes 0. A key is a string that a real quote opens
/// and a colon follows, so no text inside a string, such as a doc that
/// writes `#L6` or `"line":`, is ever touched, and the keys keep their order.
pub(crate) fn normalize_outline(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    for (n, line) in md.split_inclusive('\n').enumerate() {
        if n == 0 {
            out.push_str(line);
        } else {
            out.push_str(&normalize_record(line));
        }
    }
    out
}

/// Returns the index just past the string that opens at `start`.
fn string_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Normalizes one compact JSON record, whose text outside its strings is
/// ASCII.
fn normalize_record(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            out.push(char::from(bytes[i]));
            i += 1;
            continue;
        }
        let end = string_end(bytes, i);
        let key = bytes.get(end) == Some(&b':');
        match &line[i..end] {
            "\"url\"" if key => {
                i = string_end(bytes, end + 1);
                if out.ends_with(',') {
                    out.pop();
                } else if bytes.get(i) == Some(&b',') {
                    i += 1;
                }
            }
            "\"line\"" if key => {
                out.push_str("\"line\":0");
                i = end + 1;
                while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                    i += 1;
                }
            }
            text => {
                out.push_str(text);
                i = end;
            }
        }
    }
    out
}

#[test]
fn normalizing_drops_links_and_zeroes_lines_in_every_structural_place() {
    let stamp = "*Updated 2026-01-01T00:00:00Z*\n";
    let loc = |line: u32| {
        format!("{{\"file\":\"src/lib.rs\",\"line\":{line},\"url\":\"../src/lib.rs#L{line}\"}}")
    };
    let cases = [
        (
            format!("{{\"k\":\"fn\",\"path\":\"f\",\"sig\":\"fn f()\",\"loc\":{}}}\n", loc(12)),
            "{\"k\":\"fn\",\"path\":\"f\",\"sig\":\"fn f()\",\"loc\":{\"file\":\"src/lib.rs\",\"line\":0}}\n",
        ),
        (
            format!(
                "{{\"k\":\"lead\",\"id\":\"x\",\"mode\":\"lexical\",\"subjects\":[\"a\"],\"evidence\":\"e\",\"locs\":[{},{}]}}\n",
                loc(3),
                loc(40)
            ),
            "{\"k\":\"lead\",\"id\":\"x\",\"mode\":\"lexical\",\"subjects\":[\"a\"],\"evidence\":\"e\",\"locs\":[{\"file\":\"src/lib.rs\",\"line\":0},{\"file\":\"src/lib.rs\",\"line\":0}]}\n",
        ),
        (
            format!("{{\"k\":\"impl\",\"path\":\"A\",\"rust\":\"impl A\",\"locs\":[{}]}}\n", loc(7)),
            "{\"k\":\"impl\",\"path\":\"A\",\"rust\":\"impl A\",\"locs\":[{\"file\":\"src/lib.rs\",\"line\":0}]}\n",
        ),
        (
            "{\"k\":\"fn\",\"comments\":[{\"text\":\"// see \\\"line\\\":5 and #L6\",\"line\":9}]}\n"
                .to_owned(),
            "{\"k\":\"fn\",\"comments\":[{\"text\":\"// see \\\"line\\\":5 and #L6\",\"line\":0}]}\n",
        ),
        (
            "{\"k\":\"lead\",\"locs\":[{\"target\":\"alpha\",\"file\":\"src/lib.rs\",\"line\":2,\"url\":\"../a/src/lib.rs#L2\"}]}\n"
                .to_owned(),
            "{\"k\":\"lead\",\"locs\":[{\"target\":\"alpha\",\"file\":\"src/lib.rs\",\"line\":0}]}\n",
        ),
        (
            "{\"k\":\"mod\",\"path\":\"crate\",\"loc\":{\"file\":\"tool.rs\",\"line\":1,\"url\":\"../tool.rs#L1\"}}\n"
                .to_owned(),
            "{\"k\":\"mod\",\"path\":\"crate\",\"loc\":{\"file\":\"tool.rs\",\"line\":0}}\n",
        ),
        (
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"a \\\"url\\\":\\\"x\\\" b\"}\n".to_owned(),
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"a \\\"url\\\":\\\"x\\\" b\"}\n",
        ),
        (
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"ü ✓\"}\n".to_owned(),
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"ü ✓\"}\n",
        ),
    ];
    for (record, want) in cases {
        let got = normalize_outline(&format!("{stamp}{record}"));
        assert_eq!(got, format!("{stamp}{want}"));
    }
    assert_eq!(normalize_outline(stamp), stamp, "the stamp line stays");
}

/// Removes the source links from an HTML outline: each `Source:` line and
/// each ` (source: ...)` span go, and each link around a member name turns
/// into a plain span, with the same id if it has one.
pub(crate) fn strip_html_links(html: &str) -> String {
    const OPEN: &str = "<a class=\"src\" href=\"";
    let mut out = String::new();
    for line in html.split_inclusive('\n') {
        if line.starts_with("<p class=\"source\">") {
            continue;
        }
        let mut line = line.to_owned();
        while let Some(at) = line.find(" <span class=\"src\">(source: ") {
            let end =
                line[at..].find(")</span>").expect("a source span closes") + at + ")</span>".len();
            line.replace_range(at..end, "");
        }
        while let Some(at) = line.find(OPEN) {
            let open_end = line[at..].find('>').expect("a link tag closes") + at + 1;
            let close = line[open_end..].find("</a>").expect("a link closes") + open_end;
            let href_end = line[at + OPEN.len()..open_end]
                .find('"')
                .expect("an href closes")
                + at
                + OPEN.len()
                + 1;
            let rest = line[href_end..open_end - 1].to_owned();
            line.replace_range(close..close + "</a>".len(), "</span>");
            line.replace_range(at..open_end, &format!("<span class=\"src\"{rest}>"));
        }
        out.push_str(&line);
    }
    out
}
