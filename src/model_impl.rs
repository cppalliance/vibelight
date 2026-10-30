//! The model of impl blocks and the special members they become.

use crate::model::{
    Base, Crate, Decl, DeclId, Loc, Member, MemberKind, ModId, Note, RefId, Sig, Text,
};

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
/// HTML's `Source:` line.
#[derive(Clone, Debug)]
pub(crate) struct ImplLoc {
    /// The `impl` line.
    pub(crate) loc: Loc,
    /// The names of the members an inherent impl adds, sorted and without
    /// repeats; a trait impl adds none.
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
    /// The header as written, from `unsafe` or `impl` to the brace.
    pub(crate) header: String,
    /// The self type as written.
    pub(crate) self_text: Text,
    /// The self type's path when it is a plain path, generic arguments
    /// left out.
    pub(crate) self_ref: Option<RefId>,
    /// The trait of a trait impl.
    pub(crate) trait_: Option<Base>,
    /// An inherent impl's members; a trait impl shows none.
    pub(crate) members: Vec<Member>,
    /// A trait impl's functions, constants, and types, for analysis only:
    /// attach never moves them and no outline shows them, and every
    /// reference they write is a comment reference.
    pub(crate) trait_members: Vec<Member>,
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

impl Sig {
    /// Returns the texts of a signature that can name types: its generics,
    /// parameters, return type, and where predicates.
    pub(crate) fn texts(&self) -> Vec<&Text> {
        let mut texts = vec![&self.generics];
        texts.extend(self.params.iter().map(|p| &p.text));
        texts.extend(self.ret.iter());
        texts.extend(self.where_clause.iter().flat_map(|clause| clause.0.iter()));
        texts
    }
}

impl MemberKind {
    /// Returns the references a function, constant, or type member writes,
    /// or `None` for any other member.
    pub(crate) fn refs(&self) -> Option<Vec<RefId>> {
        let texts: Vec<&Text> = match self {
            MemberKind::Fn(sig) => sig.texts(),
            MemberKind::Const { ty, .. } | MemberKind::Type(ty) => vec![ty],
            MemberKind::Field(_) | MemberKind::Mark(_) | MemberKind::Unsupported(_) => return None,
        };
        Some(texts.into_iter().flat_map(Text::refs).collect())
    }
}

impl Decl {
    /// Tells whether a declaration is a wire type: a written derive ends in
    /// `Deserialize` or `Serialize`, or an attribute starts with `serde`.
    pub(crate) fn is_wire(&self) -> bool {
        let derived = (self.written_derives.iter())
            .any(|name| matches!(name.rsplit("::").next(), Some("Deserialize" | "Serialize")));
        derived || self.meta.attrs.iter().any(|attr| attr.starts_with("serde"))
    }
}

impl Crate {
    /// Returns the module whose path is a module's without its last
    /// segment, or `None` for the root.
    pub(crate) fn parent_module(&self, m: ModId) -> Option<ModId> {
        let (_, up) = self.modules[m].path.split_last()?;
        self.modules.iter().position(|module| module.path == up)
    }
}
