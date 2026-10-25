//! References, raw pointers, and receivers.

use std::fmt::Display;
use std::io::Write;
use std::pin::Pin;

/// Borrowed and raw views of one value.
pub struct Views<'a, T> {
    pub shared: &'a T,
    pub unique: &'a mut T,
    pub name: &'static str,
    pub items: &'a [&'a T],
    pub read: *const T,
    pub write: *mut T,
    pub shown: &'a dyn Display,
    pub check: Box<dyn Fn(&str) -> usize>,
    pub callback: &'a dyn Fn(&str) -> usize,
    pub both: &'a (dyn Display + Send),
    pub wrapped: &'a (dyn Fn(&str) -> usize),
    pub raw_call: *const dyn Fn() -> bool,
    pub step: &'a fn(u8) -> u8,
}

impl<'a, T> Views<'a, T> {
    pub fn get(&self) -> &T {
        self.shared
    }

    pub fn set(&mut self, value: T) {
        *self.unique = value;
    }

    pub fn into_name(self) -> &'static str {
        self.name
    }

    pub fn rename(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn same(self: &Self, other: &Self) -> bool {
        self.name == other.name
    }

    pub fn clear(self: &mut Self) {}

    pub fn poll(self: Pin<&mut Self>) {}

    pub fn render(&self, out: &mut impl Write, items: &[Box<dyn Display>]) {}

    pub fn lookup(
        &mut self,
        keys: &Vec<&'a str>,
        table: &std::collections::BTreeMap<String, &'a mut T>,
    ) -> Option<&'a mut T> {
        None
    }
}

/// A source of bytes.
pub trait Source {
    fn read(&mut self, buf: &mut [u8]) -> usize;
    fn name(&self) -> &str;
}

/// A borrowed table.
pub type Table<'a> = &'a [(&'a str, *const u8)];

/// Marks a type.
pub trait Marker {}

impl<T> Marker for &T {}

impl Marker for *const u8 {}

/// The default name.
pub const NAME: &str = "views";

/// Raw bytes behind a reference.
pub static BYTES: &[u8] = &[1, 2];

/// Finds the first item.
pub fn first<'a>(items: &'a [&'a str]) -> Option<&'a str> {
    items.first().copied()
}

/// Picks an item with a closure that returns a reference.
pub fn pick<'a, T>(choose: impl Fn(&T) -> &'a T, items: &'a [T]) -> Option<&'a T> {
    items.first().map(choose)
}

/// Applies a function to a string.
pub fn apply<F>(text: &mut String, f: F) -> usize
where
    F: Fn(&mut String) -> usize,
{
    f(text)
}
