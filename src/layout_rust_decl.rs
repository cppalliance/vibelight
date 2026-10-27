//! Lays out a declaration the way Rust writes it, bodies left out: the
//! item with its attributes and its fields, variants, or trait items, then
//! each inherent impl that adds members, under a line naming its file, then
//! one line for each trait impl.

use crate::layout::{Block, Line, Repeat, Span, key, mark_line, note_lines, spaced};
use crate::layout_rust::{
    above, attr_lines, code, fn_lines, line, plain, value_line, vis, where_lines,
};
use crate::model::{
    Crate, Decl, DeclId, DeclKind, ImplLoc, Member, MemberKind, Param, Shape, TypeRef, Variant,
    VariantFields, Where,
};

/// The lines of one entry of a body and the marks they repeat.
type Lines = (Vec<Line>, Vec<Repeat>);

/// Builds the Rust block of one declaration.
pub(crate) fn decl_block(krate: &Crate, id: DeclId) -> Block {
    let decl = &krate.decls[id];
    let mut block = Block::default();
    block.lines.extend(note_lines(&decl.meta.notes, 0, "///"));
    block
        .lines
        .extend(attr_lines(&decl.written_derives, &decl.meta, 0));
    item(krate, decl, &mut block);
    impls(krate, decl, &mut block);
    block
}

/// Returns the start of the item: its visibility, keyword, name, generics,
/// and a trait's supertraits.
fn head(decl: &Decl) -> Vec<Span> {
    let mut spans = vec![plain(&vis(&decl.meta))];
    if let Shape::Trait { prefix, .. } = &decl.shape {
        if !prefix.is_empty() {
            spans.push(key(prefix.trim_end()));
            spans.push(plain(" "));
        }
    }
    let keyword = match decl.kind {
        DeclKind::Struct => "struct",
        DeclKind::Enum => "enum",
        DeclKind::Union => "union",
        DeclKind::Trait | DeclKind::TraitAlias => "trait",
        DeclKind::Alias => "type",
    };
    spans.push(key(keyword));
    spans.push(plain(&format!(" {}", decl.name)));
    spans.extend(code(&decl.generics));
    if let Shape::Trait { bounds, .. } = &decl.shape {
        if !bounds.is_empty() {
            spans.push(plain(&format!(": {bounds}")));
        }
    }
    spans
}

/// Adds the item: an alias or a unit struct on its line, a tuple struct
/// with its fields, or a body in braces.
fn item(krate: &Crate, decl: &Decl, block: &mut Block) {
    let mut spans = head(decl);
    let trailing = decl.meta.trailing.clone();
    let clause = decl.where_clause.as_ref();
    match &decl.shape {
        Shape::Alias(target) => {
            spans.push(plain(" = "));
            spans.extend(code(target));
            ended(spans, clause, trailing, block);
        }
        Shape::Unit => ended(spans, clause, trailing, block),
        Shape::Tuple(fields) => {
            if fields.iter().any(expanded) {
                spans.push(plain("("));
                block.lines.push(Line {
                    indent: 0,
                    spans,
                    trailing,
                });
                for field in fields {
                    let (lines, repeats) = param_lines(field, 4);
                    block.lines.extend(lines);
                    block.repeats.extend(repeats);
                }
                ended(vec![plain(")")], clause, None, block);
            } else {
                spans.push(plain("("));
                spans.extend(joined(fields));
                spans.push(plain(")"));
                ended(spans, clause, trailing, block);
            }
        }
        Shape::Named | Shape::Enum(_) | Shape::Trait { .. } => {
            let entries = body(krate, decl);
            match clause {
                None if entries.is_empty() => spans.push(plain(" {}")),
                None => spans.push(plain(" {")),
                Some(_) => {}
            }
            block.lines.push(Line {
                indent: 0,
                spans,
                trailing,
            });
            if let Some(clause) = clause {
                block.lines.extend(where_lines(clause, 0, ","));
                block.lines.push(line(
                    0,
                    vec![plain(if entries.is_empty() { "{}" } else { "{" })],
                ));
            }
            if !entries.is_empty() {
                let mut lines = Vec::new();
                for (entry, repeats) in entries {
                    lines.push(entry);
                    block.repeats.extend(repeats);
                }
                block.lines.extend(spaced(lines));
                block.lines.push(line(0, vec![plain("}")]));
            }
        }
    }
}

/// Ends an item that has no braces: `;` on its line, or after its `where`
/// clause.
fn ended(
    mut spans: Vec<Span>,
    clause: Option<&Where>,
    trailing: Option<String>,
    block: &mut Block,
) {
    match clause {
        None => {
            spans.push(plain(";"));
            block.lines.push(Line {
                indent: 0,
                spans,
                trailing,
            });
        }
        Some(clause) => {
            block.lines.push(Line {
                indent: 0,
                spans,
                trailing,
            });
            block.lines.extend(where_lines(clause, 0, ";"));
        }
    }
}

/// Tells whether a field or a parameter has lines above it, which puts each
/// of its list's entries on a line of its own.
fn expanded(param: &Param) -> bool {
    !param.meta.notes.is_empty() || !param.meta.cfgs.is_empty() || !param.meta.attrs.is_empty()
}

/// Returns a list of fields on one line, joined by `, `.
fn joined(params: &[Param]) -> Vec<Span> {
    let mut spans = Vec::new();
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            spans.push(plain(", "));
        }
        spans.push(plain(&vis(&param.meta)));
        spans.extend(code(&param.text));
    }
    spans
}

/// Returns a field or a parameter on a line of its own, with the lines above
/// it.
fn param_lines(param: &Param, indent: usize) -> Lines {
    let mut lines = note_lines(&param.meta.notes, indent, "///");
    lines.extend(attr_lines(&[], &param.meta, indent));
    let mut spans = vec![plain(&vis(&param.meta))];
    spans.extend(code(&param.text));
    spans.push(plain(","));
    lines.push(line(indent, spans));
    (lines, Vec::new())
}

/// Returns the entries between an item's braces: the members no impl adds,
/// which are fields or trait items, or the variants, then the comments with
/// nothing after them and the macro marks of the impls.
fn body(krate: &Crate, decl: &Decl) -> Vec<Lines> {
    let held = |index: usize| decl.impl_locs.iter().any(|l| l.range.contains(&index));
    let mut entries: Vec<Lines> = decl
        .members
        .iter()
        .enumerate()
        .filter(|(index, _)| !held(*index))
        .map(|(_, member)| member_lines(member, 4, &krate.refs))
        .collect();
    if let Shape::Enum(variants) = &decl.shape {
        entries.extend(variants.iter().map(variant_lines));
    }
    let dangling = note_lines(&decl.dangling, 4, "///");
    if !dangling.is_empty() {
        entries.push((dangling, Vec::new()));
    }
    if !decl.end_marks.is_empty() {
        let mut lines = Vec::new();
        let mut repeats = Vec::new();
        for mark in &decl.end_marks {
            let (mark_line, repeat) = mark_line("MACRO", mark, 4, None);
            lines.push(mark_line);
            repeats.push(repeat);
        }
        entries.push((lines, repeats));
    }
    entries
}

/// Returns one member: a field, a function, a constant, an associated type,
/// a macro mark, or an unsupported construct.
pub(crate) fn member_lines(member: &Member, indent: usize, refs: &[TypeRef]) -> Lines {
    let (mut lines, mut repeats) = above(&member.meta, indent, member.loc);
    let vis = vis(&member.meta);
    let trailing = member.meta.trailing.clone();
    match &member.kind {
        MemberKind::Field(ty) => {
            let mut spans = vec![plain(&format!("{vis}{}: ", member.name))];
            spans.extend(code(ty));
            spans.push(plain(","));
            lines.push(Line {
                indent,
                spans,
                trailing,
            });
        }
        MemberKind::Fn(sig) => lines.extend(fn_lines(sig, &vis, indent, trailing)),
        MemberKind::Const { ty, value } => {
            let mut spans = vec![
                plain(&vis),
                key("const"),
                plain(&format!(" {}: ", member.name)),
            ];
            spans.extend(code(ty));
            lines.push(value_line(spans, value.as_deref(), indent, trailing, refs));
        }
        MemberKind::Type(text) => {
            let mut spans = vec![plain(&vis), key("type"), plain(" ")];
            spans.extend(code(text));
            spans.push(plain(";"));
            lines.push(Line {
                indent,
                spans,
                trailing,
            });
        }
        MemberKind::Mark(text) => {
            let (mark, repeat) = mark_line("MACRO", text, indent, Some(member.loc));
            lines.push(mark);
            repeats.push(repeat);
        }
        MemberKind::Unsupported(kind) => {
            let (mark, repeat) = mark_line("UNSUPPORTED:", kind, indent, Some(member.loc));
            lines.push(mark);
            repeats.push(repeat);
        }
    }
    (lines, repeats)
}

/// Returns one variant: its name, its fields, and its discriminant.
fn variant_lines(variant: &Variant) -> Lines {
    let (mut lines, repeats) = above(&variant.meta, 4, variant.loc);
    let mut spans = vec![plain(&variant.name)];
    let (fields, open, close) = match &variant.fields {
        VariantFields::Unit => (&[][..], "", ""),
        VariantFields::Tuple(fields) => (fields.as_slice(), "(", ")"),
        VariantFields::Named(fields) => (fields.as_slice(), " {", "}"),
    };
    if fields.iter().any(expanded) {
        spans.push(plain(open));
        lines.push(line(4, spans));
        for field in fields {
            lines.extend(param_lines(field, 8).0);
        }
        spans = vec![plain(close)];
    } else if !fields.is_empty() {
        let pad = if close == "}" { " " } else { "" };
        spans.push(plain(&format!("{open}{pad}")));
        spans.extend(joined(fields));
        spans.push(plain(&format!("{pad}{close}")));
    }
    if let Some(value) = &variant.discriminant {
        spans.push(plain(&format!(" = {value}")));
    }
    spans.push(plain(","));
    lines.push(Line {
        indent: 4,
        spans,
        trailing: variant.meta.trailing.clone(),
    });
    (lines, repeats)
}

/// Adds the impls after the item: each inherent impl that adds members, in
/// file and line order, then a line for each trait impl in the same order.
/// A trait impl with comments above it keeps a blank line on each side, so
/// its comments never seem to cover its neighbors.
fn impls(krate: &Crate, decl: &Decl, block: &mut Block) {
    let mut locs: Vec<&ImplLoc> = decl.impl_locs.iter().collect();
    locs.sort_by(|a, b| {
        krate.files[a.loc.file]
            .rel
            .cmp(&krate.files[b.loc.file].rel)
            .then(a.loc.line.cmp(&b.loc.line))
    });
    for loc in locs.iter().filter(|l| !l.trait_impl && !l.range.is_empty()) {
        block.lines.push(Line::default());
        let file = &krate.files[loc.loc.file].rel;
        block
            .lines
            .push(line(0, vec![Span::Comment(format!("// impl in {file}"))]));
        block.lines.extend(note_lines(&loc.notes, 0, "///"));
        block
            .lines
            .push(line(0, vec![plain(&format!("{} {{", loc.header))]));
        let mut members = Vec::new();
        for member in &decl.members[loc.range.clone()] {
            let (lines, repeats) = member_lines(member, 4, &krate.refs);
            members.push(lines);
            block.repeats.extend(repeats);
        }
        block.lines.extend(spaced(members));
        block.lines.push(line(0, vec![plain("}")]));
    }
    let mut last_bare = true;
    for (index, loc) in locs.iter().filter(|l| l.trait_impl).enumerate() {
        let bare = loc.notes.is_empty();
        if index == 0 || !bare || !last_bare {
            block.lines.push(Line::default());
        }
        last_bare = bare;
        block.lines.extend(note_lines(&loc.notes, 0, "///"));
        block
            .lines
            .push(line(0, vec![plain(&format!("{} {{}}", loc.header))]));
    }
}
