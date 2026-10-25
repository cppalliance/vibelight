//! Prompt file parser.
//!
//! A prompt is one markdown file: YAML frontmatter, a required H1, optional H1
//! blocks, then H2 sections.
//! H1 and section content are alternating sequences of exact `lua` fences and
//! prose ([`Block`]). Sections nest recursively (H3 under H2, H4 under H3, and
//! so on through H6). Classic prologue/prose/epilog is exactly `[Lua, Prose, Lua]`.
//!
//! The parser turns bytes into a [`Prompt`] tree without executing any of it.

pub use runvil_lua::LuaProgram;

mod parse;
mod prompt;

pub use prompt::{Block, Prompt, Section};

/// Shared Markdown text.
pub struct BodyText;

/// The parsed YAML frontmatter.
pub struct Frontmatter;

/// A parse failure reported to callers.
pub struct ParseError;

/// The crate's internal error.
pub struct Error;

/// The crate's result type.
pub(crate) type Result<T> = std::result::Result<T, Error>;
