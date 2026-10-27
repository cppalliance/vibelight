//! Lays out the module-level items of a module block the way Rust writes
//! them: re-exports, macro marks, constants, statics, functions, and extern
//! blocks, and the Rust form of the Other impls section.

use crate::layout::{Block, Entry, Line, join_groups, key, mark_line, note_lines, sort_entries};
use crate::layout_rust::{above, code, fn_lines, line, plain, value_line, vis};
use crate::layout_rust_decl::member_lines;
use crate::model::{Crate, Foreign, Impl, Item, ItemKind, name_order};

/// Builds one module-level item. An extern-block item or an unsupported
/// construct goes elsewhere and gets no lines here.
pub(crate) fn item_entry(krate: &Crate, item: &Item) -> Entry {
    let refs = &krate.refs;
    let (mut lines, mut repeats) = above(&item.meta, 0, item.loc);
    let vis = vis(&item.meta);
    let trailing = item.meta.trailing.clone();
    match &item.kind {
        ItemKind::Reexport { text, .. } => {
            let mut spans = vec![plain(&vis), key("use"), plain(" ")];
            spans.extend(code(text));
            spans.push(plain(";"));
            lines.push(Line {
                indent: 0,
                spans,
                trailing,
            });
        }
        ItemKind::Mark(text) => {
            let (mark, repeat) = mark_line("MACRO", text, 0, Some(item.loc));
            lines.push(Line { trailing, ..mark });
            repeats.push(repeat);
        }
        ItemKind::Const { ty, value } => {
            let mut spans = vec![
                plain(&vis),
                key("const"),
                plain(&format!(" {}: ", item.name)),
            ];
            spans.extend(code(ty));
            lines.push(value_line(spans, value.as_deref(), 0, trailing, refs));
        }
        ItemKind::Static { mutable, ty, value } => {
            let mutable = if *mutable { "mut " } else { "" };
            let mut spans = vec![
                plain(&vis),
                key("static"),
                plain(&format!(" {mutable}{}: ", item.name)),
            ];
            spans.extend(code(ty));
            lines.push(value_line(spans, value.as_deref(), 0, trailing, refs));
        }
        ItemKind::Fn(sig) => lines.extend(fn_lines(sig, &vis, 0, trailing)),
        ItemKind::Foreign { .. } | ItemKind::Unsupported(_) => {}
    }
    Entry {
        name: item.name.clone(),
        lines,
        repeats,
    }
}

/// Merges extern-block items into one `extern` block per ABI, sorted by
/// ABI, each holding its types, then its statics, then its functions.
pub(crate) fn extern_blocks(krate: &Crate, items: &[&Item]) -> Vec<Entry> {
    let mut abis: Vec<&str> = items
        .iter()
        .filter_map(|i| match &i.kind {
            ItemKind::Foreign { abi, .. } => Some(abi.as_str()),
            _ => None,
        })
        .collect();
    abis.sort_by(|a, b| name_order(a, b));
    abis.dedup();
    let mut entries = Vec::new();
    for abi in abis {
        let mut groups: Vec<Vec<Entry>> = (0..3).map(|_| Vec::new()).collect();
        for item in items {
            let ItemKind::Foreign {
                abi: own,
                item: foreign,
            } = &item.kind
            else {
                continue;
            };
            if own != abi {
                continue;
            }
            let (mut lines, repeats) = above(&item.meta, 4, item.loc);
            let vis = vis(&item.meta);
            let trailing = item.meta.trailing.clone();
            let group = match foreign {
                Foreign::Type => {
                    let spans = vec![plain(&vis), key("type"), plain(&format!(" {};", item.name))];
                    lines.push(Line {
                        indent: 4,
                        spans,
                        trailing,
                    });
                    0
                }
                Foreign::Static { mutable, ty } => {
                    let mutable = if *mutable { "mut " } else { "" };
                    let mut spans = vec![
                        plain(&vis),
                        key("static"),
                        plain(&format!(" {mutable}{}: ", item.name)),
                    ];
                    spans.extend(code(ty));
                    spans.push(plain(";"));
                    lines.push(Line {
                        indent: 4,
                        spans,
                        trailing,
                    });
                    1
                }
                Foreign::Fn(sig) => {
                    lines.extend(fn_lines(sig, &vis, 4, trailing));
                    2
                }
            };
            groups[group].push(Entry {
                name: item.name.clone(),
                lines,
                repeats,
            });
        }
        for group in &mut groups {
            sort_entries(group, &krate.refs);
        }
        let mut inner = Block::default();
        join_groups(groups, &mut inner);
        let open = if abi.is_empty() {
            "extern {".to_owned()
        } else {
            format!("extern {abi} {{")
        };
        let mut lines = vec![line(0, vec![plain(&open)])];
        lines.extend(inner.lines);
        lines.push(line(0, vec![plain("}")]));
        entries.push(Entry {
            name: abi.to_owned(),
            lines,
            repeats: inner.repeats,
        });
    }
    entries
}

/// Returns a trait impl of the Other impls section as one line, such as
/// `impl Render for &Widget {}`.
pub(crate) fn implements(item: &Impl) -> Vec<crate::layout::Span> {
    vec![plain(&format!("{} {{}}", item.header))]
}

/// Returns an inherent impl of the Other impls section as a block: its
/// header and its members.
pub(crate) fn inherent_block(krate: &Crate, item: &Impl) -> Block {
    let mut block = Block {
        lines: note_lines(&item.notes, 0, "///"),
        repeats: Vec::new(),
    };
    block
        .lines
        .push(line(0, vec![plain(&format!("{} {{", item.header))]));
    let mut members = Vec::new();
    for member in &item.members {
        let (lines, repeats) = member_lines(member, 4, &krate.refs);
        members.push(lines);
        block.repeats.extend(repeats);
    }
    let dangling = note_lines(&item.dangling, 4, "///");
    if !dangling.is_empty() {
        members.push(dangling);
    }
    for mark in &item.marks {
        let (mark, repeat) = mark_line("MACRO", mark, 4, None);
        members.push(vec![mark]);
        block.repeats.push(repeat);
    }
    block.lines.extend(crate::layout::spaced(members));
    block.lines.push(line(0, vec![plain("}")]));
    block
}
