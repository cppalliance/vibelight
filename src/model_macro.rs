//! The kill list's model: each macro definition, invocation, derive, and
//! attribute macro, and where its mark is shown.

use crate::model::{DeclId, Loc, ModId};

/// The kind of a macro, in kill list order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MacroKind {
    Definition,
    Invocation,
    Derive,
    Attribute,
}

/// Where a macro mark is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Holder {
    Decl(DeclId),
    Module(ModId),
    Impl(usize),
}

/// One kill list entry.
#[derive(Clone, Debug)]
pub(crate) struct MacroUse {
    pub(crate) kind: MacroKind,
    /// The mark text, such as `macro_rules! name`, `name!`, or
    /// `derive: Clone, Debug`.
    pub(crate) text: String,
    pub(crate) holder: Holder,
    pub(crate) loc: Loc,
}
