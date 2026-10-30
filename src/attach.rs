//! Attaches impl blocks to the types they belong to.

use crate::model::{
    Crate, DeclId, DeclKind, Holder, ImplLoc, ImplNotes, Member, MemberKind, Note, Special,
    name_order,
};
use crate::route;

/// The comments above one attached impl line and how they sort: trait
/// impls (group 0) by trait name, then inherent impls (group 1) by their
/// first member's name.
#[derive(Clone)]
struct Sorted {
    group: u8,
    key: String,
    tiebreak: String,
    notes: ImplNotes,
}

/// Returns, for each impl of a crate, the declaration it belongs to: the
/// struct, enum, or union that its self type resolves to, which is the rule
/// `attach` moves impls by.
pub(crate) fn owners(krate: &Crate) -> Vec<Option<DeclId>> {
    krate
        .impls
        .iter()
        .map(|i| {
            i.self_ref.and_then(|r| krate.refs[r].decl()).filter(|&d| {
                matches!(
                    krate.decls[d].kind,
                    DeclKind::Struct | DeclKind::Enum | DeclKind::Union
                )
            })
        })
        .collect()
}

/// Moves every impl whose self type resolves to a struct, enum, or union
/// into that type: inherent members, base-list entries, comments, and
/// marks. Returns the ids of the impls left for the Other impls section.
pub(crate) fn attach(krate: &mut Crate) -> Vec<usize> {
    let mut others = Vec::new();
    let mut notes: Vec<Vec<Sorted>> = vec![Vec::new(); krate.decls.len()];
    for (id, target) in owners(krate).into_iter().enumerate() {
        let Some(decl_id) = target else {
            others.push(id);
            continue;
        };
        let item = &mut krate.impls[id];
        item.attached = Some(decl_id);
        let members = std::mem::take(&mut item.members);
        let marks = std::mem::take(&mut item.marks);
        let dangling = std::mem::take(&mut item.dangling);
        let attributes = item.attributes.clone();
        let mut impl_notes = std::mem::take(&mut item.notes);
        let trait_ = item.trait_.clone();
        let loc = item.loc;
        let special = trait_
            .as_ref()
            .and_then(|base| route::special_kind(&krate.refs, base));
        let first_member = members
            .iter()
            .map(|m| m.name.clone())
            .min_by(|a, b| name_order(a, b))
            .unwrap_or_default();
        let decl = &mut krate.decls[decl_id];
        let adds = !members.is_empty() || trait_.is_some();
        if adds {
            decl.impl_locs.push(ImplLoc {
                loc,
                members: member_names(&members),
            });
        }
        decl.members.extend(members);
        for mark in marks {
            if !decl.end_marks.contains(&mark) {
                decl.end_marks.push(mark);
            }
        }
        decl.dangling.extend(dangling);
        for name in attributes {
            if !decl.attributes.contains(&name) {
                decl.attributes.push(name);
            }
        }
        decl.attributes.sort_by(|a, b| name_order(a, b));
        if let (Some(kind), Some(base)) = (special, &trait_) {
            decl.special.push(Special {
                kind,
                arg: base.arg.clone(),
                derived: false,
                notes: std::mem::take(&mut impl_notes),
                loc,
            });
        }
        if !impl_notes.is_empty() {
            let (group, key) = match &trait_ {
                Some(base) => (0, base.name.clone()),
                None => (1, first_member),
            };
            let base = trait_.as_ref().map(|_| decl.bases.len());
            notes[decl_id].push(Sorted {
                group,
                key,
                tiebreak: notes_text(&impl_notes, &krate.refs),
                notes: ImplNotes {
                    base,
                    notes: impl_notes,
                },
            });
        }
        if let Some(base) = trait_.filter(|_| special.is_none()) {
            decl.bases.push(base);
        }
    }
    for (decl, list) in krate.decls.iter_mut().zip(notes) {
        decl.impl_notes = sorted_notes(list);
    }
    move_impl_marks(krate);
    route::finish(krate);
    others
}

/// Moves each kill list entry of a macro in an attached impl to the type
/// the impl attached to.
fn move_impl_marks(krate: &mut Crate) {
    for mark in &mut krate.macros {
        if let Holder::Impl(id) = mark.holder {
            if let Some(decl) = krate.impls[id].attached {
                mark.holder = Holder::Decl(decl);
            }
        }
    }
}

/// Puts a type's impl comment groups in print order: trait impls by trait
/// name, then inherent impls by first member name.
fn sorted_notes(mut list: Vec<Sorted>) -> Vec<ImplNotes> {
    list.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then_with(|| name_order(&a.key, &b.key))
            .then_with(|| a.tiebreak.cmp(&b.tiebreak))
    });
    list.into_iter().map(|entry| entry.notes).collect()
}

/// Returns the names of an impl's members, sorted and without repeats,
/// leaving out its macro marks and unsupported entries.
fn member_names(members: &[Member]) -> Vec<String> {
    let mut names: Vec<String> = members
        .iter()
        .filter(|m| !matches!(m.kind, MemberKind::Mark(_) | MemberKind::Unsupported(_)))
        .map(|m| m.name.clone())
        .collect();
    names.sort_by(|a, b| name_order(a, b));
    names.dedup();
    names
}

/// Returns the visible text of notes, for ordering them.
fn notes_text(notes: &[Note], refs: &[crate::model::TypeRef]) -> String {
    let mut out = String::new();
    for note in notes {
        match note {
            Note::Doc(lines) => lines
                .iter()
                .for_each(|line| out.push_str(&line.visible(refs))),
            Note::Plain(lines) => lines.iter().for_each(|line| out.push_str(line)),
        }
        out.push('\n');
    }
    out
}
