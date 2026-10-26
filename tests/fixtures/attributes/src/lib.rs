//! Attributes that shape an API or its wire format.

/// A level.
#[non_exhaustive]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Level {
    /// The top.
    #[deprecated(since = "1.2", note = "use High")]
    Top = 2,
    /// The bottom.
    Low = 0,
}

/// A record with a wire format.
#[cfg_attr(feature = "wire", derive(serde::Serialize), serde(rename_all = "camelCase"))]
pub struct Record {
    /// The record's name.
    #[serde(rename = "id")]
    pub name: String,
    /// Kept out of the docs.
    #[doc(hidden)]
    pub cache: u8,
    #[allow(dead_code)]
    count: u8,
}

impl Record {
    /// Reads the name.
    #[must_use]
    #[track_caller]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Makes an empty record.
    #[must_use = "a record does nothing until it is stored"]
    #[inline]
    pub fn empty() -> Record {
        Record {
            name: String::new(),
            cache: 0,
            count: 0,
        }
    }
}

/// A hidden helper.
#[doc(hidden)]
#[deprecated]
pub fn helper() {}

#[doc(hidden)]
pub use Record as Row;
