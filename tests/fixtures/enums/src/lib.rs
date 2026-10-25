//! Enum shapes.

/// Every variant shape.
pub enum Shape {
    /// A unit variant.
    Unit,
    /// A tuple variant.
    Tuple(u8, String),
    /// A struct-like variant.
    Named { x: i32, y: i32 },
    /// A variant with a discriminant.
    Valued = 4,
}

/// Struct-like variants that expand.
pub enum Event {
    /// Expands because a field has a comment.
    Started {
        /// The run identifier.
        id: u64,
        name: String,
    },
    /// Expands because its one-line form is too wide.
    Finished {
        identifier: u64,
        summary: String,
        duration_in_milliseconds: u128,
    },
}

/// An enum with methods.
pub enum Level {
    Low,
    High,
}

impl Level {
    /// Returns the highest level.
    pub fn max() -> Level {
        Level::High
    }

    fn rank(&self) -> u8 {
        0
    }
}
