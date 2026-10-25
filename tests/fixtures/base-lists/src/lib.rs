//! Base lists: short names, long paths, wrapping, collisions, and supertraits.

use std::fmt;
use std::fmt::Debug as Dbg;
use std::fmt::Display as D;
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

/// Standard derives that are not special members, and comparisons with
/// another type written by hand. The crate's own Clone trait never sits on
/// the shared line, so the standard Clone keeps its short name.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ordered(u8);

impl PartialEq<u8> for Ordered {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u8> for Ordered {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

/// The standard Display written in full, with no use.
pub struct Spelled;

impl std::fmt::Display for Spelled {}

/// The standard Display through a renaming use.
pub struct Renamed;

impl D for Renamed {}

/// The standard Display after a use of the trait itself.
pub mod bare {
    use std::fmt::Display;

    /// Display through the use, which is the standard one.
    pub struct Imported;

    impl Display for Imported {}
}

/// Another crate's trait that is also named Display.
pub mod other {
    use other_crate::Display;

    /// The other crate's Display, which stays apart from the standard one.
    pub struct Foreign;

    impl Display for Foreign {}
}

/// A trait of this crate named Display.
pub mod custom {
    /// The crate's own Display.
    pub trait Display {}

    /// The crate's Display, which prints its path on its own line.
    pub struct Homemade;

    impl Display for Homemade {}
}

/// A Display that can only come from a glob import.
pub mod globbed {
    use something::*;

    /// A Display from the glob, which prints as written.
    pub struct Starred;

    impl Display for Starred {}
}
