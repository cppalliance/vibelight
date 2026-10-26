//! Front: re-exports what relay re-exports.

pub use relay::LIMIT as MAX;
pub use relay::TOTAL;
pub use relay::measure;

/// Takes a reading with [`measure`], which relay re-exports from origin.
pub fn sample() -> u32 {
    measure()
}
