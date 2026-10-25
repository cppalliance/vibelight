//! Cycles at level 1 and below.

/// Names the cycle below it.
pub struct Root {
    pub ping: net::Ping,
}

/// Half of a cycle that nothing else names.
pub struct Left {
    pub right: Option<Box<Right>>,
}

/// The other half.
pub struct Right {
    pub left: Option<Box<Left>>,
}

pub mod net {
    //! A cycle below level 1.

    /// Names Pong.
    pub struct Ping {
        pub pong: Option<Box<Pong>>,
    }

    /// Names Ping back.
    pub struct Pong {
        pub ping: Option<Box<Ping>>,
    }
}
