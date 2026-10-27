//! Rearranges fixtures in ways the determinism rule says cannot change the
//! output, and checks every file stays byte for byte the same, source
//! links aside. The markdown groups each type's functions by the impl that
//! adds them, in source order, so only a rearrangement that keeps every
//! impl where it is leaves the markdown unchanged; the HTML stays the same
//! under all of them.

use std::fs;
use std::path::Path;
use std::process::Command;

use syn::Item;
use syn::spanned::Spanned;

use crate::support::{copy_dir, fixture_names, fixtures, has_ext, input_of, outputs, rust_files};

/// The one fixture built to fail parsing, which no rearrangement can read.
const SKIP: &[&str] = &["unparsable"];

/// The files a rearrangement must leave the same.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Same {
    /// The HTML pages, for a rearrangement that moves impls.
    Pages,
    /// Every file, for one that keeps each impl in its place.
    All,
}

/// Rearranges a copy of every fixture and compares its outlines with the
/// original's.
fn check(same: Same, transform: impl Fn(&Path)) {
    let kept = |files: Vec<(String, String)>| -> Vec<(String, String)> {
        files
            .into_iter()
            .filter(|(file, _)| same == Same::All || has_ext(file, "html"))
            .collect()
    };
    for name in fixture_names() {
        if SKIP.contains(&name.as_str()) {
            continue;
        }
        let original = fixtures().join(&name);
        let expected = kept(outputs(&input_of(&original)));
        let tmp = tempfile::tempdir().expect("a temporary folder can be made");
        let copy = tmp.path().join(&name);
        copy_dir(&original, &copy);
        transform(&copy);
        let actual = kept(outputs(&input_of(&copy)));
        assert_eq!(
            expected.len(),
            actual.len(),
            "{name}: a different set of files"
        );
        for ((file, want), (_, got)) in expected.iter().zip(&actual) {
            assert!(
                want == got,
                "{name}: {file} changed\n--- before\n{want}\n--- after\n{got}"
            );
        }
    }
}

fn split_lines(text: &str) -> Vec<String> {
    text.split_inclusive('\n').map(str::to_owned).collect()
}

/// Rewrites a file with `edit`, which returns `None` to leave it alone.
fn edit(path: &Path, edit: impl FnOnce(&str, &syn::File) -> Option<String>) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    let Ok(file) = syn::parse_file(&text) else {
        return;
    };
    if let Some(new) = edit(&text, &file) {
        fs::write(path, new).expect("a fixture copy is writable");
    }
}

/// Converts a span's 1-based line to a 0-based line index.
fn line(span: proc_macro2::Span, end: bool) -> usize {
    let at = if end { span.end() } else { span.start() };
    at.line - 1
}

/// Reverses the chunks of `lines` from `from`, chunk k ending at line
/// `ends[k]`; lines after the last chunk stay where they are.
fn reverse_chunks(lines: &mut Vec<String>, from: usize, ends: &[usize]) {
    let increasing =
        ends.first().is_some_and(|&e| e >= from) && ends.windows(2).all(|w| w[0] < w[1]);
    if ends.len() < 2 || !increasing {
        return;
    }
    let mut chunks = Vec::new();
    let mut start = from;
    for &end in ends {
        let mut chunk = lines[start..=end].to_vec();
        if !chunk.last().is_some_and(|l| l.ends_with('\n')) {
            chunk.last_mut().expect("a chunk has a line").push('\n');
        }
        chunks.push(chunk);
        start = end + 1;
    }
    chunks.reverse();
    lines.splice(from..start, chunks.concat());
}

#[test]
fn reversed_items_change_nothing() {
    check(Same::Pages, |dir| {
        for path in rust_files(dir) {
            edit(&path, |text, file| {
                let mut lines = split_lines(text);
                let from = file
                    .attrs
                    .iter()
                    .map(|a| line(a.span(), true) + 1)
                    .max()
                    .unwrap_or(0);
                let ends: Vec<usize> = file.items.iter().map(|i| line(i.span(), true)).collect();
                reverse_chunks(&mut lines, from, &ends);
                Some(lines.concat())
            });
        }
    });
}

/// Reverses the names in a one-line derive attribute.
fn reverse_derive(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let Some(inner) = trimmed
        .trim_end()
        .strip_prefix("#[derive(")
        .and_then(|r| r.strip_suffix(")]"))
    else {
        return line.to_owned();
    };
    let mut names: Vec<&str> = inner.split(',').map(str::trim).collect();
    names.reverse();
    format!("{indent}#[derive({})]\n", names.join(", "))
}

#[test]
fn reversed_derives_and_attributes_change_nothing() {
    check(Same::All, |dir| {
        for path in rust_files(dir) {
            edit(&path, |text, _| {
                let mut lines: Vec<String> = split_lines(text)
                    .iter()
                    .map(|l| reverse_derive(l))
                    .collect();
                let is_attr = |l: &String| {
                    let t = l.trim_start();
                    t.starts_with("#[") && !t.starts_with("#[doc")
                };
                let mut i = 0;
                while i < lines.len() {
                    let run = lines[i..].iter().take_while(|l| is_attr(l)).count();
                    if run > 1 {
                        lines[i..i + run].reverse();
                    }
                    i += run.max(1);
                }
                Some(lines.concat())
            });
        }
    });
}

/// An inherent impl that can be cut apart: its header lines, its members'
/// last lines, and its closing line.
struct Cuttable {
    start: usize,
    open: usize,
    ends: Vec<usize>,
    close: usize,
    header: String,
}

/// Finds the inherent impls whose header ends in `{`, whose closing line
/// is only `}`, and that have no comment above them.
fn cuttable(lines: &[String], file: &syn::File) -> Vec<Cuttable> {
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Impl(i) = item else {
            continue;
        };
        let start = line(item.span(), false);
        let open = line(i.brace_token.span.open(), false);
        let close = line(i.brace_token.span.close(), false);
        let ends: Vec<usize> = i.items.iter().map(|m| line(m.span(), true)).collect();
        let commented = start > 0 && lines[start - 1].trim_start().starts_with("//");
        let documented = i.attrs.iter().any(|a| a.path().is_ident("doc"));
        let shaped = lines[open].trim_end().ends_with('{')
            && lines[close].trim() == "}"
            && ends.first().is_some_and(|&e| e > open)
            && ends.last().is_some_and(|&e| e < close)
            && ends.windows(2).all(|w| w[0] < w[1]);
        if i.trait_.is_none() && !commented && !documented && shaped {
            out.push(Cuttable {
                start,
                open,
                ends,
                close,
                header: lines[start..=open].concat(),
            });
        }
    }
    out
}

#[test]
fn split_impls_change_nothing() {
    check(Same::Pages, |dir| {
        for path in rust_files(dir) {
            edit(&path, |text, file| {
                let lines = split_lines(text);
                let mut out: Vec<String> = Vec::new();
                let mut at = 0;
                for cut in cuttable(&lines, file)
                    .into_iter()
                    .filter(|c| c.ends.len() > 1)
                {
                    out.extend_from_slice(&lines[at..cut.start]);
                    let mut from = cut.open + 1;
                    for (k, &end) in cut.ends.iter().enumerate() {
                        let last = k + 1 == cut.ends.len();
                        out.push(cut.header.clone());
                        out.extend_from_slice(
                            &lines[from..=if last { cut.close - 1 } else { end }],
                        );
                        out.push(lines[cut.close].clone());
                        from = end + 1;
                    }
                    at = cut.close + 1;
                }
                out.extend_from_slice(&lines[at..]);
                Some(out.concat())
            });
        }
    });
}

#[test]
fn merged_impls_change_nothing() {
    check(Same::Pages, |dir| {
        for path in rust_files(dir) {
            edit(&path, |text, file| {
                let lines = split_lines(text);
                let cuts = cuttable(&lines, file);
                let mut skip = vec![false; lines.len()];
                let mut insert: Vec<Vec<String>> = vec![Vec::new(); lines.len()];
                for (k, first) in cuts.iter().enumerate() {
                    if skip[first.start] {
                        continue;
                    }
                    for later in cuts[k + 1..].iter().filter(|c| c.header == first.header) {
                        insert[first.close].extend_from_slice(&lines[later.open + 1..later.close]);
                        skip[later.start..=later.close]
                            .iter_mut()
                            .for_each(|s| *s = true);
                    }
                }
                let mut out = Vec::new();
                for (i, text) in lines.iter().enumerate() {
                    out.extend(insert[i].iter().cloned());
                    if !skip[i] {
                        out.push(text.clone());
                    }
                }
                Some(out.concat())
            });
        }
    });
}

#[test]
fn impls_moved_to_the_root_file_change_nothing() {
    check(Same::Pages, |dir| {
        let root = dir.join("src").join("lib.rs");
        if !root.exists() || dir.join("src").join("main.rs").exists() {
            return;
        }
        let mut moved = String::new();
        for path in rust_files(&dir.join("src"))
            .into_iter()
            .filter(|p| *p != root)
        {
            edit(&path, |text, file| {
                let lines = split_lines(text);
                let mut keep = Vec::new();
                let mut previous = file
                    .attrs
                    .iter()
                    .map(|a| line(a.span(), true) + 1)
                    .max()
                    .unwrap_or(0);
                keep.extend_from_slice(&lines[..previous]);
                for item in &file.items {
                    let end = line(item.span(), true);
                    if end < previous {
                        return None;
                    }
                    if matches!(item, Item::Impl(_)) {
                        moved.push('\n');
                        moved.push_str(&lines[previous..=end].concat());
                    } else {
                        keep.extend_from_slice(&lines[previous..=end]);
                    }
                    previous = end + 1;
                }
                keep.extend_from_slice(&lines[previous..]);
                Some(keep.concat())
            });
        }
        let mut text = fs::read_to_string(&root).expect("the root file is readable");
        text.push_str(&moved);
        fs::write(&root, text).expect("the root file is writable");
    });
}

/// Returns the text before `mod` on a line, such as `pub `.
fn prefix(line: &str) -> Option<&str> {
    line.find("mod ").map(|at| &line[..at])
}

#[test]
fn file_modules_made_inline_change_nothing() {
    check(Same::Pages, |dir| {
        let src = dir.join("src");
        edit(&src.join("lib.rs"), |text, file| {
            let mut lines = split_lines(text);
            for item in file.items.iter().rev() {
                let Item::Mod(m) = item else {
                    continue;
                };
                let line_no = line(m.mod_token.span, false);
                let name = m.ident.to_string();
                let has_path = m.attrs.iter().any(|a| a.path().is_ident("path"));
                let candidates = [
                    src.join(format!("{name}.rs")),
                    src.join(&name).join("mod.rs"),
                ];
                let Some(found) = candidates.iter().find(|p| p.exists()) else {
                    continue;
                };
                if m.content.is_some() || has_path || line(item.span(), true) != line_no {
                    continue;
                }
                let body = fs::read_to_string(found).expect("a module file is readable");
                let lead = prefix(&lines[line_no]).unwrap_or_default().to_owned();
                lines[line_no] = format!("{lead}mod {name} {{\n{body}}}\n");
                fs::remove_file(found).expect("a copied module file can be removed");
            }
            Some(lines.concat())
        });
    });
}

#[test]
fn inline_modules_moved_to_files_change_nothing() {
    check(Same::Pages, |dir| {
        let src = dir.join("src");
        edit(&src.join("lib.rs"), |text, file| {
            let mut lines = split_lines(text);
            for item in file.items.iter().rev() {
                let Item::Mod(m) = item else {
                    continue;
                };
                let Some((brace, _)) = &m.content else {
                    continue;
                };
                let mod_line = line(m.mod_token.span, false);
                let open = line(brace.span.open(), false);
                let close = line(brace.span.close(), false);
                let file_name = m.ident.to_string().trim_start_matches("r#").to_owned();
                let target = src.join(format!("{file_name}.rs"));
                let has_path = m.attrs.iter().any(|a| a.path().is_ident("path"));
                if open != mod_line || lines[close].trim() != "}" || has_path || target.exists() {
                    continue;
                }
                fs::write(&target, lines[open + 1..close].concat())
                    .expect("a module file can be written");
                let lead = prefix(&lines[mod_line]).unwrap_or_default().to_owned();
                lines.splice(mod_line..=close, [format!("{lead}mod {};\n", m.ident)]);
            }
            Some(lines.concat())
        });
    });
}

#[test]
fn rustfmt_with_a_narrow_width_changes_nothing() {
    check(Same::All, |dir| {
        let roots: Vec<_> = ["src/lib.rs", "src/main.rs"]
            .iter()
            .map(|r| dir.join(r))
            .filter(|p| p.exists())
            .chain(
                rust_files(dir)
                    .into_iter()
                    .filter(|p| p.parent() == Some(dir)),
            )
            .collect();
        for root in roots {
            let status = Command::new("rustfmt")
                .args([
                    "--edition",
                    "2024",
                    "--config",
                    "max_width=40,style_edition=2024",
                ])
                .arg(&root)
                .status()
                .expect("rustfmt runs");
            assert!(status.success(), "rustfmt failed on {}", root.display());
        }
    });
}
