//! Relay: re-exports origin's values and links to one in a comment.

pub use origin::gauge::{LIMIT, TOTAL, measure};
pub use origin::gauge::{Reading, Scale};

pub mod taps {
    //! Where readings come from.

    pub use origin::gauge::Tap;
}

/// Takes a reading through [`origin::gauge::measure`].
pub fn read() -> u32 {
    measure()
}
