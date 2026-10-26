//! Links to functions, constants, and statics.

pub mod tools {
    //! Defines the values.

    /// The highest reading.
    pub const LIMIT: u32 = 8;

    /// A running total.
    pub static TOTAL: u32 = 0;

    /// Reads the gauge, never above [`LIMIT`] and counted in [`TOTAL`].
    pub fn measure() -> u32 {
        LIMIT
    }

    /// Draws a [`Shape`], which shares its name.
    pub fn Shape() {}

    /// A shape that [`measure`] sizes, also reached as [`crate::relay::measure`].
    pub struct Shape {
        pub sides: u8,
    }

    /// The Unix backend.
    #[cfg(unix)]
    pub fn backend() -> u8 {
        1
    }

    /// A Windows backend.
    #[cfg(windows)]
    pub fn backend() -> u8 {
        2
    }
}

mod relay {
    //! Passes values on.

    pub use crate::tools::TOTAL;
    pub use crate::tools::measure;
}

pub use relay::measure;
pub use tools::LIMIT;
pub use tools::TOTAL as GRAND_TOTAL;
pub use tools::{Shape, backend};
