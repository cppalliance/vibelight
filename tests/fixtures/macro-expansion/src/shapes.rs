//! A macro that the root module invokes, with an internal rule it invokes
//! itself.

/// Writes one marker type per name, one name at a time.
macro_rules! markers {
    (@one $name:ident) => {
        /// A marker that the macro writes.
        pub struct $name;
    };
    ($($name:ident),* $(,)?) => {
        $( markers!(@one $name); )*
    };
}
