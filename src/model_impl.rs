//! The model of impl blocks and the special members they become.

use crate::model::{Base, DeclId, Loc, Member, Note, RefId, Text};

/// The standard trait behind a C++ special member function, a constructor
/// or the destructor, in print order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpecialKind {
    /// The default constructor, `Type();`
    Default,
    /// A converting constructor, `Type(T&);`
    From,
    /// The destructor, `~Type();`
    Drop,
}

/// One special member of a type.
#[derive(Clone, Debug)]
pub(crate) struct Special {
    pub(crate) kind: SpecialKind,
    /// The argument of a converting constructor, already ending in a
    /// reference.
    pub(crate) arg: Option<Text>,
    /// Whether a derive wrote it, which prints `= default`.
    pub(crate) derived: bool,
    /// Comments above the impl it came from.
    pub(crate) notes: Vec<Note>,
    pub(crate) loc: Loc,
}

/// An attached impl block that adds members or base-list entries, for the
/// `Source:` line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImplLoc {
    /// The `impl` line.
    pub(crate) loc: Loc,
    /// The names of the members an inherent impl adds, sorted; a trait
    /// impl adds none.
    pub(crate) members: Vec<String>,
}

/// The comments above one attached impl line, kept together so the outline
/// can print an `// impl` header above them.
#[derive(Clone, Debug)]
pub(crate) struct ImplNotes {
    /// The index of the impl's entry in the type's base list, or `None` for
    /// an inherent impl. The header reads the entry after routing, so it
    /// names the trait the way the base list prints it.
    pub(crate) base: Option<usize>,
    pub(crate) notes: Vec<Note>,
}

/// One impl block.
#[derive(Clone, Debug)]
pub(crate) struct Impl {
    /// The `impl` line.
    pub(crate) loc: Loc,
    /// The self type as written.
    pub(crate) self_text: Text,
    /// The self type's path when it is a plain path, generic arguments
    /// left out.
    pub(crate) self_ref: Option<RefId>,
    /// The trait of a trait impl.
    pub(crate) trait_: Option<Base>,
    /// An inherent impl's members; a trait impl shows none.
    pub(crate) members: Vec<Member>,
    /// Macro invocations in the impl's body.
    pub(crate) marks: Vec<String>,
    /// Attribute macros on the impl, sorted.
    pub(crate) attributes: Vec<String>,
    /// Comments above the impl line.
    pub(crate) notes: Vec<Note>,
    /// Comments with nothing after them in an inherent impl's body.
    pub(crate) dangling: Vec<Note>,
    /// The declaration this impl attached to, filled in by attach.
    pub(crate) attached: Option<DeclId>,
}
