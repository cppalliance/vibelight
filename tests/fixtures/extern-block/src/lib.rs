//! Extern blocks.

unsafe extern "C" {
    /// Absolute value.
    pub fn abs(x: i32) -> i32;
    /// The last error.
    pub static errno: i32;
    fn secret() -> u8;
}

unsafe extern "C" {
    /// Exits the process.
    pub fn exit(code: i32) -> !;
}

unsafe extern "system" {
    pub fn GetTickCount() -> u32;
}
