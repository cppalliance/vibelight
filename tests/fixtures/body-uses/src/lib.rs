//! Types that only function bodies name.

/// Named only inside a free function's body.
pub struct Listed;

/// Named only inside a method body of another type.
pub struct Probe;

/// Named only inside its own method body, so it stays unconnected.
pub struct Lonely;

/// Named only in a trait's default method body.
pub struct Marker;

/// Runs things.
pub struct Runner;

impl Runner {
    /// Makes a probe.
    pub fn probe(&self) {
        let _ = Probe;
    }
}

impl Lonely {
    /// Makes another one.
    pub fn again() {
        let _ = Lonely;
    }
}

/// Lists things.
pub trait Lister {
    /// Marks.
    fn mark(&self) {
        let _ = Marker;
    }
}

/// Named only inside an impl on a wrapper of itself, so it stays
/// unconnected.
pub struct Wrapped;

impl Default for Box<Wrapped> {
    fn default() -> Self {
        Box::new(Wrapped)
    }
}

/// Lists one thing.
pub fn list() {
    let _listed = Listed;
}
