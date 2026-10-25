//! Base lists: short names, long paths, wrapping, collisions, and supertraits.

use std::fmt;
use std::fmt::Debug as Dbg;
use std::io;

/// The crate's own error, whose name collides with std::error::Error.
#[derive(Debug)]
pub enum Error {
    Failed,
}

impl std::error::Error for Error {}

impl Named for Error {
    fn name(&self) -> String {
        String::new()
    }
}

/// A trait of this crate.
pub trait Named {
    fn name(&self) -> String;
}

/// A standard supertrait and a supertrait of this crate.
pub trait Labeled: fmt::Display + Named {}

/// Standard supertraits and a lifetime bound.
pub trait Plain: Send + Sync + 'static {}

/// Standard traits only, so the list stays on the head line.
pub struct Run {
    pub id: u32,
}

impl fmt::Debug for Run {}

impl fmt::Display for Run {}

/// The crate's Error makes std::error::Error keep its full path.
pub struct RunError {
    pub reason: String,
}

impl fmt::Debug for RunError {}

impl fmt::Display for RunError {}

impl std::error::Error for RunError {}

/// A trait of this crate and one of another crate take the comma form.
pub struct Mixed;

impl fmt::Debug for Mixed {}

impl Named for Mixed {
    fn name(&self) -> String {
        String::new()
    }
}

impl serde::Serialize for Mixed {}

/// Two traits named Write keep their full paths.
pub struct Sink;

impl fmt::Write for Sink {}

impl io::Write for Sink {}

/// So many standard traits that the list wraps.
#[derive(Copy, Debug, Eq, Hash, Ord)]
pub struct Wide(u64);

impl fmt::Binary for Wide {}

impl fmt::Display for Wide {}

impl fmt::LowerExp for Wide {}

impl fmt::LowerHex for Wide {}

impl fmt::Octal for Wide {}

impl fmt::Pointer for Wide {}

impl fmt::UpperExp for Wide {}

impl fmt::UpperHex for Wide {}

/// Generics, a where clause, and a trait of this crate.
pub struct Holder<T>
where
    T: Copy,
{
    pub value: T,
}

impl<T: Copy> fmt::Debug for Holder<T> {}

impl<T> Named for Holder<T>
where
    T: Copy,
{
    fn name(&self) -> String {
        String::new()
    }
}

/// A union with standard derives.
#[derive(Copy, Debug)]
pub union Bits {
    pub int: u32,
    pub float: f32,
}

/// A class with a standard trait.
pub struct Secret {
    value: u8,
}

impl fmt::Display for Secret {}

/// A unit struct whose list fits on the head line.
#[derive(Debug, Eq, Hash)]
pub struct Marker;

/// A trait of this crate that shares a prelude name.
pub trait Clone {}

/// The crate's own Clone, a trait nothing declares, and a renamed standard
/// trait.
pub struct Local;

impl Clone for Local {}

impl Unknown for Local {}

impl Dbg for Local {}

/// A bare derive of From comes from another crate, so it stays on the mark.
#[derive(From, Debug)]
pub struct Wrapped(u8);
