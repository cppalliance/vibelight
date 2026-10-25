//! Source links.

mod parts;

/// A type with an impl in another file.
pub struct Engine {
    pub speed: u32,
}

impl Engine {
    /// Starts the engine.
    pub fn start(&self) {}
}

macro_rules! noop {
    () => {};
}

impl Clone for Box<Engine> {
    fn clone(&self) -> Self {
        Box::new(Engine { speed: 0 })
    }
}

/// A dial with an optional limit.
pub struct Dial {
    pub limit: std::option::Option<u32>,
}
