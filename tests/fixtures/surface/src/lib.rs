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

pub use detail::Handle;
pub use detail::tools as kit;

/// The number of calls so far.
pub static CALLS: u8 = 0;

/// Starts the library.
pub fn start() {}

/// A crate-only type.
pub(crate) struct Internal;
