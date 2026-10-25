//! Re-exports.

mod inner {
    //! Holds the items.

    /// A thing.
    pub struct Thing;

    /// Another thing.
    pub struct Other;

    /// A helper.
    pub fn helper() {}
}

pub use inner::{Other, Thing};
pub use inner::helper as assist;
pub use inner::*;
pub(crate) use inner::Thing as CrateThing;
use std::fmt;
