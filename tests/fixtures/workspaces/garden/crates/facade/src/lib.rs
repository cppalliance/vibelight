//! Facade: re-exports the workspace's public types.

pub use alpha::Meter;
pub use alpha::Reads;
pub use beta::Gauge;

/// A panel that reads through alpha's trait.
pub struct Panel {
    /// A meter from alpha.
    pub meter: alpha::Meter,
}

impl alpha::Reads for Panel {
    fn read(&self) -> u32 {
        0
    }
}

/// Shows any reader.
pub fn show<T: alpha::Reads>(reader: &T) -> u32 {
    reader.read()
}
