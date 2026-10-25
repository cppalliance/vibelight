//! Comments on impls and modules.

/// The type.
pub struct Engine;

// Above the inherent impl.
impl Engine {
    /// Stays the method's doc comment.
    pub fn start(&self) {}
}

/// Above the Display impl.
impl std::fmt::Display for Engine {
    // A comment on a trait impl member is not shown.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

// Above the Clone impl.
impl Clone for Engine {
    fn clone(&self) -> Engine {
        Engine
    }
}

// Above an impl that goes to Other impls.
impl Default for Box<Engine> {
    fn default() -> Self {
        Box::new(Engine)
    }
}

// An ordinary comment on a mod line.
/// The outer doc of the module.
pub mod parts;
