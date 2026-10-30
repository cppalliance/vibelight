//! Reads the raw doc text of a run: where each node's doc text is, the
//! compiled Rust code blocks that rustdoc tests, which the census scans as
//! doc tests, and each doc's first paragraph, its summary.

use crate::model::{Crate, ItemKind, Loc, Param, Shape, VariantFields};

/// The words of a code block's info string that rustdoc reads itself; a
/// block whose info string holds only these, or nothing, is Rust.
const RUSTDOC_WORDS: [&str; 8] = [
    "allow_fail",
    "compile_fail",
    "ignore",
    "no_run",
    "rust",
    "should_panic",
    "standalone_crate",
    "test_harness",
];

/// Returns each doc text of a crate's nodes, with the node's location: the
/// modules, the declarations, their members, fields, and variants, the
/// members of every impl, and the module items other than re-exports, which
/// share their `use` item's doc text.
pub(crate) fn doc_texts<'a>(krate: &'a Crate) -> Vec<(Loc, &'a str)> {
    let mut out = Vec::new();
    let mut add = |loc: Loc, doc: &'a Option<String>| {
        if let Some(doc) = doc {
            out.push((loc, doc.as_str()));
        }
    };
    for module in &krate.modules {
        let loc = module.inline.unwrap_or(Loc {
            file: module.file,
            line: 1,
        });
        add(loc, &module.raw_doc);
        for item in &module.items {
            if !matches!(item.kind, ItemKind::Reexport { .. }) {
                add(item.loc, &item.meta.raw_doc);
            }
        }
    }
    for decl in &krate.decls {
        add(decl.loc, &decl.meta.raw_doc);
        for member in &decl.members {
            add(member.loc, &member.meta.raw_doc);
        }
        let mut fields: Vec<(Loc, &'a [Param])> = Vec::new();
        match &decl.shape {
            Shape::Tuple(params) => fields.push((decl.loc, params)),
            Shape::Enum(variants) => {
                for variant in variants {
                    add(variant.loc, &variant.meta.raw_doc);
                    if let VariantFields::Tuple(params) | VariantFields::Named(params) =
                        &variant.fields
                    {
                        fields.push((variant.loc, params));
                    }
                }
            }
            _ => {}
        }
        for (loc, params) in fields {
            for param in params {
                add(loc, &param.meta.raw_doc);
            }
        }
    }
    for member in krate
        .impls
        .iter()
        .flat_map(|i| i.members.iter().chain(&i.trait_members))
    {
        add(member.loc, &member.meta.raw_doc);
    }
    out
}

/// Returns the fence a line opens or closes: its character, how many of
/// them it has, and the info string after them. A backtick fence's info
/// string holds no backtick.
pub(crate) fn fence(line: &str) -> Option<(char, usize, &str)> {
    let line = line.trim_start();
    let mark = line.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let count = line.chars().take_while(|&c| c == mark).count();
    let info = line[count..].trim();
    (count >= 3 && !(mark == '`' && info.contains('`'))).then_some((mark, count, info))
}

/// Tells whether a code block with this info string is a doc test rustdoc
/// compiles: Rust, and neither `ignore`d nor `compile_fail`; a `no_run`
/// block counts.
fn runnable(info: &str) -> bool {
    let mut words = info
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|w| !w.is_empty());
    words.all(|word| {
        let known = RUSTDOC_WORDS.contains(&word) || word.starts_with("edition");
        known && word != "compile_fail" && !word.starts_with("ignore")
    })
}

/// Returns the code of each doc test a doc text fences, with the marker
/// of each hidden line removed: `# ` or a lone `#`, and the first `#` of
/// `##`. A block that is never closed runs to the end of the text.
pub(crate) fn doctests(doc: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut open: Option<(char, usize, bool)> = None;
    let mut code = String::new();
    for line in doc.lines() {
        match open {
            None => {
                if let Some((mark, count, info)) = fence(line) {
                    open = Some((mark, count, runnable(info)));
                    code.clear();
                }
            }
            Some((mark, count, run)) => {
                let closed = fence(line)
                    .is_some_and(|(m, n, info)| m == mark && n >= count && info.is_empty());
                if closed {
                    if run {
                        out.push(std::mem::take(&mut code));
                    }
                    open = None;
                } else if run {
                    code.push_str(shown(line));
                    code.push('\n');
                }
            }
        }
    }
    if let Some((_, _, true)) = open {
        out.push(code);
    }
    out
}

/// Returns a doc test line as rustdoc compiles it.
fn shown(line: &str) -> &str {
    match line.trim_start().strip_prefix('#') {
        Some("") => "",
        Some(rest) => rest
            .strip_prefix(' ')
            .or_else(|| rest.starts_with('#').then_some(rest))
            .unwrap_or(line),
        None => line,
    }
}

/// Returns a doc text's summary: its lines up to the first blank line or
/// code fence, with their line breaks; `None` when that leaves nothing.
pub(crate) fn summary(doc: &str) -> Option<String> {
    let lines: Vec<&str> = doc
        .lines()
        .take_while(|line| !line.trim().is_empty() && fence(line).is_none())
        .collect();
    let text = lines.join("\n");
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_runnable_rust_blocks_are_doc_tests_with_hidden_markers_removed() {
        let doc = "Runs it.\n\n```\nlet a = t::one();\n# let hidden = 1;\n##[derive]\n```\n\
                   ```rust,no_run\nt::two();\n```\n```ignore\nt::three();\n```\n\
                   ```compile_fail\nt::four();\n```\n```text\nt::five();\n```\n\
                   ~~~edition2021\nt::six();\n~~~\n  ```should_panic\n  t::seven();\n";
        assert_eq!(
            doctests(doc),
            [
                "let a = t::one();\nlet hidden = 1;\n#[derive]\n",
                "t::two();\n",
                "t::six();\n",
                "  t::seven();\n"
            ]
        );
    }

    #[test]
    fn a_summary_is_the_first_paragraph_before_any_fence() {
        assert_eq!(
            summary("First line\nsecond line.\n\nMore.").as_deref(),
            Some("First line\nsecond line.")
        );
        assert_eq!(
            summary("Reads it:\n```\ncode\n```").as_deref(),
            Some("Reads it:")
        );
        assert_eq!(summary("```\ncode\n```"), None);
        assert_eq!(fence("  ````rust "), Some(('`', 4, "rust")));
        assert_eq!(fence("``not`"), None);
    }
}
