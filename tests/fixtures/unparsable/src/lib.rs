//! Constructs the collector cannot read.

mod broken;

/// Still collected.
pub struct Fine;

impl Fine {
    fn no_body(&self);

    /// Collected normally.
    pub fn kept(&self) {}
}

pub macro twice($e:expr) {
    $e + $e
}
