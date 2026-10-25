//! Spells references and raw pointers with the sigil after the inner type,
//! so `&'a mut T` prints as `T mut& 'a` and `*const T` as `T*`.

use std::ops::Range;

use syn::punctuated::Punctuated;
use syn::{PathArguments, ReturnType, Type, TypeParamBound};

use crate::typerefs::Found;

/// The word that marks what can change. A reference, a pointer, or a
/// receiver without it is read-only.
pub(crate) const MUT: &str = "mut";

/// Returns what follows the inner type of a reference or a pointer: the
/// sigil, after ` mut` when the target can change.
pub(crate) fn marked(sigil: char, mutable: bool) -> String {
    if mutable {
        format!(" {MUT}{sigil}")
    } else {
        sigil.to_string()
    }
}

/// Tells whether the inner type of a reference or a pointer prints inside
/// parentheses: a trait object or an `impl` type with a return type or more
/// than one bound, or a function pointer with a return type. Without them,
/// the suffix would read as part of the last bound or of the return type.
pub(crate) fn needs_parens(inner: &Type) -> bool {
    match inner {
        Type::TraitObject(object) => ambiguous(&object.bounds),
        Type::ImplTrait(bounded) => ambiguous(&bounded.bounds),
        Type::FnPtr(pointer) => matches!(pointer.output, ReturnType::Type(..)),
        _ => false,
    }
}

fn ambiguous<P>(bounds: &Punctuated<TypeParamBound, P>) -> bool {
    bounds.len() > 1
        || bounds.iter().any(|bound| match bound {
            TypeParamBound::Trait(t) => t.path.segments.iter().any(|segment| {
                matches!(&segment.arguments, PathArguments::Parenthesized(args)
                    if matches!(args.output, ReturnType::Type(..)))
            }),
            _ => false,
        })
}

/// One step of building a slice, at a source range.
pub(crate) enum Piece<'s> {
    /// The kept type name at this index.
    Mark(usize),
    /// Source to drop, such as the `&'a mut ` before an inner type.
    Cut,
    /// Text to write where an inner type ends.
    Insert(&'s str),
}

/// Orders a slice's type-name marks and reference edits by position. When an
/// edit falls outside the slice, inside a type name, or across another edit,
/// every edit is dropped and the slice keeps its source spelling.
pub(crate) fn pieces<'s>(
    kept: &[Found],
    cuts: &[Range<usize>],
    inserts: &'s [(usize, String)],
    range: &Range<usize>,
) -> Vec<(Range<usize>, Piece<'s>)> {
    let marks = || {
        kept.iter()
            .enumerate()
            .map(|(index, item)| (item.range.clone(), Piece::Mark(index)))
    };
    let mut all: Vec<(Range<usize>, Piece<'s>)> = inserts
        .iter()
        .map(|(at, suffix)| (*at..*at, Piece::Insert(suffix.as_str())))
        .chain(cuts.iter().map(|cut| (cut.clone(), Piece::Cut)))
        .chain(marks())
        .collect();
    all.sort_by_key(|(at, _)| at.start);
    let inside = |at: &Range<usize>| range.start <= at.start && at.end <= range.end;
    let apart = all.windows(2).all(|pair| pair[0].0.end <= pair[1].0.start);
    if apart && all.iter().all(|(at, _)| inside(at)) {
        all
    } else {
        marks().collect()
    }
}

#[cfg(test)]
mod tests {
    use syn::visit::Visit;

    use crate::model::{Seg, Text, TypeRef};
    use crate::source::SourceFile;
    use crate::typerefs::Slicer;

    /// Slices the type of `type A = <ty>;`.
    fn slice(ty: &str) -> (Text, Vec<TypeRef>) {
        let file = SourceFile::new("a.rs".into(), &format!("type A = {ty};"));
        let item: syn::ItemType = syn::parse_str(&file.text).unwrap();
        let mut refs = Vec::new();
        let mut slicer = Slicer {
            file: &file,
            file_id: 0,
            refs: &mut refs,
            module: 0,
            generics: &[],
        };
        let text = slicer.node(&*item.ty, |f| f.visit_type(&item.ty));
        (text, refs)
    }

    fn spell(ty: &str) -> String {
        let (text, refs) = slice(ty);
        text.visible(&refs)
    }

    #[test]
    fn references_and_pointers_put_the_sigil_after_the_type() {
        assert_eq!(spell("&T"), "T&");
        assert_eq!(spell("&mut T"), "T mut&");
        assert_eq!(spell("*const T"), "T*");
        assert_eq!(spell("*mut T"), "T mut*");
        assert_eq!(spell("&Self"), "Self&");
    }

    #[test]
    fn a_lifetime_follows_the_reference() {
        assert_eq!(spell("&'a T"), "T& 'a");
        assert_eq!(spell("&'a mut T"), "T mut& 'a");
        assert_eq!(spell("&'static str"), "str& 'static");
    }

    #[test]
    fn slices_trait_objects_and_impl_traits_keep_their_spelling() {
        assert_eq!(spell("&str"), "str&");
        assert_eq!(spell("&[T]"), "[T]&");
        assert_eq!(spell("&dyn Trait"), "dyn Trait&");
        assert_eq!(spell("&mut impl Write"), "impl Write mut&");
        assert_eq!(spell("&(dyn Trait + Send)"), "(dyn Trait + Send)&");
    }

    #[test]
    fn nested_references_come_out_inside_out() {
        assert_eq!(spell("&Vec<&T>"), "Vec<T&>&");
        assert_eq!(
            spell("Box<dyn Fn(&str) -> usize>"),
            "Box<dyn Fn(str&) -> usize>"
        );
        assert_eq!(spell("Pin<&mut Self>"), "Pin<Self mut&>");
        assert_eq!(spell("&&T"), "T&&");
        assert_eq!(spell("*const *mut T"), "T mut**");
        assert_eq!(spell("&'a [&'a mut T]"), "[T mut& 'a]& 'a");
        assert_eq!(spell("fn(&str) -> &str"), "fn(str&) -> str&");
    }

    #[test]
    fn a_closure_or_a_bound_list_behind_a_reference_prints_in_parentheses() {
        assert_eq!(spell("&'a dyn Fn(&str) -> R"), "(dyn Fn(str&) -> R)& 'a");
        assert_eq!(
            spell("&mut impl FnMut(u8) -> bool"),
            "(impl FnMut(u8) -> bool) mut&"
        );
        assert_eq!(spell("*const dyn Fn() -> R"), "(dyn Fn() -> R)*");
        assert_eq!(spell("&fn(u8) -> u8"), "(fn(u8) -> u8)&");
        assert_eq!(spell("&dyn Fn() -> &R"), "(dyn Fn() -> R&)&");
        assert_eq!(spell("&mut (dyn A + B)"), "(dyn A + B) mut&");
        assert_eq!(
            spell("&(dyn Fn(&str) -> R + Send)"),
            "(dyn Fn(str&) -> R + Send)&"
        );
    }

    #[test]
    fn a_closure_that_returns_a_reference_stays_bare() {
        assert_eq!(
            spell("impl Fn(&T) -> &'a Section"),
            "impl Fn(T&) -> Section& 'a"
        );
        assert_eq!(spell("&dyn Fn(u8)"), "dyn Fn(u8)&");
        assert_eq!(spell("&fn(u8)"), "fn(u8)&");
    }

    #[test]
    fn comments_inside_a_reference_go_with_its_prefix() {
        assert_eq!(spell("& /* note */ mut T"), "T mut&");
        assert_eq!(spell("Vec<&'a /* note */ str>"), "Vec<str& 'a>");
        assert_eq!(spell("Vec<&T /* note */>"), "Vec<T&>");
    }

    #[test]
    fn type_names_inside_stay_links() {
        let (text, refs) = slice("&mut Vec<&'a Block>");
        assert_eq!(text.visible(&refs), "Vec<Block& 'a> mut&");
        let names: Vec<String> = refs.iter().map(|r| r.path.join("::")).collect();
        assert_eq!(names, ["Vec", "Block"]);
        assert!(matches!(
            text.0.as_slice(),
            [Seg::Ref(0), Seg::Plain(open), Seg::Ref(1), Seg::Plain(close)]
                if open == "<" && close == "& 'a> mut&"
        ));
    }

    #[test]
    fn a_reference_inside_a_path_segment_stays_as_written() {
        assert_eq!(spell("a::B<&T>::C"), "a::B<&T>::C");
    }
}
