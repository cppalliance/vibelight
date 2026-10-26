//! Members grouped by impl.

mod more;

/// A type with methods in two files.
pub struct Engine {
    speed: u32,
}

impl Engine {
    /// Makes one.
    pub fn new() -> Engine {
        Engine { speed: 0 }
    }

    /// Starts it.
    pub fn start(&mut self) {
        self.speed = 1;
    }

    /// Checks it.
    fn check(&self) -> bool {
        self.speed > 0
    }
}
