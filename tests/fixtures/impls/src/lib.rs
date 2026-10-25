//! Impl blocks of every kind.

use std::fmt;

/// A type with split impls and trait impls.
pub struct Widget {
    id: u32,
}

impl Widget {
    /// Makes a widget.
    pub fn new(id: u32) -> Widget {
        Widget { id }
    }
}

impl fmt::Display for Widget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

unsafe impl Send for Widget {}

impl !Sync for Widget {}

impl Widget {
    /// Returns the identifier.
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl<'a> From<&'a str> for Widget {
    fn from(_: &'a str) -> Widget {
        Widget { id: 0 }
    }
}

/// A generic holder.
pub struct Holder<'a, T> {
    item: &'a T,
}

impl<'a, T> Holder<'a, T> {
    /// Returns the held item.
    pub fn item(&self) -> &'a T {
        self.item
    }
}

/// Describes itself.
pub trait Describe {
    /// Returns a description.
    fn describe(&self) -> String;
}

impl<T: fmt::Debug> Describe for T {
    fn describe(&self) -> String {
        format!("{self:?}")
    }
}

/// Renders itself.
pub trait Render {
    /// Renders to a string.
    fn render(&self) -> String;
}

impl Render for &Widget {
    fn render(&self) -> String {
        String::new()
    }
}

impl Render for Box<Widget> {
    fn render(&self) -> String {
        String::new()
    }
}

impl Render for (u8, u8) {
    fn render(&self) -> String {
        String::new()
    }
}

impl Render for [Widget] {
    fn render(&self) -> String {
        String::new()
    }
}
