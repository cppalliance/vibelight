//! Special members: the standard traits that become C++ constructors and
//! destructors.

/// The one derivable special member, and a derive from another crate.
#[derive(Default, serde::Serialize)]
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

/// An enum with a derived default.
#[derive(Default)]
pub enum Mode {
    #[default]
    On,
    Off,
}

/// A tuple struct with a derived and a hand-written member.
#[derive(Default)]
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
