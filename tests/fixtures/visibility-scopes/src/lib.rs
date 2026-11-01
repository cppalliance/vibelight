//! Visibility scopes: restricted spellings, the reach each one has, and an
//! item whose uses need less reach than it has.

mod engine;

/// Runs the engine.
pub fn launch() -> u8 {
    engine::run::start()
}
