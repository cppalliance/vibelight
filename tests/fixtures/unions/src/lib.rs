//! Unions.

/// Raw bits viewed two ways.
#[repr(C)]
pub union Bits {
    /// As an integer.
    pub int: u32,
    /// As a float.
    pub float: f32,
    raw: [u8; 4],
}

/// A union with only private fields.
pub union Hidden {
    b: i8,
    a: u8,
}
