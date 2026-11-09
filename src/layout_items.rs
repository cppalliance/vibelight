//! Lays out module trailing blocks, docs fences, and Other impls entries.

use std::collections::BTreeMap;

use crate::access::Access;
use crate::access_stamp::Stamp;
use crate::layout::{
    Block, Line, Span, above, code, const_line, fn_lines, key, line_text, mark_line, note_lines,
    plain,
};
use crate::layout_decl::{BodyParts, body};
use crate::layout_groups::{
    Entry, Label, Sections, add, join_sections, label, reaches_line, sort_entries,
};
use crate::layout_marker::Marker;
use crate::macros;
use crate::model::{Crate, Foreign, Impl, Item, ItemKind, Loc, ModId, Note, ValueRef, name_order};

/// Builds a module's trailing block: first what has no label of its own,
/// macro marks, one extern block per ABI with its items labeled inside,
/// and unsupported lines; then the items in sections by visibility, every
/// section opened by its label and grouped as re-exports, constants,
/// statics, and free functions; then comments with nothing after them.
pub(crate) fn module_block(krate: &Crate, id: ModId) -> Option<Block> {
    let module = &krate.modules[id];
    let shown = module.access.as_ref().and_then(Stamp::note);
    let entry = |item: &Item, at: ValueRef| item_entry(krate, item, at, shown);
    let mut sections = Sections::new();
    let mut marks = Vec::new();
    let mut foreign: Vec<&Item> = Vec::new();
    let mut unsupported = Vec::new();
    let mut use_groups: BTreeMap<(Label, usize), Vec<Entry>> = BTreeMap::new();
    for (index, item) in module.items.iter().enumerate() {
        let at = ValueRef {
            module: id,
            item: index,
        };
        if let ItemKind::Reexport {
            group: Some(set), ..
        } = item.kind
        {
            let entries = use_groups.entry((label(&item.meta), set)).or_default();
            entries.push(entry(item, at));
            continue;
        }
        let group = match &item.kind {
            ItemKind::Reexport { .. } => 0,
            ItemKind::Const { .. } => 1,
            ItemKind::Static { .. } => 2,
            ItemKind::Fn(_) => 3,
            ItemKind::Mark(_) => {
                marks.push(entry(item, at));
                continue;
            }
            ItemKind::Foreign { .. } => {
                foreign.push(item);
                continue;
            }
            ItemKind::Unsupported(kind) => {
                unsupported.push(mark_line("UNSUPPORTED:", kind, 0, Some(item.loc)));
                continue;
            }
        };
        add(&mut sections, label(&item.meta), 4, group, entry(item, at));
    }
    for ((at, _), mut entries) in use_groups {
        sort_entries(&mut entries, &krate.refs);
        let merged = Entry {
            name: entries[0].name.clone(),
            lines: entries.iter_mut().flat_map(|e| e.lines.drain(..)).collect(),
            repeats: entries
                .iter_mut()
                .flat_map(|e| e.repeats.drain(..))
                .collect(),
        };
        add(&mut sections, at, 4, 0, merged);
    }
    for groups in sections.values_mut() {
        for group in groups.iter_mut() {
            sort_entries(group, &krate.refs);
        }
        let reexports = std::mem::take(&mut groups[0]);
        if !reexports.is_empty() {
            groups[0] = vec![packed(reexports, "using")];
        }
    }
    sort_entries(&mut marks, &krate.refs);
    let (lines, repeats): (Vec<Line>, Vec<_>) = unsupported.into_iter().unzip();
    let unsupported = (!lines.is_empty()).then(|| Entry {
        name: String::new(),
        lines,
        repeats,
    });
    let first = vec![
        marks,
        extern_blocks(krate, &foreign, shown),
        unsupported.into_iter().collect(),
    ];
    let mut block = Block::default();
    join_sections(first, [sections, Sections::new()], &mut block);
    let dangling = note_lines(&module.dangling, 0, Marker::DOC);
    if !dangling.is_empty() {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        block.lines.extend(dangling);
    }
    (!block.lines.is_empty()).then_some(block)
}

/// Builds the block of one module-level function, constant, or static, as
/// the module's block lays it out, under the name a user writes for it.
pub(crate) fn item_block(krate: &Crate, at: ValueRef, name: &str) -> Block {
    let mut item = krate.modules[at.module].items[at.item].clone();
    name.clone_into(&mut item.name);
    let entry = item_entry(krate, &item, at, None);
    Block {
        lines: entry.lines,
        repeats: entry.repeats,
    }
}

/// Returns the text a module-level function, constant, or static prints
/// as, which orders the items whose ids would repeat.
pub(crate) fn item_text(krate: &Crate, at: ValueRef) -> String {
    let item = &krate.modules[at.module].items[at.item];
    let entry = item_entry(krate, item, at, None);
    let lines: Vec<String> = entry
        .lines
        .iter()
        .map(|line| line_text(line, &krate.refs))
        .collect();
    lines.join("\n")
}

/// Returns a module item's entry, with a `reaches` line above it when it
/// reaches less than its visibility gives and its module's note, `shown`,
/// says otherwise.
fn item_entry(krate: &Crate, item: &Item, at: ValueRef, shown: Option<Access>) -> Entry {
    let refs = &krate.refs;
    let attrs_above = !matches!(item.kind, ItemKind::Fn(_));
    let (mut lines, mut repeats) = above(&item.meta, 0, item.loc, attrs_above);
    lines.extend(reaches_line(&item.meta, 0, shown));
    let name = Span::Value(item.name.clone(), item.loc, at);
    let trailing = item.meta.trailing.clone();
    match &item.kind {
        ItemKind::Reexport { text, .. } => {
            let mut spans = vec![key("using"), plain(" ")];
            spans.extend(code(text));
            lines.push(Line {
                indent: 0,
                spans,
                trailing,
            });
        }
        ItemKind::Mark(text) => {
            let (line, repeat) = mark_line("MACRO", text, 0, Some(item.loc));
            lines.push(Line { trailing, ..line });
            repeats.push(repeat);
        }
        ItemKind::Const { ty, value } => {
            let mut spans = vec![key("constexpr"), plain(" "), name, plain(": ")];
            spans.extend(code(ty));
            lines.push(const_line(spans, value.as_deref(), 0, trailing, refs));
        }
        ItemKind::Static { mutable, ty, value } => {
            let mut spans = vec![key("static"), plain(" ")];
            if *mutable {
                spans.push(key("mut"));
                spans.push(plain(" "));
            }
            spans.extend([name, plain(": ")]);
            spans.extend(code(ty));
            lines.push(const_line(spans, value.as_deref(), 0, trailing, refs));
        }
        ItemKind::Fn(sig) => lines.extend(fn_lines(
            krate,
            sig,
            name,
            trailing,
            0,
            false,
            &item.meta.attrs,
        )),
        ItemKind::Foreign { .. } | ItemKind::Unsupported(_) => {}
    }
    Entry {
        name: item.name.clone(),
        lines,
        repeats,
    }
}

/// Packs sorted re-export entries into one entry, so their `using` lines
/// sit together; `word` is the keyword. An entry with a line above
/// its first `using` line, such as a comment or a `// cfg:` line, keeps a
/// blank line on each side, so that line never seems to cover its
/// neighbors.
fn packed(entries: Vec<Entry>, word: &str) -> Entry {
    let plain = |entry: &Entry| {
        entry
            .lines
            .first()
            .is_some_and(|line| line.spans.contains(&key(word)))
    };
    let name = entries[0].name.clone();
    let mut lines = Vec::new();
    let mut repeats = Vec::new();
    let mut last_plain = true;
    for entry in entries {
        let this_plain = plain(&entry);
        if !lines.is_empty() && (!last_plain || !this_plain) {
            lines.push(Line::default());
        }
        last_plain = this_plain;
        lines.extend(entry.lines);
        repeats.extend(entry.repeats);
    }
    Entry {
        name,
        lines,
        repeats,
    }
}

/// Merges extern-block items into one block per ABI, sorted by ABI, each
/// with its items in sections by visibility inside its braces.
fn extern_blocks(krate: &Crate, items: &[&Item], shown: Option<Access>) -> Vec<Entry> {
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
        let mut sections = Sections::new();
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
            let attrs_above = !matches!(foreign, Foreign::Fn(_));
            let (mut lines, repeats) = above(&item.meta, 4, item.loc, attrs_above);
            lines.extend(reaches_line(&item.meta, 4, shown));
            let name = Span::Name(item.name.clone(), item.loc);
            let trailing = item.meta.trailing.clone();
            let group = match foreign {
                Foreign::Type => {
                    lines.push(Line {
                        indent: 4,
                        spans: vec![key("struct"), plain(" "), name],
                        trailing,
                    });
                    0
                }
                Foreign::Static { mutable, ty } => {
                    let mut spans = vec![key("static"), plain(" ")];
                    if *mutable {
                        spans.push(key("mut"));
                        spans.push(plain(" "));
                    }
                    spans.extend([name, plain(": ")]);
                    spans.extend(code(ty));
                    lines.push(Line {
                        indent: 4,
                        spans,
                        trailing,
                    });
                    1
                }
                Foreign::Fn(sig) => {
                    let fn_name = Span::Name(sig.name.clone(), item.loc);
                    let attrs = &item.meta.attrs;
                    lines.extend(fn_lines(krate, sig, fn_name, trailing, 4, false, attrs));
                    2
                }
            };
            let entry = Entry {
                name: item.name.clone(),
                lines,
                repeats,
            };
            add(&mut sections, label(&item.meta), 3, group, entry);
        }
        for group in sections.values_mut().flatten() {
            sort_entries(group, &krate.refs);
        }
        let mut inner = Block::default();
        join_sections(Vec::new(), [sections, Sections::new()], &mut inner);
        let mut lines = vec![
            Line {
                indent: 0,
                spans: vec![key("extern"), plain(&format!(" {abi}"))],
                trailing: None,
            },
            Line {
                indent: 0,
                spans: vec![plain("{")],
                trailing: None,
            },
        ];
        lines.extend(inner.lines);
        lines.push(Line {
            indent: 0,
            spans: vec![plain("}")],
            trailing: None,
        });
        entries.push(Entry {
            name: abi.to_owned(),
            lines,
            repeats: inner.repeats,
        });
    }
    entries
}

/// Builds a docs fence: `//!` lines, then a `// cfg:` line per `cfg`.
pub(crate) fn docs_block(notes: &[Note], cfgs: &[String]) -> Option<Block> {
    let mut lines = note_lines(notes, 0, Marker::MODULE);
    for cfg in cfgs {
        lines.push(Line {
            indent: 0,
            spans: vec![Span::Marker(Marker::CFG.spaced), Span::Comment(cfg.clone())],
            trailing: None,
        });
    }
    (!lines.is_empty()).then(|| Block {
        lines,
        repeats: Vec::new(),
    })
}

/// Returns `implements: Trait for Type;`.
fn implements(item: &Impl) -> Vec<Span> {
    let mut spans = vec![key("implements:"), plain(" ")];
    if let Some(base) = &item.trait_ {
        spans.extend(code(&base.text));
    }
    spans.push(plain(" for "));
    spans.extend(code(&item.self_text));
    spans
}

/// One trait-impl line of the Other impls section.
pub(crate) struct Implements {
    /// Comment lines printed above the line.
    pub(crate) notes: Vec<Line>,
    /// `implements: Trait for Type;`
    pub(crate) spans: Vec<Span>,
    /// Macro marks from the impl's body and attributes.
    pub(crate) marks: Vec<String>,
    pub(crate) loc: Loc,
}

/// Builds the Other impls section: a line for each trait impl, sorted by
/// trait and then by self type, `implements: Trait for Type;`, and a block
/// for each inherent impl, sorted by self type.
pub(crate) fn other_impls(krate: &Crate, ids: &[usize]) -> (Vec<Implements>, Vec<(Block, Loc)>) {
    let refs = &krate.refs;
    let mut traits: Vec<&Impl> = Vec::new();
    let mut inherent: Vec<&Impl> = Vec::new();
    for &id in ids {
        let item = &krate.impls[id];
        if item.trait_.is_some() {
            traits.push(item);
        } else {
            inherent.push(item);
        }
    }
    traits.sort_by(|a, b| {
        let (ta, tb) = (a.trait_.as_ref(), b.trait_.as_ref());
        let name = |t: Option<&crate::model::Base>| t.map(|t| t.name.clone()).unwrap_or_default();
        let text =
            |t: Option<&crate::model::Base>| t.map(|t| t.text.visible(refs)).unwrap_or_default();
        name_order(&name(ta), &name(tb))
            .then_with(|| name_order(&text(ta), &text(tb)))
            .then_with(|| name_order(&a.self_text.visible(refs), &b.self_text.visible(refs)))
    });
    inherent.sort_by(|a, b| name_order(&a.self_text.visible(refs), &b.self_text.visible(refs)));
    let lines = traits
        .into_iter()
        .map(|item| {
            let spans = implements(item);
            let mut marks = item.marks.clone();
            if !item.attributes.is_empty() {
                marks.push(macros::mark_text(macros::ATTRIBUTE, &item.attributes));
            }
            Implements {
                notes: note_lines(&item.notes, 0, Marker::DOC),
                spans,
                marks,
                loc: item.loc,
            }
        })
        .collect();
    let blocks = inherent
        .into_iter()
        .map(|item| {
            let mut block = Block {
                lines: note_lines(&item.notes, 0, Marker::DOC),
                repeats: Vec::new(),
            };
            let mut header = vec![key("impl"), plain(" ")];
            header.extend(code(&item.self_text));
            block.lines.push(Line {
                indent: 0,
                spans: header,
                trailing: None,
            });
            block.lines.push(Line {
                indent: 0,
                spans: vec![plain("{")],
                trailing: None,
            });
            let inner = body(
                krate,
                &BodyParts {
                    members: &item.members,
                    variants: None,
                    unlabeled: false,
                    dangling: &item.dangling,
                    end_marks: &item.marks,
                    special: &[],
                    name: "",
                    own: (0, "pub".to_owned()),
                    shown: None,
                },
            );
            block.lines.extend(inner.lines);
            block.repeats.extend(inner.repeats);
            block.lines.push(Line {
                indent: 0,
                spans: vec![plain("}")],
                trailing: None,
            });
            (block, item.loc)
        })
        .collect();
    (lines, blocks)
}
