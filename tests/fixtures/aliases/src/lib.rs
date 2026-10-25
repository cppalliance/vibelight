//! Type aliases.

/// The crate's result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The crate's error.
#[derive(Debug)]
pub struct Error {
    message: String,
}

/// A callback with a bound.
pub type Callback<'a, T> = Box<dyn Fn(T) -> Result<()> + 'a>;
