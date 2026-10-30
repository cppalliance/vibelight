//! Shared helpers: running the binary, copying fixtures, and reading outputs.

use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

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

/// The binary, with the stamp pinned to `EPOCH`. It starts in the system's
/// temporary folder, so the repository's own `vibelight.toml` is never the
/// config of a test run; a test that needs a folder sets `current_dir`.
pub(crate) fn vibelight() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vibelight"));
    command.env("SOURCE_DATE_EPOCH", EPOCH);
    command.current_dir(std::env::temp_dir());
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

/// Like `read_run`, with the source links and the markdown's `root`
/// removed, so runs from different folders compare equal, and with every
/// source line of the markdown made 0, so a change of layout that moves no
/// declaration changes no golden.
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

/// Normalizes a markdown outline for its golden. The stamp line stays, the
/// header loses its `root`, and on every record line each structural
/// location string ends in line 0 and each `line` value becomes 0. A
/// location string is the value of a `loc` key, an element of a `locs`
/// array, or an item of a `more` record that continues a `locs` array. A
/// key is a string that a real quote opens and a colon follows, so no text
/// inside another string, such as a doc that writes `src/lib.rs:6` or
/// `"line":`, is ever touched, and the keys keep their order.
pub(crate) fn normalize_outline(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    for (n, line) in md.split_inclusive('\n').enumerate() {
        if n == 0 {
            out.push_str(line);
        } else {
            out.push_str(&normalize_record(line, n == 1));
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

/// Returns a quoted location string with its line made 0.
fn zero_line(text: &str) -> String {
    match text.rfind(':') {
        Some(at) => format!("{}0\"", &text[..=at]),
        None => text.to_owned(),
    }
}

/// Normalizes one compact JSON record, whose text outside its strings is
/// ASCII; the header also loses its `root`.
fn normalize_record(line: &str, header: bool) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    // Each open object or array, with the key whose value it is.
    let mut open: Vec<(u8, String)> = Vec::new();
    let mut key = String::new();
    let mut field = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte != b'"' {
            match byte {
                b'{' | b'[' => open.push((byte, mem::take(&mut key))),
                b'}' | b']' => key = open.pop().map(|(_, of)| of).unwrap_or_default(),
                _ => {}
            }
            out.push(char::from(byte));
            i += 1;
            continue;
        }
        let end = string_end(bytes, i);
        let text = &line[i..end];
        if bytes.get(end) == Some(&b':') {
            text.trim_matches('"').clone_into(&mut key);
            if header && open.len() == 1 && key == "root" {
                i = string_end(bytes, end + 1);
                if out.ends_with(',') {
                    out.pop();
                } else if bytes.get(i) == Some(&b',') {
                    i += 1;
                }
                continue;
            }
            out.push_str(text);
            out.push(':');
            i = end + 1;
            if key == "line" {
                out.push('0');
                while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                    i += 1;
                }
            }
            continue;
        }
        let (kind, of) = open.last().map_or((b' ', ""), |(b, of)| (*b, of.as_str()));
        let located = match kind {
            b'{' => key == "loc",
            b'[' => of == "locs" || (of == "items" && field.rsplit('.').next() == Some("locs")),
            _ => false,
        };
        if kind == b'{' && open.len() == 1 && key == "field" {
            text.trim_matches('"').clone_into(&mut field);
        }
        if located {
            out.push_str(&zero_line(text));
        } else {
            out.push_str(text);
        }
        i = end;
    }
    out
}

/// Returns every source location of a markdown outline: each `loc`, each
/// element of a `locs` array, and each item of a `more` record that
/// continues a `locs` array.
pub(crate) fn locations(md: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in md.lines().skip(1) {
        let value: Value = serde_json::from_str(line).expect("a record is JSON");
        let field = value["field"].as_str().unwrap_or_default();
        if value["k"] == "more" && field.rsplit('.').next() == Some("locs") {
            out.extend(strings(&value["items"]));
        }
        collect_locations(&value, &mut out);
    }
    out
}

/// Returns the strings of a JSON array.
fn strings(list: &Value) -> Vec<String> {
    let items = list.as_array().map_or(&[][..], Vec::as_slice);
    items
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// Adds each `loc` and each element of a `locs` array inside a value.
fn collect_locations(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                match key.as_str() {
                    "loc" => out.extend(inner.as_str().map(str::to_owned)),
                    "locs" => out.extend(strings(inner)),
                    _ => collect_locations(inner, out),
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_locations(item, out)),
        _ => {}
    }
}

/// Returns the header's `root` of a markdown outline: the crate's folder
/// from the markdown folder.
pub(crate) fn root_of(md: &str) -> String {
    let header = md.lines().nth(1).expect("an outline has a header");
    let value: Value = serde_json::from_str(header).expect("the header is JSON");
    value["root"]
        .as_str()
        .expect("the header has a root")
        .to_owned()
}

/// Splits a location string from the right into the stem of the target
/// that holds its file, when that is another target, and the file.
pub(crate) fn split_location(loc: &str) -> (Option<&str>, &str) {
    let mut parts = loc.rsplitn(3, ':').skip(1);
    let file = parts.next().expect("a location names a file");
    (parts.next(), file)
}

#[test]
fn normalizing_drops_the_root_and_zeroes_lines_in_every_structural_place() {
    let stamp = "*Updated 2026-01-01T00:00:00Z*\n";
    let header = "{\"k\":\"vibelight-outline\",\"schema\":1,\"crate\":\"a\",\"target\":\"lib\"}\n";
    let rooted = header.replace("\"lib\"}", "\"lib\",\"root\":\"../a b\"}");
    assert_eq!(
        normalize_outline(&format!("{stamp}{rooted}")),
        format!("{stamp}{header}")
    );
    let cases = [
        (
            "{\"k\":\"fn\",\"path\":\"f\",\"sig\":\"fn f()\",\"loc\":\"src/lib.rs:12\"}",
            "{\"k\":\"fn\",\"path\":\"f\",\"sig\":\"fn f()\",\"loc\":\"src/lib.rs:0\"}",
        ),
        (
            "{\"k\":\"lead\",\"id\":\"x\",\"subjects\":[\"a\"],\"evidence\":\"at x.rs:3\",\"locs\":[\"src/lib.rs:3\",\"alpha:src/lib.rs:40\"]}",
            "{\"k\":\"lead\",\"id\":\"x\",\"subjects\":[\"a\"],\"evidence\":\"at x.rs:3\",\"locs\":[\"src/lib.rs:0\",\"alpha:src/lib.rs:0\"]}",
        ),
        (
            "{\"k\":\"fn\",\"comments\":[{\"text\":\"// see \\\"line\\\":5 and x.rs:6\",\"line\":9}]}",
            "{\"k\":\"fn\",\"comments\":[{\"text\":\"// see \\\"line\\\":5 and x.rs:6\",\"line\":0}]}",
        ),
        (
            "{\"k\":\"spelling\",\"path\":\"p\",\"spellings\":[{\"text\":\"a:1\",\"count\":1,\"locs\":[\"src/a.rs:4\"]}]}",
            "{\"k\":\"spelling\",\"path\":\"p\",\"spellings\":[{\"text\":\"a:1\",\"count\":1,\"locs\":[\"src/a.rs:0\"]}]}",
        ),
        (
            "{\"k\":\"more\",\"path\":\"p\",\"field\":\"spellings.0.locs\",\"part\":2,\"items\":[\"src/a.rs:5\",\"b:src/b.rs:16\"]}",
            "{\"k\":\"more\",\"path\":\"p\",\"field\":\"spellings.0.locs\",\"part\":2,\"items\":[\"src/a.rs:0\",\"b:src/b.rs:0\"]}",
        ),
        (
            "{\"k\":\"more\",\"path\":\"f\",\"field\":\"comments\",\"part\":2,\"items\":[{\"text\":\"// a:1\",\"line\":12}]}",
            "{\"k\":\"more\",\"path\":\"f\",\"field\":\"comments\",\"part\":2,\"items\":[{\"text\":\"// a:1\",\"line\":0}]}",
        ),
        (
            "{\"k\":\"more\",\"path\":\"a\",\"id\":\"x\",\"field\":\"subjects\",\"part\":2,\"items\":[\"b:7\"]}",
            "{\"k\":\"more\",\"path\":\"a\",\"id\":\"x\",\"field\":\"subjects\",\"part\":2,\"items\":[\"b:7\"]}",
        ),
        (
            "{\"k\":\"more\",\"path\":\"a\",\"field\":\"text\",\"part\":3,\"text\":\"see src/lib.rs:5\"}",
            "{\"k\":\"more\",\"path\":\"a\",\"field\":\"text\",\"part\":3,\"text\":\"see src/lib.rs:5\"}",
        ),
        (
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"a \\\"loc\\\":\\\"x:5\\\", \\\"root\\\":\\\"y\\\" b\"}",
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"a \\\"loc\\\":\\\"x:5\\\", \\\"root\\\":\\\"y\\\" b\"}",
        ),
        (
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"ü ✓\"}",
            "{\"k\":\"doc\",\"path\":\"a\",\"text\":\"ü ✓\"}",
        ),
    ];
    for (record, want) in cases {
        let got = normalize_outline(&format!("{stamp}{header}{record}\n"));
        assert_eq!(got, format!("{stamp}{header}{want}\n"));
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
