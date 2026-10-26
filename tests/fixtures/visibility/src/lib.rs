//! Visibility sections: public, internal, and private.

pub mod outer {
    //! Module items in every section.

    /// Visible everywhere.
    pub fn open() {}

    /// Visible in the crate.
    pub(crate) fn shared() {}

    /// Visible in the parent module.
    pub(super) fn parent() {}

    /// Visible only here.
    fn hidden() {}

    /// Also visible only here.
    pub(self) fn own() {}

    /// Visible in the crate, through a path.
    pub(in crate) const LIMIT: u8 = 4;

    /// A shared counter.
    pub(crate) static COUNT: u8 = 0;

    unsafe extern "C" {
        /// Opens a stream.
        pub fn ext_open() -> i32;

        /// Shares a stream.
        pub(crate) fn ext_share() -> i32;
    }
}

/// A struct with every section.
pub struct Mixed {
    /// A public field.
    pub name: String,
    /// An internal field.
    pub(crate) size: usize,
    /// A private field.
    secret: u8,
}

impl Mixed {
    /// Makes one.
    pub fn new() -> Mixed {
        Mixed {
            name: String::new(),
            size: 0,
            secret: 0,
        }
    }

    /// Takes the name out.
    pub fn into_name(self) -> String {
        self.name
    }

    /// Grows it.
    pub(crate) fn grow(&mut self) {
        self.size += 1;
    }

    /// Checks it.
    fn check(&self) -> bool {
        self.secret == 0
    }
}

/// A class with internal and private members.
pub struct Inner {
    /// Seen in the crate.
    pub(crate) id: u32,
    /// Seen only here.
    state: u8,
}

/// A class with private members only.
pub struct Hidden {
    value: u8,
}

/// An enum seen only in the crate.
#[non_exhaustive]
pub(crate) enum Mode {
    Fast,
    Slow,
}

/// A type alias seen only here.
type Count = u32;
