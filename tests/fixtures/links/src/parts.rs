//! Parts of the engine.

impl crate::Engine {
    /// Stops the engine.
    pub fn stop(&self) {}
}

/// A part.
pub struct Part;

/// A gauge that writes one standard type two ways.
pub struct Gauge {
    pub reading: Option<u32>,
    pub spare: std::option::Option<u32>,
}
