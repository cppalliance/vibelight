//! Beta: re-exports a type from one of its modules.

pub mod parts {
    //! The parts of a meter.

    /// A gauge.
    pub struct Gauge;

    /// A valve.
    pub struct Valve;
}

pub use parts::Gauge;
