//! Spellings: each item from outside the crate is written one way, so the
//! outline has no Spellings section.

use std::fmt;

/// A point whose standard names each have one spelling.
#[derive(Clone, PartialEq)]
pub struct Point {
    pub label: Option<String>,
    pub tags: Vec<String>,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unused: std::option::Option<u8> = None;
        write!(f, "{unused:?}")
    }
}

/// A local type written two ways, which the section leaves out.
pub struct Line {
    pub start: crate::Point,
    pub end: Point,
}

impl Line {
    /// Returns the label; see `std::option::Option`.
    pub fn label<T: Clone>(&self, fallback: T) -> Option<T> {
        let unused: std::option::Option<T> = Some(fallback);
        unused
    }
}
