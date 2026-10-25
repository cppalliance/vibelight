//! Module layouts.

pub mod files;
mod nested;
pub(crate) mod only_modules;

/// An inline module.
pub mod inline {
    //! Inner docs of the inline module.

    /// A type inside the inline module.
    pub struct Inside;

    pub mod deeper {
        /// The deepest type.
        pub struct Deepest;
    }
}

#[path = "custom-name.rs"]
mod renamed;
