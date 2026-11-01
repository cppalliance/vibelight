//! Macro look-through: the items that the crate's own macros write, from a
//! macro in this file and one in another.

#[macro_use]
mod shapes;
mod events;

pub use events::Event;

markers!(Alpha, Beta);

markers!(=> nothing);

/// Reads the run of an event.
pub fn run_of(event: &Event) -> &str {
    event.execution()
}
