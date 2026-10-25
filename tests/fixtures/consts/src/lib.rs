//! Constants and statics.

use std::sync::atomic::AtomicUsize;

/// The default limit.
pub const LIMIT: usize = 64;

/// A long value that does not fit.
pub const GREETING: &str = "a long greeting that will not fit in the outline's eighty column limit";

/// A value with a macro call.
pub const NAME: &str = concat!("vibe", "light");

/// A counter.
pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A mutable static.
pub static mut STATE: u8 = 0;

const _: () = assert!(LIMIT > 0);

const PRIVATE: u8 = 1;

/// A type with associated constants.
pub struct Config;

impl Config {
    /// The version.
    pub const VERSION: u32 = 3;
    /// A computed value.
    pub const DOUBLE: u32 = Self::VERSION * 2;
}
