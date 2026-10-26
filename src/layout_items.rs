//! Lays out module trailing blocks, docs fences, and Other impls entries.

use std::collections::BTreeMap;

use crate::layout::{
    Block, Entry, Line, Span, above, code, const_line, fn_lines, join_groups, key, mark_line,
    note_lines, plain, sort_entries,
};
use crate::layout_decl::{BodyParts, body};
use crate::model::{Crate, Foreign, Impl, Item, ItemKind, Loc, ModId, Note, name_order};

/// Builds a module's trailing block: the public items, then `private:` and
/// the private items, each section grouped as re-exports, macro marks,
/// constants, statics, free functions, and extern blocks.
pub(crate) fn module_block(krate: &Crate, id: ModId) -> Option<Block> {
    let module = &krate.modules[id];
    let mut public: Vec<Vec<Entry>> = (0..6).map(|_| Vec::new()).collect();
    let mut private: Vec<Vec<Entry>> = (0..6).map(|_| Vec::new()).collect();
    let mut foreign: [Vec<&Item>; 2] = [Vec::new(), Vec::new()];
    let mut unsupported = Vec::new();
    let mut use_groups: BTreeMap<(bool, usize), Vec<Entry>> = BTreeMap::new();
    for item in &module.items {
        if let ItemKind::Reexport {
            group: Some(id), ..
        } = item.kind
        {
            let entries = use_groups.entry((item.public, id)).or_default();
            entries.push(item_entry(krate, item));
            continue;
        }
        let group = match &item.kind {
            ItemKind::Reexport { .. } => 0,
            ItemKind::Mark(_) => 1,
            ItemKind::Const { .. } => 2,
            ItemKind::Static { .. } => 3,
            ItemKind::Fn(_) => 4,
            ItemKind::Foreign { .. } => {
                foreign[usize::from(!item.public)].push(item);
                continue;
            }
            ItemKind::Unsupported(kind) => {
                unsupported.push((kind.as_str(), item.loc));
                continue;
            }
        };
        let entry = item_entry(krate, item);
        if item.public {
            public[group].push(entry);
        } else {
            private[group].push(entry);
        }
    }
    for ((is_public, _), mut entries) in use_groups {
        sort_entries(&mut entries, &krate.refs);
        let merged = Entry {
            name: entries[0].name.clone(),
            lines: entries.iter_mut().flat_map(|e| e.lines.drain(..)).collect(),
            repeats: entries
                .iter_mut()
                .flat_map(|e| e.repeats.drain(..))
                .collect(),
        };
        if is_public {
            public[0].push(merged);
        } else {
            private[0].push(merged);
        }
    }
    for group in public.iter_mut().chain(private.iter_mut()) {
        sort_entries(group, &krate.refs);
    }
    let [public_foreign, private_foreign] = foreign;
    public[5] = extern_blocks(krate, &public_foreign);
    private[5] = extern_blocks(krate, &private_foreign);

    let mut block = Block::default();
    join_groups(public, &mut block);
    if private.iter().any(|g| !g.is_empty()) {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        block.lines.push(Line {
            indent: 0,
            spans: vec![key("private:")],
            trailing: None,
        });
        let mut section = Block::default();
        join_groups(private, &mut section);
        block.lines.extend(section.lines);
        block.repeats.extend(section.repeats);
    }
    let dangling = note_lines(&module.dangling, 0, "///");
    if !dangling.is_empty() {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        block.lines.extend(dangling);
    }
    if !unsupported.is_empty() {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        for (kind, loc) in unsupported {
            let (line, repeat) = mark_line("UNSUPPORTED:", kind, 0, Some(loc));
            block.lines.push(line);
            block.repeats.push(repeat);
        }
    }
    (!block.lines.is_empty()).then_some(block)
}

fn item_entry(krate: &Crate, item: &Item) -> Entry {
    let refs = &krate.refs;
    let (mut lines, mut repeats) = above(&item.meta, 0, item.loc);
    let name = Span::Name(item.name.clone(), item.loc);
    let trailing = item.meta.trailing.clone();
    match &item.kind {
        ItemKind::Reexport { text, .. } => {
            let mut spans = vec![key("using"), plain(" ")];
            spans.extend(code(text));
            spans.push(plain(";"));
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
        ItemKind::Fn(sig) => lines.extend(fn_lines(krate, sig, item.loc, trailing, 0, false)),
        ItemKind::Foreign { .. } | ItemKind::Unsupported(_) => {}
    }
    Entry {
        name: item.name.clone(),
        lines,
        repeats,
    }
}

/// Merges extern-block items into one block per ABI, sorted by ABI.
fn extern_blocks(krate: &Crate, items: &[&Item]) -> Vec<Entry> {
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
            let name = Span::Name(item.name.clone(), item.loc);
            let trailing = item.meta.trailing.clone();
            let group = match foreign {
                Foreign::Type => {
                    lines.push(Line {
                        indent: 4,
                        spans: vec![key("struct"), plain(" "), name, plain(";")],
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
                    spans.push(plain(";"));
                    lines.push(Line {
                        indent: 4,
                        spans,
                        trailing,
                    });
                    1
                }
                Foreign::Fn(sig) => {
                    lines.extend(fn_lines(krate, sig, item.loc, trailing, 4, false));
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

/// Builds a docs fence: `//!` lines, then one `// cfg:` line per `cfg`.
pub(crate) fn docs_block(notes: &[Note], cfgs: &[String]) -> Option<Block> {
    let mut lines = note_lines(notes, 0, "//!");
    for cfg in cfgs {
        lines.push(Line {
            indent: 0,
            spans: vec![Span::Comment(format!("// cfg: {cfg}"))],
            trailing: None,
        });
    }
    (!lines.is_empty()).then(|| Block {
        lines,
        repeats: Vec::new(),
    })
}

/// One `implements:` line of the Other impls section.
pub(crate) struct Implements {
    /// Comment lines printed above the line.
    pub(crate) notes: Vec<Line>,
    /// `implements: Trait for Type;`
    pub(crate) spans: Vec<Span>,
    /// Macro marks from the impl's body and attributes.
    pub(crate) marks: Vec<String>,
    pub(crate) loc: Loc,
}

/// Builds the Other impls section: `implements:` lines for trait impls,
/// sorted by trait and then by self type, and a block for each inherent
/// impl, sorted by self type.
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
            let mut spans = vec![key("implements:"), plain(" ")];
            if let Some(base) = &item.trait_ {
                spans.extend(code(&base.text));
            }
            spans.push(plain(" for "));
            spans.extend(code(&item.self_text));
            spans.push(plain(";"));
            let mut marks = item.marks.clone();
            if !item.attributes.is_empty() {
                marks.push(format!("attribute: {}", item.attributes.join(", ")));
            }
            Implements {
                notes: note_lines(&item.notes, 0, "///"),
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
                lines: note_lines(&item.notes, 0, "///"),
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
                    class: false,
                    all_public: false,
                    dangling: &item.dangling,
                    end_marks: &item.marks,
                    special: &[],
                    name: "",
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
