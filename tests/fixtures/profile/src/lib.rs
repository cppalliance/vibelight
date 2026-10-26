//! A crate outlined with the analysis profile.

/// Defines a constant.
macro_rules! constant {
    ($name:ident) => {
        pub const $name: u8 = 1;
    };
}

constant!(ONE);

pub mod shapes {
    //! Shapes.

    /// A point.
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    /// A size.
    pub struct Size {
        pub x: i32,
        pub y: i32,
    }
}

/// A canvas.
pub struct Canvas {
    /// Its corner.
    pub corner: shapes::Point,
}
