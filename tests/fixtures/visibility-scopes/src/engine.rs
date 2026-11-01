//! The engine: two layers under one group, and a type whose impl is
//! written in a sibling module.

pub mod parse {
    //! The first layer.

    /// A token, widened to the group so the entry point can match it.
    pub(in crate::engine) struct Token;

    /// Reads one token.
    pub(super) fn lex() -> Token {
        Token
    }

    pub mod inner {
        //! Deeper.

        /// Spelled for the parent, which `pub(super)` says.
        pub(in crate::engine::parse) fn helper() -> u8 {
            1
        }
    }

    /// Used only by the other layer of the group.
    pub(crate) fn local() -> u8 {
        inner::helper()
    }
}

pub mod run {
    //! The second layer.

    /// Starts a run.
    pub fn start() -> u8 {
        let _token = super::parse::lex();
        super::parse::local()
    }
}

/// A type with members in an impl of another module.
pub struct Gauge;

mod reading {
    impl super::Gauge {
        /// Reachable from the engine.
        pub(super) fn level(&self) -> u8 {
            self.private_level()
        }

        fn private_level(&self) -> u8 {
            0
        }
    }
}
