//! A crate with macros, outlined with only the kill list's counts.

/// Defines a constant.
macro_rules! constant {
    ($name:ident) => {
        pub const $name: u8 = 1;
    };
}

constant!(ONE);

/// A type with a derive from another crate and an attribute macro.
#[derive(Debug, serde::Serialize)]
#[tracing::instrument]
pub struct Event;
