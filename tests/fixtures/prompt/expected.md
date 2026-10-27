*Updated 2026-01-01T00:00:00Z*

<a id="crate.runvil_parser"></a>
# runvil_parser

```
//! Prompt file parser.
//!
//! A prompt is one markdown file: YAML frontmatter, a required H1, optional H1
//! blocks, then H2 sections.
//! H1 and section content are alternating sequences of exact `lua` fences and
//! prose (`Block`). Sections nest recursively (H3 under H2, H4 under H3, and
//! so on through H6). Classic prologue/prose/epilog is exactly `[Lua, Prose, Lua]`.
//!
//! The parser turns bytes into a `Prompt` tree without executing any of it.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations by level, and Levels indexes them: level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Levels](#levels)
- [runvil_parser](#mod.runvil_parser)
  - [ParseError](#type.runvil_parser.ParseError)
  - [Frontmatter](#type.runvil_parser.Frontmatter)
  - [Result](#type.runvil_parser.Result)
  - [Error](#type.runvil_parser.Error)
  - [BodyText](#type.runvil_parser.BodyText)
- [parse](#mod.runvil_parser.parse)
- [prompt](#mod.runvil_parser.prompt)
  - [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
  - [prompt::Section](#type.runvil_parser.prompt.Section)
  - [prompt::Block](#type.runvil_parser.prompt.Block)
- [External items](#external)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Block](#type.runvil_parser.prompt.Block)
- [BodyText](#type.runvil_parser.BodyText)
- [Error](#type.runvil_parser.Error)
- [Frontmatter](#type.runvil_parser.Frontmatter)
- LuaProgram (runvil_lua::LuaProgram)
- [ParseError](#type.runvil_parser.ParseError)
- [Prompt](#type.runvil_parser.prompt.Prompt)
- [Section](#type.runvil_parser.prompt.Section)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
- Level 2
  - [ParseError](#type.runvil_parser.ParseError)
  - [Frontmatter](#type.runvil_parser.Frontmatter)
  - [prompt::Section](#type.runvil_parser.prompt.Section)
  - [Result](#type.runvil_parser.Result)
- Level 3
  - [prompt::Block](#type.runvil_parser.prompt.Block)
  - [Error](#type.runvil_parser.Error)
- Level 4
  - [BodyText](#type.runvil_parser.BodyText)

<a id="mod.runvil_parser"></a>
## runvil_parser

```
pub use prompt::Block;
pub use runvil_lua::LuaProgram;
pub use prompt::Prompt;
pub use prompt::Section;
```

<a id="type.runvil_parser.ParseError"></a>
### ParseError

Counts: none

Used by: [prompt::Prompt](#type.runvil_parser.prompt.Prompt)

```
/// A parse failure reported to callers.
pub struct ParseError;
```

<a id="type.runvil_parser.Frontmatter"></a>
### Frontmatter

Counts: none

Used by: [prompt::Prompt](#type.runvil_parser.prompt.Prompt)

```
/// The parsed YAML frontmatter.
pub struct Frontmatter;
```

<a id="type.runvil_parser.Result"></a>
### Result

Counts: none

Used by: [prompt::Prompt](#type.runvil_parser.prompt.Prompt)

```
/// The crate's result type.
pub(crate) type Result<T> = std::result::Result<T, Error>;
```

<a id="type.runvil_parser.Error"></a>
### Error

Counts: none

Used by: [Result](#type.runvil_parser.Result)

```
/// The crate's internal error.
pub struct Error;
```

<a id="type.runvil_parser.BodyText"></a>
### BodyText

Counts: none

Used by: [prompt::Block](#type.runvil_parser.prompt.Block), [prompt::Section](#type.runvil_parser.prompt.Section)

```
/// Shared Markdown text.
pub struct BodyText;
```

<a id="mod.runvil_parser.parse"></a>
## parse

```
//! Parses a prompt's source into a `Prompt`: frontmatter, H1, and sections.
```

<a id="mod.runvil_parser.prompt"></a>
## prompt

```
//! Holds the parsed prompt tree: `Prompt`, its `Section`s, and their `Block`s.
//!
//! Code outside the Engine reads a prompt's title and frontmatter; the tree
//! below them is the Engine's, reached through `crate::detail`.
//!
//! Construction happens in the parsing modules; this module holds the value
//! types and the invariant-preserving operations on them.
```

<a id="type.runvil_parser.prompt.Prompt"></a>
### prompt::Prompt

Counts: 4 fields, 8 functions

Used by: [runvil_parser](#mod.runvil_parser)

Fields:

- `frontmatter`: frontmatter
- `title`: title
- `h1_blocks`: h1_blocks
- `sections`: entry, sections

```
/// A fully parsed prompt file.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Prompt {
    /// The parsed YAML frontmatter.
    pub(crate) frontmatter: Frontmatter,

    /// The required H1 title.
    pub(crate) title: String,

    /// Ordered live Lua and prose blocks from the H1.
    pub(crate) h1_blocks: Vec<Block>,

    /// Top-level sections (H2s) in file order.
    pub(crate) sections: Vec<Section>,
}

// impl in src/parse.rs
impl Prompt {
    /// Parses the full source text of a prompt file into a `Prompt` and
    /// returns the events reported during parsing alongside the result.
    ///
    /// The caller supplies `execution`, an identifier that every event
    /// carries. Every event is reported under task `0` because parsing happens
    /// before any run starts.
    ///
    /// The events come in this order: `ParseStarted`, then a
    /// `LuaCompilationStarted` event and a `LuaCompilationSucceeded` or
    /// `LuaCompilationFailed` event for each Lua block the parser compiles,
    /// then `ParseSucceeded` or `ParseFailed`. They exist only for the caller
    /// to log.
    ///
    /// # Errors
    /// On failure, the first element of the pair is a `ParseError` whose
    /// kind is:
    ///
    /// - `Frontmatter` when the `---` delimiters are missing, the frontmatter
    ///   fails to decode, a `uses:` entry is neither a Pack id nor
    ///   vendor/name, or two entries share a Pack id.
    /// - `Structure` when the H1 is missing, there is more than one H1, the H1
    ///   title is empty, a section heading skips a level (such as an H4
    ///   directly under an H2), a section heading is empty, or two sibling
    ///   sections have the same name.
    /// - `Fence` when a `lua` fence is left open, or anywhere in the body a
    ///   fence or line breaks the strict Lua fence rule. A `lua` fence opens
    ///   with a line that is exactly three backticks then `lua` and closes
    ///   with a line that is exactly three backticks. Label: a fence's label
    ///   starts with `lua` in any letter case without being exactly `lua`,
    ///   such as `Lua` or `lua extra`. Opener: a fence labeled `lua` opens
    ///   with any other line, such as one that is indented, uses tildes or
    ///   four backticks, or sits in a block quote or list item. Closer: a
    ///   fence with an exact opening line closes with any other line.
    ///   Lookalike: a line outside every fence looks like a `lua` opening
    ///   line, such as one in an indented code block. The first violation in
    ///   source order fails the parse at its line, before any Lua compiles.
    ///   Lines inside the body of another fenced code block are never
    ///   inspected, so a document can show a Lua fence as an example.
    /// - `List` when a section made up only of list items has an empty item.
    /// - `Lua` when a Lua block in the H1 or in a section fails to compile as
    ///   Lua.
    /// - `Internal` when the parser fails through no fault of the prompt: one
    ///   of its own invariants breaks, or the Lua compiler cannot start.
    pub fn parse(input: &str, execution: &str) -> (std::result::Result<Prompt, ParseError>, Vec<Event>);

    fn parse_inner(input: &str, emitter: &Emitter) -> Result<Prompt>;

    fn parse_body(frontmatter: Frontmatter, shared: &Arc<str>, frontmatter_lines: u32, emitter: &Emitter) -> Result<Prompt>;
}

// impl in src/prompt.rs
impl Prompt {
    /// Returns the parsed frontmatter.
    #[must_use]
    pub fn frontmatter(&self) -> &Frontmatter;

    /// Returns the prompt's title, the text of its required H1 heading.
    #[must_use]
    pub fn title(&self) -> &str;

    /// Returns the ordered live Lua and prose blocks from the H1; the Engine
    /// reaches them through `crate::detail::h1_blocks`.
    #[must_use]
    pub(crate) fn h1_blocks(&self) -> &[Block];

    /// Returns the top-level H2 sections in file order; the Engine reaches
    /// them through `crate::detail::sections`.
    #[must_use]
    pub(crate) fn sections(&self) -> &[Section];

    /// The entry-point section: the first top-level section in file order;
    /// the Engine reaches it through `crate::detail::entry`.
    #[must_use]
    pub(crate) fn entry(&self) -> Option<&Section>;
}
```

<a id="type.runvil_parser.prompt.Section"></a>
### prompt::Section

Counts: 6 fields, 9 functions

Used by: [prompt::Prompt](#type.runvil_parser.prompt.Prompt), [runvil_parser](#mod.runvil_parser)

Fields:

- `name`: name
- `level`: level
- `blocks`: blocks, epilog, prologue, prose
- `children`: children
- `items`: items
- `markdown`: markdown

```
/// One section of a prompt: a heading, ordered blocks, and children.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Section {
    /// The heading text (the section's address).
    pub(crate) name: String,

    /// The heading level, 2 through 6.
    pub(crate) level: u8,

    /// Ordered lua/prose blocks for this section.
    pub(crate) blocks: Vec<Block>,

    /// Child sections nested under this one (deeper heading levels).
    pub(crate) children: Vec<Section>,

    /// Pre-parsed bullet items for list-only sections (no lua blocks).
    ///
    /// Empty for non-list sections.
    pub(crate) items: Vec<BodyText>,

    /// The section's own raw Markdown, without its heading line or children.
    pub(crate) markdown: BodyText,
}

// impl in src/prompt.rs
impl Section {
    /// Returns the heading text (the section's address).
    #[must_use]
    pub fn name(&self) -> &str;

    /// Returns the heading level (2 through 6).
    #[must_use]
    pub fn level(&self) -> u8;

    /// Returns the ordered Lua and prose blocks of this section.
    #[must_use]
    pub fn blocks(&self) -> &[Block];

    /// Returns the child sections nested under this one.
    #[must_use]
    pub fn children(&self) -> &[Section];

    /// Returns the pre-parsed bullet items for a list-only section.
    #[must_use]
    pub fn items(&self) -> &[BodyText];

    /// The section's own raw Markdown: from just after its heading line to the next
    /// heading of any level, leading blank lines and trailing whitespace trimmed.
    #[must_use]
    pub fn markdown(&self) -> &str;

    /// Classic leading Lua fence when the first block is Lua.
    #[must_use]
    pub fn prologue(&self) -> Option<&LuaProgram>;

    /// Text of the last prose block, or `""` when the section has none.
    #[must_use]
    pub fn prose(&self) -> &str;

    /// Classic trailing Lua fence when the last block is Lua and not the sole
    /// leading prologue (a section that is only one Lua block has no epilog).
    #[must_use]
    pub fn epilog(&self) -> Option<&LuaProgram>;
}
```

<a id="type.runvil_parser.prompt.Block"></a>
### prompt::Block

Counts: 2 variants

Used by: [prompt::Prompt](#type.runvil_parser.prompt.Prompt), [prompt::Section](#type.runvil_parser.prompt.Section), [runvil_parser](#mod.runvil_parser)

```
/// One executable block inside a section: a compiled Lua fence or prose.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Block {
    /// An exact `lua` fence compiled at parse time.
    Lua(LuaProgram),

    /// Author prose: the pending Markdown accumulated since the nearest
    /// preceding heading, `lua` fence, or thematic break.
    ///
    /// The executor installs it as the following Lua block's lazy `prose`
    /// template.
    #[non_exhaustive]
    Prose {
        /// Captured Markdown, trimmed of surrounding blank lines.
        text: BodyText,
    },
}
```

<a id="external"></a>
## External items

- `runvil_lua`
  - `runvil_lua::LuaProgram`, 3 uses by [prompt::Block](#type.runvil_parser.prompt.Block), [prompt::Section](#type.runvil_parser.prompt.Section)
- `runvil_types`
  - `runvil_types::emitter::Emitter`, 2 uses by [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
  - `runvil_types::event::Event`, 1 use by [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
