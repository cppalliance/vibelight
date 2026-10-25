//! Name resolution.

pub mod api;
pub mod model;

pub use model::Record as Entry;

/// Uses names resolved every way rule 39 allows.
pub struct Catalog {
    /// Through a prefix path.
    pub first: crate::model::Record,
    /// Through a pub use rename at the root.
    pub second: Entry,
    /// Through a glob import.
    pub third: api::Handle,
    /// A unique name across the target.
    pub fourth: Token,
    /// An ambiguous name stays plain.
    pub fifth: Shared,
}

/// A generic parameter that shares a type's name stays plain.
pub struct Wrapper<Record> {
    pub inner: Record,
}

/// Escapes in HTML: <, >, and & in types and comments.
pub type Pairs<'a> = &'a [(Token, Option<Vec<Token>>)];

/// The first of two types named Twin, under different cfgs.
#[cfg(unix)]
pub struct Twin;

/// The second Twin.
#[cfg(windows)]
pub struct Twin;
