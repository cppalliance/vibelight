//! Macros of every kind.

/// Defines the events.
macro_rules! events {
    ($($name:ident),*) => {};
}

events!(Started, Finished);

/// A type with derives and an attribute macro.
#[derive(Debug, Clone)]
#[derive(PartialEq)]
#[serde_like::tag]
pub struct Event {
    #[serde(rename = "n")]
    pub name: String,
}

/// Named by the module-level invocations.
pub struct Started;

impl Event {
    forward_methods!();

    /// Has an attribute macro.
    #[tracing::instrument]
    pub fn emit(&self) {}
}

lifecycle_constants!(Started);

/// The entry point.
#[tokio::main]
pub async fn main() {}

/// A value with a macro.
pub const NAME: &str = stringify!(macros);
