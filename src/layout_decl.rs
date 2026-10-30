//! Lays out declaration blocks: headers and bodies.

use std::ops::Range;

use crate::layout::{
    Block, Entry, Line, Sections, Span, above, code, join_sections, key, mark_line, note_lines,
    plain, section, sections, sort_entries, where_parts,
};
use crate::layout_head::header_lines;
use crate::layout_marker::Marker;
use crate::layout_members::{member_entry, variant_entry};
use crate::layout_special::special_entry;
use crate::model::{
    Crate, Decl, DeclId, DeclKind, Member, MemberKind, Meta, Note, Shape, Special, Text, TypeRef,
    Variant, Vis, name_order,
};
use crate::settings::MembersBy;

/// Builds the block of one declaration, its functions ordered by `members`.
pub(crate) fn decl_block(krate: &Crate, id: DeclId, members: MembersBy) -> Block {
    let decl = &krate.decls[id];
    let refs = &krate.refs;
    let mut block = Block::default();
    block
        .lines
        .extend(note_lines(&decl.meta.notes, 0, Marker::DOC));
    for group in &decl.impl_notes {
        block.lines.push(impl_header(decl, group.base));
        block.lines.extend(note_lines(&group.notes, 0, Marker::DOC));
    }
    let cfgs = Meta {
        cfgs: decl.meta.cfgs.clone(),
        attrs: decl.meta.attrs.clone(),
        ..Meta::default()
    };
    block.lines.extend(above(&cfgs, 0, decl.loc).0);
    if let Shape::Alias(target) = &decl.shape {
        block.lines.push(alias_line(decl, target));
        return block;
    }
    let class = is_class(decl);
    let has_body = !matches!(decl.shape, Shape::Unit | Shape::Tuple(_))
        || !decl.members.is_empty()
        || !decl.special.is_empty()
        || !decl.end_marks.is_empty()
        || !decl.dangling.is_empty();
    block
        .lines
        .extend(header_lines(decl, head_spans(decl, class), has_body, refs));
    if !has_body {
        return block;
    }
    let variants = match &decl.shape {
        Shape::Enum(variants) => Some(variants.as_slice()),
        _ => None,
    };
    let impls = match members {
        MembersBy::Name => Vec::new(),
    };
    let body = body(
        krate,
        &BodyParts {
            members: &decl.members,
            variants,
            class,
            all_public: matches!(decl.shape, Shape::Trait { .. }),
            dangling: &decl.dangling,
            end_marks: &decl.end_marks,
            special: &decl.special,
            name: &decl.name,
            impls: &impls,
        },
    );
    block.lines.push(brace("{"));
    block.lines.extend(body.lines);
    block.repeats.extend(body.repeats);
    block.lines.push(brace("}"));
    block
}

/// Returns the line above a module's free item that is not public, which
/// gives its own visibility: `// internal` or `// private`. A type does not
/// take one: its members' `internal:` and `private:` sections already say
/// what a reader needs, and a line above the header was only noise.
pub(crate) fn visibility_line(vis: Vis, indent: usize) -> Option<Line> {
    let text = match vis {
        Vis::Public => return None,
        Vis::Internal => "internal",
        Vis::Private => "private",
    };
    Some(Line {
        indent,
        spans: vec![
            Span::Marker(Marker::VISIBILITY.spaced),
            Span::Comment(text.to_owned()),
        ],
        trailing: None,
    })
}

/// Sorts every group of a body's members, then adds each impl's functions,
/// in source order, after the other functions of their section. The fields
/// and the variants are not here: they keep their declared order in the
/// data sections.
fn order_groups(
    sections: &mut Sections,
    impls: &[(String, Range<usize>)],
    by_impl: Vec<[Vec<Entry>; 3]>,
    refs: &[TypeRef],
) {
    for groups in sections.iter_mut() {
        for group in groups.iter_mut() {
            sort_entries(group, refs);
        }
    }
    for ((file, _), held) in impls.iter().zip(by_impl) {
        for (groups, mut entries) in sections.iter_mut().zip(held) {
            if let Some(first) = entries.first_mut() {
                first.lines.insert(0, impl_line(file));
            }
            groups[5].extend(entries);
        }
    }
}

/// Returns the line that starts the functions of one inherent impl when
/// functions are ordered by impl.
fn impl_line(file: &str) -> Line {
    Line {
        indent: 4,
        spans: vec![
            Span::Marker(Marker::IMPL_IN.spaced),
            Span::Comment(file.to_owned()),
        ],
        trailing: None,
    }
}

fn brace(text: &str) -> Line {
    Line {
        indent: 0,
        spans: vec![plain(text)],
        trailing: None,
    }
}

/// Returns the `// impl` line above one impl's comments: the trait as the
/// base list prints it, or the type's own name for an inherent impl. The
/// trait is a comment reference, so it links in HTML and names nothing.
fn impl_header(decl: &Decl, base: Option<usize>) -> Line {
    let mut spans = vec![Span::Marker(Marker::IMPL.spaced)];
    match base.and_then(|index| decl.bases.get(index)) {
        Some(base) => spans.extend(code(&base.text).into_iter().map(|span| match span {
            Span::Ref(id) => Span::DocRef(id),
            Span::Code(text) => Span::Comment(text),
            other => other,
        })),
        None => spans.push(Span::Comment(decl.name.clone())),
    }
    Line {
        indent: 0,
        spans,
        trailing: None,
    }
}

/// Tells whether a struct prints as a class: named fields, at least one
/// member, and no public member.
fn is_class(decl: &Decl) -> bool {
    let mut real = decl
        .members
        .iter()
        .filter(|m| !matches!(m.kind, MemberKind::Mark(_) | MemberKind::Unsupported(_)))
        .peekable();
    decl.kind == DeclKind::Struct
        && matches!(decl.shape, Shape::Named)
        && real.peek().is_some()
        && real.all(|m| m.vis != Vis::Public)
}

/// Returns a type alias or trait alias as one line.
fn alias_line(decl: &Decl, target: &Text) -> Line {
    let keyword = if decl.kind == DeclKind::TraitAlias {
        "trait"
    } else {
        "using"
    };
    let mut spans = vec![key(keyword), plain(&format!(" {}", decl.name))];
    spans.extend(code(&decl.generics));
    spans.push(plain(" = "));
    spans.extend(code(target));
    if let Some(clause) = &decl.where_clause {
        spans.push(plain(" "));
        spans.extend(where_parts(clause).one);
    }
    Line {
        indent: 0,
        spans,
        trailing: decl.meta.trailing.clone(),
    }
}

/// Returns the start of a header: the keyword, the name, and the generics.
fn head_spans(decl: &Decl, class: bool) -> Vec<Span> {
    let mut spans = Vec::new();
    if let Shape::Trait { prefix, .. } = &decl.shape {
        if !prefix.is_empty() {
            spans.push(key(prefix.trim_end()));
            spans.push(plain(" "));
        }
        spans.push(key("trait"));
        spans.push(plain(&format!(" {}", decl.name)));
        spans.extend(code(&decl.generics));
        return spans;
    }
    let keyword = match decl.kind {
        DeclKind::Enum => "enum",
        DeclKind::Union => "union",
        _ if class => "class",
        _ => "struct",
    };
    spans.push(key(keyword));
    spans.push(plain(&format!(" {}", decl.name)));
    spans.extend(code(&decl.generics));
    spans
}

/// What goes between a type's braces.
pub(crate) struct BodyParts<'b> {
    pub(crate) members: &'b [Member],
    /// An enum's variants, which print as its data, in declared order.
    pub(crate) variants: Option<&'b [Variant]>,
    /// A class starts private and any other type starts public, and a label
    /// prints only where the access changes from there.
    pub(crate) class: bool,
    /// Every member of a trait is public.
    pub(crate) all_public: bool,
    pub(crate) dangling: &'b [Note],
    pub(crate) end_marks: &'b [String],
    /// The special members, printed as one packed group after the data and
    /// the constants.
    pub(crate) special: &'b [Special],
    /// The type's name, which the special members spell.
    pub(crate) name: &'b str,
    /// When functions are ordered by impl: each inherent impl's file and the
    /// indexes of the members it adds, in print order.
    pub(crate) impls: &'b [(String, Range<usize>)],
}

/// Builds the lines between a type's braces: the data first, a struct's
/// fields or an enum's variants, then every other member, each set in the
/// order public, `internal:`, `private:` with a label wherever the access
/// changes, then comments with nothing after them, macro marks, and
/// unsupported lines.
pub(crate) fn body(krate: &Crate, parts: &BodyParts<'_>) -> Block {
    let refs = &krate.refs;
    let mut data = sections(6);
    let mut sections = sections(6);
    let mut by_impl: Vec<[Vec<Entry>; 3]> =
        parts.impls.iter().map(|_| Default::default()).collect();
    let real = parts
        .members
        .iter()
        .enumerate()
        .filter(|(_, m)| !matches!(m.kind, MemberKind::Mark(_) | MemberKind::Unsupported(_)));
    for (index, member) in real {
        let group = match &member.kind {
            MemberKind::Type(_) => 0,
            MemberKind::Const { .. } => 1,
            MemberKind::Field(_) => 2,
            MemberKind::Fn(_) if member.kind.is_static_fn() => 4,
            _ => 5,
        };
        let vis = if parts.all_public {
            Vis::Public
        } else {
            member.vis
        };
        let held = parts
            .impls
            .iter()
            .position(|(_, range)| range.contains(&index));
        let entry = member_entry(krate, member);
        match held.filter(|_| matches!(member.kind, MemberKind::Fn(_))) {
            Some(k) => by_impl[k][section(vis)].push(entry),
            None if group == 2 => data[section(vis)][group].push(entry),
            None => sections[section(vis)][group].push(entry),
        }
    }
    order_groups(&mut sections, parts.impls, by_impl, refs);
    if let Some(variants) = parts.variants {
        data[0][2] = variants.iter().map(|v| variant_entry(v, refs)).collect();
    }
    let default = if parts.class {
        Vis::Private
    } else {
        Vis::Public
    };
    if let Some(entry) = special_entry(parts.special, parts.name, refs) {
        sections[section(default)][3].push(entry);
    }
    let mut block = Block::default();
    join_sections(data, sections, default, &mut block);
    let dangling = note_lines(parts.dangling, 4, Marker::DOC);
    if !dangling.is_empty() {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        block.lines.extend(dangling);
    }
    let mut marks: Vec<&str> = parts.end_marks.iter().map(String::as_str).collect();
    marks.extend(parts.members.iter().filter_map(|m| match &m.kind {
        MemberKind::Mark(text) => Some(text.as_str()),
        _ => None,
    }));
    marks.sort_by(|a, b| name_order(a, b));
    marks.dedup();
    let unsupported: Vec<&Member> = parts
        .members
        .iter()
        .filter(|m| matches!(m.kind, MemberKind::Unsupported(_)))
        .collect();
    if !marks.is_empty() {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        for mark in marks {
            let (line, repeat) = mark_line("MACRO", mark, 4, None);
            block.lines.push(line);
            block.repeats.push(repeat);
        }
    }
    if !unsupported.is_empty() {
        if !block.lines.is_empty() {
            block.lines.push(Line::default());
        }
        for member in unsupported {
            if let MemberKind::Unsupported(kind) = &member.kind {
                let (line, repeat) = mark_line("UNSUPPORTED:", kind, 4, Some(member.loc));
                block.lines.push(line);
                block.repeats.push(repeat);
            }
        }
    }
    block
}
