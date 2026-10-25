//! Lays out declaration blocks: headers and bodies.

use crate::layout::{
    Block, Entry, Line, Span, above, code, join_groups, key, mark_line, note_lines, plain,
    sort_entries, where_spans,
};
use crate::layout_members::{member_entry, param_lines, variant_entry};
use crate::model::{
    Base, Crate, Decl, DeclId, DeclKind, Member, MemberKind, Meta, Note, Shape, Text, TypeRef,
    Variant, name_order,
};
use crate::wrap;

/// Builds the block of one declaration.
pub(crate) fn decl_block(krate: &Crate, id: DeclId) -> Block {
    let decl = &krate.decls[id];
    let refs = &krate.refs;
    let mut block = Block::default();
    block.lines.extend(note_lines(&decl.meta.notes, 0, "///"));
    block.lines.extend(note_lines(&decl.impl_notes, 0, "///"));
    let cfgs = Meta {
        cfgs: decl.meta.cfgs.clone(),
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
    let body = body(
        krate,
        &BodyParts {
            members: &decl.members,
            variants,
            class,
            all_public: matches!(decl.shape, Shape::Trait { .. }),
            dangling: &decl.dangling,
            end_marks: &decl.end_marks,
        },
    );
    block.lines.push(brace("{"));
    block.lines.extend(body.lines);
    block.repeats.extend(body.repeats);
    block.lines.push(brace("}"));
    block
}

fn brace(text: &str) -> Line {
    Line {
        indent: 0,
        spans: vec![plain(text)],
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
        && real.all(|m| !m.public)
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
        spans.extend(where_spans(clause));
    }
    spans.push(plain(";"));
    Line {
        indent: 0,
        spans,
        trailing: decl.meta.trailing.clone(),
    }
}

/// Returns the start of a header: the keyword, the name, the generics, and
/// a trait's supertraits.
fn head_spans(decl: &Decl, class: bool) -> Vec<Span> {
    let mut spans = Vec::new();
    if let Shape::Trait {
        prefix,
        supertraits,
    } = &decl.shape
    {
        if !prefix.is_empty() {
            spans.push(key(prefix.trim_end()));
            spans.push(plain(" "));
        }
        spans.push(key("trait"));
        spans.push(plain(&format!(" {}", decl.name)));
        spans.extend(code(&decl.generics));
        if let Some(supertraits) = supertraits {
            spans.push(plain(": "));
            spans.extend(code(supertraits));
        }
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

/// Finishes a header with any tuple fields, the base list, and the `where`
/// clause; a type with no body ends in `;`.
fn header_lines(decl: &Decl, mut header: Vec<Span>, has_body: bool, refs: &[TypeRef]) -> Vec<Line> {
    let mut bases = bases(&decl.bases, refs);
    let where_clause = decl.where_clause.as_ref().map(where_spans);
    if let Shape::Tuple(fields) = &decl.shape {
        header.push(plain("("));
        if fields.iter().any(|f| !f.meta.notes.is_empty()) {
            let mut lines = vec![Line {
                indent: 0,
                spans: header,
                trailing: None,
            }];
            let mut fields = param_lines(fields, 4);
            let mut last = fields.pop().unwrap_or_default();
            last.spans.push(plain(")"));
            last.spans.append(&mut bases);
            if !has_body {
                last.spans.push(plain(";"));
            }
            lines.extend(fields);
            lines.push(last);
            return lines;
        }
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                header.push(plain(", "));
            }
            header.extend(code(&field.text));
        }
        header.push(plain(")"));
    }
    header.append(&mut bases);
    if has_body {
        return wrap::header(header, where_clause, refs);
    }
    if let Some(clause) = where_clause {
        header.push(plain(" "));
        header.extend(clause);
    }
    header.push(plain(";"));
    vec![Line {
        indent: 0,
        spans: header,
        trailing: decl.meta.trailing.clone(),
    }]
}
/// What goes between a type's braces.
pub(crate) struct BodyParts<'b> {
    pub(crate) members: &'b [Member],
    /// An enum's variants, which take the fields' place in declared order.
    pub(crate) variants: Option<&'b [Variant]>,
    /// A class prints its private members without a label.
    pub(crate) class: bool,
    /// Every member of a trait is public.
    pub(crate) all_public: bool,
    pub(crate) dangling: &'b [Note],
    pub(crate) end_marks: &'b [String],
}

/// Builds the lines between a type's braces: the public groups, then
/// `private:` and the private groups, then comments with nothing after
/// them, macro marks, and unsupported lines.
pub(crate) fn body(krate: &Crate, parts: &BodyParts<'_>) -> Block {
    let refs = &krate.refs;
    let mut public: Vec<Vec<Entry>> = (0..5).map(|_| Vec::new()).collect();
    let mut private: Vec<Vec<Entry>> = (0..5).map(|_| Vec::new()).collect();
    let real = parts
        .members
        .iter()
        .filter(|m| !matches!(m.kind, MemberKind::Mark(_) | MemberKind::Unsupported(_)));
    for member in real {
        let group = match &member.kind {
            MemberKind::Type(_) => 0,
            MemberKind::Const { .. } => 1,
            MemberKind::Field(_) => 2,
            MemberKind::Fn(_) if member.kind.is_static_fn() => 3,
            _ => 4,
        };
        let entry = member_entry(krate, member);
        if member.public || parts.all_public {
            public[group].push(entry);
        } else {
            private[group].push(entry);
        }
    }
    for group in public.iter_mut().chain(private.iter_mut()) {
        sort_entries(group, refs);
    }
    if let Some(variants) = parts.variants {
        public[2] = variants.iter().map(|v| variant_entry(v, refs)).collect();
    }
    let mut block = Block::default();
    join_groups(public, &mut block);
    let has_private = private.iter().any(|g| !g.is_empty());
    if has_private {
        let mut section = Block::default();
        join_groups(private, &mut section);
        if !parts.class {
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            block.lines.push(Line {
                indent: 0,
                spans: vec![key("private:")],
                trailing: None,
            });
        }
        block.lines.extend(section.lines);
        block.repeats.extend(section.repeats);
    }
    let dangling = note_lines(parts.dangling, 4, "///");
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

/// Returns a base list, ` : A, B`, sorted by trait name.
fn bases(bases: &[Base], refs: &[TypeRef]) -> Vec<Span> {
    let mut sorted: Vec<&Base> = bases.iter().collect();
    sorted.sort_by(|a, b| {
        name_order(&a.name, &b.name).then_with(|| a.text.visible(refs).cmp(&b.text.visible(refs)))
    });
    let mut spans = Vec::new();
    for (i, base) in sorted.into_iter().enumerate() {
        spans.push(plain(if i == 0 { " : " } else { ", " }));
        spans.extend(code(&base.text));
    }
    spans
}
