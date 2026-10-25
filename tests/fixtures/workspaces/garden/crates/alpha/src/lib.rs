//! Alpha: a crate that names types of its dependencies.

use b::Gauge;

/// A meter that reads a gauge of beta.
pub struct Meter {
    /// Through a use of the re-export at beta's root.
    pub gauge: Gauge,
    /// Through beta's module path.
    pub valve: b::parts::Valve,
    /// A crate outside the run stays plain.
    pub stray: delta::Stray,
    /// A crate that no follow glob allows stays plain.
    pub vendored: gamma::Vendored,
}

/// A trait other crates implement.
pub trait Reads {
    fn read(&self) -> u32;
}
