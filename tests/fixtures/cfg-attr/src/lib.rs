//! cfg and cfg_attr.

/// Derives through cfg_attr.
#[cfg_attr(feature = "serde", derive(Clone))]
#[derive(Debug)]
pub struct Settings {
    /// Only with a feature.
    #[cfg(feature = "extra")]
    pub extra: u32,
    #[cfg(unix)]
    #[cfg(feature = "fast")]
    pub both: u8,
}

#[cfg(feature = "methods")]
impl Settings {
    /// Gated by the impl's cfg.
    pub fn gated(&self) {}

    /// Also gated, with its own cfg.
    #[cfg(windows)]
    pub fn doubly(&self) {}
}

/// A test-only cfg_attr is ignored.
#[cfg_attr(test, derive(PartialEq))]
pub struct Plain;
