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

/// The fixtures the no-link checks run over: every folder but `links`,
/// whose golden is rendered with source links on, and `workspaces`, which
/// holds multi-crate runs.
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
pub(crate) fn run(input: &Path, out: &Path, links: bool) -> Output {
    run_with(input, out, links, &[])
}

/// Runs the binary on `input`, writing into `out`, with more switches.
pub(crate) fn run_with(input: &Path, out: &Path, links: bool, extra: &[String]) -> Output {
    let mut command = vibelight();
    command.arg(input).arg("-o").arg(out).args(extra);
    if !links {
        command.arg("--no-source-links");
    }
    command.output().expect("the vibelight binary runs")
}

/// Runs the binary into a fresh folder and returns each written file's
/// name and text, sorted by name.
pub(crate) fn outputs(input: &Path, links: bool) -> Vec<(String, String)> {
    outputs_with(input, links, &[])
}

/// Like `outputs`, with more switches.
pub(crate) fn outputs_with(input: &Path, links: bool, extra: &[String]) -> Vec<(String, String)> {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let result = run_with(input, tmp.path(), links, extra);
    assert!(
        result.status.success(),
        "vibelight failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    read_outputs(tmp.path())
}

/// Reads every file in a folder, sorted by name.
pub(crate) fn read_outputs(dir: &Path) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = fs::read_dir(dir)
        .expect("the output folder is readable")
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

/// The headings, marks, and code block texts of one outline file.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Parts {
    pub(crate) headings: Vec<String>,
    pub(crate) marks: Vec<String>,
    pub(crate) blocks: Vec<String>,
}

/// Reads the headings, marks, and block texts of a markdown outline.
pub(crate) fn markdown_parts(md: &str) -> Parts {
    let mut parts = Parts {
        headings: Vec::new(),
        marks: Vec::new(),
        blocks: Vec::new(),
    };
    let mut fence: Option<String> = None;
    let mut block = Vec::new();
    for line in md.lines() {
        if let Some(marks) = &fence {
            if line == marks {
                parts.blocks.push(block.join("\n"));
                block.clear();
                fence = None;
            } else {
                block.push(line.to_owned());
            }
            continue;
        }
        if line.len() >= 3 && line.chars().all(|c| c == '`') {
            fence = Some(line.to_owned());
        } else if line.starts_with('#') {
            parts
                .headings
                .push(line.trim_start_matches('#').trim().to_owned());
        } else if line.starts_with("**MACRO**") || line.starts_with("**UNSUPPORTED:**") {
            parts.marks.push(without_source(&line.replace("**", "")));
        }
    }
    parts
}

/// Reads the headings, marks, and block texts of an HTML outline.
pub(crate) fn html_parts(html: &str) -> Parts {
    let mut parts = Parts {
        headings: Vec::new(),
        marks: Vec::new(),
        blocks: Vec::new(),
    };
    let mut rest = html;
    while let Some(start) = rest.find("<pre><code>") {
        let after = &rest[start + "<pre><code>".len()..];
        let end = after.find("</code></pre>").expect("every block closes");
        parts.blocks.push(unescape(&strip_tags(&after[..end])));
        rest = &after[end..];
    }
    for line in html.lines() {
        if line.starts_with("<h1") || line.starts_with("<h2") || line.starts_with("<h3") {
            parts.headings.push(unescape(&strip_tags(line)));
        } else if line.starts_with("<p class=\"mark\"") {
            parts
                .marks
                .push(without_source(&unescape(&strip_tags(line))));
        }
    }
    parts
}

fn without_source(line: &str) -> String {
    line.find(" (source: ")
        .map_or(line, |at| &line[..at])
        .to_owned()
}

/// Removes every tag.
pub(crate) fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;
    for c in text.chars() {
        match c {
            '<' => inside = true,
            '>' if inside => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out
}

/// Turns the escapes vibelight writes back into characters.
pub(crate) fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

/// Removes the source links from a markdown outline: each `Source:` line
/// with its one blank line, and each ` (source: ...)` suffix.
pub(crate) fn strip_markdown_links(md: &str) -> String {
    let mut out = String::new();
    let mut skip_blank = false;
    for line in md.split_inclusive('\n') {
        if skip_blank && line == "\n" {
            skip_blank = false;
            continue;
        }
        skip_blank = false;
        if line.starts_with("Source: ") {
            skip_blank = true;
            continue;
        }
        match line.find(" (source: [") {
            Some(at) => {
                out.push_str(&line[..at]);
                out.push('\n');
            }
            None => out.push_str(line),
        }
    }
    out
}

/// Removes the source links from an HTML outline: each `Source:` line and
/// each ` (source: ...)` span go, and each link around a member name turns
/// into the span the no-links output has.
pub(crate) fn strip_html_links(html: &str) -> String {
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
        while let Some(at) = line.find("<a class=\"src\" href=\"") {
            let open_end = line[at..].find('>').expect("a link tag closes") + at + 1;
            let close = line[open_end..].find("</a>").expect("a link closes") + open_end;
            line.replace_range(close..close + "</a>".len(), "</span>");
            line.replace_range(at..open_end, "<span class=\"src\">");
        }
        out.push_str(&line);
    }
    out
}
