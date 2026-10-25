//! Member order inside a block.

/// Members in scrambled order.
#[allow(non_snake_case)]
pub struct Scrambled {
    pub zeta: u8,
    pub alpha: u8,
    beta: u8,
    pub Alpha: u8,
}

#[allow(non_snake_case)]
impl Scrambled {
    pub fn zed(&self) {}

    pub const MAX: u8 = 9;

    pub fn make() -> Scrambled {
        todo!()
    }

    fn hidden() {}

    pub fn apply(&self) {}

    pub fn Apply(&self) {}
}

/// Trait items in scrambled order.
pub trait Grouped {
    fn member(&self);
    fn build() -> Self;
    const LIMIT: u8;
    type Out;
}

/// Variants keep their declared order.
pub enum Order {
    Zulu,
    Alpha,
    Mike,
}

/// Tuple fields keep their declared order.
pub struct Triple(pub u8, pub String, pub bool);

/// The base list is sorted.
pub struct Based;

impl Default for Based {
    fn default() -> Based {
        Based
    }
}

impl std::fmt::Debug for Based {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Clone for Based {
    fn clone(&self) -> Based {
        Based
    }
}

impl AsRef<str> for Based {
    fn as_ref(&self) -> &str {
        ""
    }
}
