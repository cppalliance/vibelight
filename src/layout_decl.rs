//! Lays out declaration blocks as Rust writes them: a type's attributes,
//! its head and body, one impl holding the members of all its inherent
//! impls, and the traits it implements.

use crate::access::Access;
use crate::access_stamp::Stamp;
use crate::layout::{
    Block, Line, Span, above, code, key, mark_line, note_lines, plain, span_text, vis_spans,
    where_parts,
};
use crate::layout_generics::generic_params;
use crate::layout_groups::{
    Entry, Sections, add, join_groups, join_sections, label, reaches_line, sort_entries,
};
use crate::layout_head::header_lines;
use crate::layout_marker::Marker;
use crate::layout_members::{member_entry, variant_entry};
use crate::model::{
    Crate, Decl, DeclId, DeclKind, Member, MemberKind, Meta, Note, Shape, Special, SpecialKind,
    Text, TypeRef, Variant, name_order,
};

/// Builds the block of one declaration.
pub(crate) fn decl_block(krate: &Crate, id: DeclId) -> Block {
    block_of(krate, &krate.decls[id], false)
}

/// Builds the block of one declaration as a user of a public surface sees
/// it, under the name that user writes: its `pub` members, none hidden from
/// the docs, and no word of how far anything reaches. A struct that hides
/// fields says `/* private fields */`, and a hidden tuple field reads
/// `/* private field */`.
pub(crate) fn public_block(krate: &Crate, id: DeclId, name: &str) -> Block {
    let shown = |meta: &Meta| meta.vis == "pub" && !meta.attrs.iter().any(|a| a == "doc(hidden)");
    let mut decl = krate.decls[id].clone();
    name.clone_into(&mut decl.name);
    let is_trait = matches!(decl.shape, Shape::Trait { .. });
    let hidden_fields = !is_trait
        && (decl.members.iter()).any(|m| matches!(m.kind, MemberKind::Field(_)) && !shown(&m.meta));
    if !is_trait {
        decl.members.retain(|m| shown(&m.meta));
    }
    decl.meta.access = None;
    for member in &mut decl.members {
        member.meta.access = None;
    }
    if let Shape::Tuple(fields) = &mut decl.shape {
        for field in fields.iter_mut().filter(|f| f.meta.vis != "pub") {
            field.text = Text::plain("/* private field */");
            field.meta = Meta::default();
        }
    }
    decl.end_marks.clear();
    decl.dangling.clear();
    block_of(krate, &decl, hidden_fields)
}

/// Builds a declaration's block, with `/* private fields */` closing its body
/// when `hidden_fields` is set.
fn block_of(krate: &Crate, decl: &Decl, hidden_fields: bool) -> Block {
    let refs = &krate.refs;
    let mut block = Block::default();
    block
        .lines
        .extend(note_lines(&decl.meta.notes, 0, Marker::DOC));
    if !decl.written_derives.is_empty() {
        block.lines.push(Line {
            indent: 0,
            spans: vec![Span::Dim(format!(
                "#[derive({})]",
                decl.written_derives.join(", ")
            ))],
            trailing: None,
        });
    }
    let attrs = Meta {
        cfgs: decl.meta.cfgs.clone(),
        attrs: decl.meta.attrs.clone(),
        ..Meta::default()
    };
    block.lines.extend(above(&attrs, 0, decl.loc).0);
    block.lines.extend(reaches_line(&decl.meta, 0, None));
    if let Shape::Alias(target) = &decl.shape {
        block.lines.push(alias_line(decl, target));
        return block;
    }
    let variants = match &decl.shape {
        Shape::Enum(variants) => Some(variants.as_slice()),
        _ => None,
    };
    let mut bodies = body(
        krate,
        &BodyParts {
            members: &decl.members,
            variants,
            unlabeled: matches!(decl.shape, Shape::Trait { .. }),
            dangling: &decl.dangling,
            end_marks: &decl.end_marks,
            shown: decl.meta.access.as_ref().and_then(Stamp::note),
        },
    );
    let has_body = matches!(
        decl.shape,
        Shape::Named | Shape::Enum(_) | Shape::Trait { .. }
    );
    if hidden_fields {
        bodies.inner.lines.push(Line {
            indent: 4,
            spans: vec![Span::Comment("/* private fields */".to_owned())],
            trailing: None,
        });
    }
    block
        .lines
        .extend(header_lines(decl, head_spans(decl), has_body, refs));
    if has_body && bodies.inner.lines.is_empty() {
        // An empty body closes on its head line, as `pub struct Empty {}`.
        if let Some(head) = block.lines.last_mut() {
            head.spans.push(plain("}"));
        }
    } else if has_body {
        block.lines.extend(bodies.inner.lines);
        block.repeats.extend(bodies.inner.repeats);
        block.lines.push(brace("}"));
    } else if !bodies.inner.lines.is_empty() {
        // A unit or tuple struct has no body of its own, so what would go
        // in it, its end marks and its dangling comments, goes in its impl.
        let mut inner = bodies.inner;
        if !bodies.methods.lines.is_empty() {
            inner.lines.push(Line::default());
        }
        inner.lines.append(&mut bodies.methods.lines);
        inner.repeats.append(&mut bodies.methods.repeats);
        bodies.methods = inner;
    }
    if !bodies.methods.lines.is_empty() {
        block.lines.push(Line::default());
        for group in decl.impl_notes.iter().filter(|g| g.base.is_none()) {
            block.lines.extend(note_lines(&group.notes, 0, Marker::DOC));
        }
        block.lines.push(impl_line(decl, refs));
        block.lines.extend(bodies.methods.lines);
        block.repeats.extend(bodies.methods.repeats);
        block.lines.push(brace("}"));
    }
    block.lines.extend(implements_line(decl, refs));
    block.lines.extend(trait_impl_notes(decl));
    block
}

/// Returns the comments above the type's trait impls, each set under an
/// `// impl Trait` line, after the line that names those traits.
fn trait_impl_notes(decl: &Decl) -> Vec<Line> {
    let mut lines = Vec::new();
    for group in decl.impl_notes.iter().filter(|g| g.base.is_some()) {
        lines.push(impl_header(decl, group.base));
        lines.extend(note_lines(&group.notes, 0, Marker::DOC));
    }
    for special in decl.special.iter().filter(|s| !s.notes.is_empty()) {
        let mut spans = vec![Span::Marker(Marker::IMPL.spaced)];
        spans.extend(commented(special_spans(special)));
        lines.push(Line {
            indent: 0,
            spans,
            trailing: None,
        });
        lines.extend(note_lines(&special.notes, 0, Marker::DOC));
    }
    lines
}

fn brace(text: &str) -> Line {
    Line {
        indent: 0,
        spans: vec![plain(text)],
        trailing: None,
    }
}

/// Returns the `// impl Trait` line above one trait impl's comments, the
/// trait as a comment reference, so it links in HTML and names nothing.
fn impl_header(decl: &Decl, base: Option<usize>) -> Line {
    let mut spans = vec![Span::Marker(Marker::IMPL.spaced)];
    if let Some(base) = base.and_then(|index| decl.bases.get(index)) {
        spans.extend(commented(code(&base.text)));
    }
    Line {
        indent: 0,
        spans,
        trailing: None,
    }
}

/// Turns code spans into comment spans, a type name into a comment
/// reference, so it links and names nothing.
fn commented(spans: Vec<Span>) -> impl Iterator<Item = Span> {
    spans.into_iter().map(|span| match span {
        Span::Ref(id) => Span::DocRef(id),
        Span::Code(text) => Span::Comment(text),
        other => other,
    })
}

/// Returns the line that names the traits a type implements, other than
/// those it derives, which its `#[derive]` line names: a dim comment such
/// as `// implements Display, From<String>`, each trait linked, by name and
/// then by path.
fn implements_line(decl: &Decl, refs: &[TypeRef]) -> Option<Line> {
    let text = |spans: &[Span]| -> String { spans.iter().map(|s| span_text(s, refs)).collect() };
    let mut traits: Vec<(String, Vec<Span>)> = (decl.bases.iter())
        .filter(|base| !base.derived)
        .map(|base| (base.name.clone(), code(&base.text)))
        .collect();
    for special in decl.special.iter().filter(|special| !special.derived) {
        let name = match special.kind {
            SpecialKind::Default => "Default",
            SpecialKind::Drop => "Drop",
            SpecialKind::From => "From",
        };
        traits.push((name.to_owned(), special_spans(special)));
    }
    if traits.is_empty() {
        return None;
    }
    traits.sort_by(|a, b| name_order(&a.0, &b.0).then_with(|| text(&a.1).cmp(&text(&b.1))));
    let mut spans = vec![
        Span::Marker(Marker::PLAIN.spaced),
        Span::Comment("implements ".to_owned()),
    ];
    for (i, (_, entry)) in traits.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::Comment(", ".to_owned()));
        }
        spans.extend(commented(entry));
    }
    Some(Line {
        indent: 0,
        spans,
        trailing: None,
    })
}

/// Returns the trait of a special impl as code: `Default`, `Drop`, or
/// `From<T>`.
fn special_spans(special: &Special) -> Vec<Span> {
    match special.kind {
        SpecialKind::Default => vec![plain("Default")],
        SpecialKind::Drop => vec![plain("Drop")],
        SpecialKind::From => {
            let mut spans = vec![plain("From<")];
            spans.extend(special.arg.iter().flat_map(code));
            spans.push(plain(">"));
            spans
        }
    }
}

/// Returns the line that opens a type's impl: `impl`, the type's generic
/// parameters less their defaults, and the type with their names.
fn impl_line(decl: &Decl, refs: &[TypeRef]) -> Line {
    let params = generic_params(&decl.generics.rust_visible(refs));
    let mut spans = vec![key("impl")];
    let list = |parts: Vec<&str>| format!("<{}>", parts.join(", "));
    if !params.is_empty() {
        spans.push(plain(&list(params.iter().map(|p| p.0.as_str()).collect())));
    }
    spans.push(plain(&format!(" {}", decl.name)));
    if !params.is_empty() {
        spans.push(plain(&list(params.iter().map(|p| p.1.as_str()).collect())));
    }
    spans.push(plain(" {"));
    Line {
        indent: 0,
        spans,
        trailing: None,
    }
}

/// Returns a type alias or trait alias as one line, after its visibility.
fn alias_line(decl: &Decl, target: &Text) -> Line {
    let keyword = if decl.kind == DeclKind::TraitAlias {
        "trait"
    } else {
        "type"
    };
    let mut spans = vis_spans(&decl.meta.vis);
    spans.extend([key(keyword), plain(&format!(" {}", decl.name))]);
    spans.extend(code(&decl.generics));
    spans.push(plain(" = "));
    spans.extend(code(target));
    if let Some(clause) = &decl.where_clause {
        spans.push(plain(" "));
        spans.extend(where_parts(clause).one);
    }
    spans.push(plain(";"));
    Line {
        indent: 0,
        spans,
        trailing: decl.meta.trailing.clone(),
    }
}

/// Returns the start of a header: the visibility as written, the Rust
/// keyword with any trait prefix, the name, and the generics.
fn head_spans(decl: &Decl) -> Vec<Span> {
    let mut spans = vis_spans(&decl.meta.vis);
    let keyword = match (&decl.shape, decl.kind) {
        (Shape::Trait { prefix, .. }, _) => {
            if !prefix.is_empty() {
                spans.extend([key(prefix.trim_end()), plain(" ")]);
            }
            "trait"
        }
        (_, DeclKind::Enum) => "enum",
        (_, DeclKind::Union) => "union",
        _ => "struct",
    };
    spans.push(key(keyword));
    spans.push(plain(&format!(" {}", decl.name)));
    spans.extend(code(&decl.generics));
    spans
}

/// What goes between a type's braces and in its impl.
pub(crate) struct BodyParts<'b> {
    pub(crate) members: &'b [Member],
    /// An enum's variants, which print first, in declared order.
    pub(crate) variants: Option<&'b [Variant]>,
    /// Every member of a trait is as visible as the trait and goes in its
    /// body, so a trait has no impl.
    pub(crate) unlabeled: bool,
    pub(crate) dangling: &'b [Note],
    pub(crate) end_marks: &'b [String],
    /// The access the type's `reaches` note shows, if it shows one.
    pub(crate) shown: Option<Access>,
}

/// The two blocks of a type: what goes between its braces, and what goes
/// in its impl.
pub(crate) struct Bodies {
    /// An enum's variants, a struct's fields, or a trait's members, then the
    /// macro marks at the end of the body and its dangling comments.
    pub(crate) inner: Block,
    /// The constants, types, and functions of the type's inherent impls, by
    /// visibility, the widest first, each set in the order they are written,
    /// then the macro marks and unsupported lines in those impls.
    pub(crate) methods: Block,
}

/// Builds a type's two blocks. In each, a set of entries of one visibility
/// follows the wider sets after a blank line.
pub(crate) fn body(krate: &Crate, parts: &BodyParts<'_>) -> Bodies {
    let refs = &krate.refs;
    let mut first: Vec<Vec<Entry>> = (0..6).map(|_| Vec::new()).collect();
    let mut data = Sections::new();
    let mut members = Sections::new();
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
        let entry = member_entry(krate, member, parts.shown);
        match (parts.unlabeled, group) {
            (true, _) => first[group].push(entry),
            (false, 2) => add(&mut data, label(&member.meta), 5, group, entry),
            (false, _) => add(&mut members, label(&member.meta), 5, group, entry),
        }
    }
    for group in first.iter_mut().chain(members.values_mut().flatten()) {
        sort_entries(group, refs);
    }
    if let Some(variants) = parts.variants {
        first[2] = variants.iter().map(|v| variant_entry(v, refs)).collect();
    }
    let (impl_marks, end_marks) = marks(parts);
    let mut inner = Block::default();
    let mut methods = Block::default();
    if parts.unlabeled {
        first.push(impl_marks.into_iter().chain(end_marks).collect());
        join_sections(first, data, &mut inner);
    } else {
        join_sections(first, data, &mut inner);
        append(&mut inner, end_marks);
        join_sections(Vec::new(), members, &mut methods);
        append(&mut methods, impl_marks);
    }
    let dangling = note_lines(parts.dangling, 4, Marker::DOC);
    if !dangling.is_empty() {
        if !inner.lines.is_empty() {
            inner.lines.push(Line::default());
        }
        inner.lines.extend(dangling);
    }
    Bodies { inner, methods }
}

/// Adds entries to the end of a block, after a blank line.
fn append(block: &mut Block, entries: Vec<Entry>) {
    if entries.is_empty() {
        return;
    }
    if !block.lines.is_empty() {
        block.lines.push(Line::default());
    }
    join_groups(vec![entries], block);
}

/// Returns a body's macro marks and unsupported lines, each kind as one
/// packed entry: first those the impls hold, then those at the end of the
/// type's own body.
fn marks(parts: &BodyParts<'_>) -> (Vec<Entry>, Vec<Entry>) {
    let mut texts: Vec<&str> = parts
        .members
        .iter()
        .filter_map(|m| match &m.kind {
            MemberKind::Mark(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    texts.sort_by(|a, b| name_order(a, b));
    texts.dedup();
    let mut end: Vec<&str> = parts.end_marks.iter().map(String::as_str).collect();
    end.sort_by(|a, b| name_order(a, b));
    end.dedup();
    let unsupported = parts
        .members
        .iter()
        .filter_map(|member| match &member.kind {
            MemberKind::Unsupported(kind) => {
                Some(mark_line("UNSUPPORTED:", kind, 4, Some(member.loc)))
            }
            _ => None,
        });
    let packed = |lines: Vec<(Line, crate::layout::Repeat)>| {
        let (lines, repeats): (Vec<Line>, Vec<_>) = lines.into_iter().unzip();
        (!lines.is_empty()).then(|| Entry {
            name: String::new(),
            lines,
            repeats,
        })
    };
    let as_marks = |texts: Vec<&str>| {
        packed(
            texts
                .into_iter()
                .map(|text| mark_line("MACRO", text, 4, None))
                .collect(),
        )
    };
    let in_impls = as_marks(texts)
        .into_iter()
        .chain(packed(unsupported.collect()))
        .collect();
    (in_impls, as_marks(end).into_iter().collect())
}
