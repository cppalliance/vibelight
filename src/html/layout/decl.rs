//! Lays out declaration blocks: headers and bodies.

use crate::access::Access;
use crate::access_stamp::Stamp;
use crate::html::layout::groups::{
    Entry, Label, Labels, Sections, add, join_sections, label, reaches_line, sort_entries, spaced,
};
use crate::html::layout::head::header_lines;
use crate::html::layout::marker::Marker;
use crate::html::layout::members::{member_entry, variant_entry};
use crate::html::layout::special::special_entry;
use crate::html::layout::{
    Block, Line, Span, above, code, key, mark_line, note_lines, plain, vis_spans, where_parts,
};
use crate::model::{
    Crate, Decl, DeclId, DeclKind, Member, MemberKind, Meta, Note, Shape, Special, Text, Variant,
    name_order,
};

/// How a declaration's header opens.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Head {
    /// With the type's visibility as written.
    Written,
    /// Without `pub`, as a public surface shows a type: every type there is
    /// `pub`, so the word tells its reader nothing. Any other visibility
    /// stays as written.
    Surface,
}

/// Builds the block of one declaration, its functions ordered by name.
pub(crate) fn decl_block(krate: &Crate, id: DeclId) -> Block {
    block_of(krate, &krate.decls[id], Head::Written)
}

/// Builds the block of one declaration as a user of a public surface sees
/// it, under the name that user writes: its header without `pub`, its `pub`
/// members, none hidden from the docs, and no word of how far anything
/// reaches. A tuple struct whose fields are all private shows no fields,
/// which tell that user nothing. When only some are, a private one reads
/// `_`, so each public field keeps its number.
pub(crate) fn public_block(krate: &Crate, id: DeclId, name: &str) -> Block {
    block_of(krate, &public_decl(krate, id, name), Head::Surface)
}

/// Returns a declaration as a user of a public surface sees it, under the
/// name that user writes.
fn public_decl(krate: &Crate, id: DeclId, name: &str) -> Decl {
    let shown = |meta: &Meta| meta.vis == "pub" && !meta.attrs.iter().any(|a| a == "doc(hidden)");
    let mut decl = krate.decls[id].clone();
    name.clone_into(&mut decl.name);
    if !matches!(decl.shape, Shape::Trait { .. }) {
        decl.members.retain(|m| shown(&m.meta));
    }
    decl.meta.access = None;
    for member in &mut decl.members {
        member.meta.access = None;
    }
    if let Shape::Tuple(fields) = &mut decl.shape {
        if !fields.is_empty() && fields.iter().all(|f| f.meta.vis != "pub") {
            decl.shape = Shape::Unit;
        } else {
            for field in fields.iter_mut().filter(|f| f.meta.vis != "pub") {
                field.text = Text::plain("_");
                field.meta = Meta::default();
            }
        }
    }
    decl.end_marks.clear();
    decl.dangling.clear();
    decl
}

/// Builds a declaration's block, its header opened as `head` says. A comment
/// above one of its impls goes with what the impl adds: above the trait in
/// the base list, or, for an inherent impl, at the top of the body.
fn block_of(krate: &Crate, decl: &Decl, head: Head) -> Block {
    let refs = &krate.refs;
    let mut block = Block::default();
    block
        .lines
        .extend(note_lines(&decl.meta.notes, 0, Marker::DOC));
    let inherent = spaced(
        (decl.impl_notes.iter())
            .filter(|group| group.base.is_none())
            .map(|group| note_lines(&group.notes, 4, Marker::DOC)),
    );
    let cfgs = Meta {
        cfgs: decl.meta.cfgs.clone(),
        attrs: decl.meta.attrs.clone(),
        ..Meta::default()
    };
    block.lines.extend(above(&cfgs, 0, decl.loc, true).0);
    block.lines.extend(reaches_line(&decl.meta, 0, None));
    if let Shape::Alias(target) = &decl.shape {
        block.lines.push(alias_line(decl, target, head));
        return block;
    }
    let has_body = !matches!(decl.shape, Shape::Unit | Shape::Tuple(_))
        || !decl.members.is_empty()
        || !decl.special.is_empty()
        || !decl.end_marks.is_empty()
        || !decl.dangling.is_empty()
        || !inherent.is_empty();
    let variants = match &decl.shape {
        Shape::Enum(variants) => Some(variants.as_slice()),
        _ => None,
    };
    // The body's parts are sorted before the header is laid out, because
    // the header's keyword comes from the first member the body shows.
    let parts = BodyParts {
        members: &decl.members,
        variants,
        unlabeled: matches!(decl.shape, Shape::Trait { .. }),
        dangling: &decl.dangling,
        end_marks: &decl.end_marks,
        special: &decl.special,
        name: &decl.name,
        own: label(&decl.meta),
        shown: decl.meta.access.as_ref().and_then(Stamp::note),
    };
    let sorted = Sorted::of(krate, &parts);
    let keyword = keyword(decl, sorted.first_label());
    let header = head_spans(decl, head, keyword);
    block
        .lines
        .extend(header_lines(decl, header, has_body, refs));
    if !has_body {
        return block;
    }
    let in_force = match keyword {
        "struct" => Some("pub"),
        "class" => Some("pub(self)"),
        _ => None,
    };
    let body = sorted.join(Labels::Changes(in_force), &decl.dangling);
    block.lines.push(brace("{"));
    let parts = [inherent, body.lines];
    block
        .lines
        .extend(spaced(parts.into_iter().filter(|lines| !lines.is_empty())));
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

/// Returns the visibility a header opens with, and the space after it.
fn head_vis(decl: &Decl, head: Head) -> Vec<Span> {
    if head == Head::Surface && decl.meta.vis == "pub" {
        Vec::new()
    } else {
        vis_spans(&decl.meta.vis)
    }
}

/// Returns a type alias or trait alias as one line, after its visibility.
fn alias_line(decl: &Decl, target: &Text, head: Head) -> Line {
    let keyword = if decl.kind == DeclKind::TraitAlias {
        "trait"
    } else {
        "using"
    };
    let mut spans = head_vis(decl, head);
    spans.extend([key(keyword), plain(&format!(" {}", decl.name))]);
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

/// Returns the keyword of a declaration's header: Rust's own for a trait,
/// an enum, and a union, and for a struct the C++ keyword that fits the
/// first member shown, `struct` when that is `pub` and `class` when it is
/// not. The first member shown is a tuple struct's first field, or else
/// what the body's first section holds, labeled `first`; a struct that
/// shows no member is a `struct`. The keyword sets the visibility in force
/// at the opening brace, as in C++, so a body's first label prints only
/// when it says something else.
fn keyword(decl: &Decl, first: Option<&str>) -> &'static str {
    let public = match (&decl.shape, decl.kind) {
        (Shape::Trait { .. }, _) => return "trait",
        (_, DeclKind::Enum) => return "enum",
        (_, DeclKind::Union) => return "union",
        (Shape::Tuple(fields), _) if !fields.is_empty() => fields[0].meta.vis == "pub",
        _ => first.is_none_or(|label| label == "pub"),
    };
    if public { "struct" } else { "class" }
}

/// Returns the start of a header: the visibility `head` asks for, any
/// trait prefix, the keyword, the name, and the generics.
fn head_spans(decl: &Decl, head: Head, keyword: &str) -> Vec<Span> {
    let mut spans = head_vis(decl, head);
    if let Shape::Trait { prefix, .. } = &decl.shape {
        if !prefix.is_empty() {
            spans.extend([key(prefix.trim_end()), plain(" ")]);
        }
    }
    spans.push(key(keyword));
    spans.push(plain(&format!(" {}", decl.name)));
    spans.extend(code(&decl.generics));
    spans
}

/// What goes between a type's braces.
pub(crate) struct BodyParts<'b> {
    pub(crate) members: &'b [Member],
    /// An enum's variants, which print first, unlabeled, in declared order.
    pub(crate) variants: Option<&'b [Variant]>,
    /// Every member of a trait is as visible as the trait, so a trait's
    /// members print unlabeled.
    pub(crate) unlabeled: bool,
    pub(crate) dangling: &'b [Note],
    pub(crate) end_marks: &'b [String],
    /// The special members, printed as one packed group under the type's
    /// own label.
    pub(crate) special: &'b [Special],
    /// The type's name, which the special members spell.
    pub(crate) name: &'b str,
    /// The type's own label.
    pub(crate) own: Label,
    /// The access the type's `reaches` note shows, if it shows one.
    pub(crate) shown: Option<Access>,
}

/// Builds the lines between the braces of a block that has no keyword of
/// its own to put a visibility in force, such as an impl with no declared
/// type: what `Sorted` holds, each section opened by its label, then the
/// comments with nothing after them.
pub(crate) fn body(krate: &Crate, parts: &BodyParts<'_>) -> Block {
    Sorted::of(krate, parts).join(Labels::Changes(None), parts.dangling)
}

/// What goes between a type's braces, sorted and not yet joined: first what
/// has no label of its own, an enum's variants or a trait's members, then
/// macro marks and unsupported lines; then the data, a struct's fields, and
/// then every other member, each set in sections by visibility.
struct Sorted {
    first: Vec<Vec<Entry>>,
    data: Sections,
    members: Sections,
}

impl Sorted {
    fn of(krate: &Crate, parts: &BodyParts<'_>) -> Self {
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
                MemberKind::Fn(_) if member.kind.is_static_fn() => 4,
                _ => 5,
            };
            let entry = member_entry(krate, member, parts.shown);
            match (parts.unlabeled, group) {
                (true, _) => first[group].push(entry),
                (false, 2) => add(&mut data, label(&member.meta), 6, group, entry),
                (false, _) => add(&mut members, label(&member.meta), 6, group, entry),
            }
        }
        for group in first.iter_mut().chain(members.values_mut().flatten()) {
            sort_entries(group, refs);
        }
        if let Some(variants) = parts.variants {
            first[2] = variants.iter().map(|v| variant_entry(v, refs)).collect();
        }
        if let Some(entry) = special_entry(parts.special, parts.name, refs) {
            add(&mut members, parts.own.clone(), 6, 3, entry);
        }
        first.push(marks(parts));
        Self {
            first,
            data,
            members,
        }
    }

    /// Returns the label of the first section: the data's first, or else
    /// the other members' first. It is the visibility of the first member
    /// the body shows.
    fn first_label(&self) -> Option<&str> {
        let (_, spelling) = self.data.keys().chain(self.members.keys()).next()?;
        Some(spelling)
    }

    /// Joins the parts into the body's lines, the sections opened by their
    /// labels as `labels` says, then the comments with nothing after them,
    /// `dangling`.
    fn join(self, labels: Labels<'_>, dangling: &[Note]) -> Block {
        let mut block = Block::default();
        join_sections(self.first, [self.data, self.members], labels, &mut block);
        let dangling = note_lines(dangling, 4, Marker::DOC);
        if !dangling.is_empty() {
            if !block.lines.is_empty() {
                block.lines.push(Line::default());
            }
            block.lines.extend(dangling);
        }
        block
    }
}

/// Returns a body's macro marks, sorted, then its unsupported lines, as
/// one packed entry each.
fn marks(parts: &BodyParts<'_>) -> Vec<Entry> {
    let mut texts: Vec<&str> = parts.end_marks.iter().map(String::as_str).collect();
    texts.extend(parts.members.iter().filter_map(|m| match &m.kind {
        MemberKind::Mark(text) => Some(text.as_str()),
        _ => None,
    }));
    texts.sort_by(|a, b| name_order(a, b));
    texts.dedup();
    let marks = texts
        .into_iter()
        .map(|text| mark_line("MACRO", text, 4, None));
    let unsupported = parts
        .members
        .iter()
        .filter_map(|member| match &member.kind {
            MemberKind::Unsupported(kind) => {
                Some(mark_line("UNSUPPORTED:", kind, 4, Some(member.loc)))
            }
            _ => None,
        });
    let packed = |lines: Vec<(Line, crate::html::layout::Repeat)>| {
        let (lines, repeats): (Vec<Line>, Vec<_>) = lines.into_iter().unzip();
        (!lines.is_empty()).then(|| Entry {
            name: String::new(),
            lines,
            repeats,
        })
    };
    let marks = packed(marks.collect());
    let unsupported = packed(unsupported.collect());
    marks.into_iter().chain(unsupported).collect()
}
