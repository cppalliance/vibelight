//! Prelude order: a bare name is the prelude's unless the module declares
//! it, imports it, or a glob brings it in.

/// The first module with a private Result alias.
pub mod alpha {
    /// Alpha's error.
    pub struct AlphaError;

    /// The results of alpha.
    type Result<T> = std::result::Result<T, AlphaError>;
}

/// The second module with a private Result alias.
pub mod beta {
    /// Beta's error.
    pub struct BetaError;

    /// The results of beta.
    type Result<T> = std::result::Result<T, BetaError>;
}

/// A module that declares its own Result and uses it.
pub mod own {
    /// The error of this module.
    pub struct OwnError;

    /// The results of this module.
    pub type Result<T> = std::result::Result<T, OwnError>;

    /// A value in this module's Result.
    pub struct Owned {
        pub value: Result<u8>,
    }
}

/// A module that names the prelude Result and imports nothing.
pub mod plain {
    /// A report whose outcome is the standard Result.
    pub struct Report {
        pub outcome: Result<u8, String>,
    }
}

/// A module whose glob import brings in the Result of own.
pub mod starred {
    use crate::own::*;

    /// A value in the Result that the glob brings in.
    pub struct Starred {
        pub value: Result<u16>,
    }
}
