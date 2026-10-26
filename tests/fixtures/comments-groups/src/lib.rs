//! Comments on impl blocks and on use groups.

pub mod store {
    //! Keys and values.

    /// A key.
    pub struct Key;

    /// A value.
    pub struct Value;

    /// Reads a value.
    pub fn read() {}

    /// Writes a value.
    pub fn write() {}
}

pub mod traits {
    /// Something that persists.
    pub trait Persist {
        fn save(&self) {}
    }
}

// Store operations, for scripts.
pub use store::{
    read,
    // Writes replace the old value.
    write,
};

pub use store::{
    // The key type.
    Key,
    Value, // The value type.
};

/// A handle.
pub struct Handle;

// Opening.
impl Handle {
    /// Opens a handle.
    pub fn open() -> Handle {
        Handle
    }
}

// Reading.
impl Handle {
    /// Reads a byte.
    pub fn get(&self) -> u8 {
        0
    }
}

/// Shows the handle.
impl std::fmt::Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("handle")
    }
}

// Persists the handle.
impl traits::Persist for Handle {}
