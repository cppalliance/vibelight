//! Raw identifiers.

/// Raw names in a struct.
pub struct Keywords {
    /// A raw field.
    pub r#type: u8,
    pub r#match: u8,
    kind: u8,
}

impl Keywords {
    /// A raw method.
    pub fn r#as(&self) -> u8 {
        self.r#type
    }

    pub fn r#async() {}
}

/// A module with a raw name.
pub mod r#impl {
    /// Lives in a raw module.
    pub struct Inner;
}

pub fn r#fn() {}
