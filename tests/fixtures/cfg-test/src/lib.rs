//! Test-only items are skipped.

/// Kept.
pub struct Kept {
    /// A kept field.
    pub value: u32,
    /// A test-only field.
    #[cfg(test)]
    pub probe: u32,
}

/// A kept enum.
pub enum Mode {
    On,
    /// A test-only variant.
    #[cfg(test)]
    Fake,
    #[cfg(any(test, feature = "debug"))]
    Debug,
}

impl Kept {
    /// A kept method.
    pub fn get(&self) -> u32 {
        self.value
    }

    /// A test-only method.
    #[cfg(test)]
    pub fn poke(&mut self) {}
}

#[cfg(test)]
impl Kept {
    /// From a test-only impl.
    pub fn hidden(&self) {}
}

/// A test-only trait item stays out.
pub trait Probe {
    /// Kept.
    fn check(&self);
    /// Skipped.
    #[cfg(test)]
    fn fake(&self);
}

#[cfg(test)]
use std::collections::BTreeMap;

// A comment on a test-only module.
#[cfg(test)]
mod tests {
    #[test]
    fn works() {}
}

#[cfg(all(test, feature = "slow"))]
pub fn slow_helper() {}

#[cfg(not(test))]
pub fn production_only() {}
