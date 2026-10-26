//! Relay: re-exports origin's values and links to one in a comment.

pub use origin::gauge::{LIMIT, TOTAL, measure};

/// Takes a reading through [`origin::gauge::measure`].
pub fn read() -> u32 {
    measure()
}
