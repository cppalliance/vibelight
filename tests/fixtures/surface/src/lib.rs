//! The public surface of a library.

pub mod api {
    //! The public API.

    pub mod v1 {
        //! The first version.

        /// A request.
        pub struct Request;

        /// Sends a request.
        pub fn send(request: Request) -> u8 {
            let _ = request;
            0
        }
    }

    /// The version in use.
    pub const VERSION: u8 = 1;
}

mod detail {
    //! Hidden helpers.

    /// A handle made public through a re-export.
    pub struct Handle;

    /// A helper that stays hidden.
    pub fn helper() {}

    pub mod tools {
        //! Tools reached only through a re-export.

        /// A tool.
        pub struct Tool;
    }
}

mod values {
    //! Values made public through re-exports.

    /// Builds a value.
    pub fn build() -> u8 {
        0
    }

    /// The largest value.
    pub const MAX: u8 = 9;

    /// The current value.
    pub static CURRENT: u8 = 0;

    pub use self::inner::deep;

    mod inner {
        /// A function two re-exports away.
        pub fn deep() {}
    }
}

mod prelude {
    //! Names a glob brings in.

    /// A marker a glob re-exports.
    pub struct Marker;

    /// A function a glob re-exports.
    pub fn mark() {}
}

pub use detail::Handle;
pub use detail::tools as kit;
pub use prelude::*;
pub use std::collections::BTreeMap;
pub use values::deep;
pub use values::{CURRENT, MAX, build};

/// Says hello.
#[macro_export]
macro_rules! hello {
    () => {};
}

/// The number of calls so far.
pub static CALLS: u8 = 0;

/// Starts the library.
pub fn start() {}

/// A crate-only type.
pub(crate) struct Internal;
