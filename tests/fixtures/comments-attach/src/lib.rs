//! Where comments attach.

// Attached to the struct below.
/// Docs for Config.
pub struct Config {
    // Directly above the field.
    pub first: u8,

    // Separated from the field by a blank line.

    pub second: u8, // Trailing comment on the same line.
    pub third: u8,
    // Nothing follows this comment in the struct.
}

impl Config {
    /// Takes commented parameters.
    pub fn build(
        // The first input.
        first: u8,
        second: u8,
    ) -> Config {
        // A comment inside the body is never shown.
        Config {
            first,
            second,
            third: 0,
        }
    }

    /// Holds a comment inside its type.
    pub fn typed(&self, items: Vec</* the count */ u32>) {}
}

/// A field whose type holds a comment.
pub struct Holder {
    pub items: Vec</* inner */ u8>,
}

// A comment on a plain use is not shown.
use std::fmt;

macro_rules! noop {
    // A comment inside a macro_rules! body is not shown.
    () => {};
}

/// Holds a string with comment-like text.
pub const PATTERN: &str = "// not a comment";
