//! Re-exports through a glob.

pub use self::inner::*;

pub mod inner {
    /// A handle.
    pub struct Handle;

    /// The other type named Shared.
    pub struct Shared;
}

/// A unique token type.
pub struct Token;
