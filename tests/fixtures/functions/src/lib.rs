//! Functions and signatures.

/// Computes at compile time.
pub const fn square(x: u32) -> u32 {
    x * x
}

/// Waits for input.
pub async fn wait(seconds: u64) {}

/// Reads raw memory.
///
/// # Safety
/// The pointer must be valid.
pub unsafe fn peek(ptr: *const u8) -> u8 {
    unsafe { *ptr }
}

/// Exported to C.
pub extern "C" fn callback(code: i32) -> i32 {
    code
}

/// Every qualifier at once.
pub const unsafe extern "C" fn all_qualifiers() {}

fn private_helper() {}

/// Takes impl and dyn traits.
pub fn render(out: &mut impl std::io::Write, items: &[Box<dyn std::fmt::Display>]) {}

/// Uses a higher-ranked bound.
pub fn apply<F>(f: F) -> usize
where
    F: for<'a> Fn(&'a str) -> usize,
{
    f("x")
}

/// Has a where clause on several lines.
pub fn merge<A, B, C>(a: A, b: B) -> C
where
    A: Into<C>,
    B: Into<C>,
    C: std::ops::Add<Output = C>,
{
    a.into() + b.into()
}

/// A rustfmt-style signature over several lines.
pub fn parse(
    input: &str,
    execution: &str,
) -> (std::result::Result<String, String>, Vec<String>) {
    todo!()
}
