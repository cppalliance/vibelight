//! Origin: defines the values other crates re-export.

pub mod gauge {
    //! Gauge values.

    /// The highest reading.
    pub const LIMIT: u32 = 8;

    /// A running total.
    pub static TOTAL: u32 = 0;

    /// Takes one reading.
    pub fn measure() -> u32 {
        LIMIT
    }
}
