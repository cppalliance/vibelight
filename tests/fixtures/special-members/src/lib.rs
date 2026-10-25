//! Special members: the standard traits with a C++ equivalent.

/// Every derivable special member, and a derive from another crate.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, serde::Serialize)]
pub struct Derived {
    pub value: u8,
}

/// Every hand-written special member.
pub struct Written {
    pub value: u8,
}

impl Default for Written {
    fn default() -> Self {
        Written { value: 0 }
    }
}

impl Clone for Written {
    fn clone(&self) -> Self {
        Written { value: self.value }
    }
}

/// Converts from a byte.
impl From<u8> for Written {
    fn from(value: u8) -> Self {
        Written { value }
    }
}

impl From<&str> for Written {
    fn from(text: &str) -> Self {
        Written { value: text.len() as u8 }
    }
}

impl Drop for Written {
    fn drop(&mut self) {}
}

impl PartialEq for Written {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd<u8> for Written {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

impl Written {
    pub fn new() -> Self {
        Written { value: 0 }
    }

    pub fn get(&self) -> u8 {
        self.value
    }
}

/// A name from anything that converts into a String.
pub struct Name(String);

impl<T: Into<String>> From<T> for Name {
    fn from(value: T) -> Self {
        Name(value.into())
    }
}

/// An enum with derived members.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    On,
    Off,
}

/// A tuple struct with derived and hand-written members.
#[derive(Clone, Default)]
pub struct Handle(u32);

impl Drop for Handle {
    fn drop(&mut self) {}
}

/// A class with a hand-written destructor.
pub struct Scheduler {
    jobs: Vec<u32>,
}

impl Drop for Scheduler {
    fn drop(&mut self) {}
}

/// A struct with a private section.
#[derive(Default)]
pub struct Counter {
    count: u32,
}

impl Counter {
    pub fn count(&self) -> u32 {
        self.count
    }
}
