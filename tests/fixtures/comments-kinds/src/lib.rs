//! Comment kinds and their order.
//!
//! Crate docs print as `//!` lines.

/// A doc comment first.
// Then an ordinary comment.
pub struct DocFirst;

// An ordinary comment first.
/// Then a doc comment.
pub struct PlainFirst;

/* A block comment
   kept as written. */
pub struct BlockComment;

/// Docs with their own code fence:
///
/// ```
/// let x = [1, 2];
/// ```
///
/// And a ```` four-backtick run inside.
pub struct Fenced;

/// Links: [`DocFirst`], [the plain one](PlainFirst), and `inline code`.
pub struct Linked;

/*
```
not code
```
*/
pub fn after_block() {}
