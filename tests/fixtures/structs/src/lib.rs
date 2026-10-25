//! Struct shapes: named, tuple, unit, and generic.

/// A named struct with public and private fields.
pub struct Point {
    /// Horizontal position.
    pub x: i32,
    /// Vertical position.
    pub y: i32,
    label: String,
}

/// A tuple struct.
pub struct Meters(pub f64);

/// A tuple struct with two fields, one private.
pub struct Pair(pub u8, u16);

/// A unit struct.
pub struct Marker;

/// Generics with a lifetime, a bound, a default, and a const generic.
pub struct Buffer<'a, T: Clone = u8, const N: usize = 16> {
    pub data: &'a [T; N],
    len: usize,
}

/// A tuple struct whose fields carry comments.
pub struct Span(
    /// The first byte.
    pub usize,
    /// One past the last byte.
    pub usize,
);

/// A tuple struct with methods.
pub struct Index(usize);

impl Index {
    /// Returns the index.
    pub fn get(&self) -> usize {
        self.0
    }
}

/// A unit struct with a method.
pub struct Token;

impl Token {
    /// Makes a token.
    pub fn new() -> Token {
        Token
    }
}

/// An empty braced struct.
pub struct Empty {}
