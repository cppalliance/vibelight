//! Front: re-exports what relay re-exports.

pub use relay::LIMIT as MAX;
pub use relay::TOTAL;
pub use relay::measure;

/// Takes a reading with [`measure`], which relay re-exports from origin.
pub fn sample() -> u32 {
    measure()
}

/// Returns the level of a reading, a type that front does not re-export.
pub fn level_of(reading: &relay::Reading) -> u32 {
    reading.level
}
