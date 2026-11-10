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

    /// A reading and how it was taken.
    pub struct Reading {
        /// The level read.
        pub level: u32,
        /// The scale it was read on.
        pub scale: Scale,
        /// How many tries it took.
        tries: u32,
        /// The tap it came from.
        tap: Tap,
        /// The marks made on the way.
        marks: Vec<Mark>,
    }

    /// The scale of a reading.
    pub struct Scale;

    /// Where a reading comes from.
    pub struct Tap(u32);

    /// A mark made while reading.
    pub struct Mark;
}
