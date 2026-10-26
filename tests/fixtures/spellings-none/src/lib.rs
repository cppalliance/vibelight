//! Spellings: each item from outside the crate is written one way, so the
//! outline has no Spellings section. A derive name is never counted, so a
//! derived Debug beside `fmt::Debug` in an impl is still one way.

use std::fmt;

/// A point whose standard names each have one spelling.
#[derive(Clone, Debug, PartialEq)]
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

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line")
    }
}

impl Line {
    /// Returns the label; see `std::option::Option`.
    pub fn label<T: Clone>(&self, fallback: T) -> Option<T> {
        let unused: std::option::Option<T> = Some(fallback);
        unused
    }
}
