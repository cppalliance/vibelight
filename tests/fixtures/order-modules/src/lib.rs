//! Declarations shown one module at a time.

pub mod engine {
    //! Runs things.

    /// A runner.
    pub struct Runner {
        /// Its plan.
        pub plan: crate::plan::Plan,
    }

    /// Starts the engine.
    pub fn start() {}
}

pub mod plan {
    //! Plans runs.

    /// A plan.
    pub struct Plan {
        /// Its steps.
        pub steps: Vec<Step>,
    }

    /// One step.
    pub struct Step;

    /// Starts the engine for a plan, a call that closes a cycle no `use`
    /// item writes.
    pub fn launch() {
        crate::engine::start();
    }

    /// A step for tests and benches.
    #[cfg(any(test, feature = "test-support"))]
    pub struct FakeStep;
}

/// A value nothing uses.
pub struct Orphan;

impl Clone for Box<Orphan> {
    fn clone(&self) -> Self {
        Box::new(Orphan)
    }
}
