<a id="crate.runvil_parser"></a>
# runvil_parser

```
//! Prompt file parser.
//!
//! A prompt is one markdown file: YAML frontmatter, a required H1, optional H1
//! blocks, then H2 sections.
//! H1 and section content are alternating sequences of exact lua fences and
//! prose (Block). Sections nest recursively (H3 under H2, H4 under H3, and
//! so on through H6). Classic prologue/prose/epilog is exactly [Lua, Prose, Lua].
//!
//! The parser turns bytes into a Prompt tree without executing any of it.
```

- [Level 1: top-level types](#level.1)
  - [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
- [Level 2](#level.2)
  - [ParseError](#type.runvil_parser.ParseError)
  - [Frontmatter](#type.runvil_parser.Frontmatter)
  - [prompt::Section](#type.runvil_parser.prompt.Section)
  - [Result](#type.runvil_parser.Result)
- [Level 3](#level.3)
  - [prompt::Block](#type.runvil_parser.prompt.Block)
  - [Error](#type.runvil_parser.Error)
- [Level 4](#level.4)
  - [BodyText](#type.runvil_parser.BodyText)
- [Modules](#modules)
  - [runvil_parser](#mod.runvil_parser)
  - [parse](#mod.runvil_parser.parse)
  - [prompt](#mod.runvil_parser.prompt)

**MACROS:** 3 derive lists
- **MACRO** `derive: Clone, Debug, Eq, PartialEq`, in [prompt::Block](#type.runvil_parser.prompt.Block)
- **MACRO** `derive: Clone, Debug, Eq, PartialEq`, in [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
- **MACRO** `derive: Clone, Debug, Eq, PartialEq`, in [prompt::Section](#type.runvil_parser.prompt.Section)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.runvil_parser.prompt.Prompt"></a>
### prompt::Prompt

**MACRO** derive: Clone, Debug, Eq, PartialEq

```
/// A fully parsed prompt file.
struct Prompt
{
    /// Parses the full source text of a prompt file into a Prompt and
    /// returns the events reported during parsing alongside the result.
    ///
    /// The caller supplies execution, an identifier that every event
    /// carries. Every event is reported under task 0 because parsing happens
    /// before any run starts.
    ///
    /// The events come in this order: ParseStarted, then a
    /// LuaCompilationStarted event and a LuaCompilationSucceeded or
    /// LuaCompilationFailed event for each Lua block the parser compiles,
    /// then ParseSucceeded or ParseFailed. They exist only for the caller
    /// to log.
    ///
    /// # Errors
    /// On failure, the first element of the pair is a ParseError whose
    /// kind is:
    ///
    /// - Frontmatter when the --- delimiters are missing, the frontmatter
    ///   fails to decode, a uses: entry is neither a Pack id nor
    ///   vendor/name, or two entries share a Pack id.
    /// - Structure when the H1 is missing, there is more than one H1, the H1
    ///   title is empty, a section heading skips a level (such as an H4
    ///   directly under an H2), a section heading is empty, or two sibling
    ///   sections have the same name.
    /// - Fence when a lua fence is left open, or anywhere in the body a
    ///   fence or line breaks the strict Lua fence rule. A lua fence opens
    ///   with a line that is exactly three backticks then lua and closes
    ///   with a line that is exactly three backticks. Label: a fence's label
    ///   starts with lua in any letter case without being exactly lua,
    ///   such as Lua or lua extra. Opener: a fence labeled lua opens
    ///   with any other line, such as one that is indented, uses tildes or
    ///   four backticks, or sits in a block quote or list item. Closer: a
    ///   fence with an exact opening line closes with any other line.
    ///   Lookalike: a line outside every fence looks like a lua opening
    ///   line, such as one in an indented code block. The first violation in
    ///   source order fails the parse at its line, before any Lua compiles.
    ///   Lines inside the body of another fenced code block are never
    ///   inspected, so a document can show a Lua fence as an example.
    /// - List when a section made up only of list items has an empty item.
    /// - Lua when a Lua block in the H1 or in a section fails to compile as
    ///   Lua.
    /// - Internal when the parser fails through no fault of the prompt: one
    ///   of its own invariants breaks, or the Lua compiler cannot start.
    static parse(input: &str, execution: &str) ->
        (std::result::Result<Prompt, ParseError>, Vec<Event>);

    /// Returns the parsed frontmatter.
    frontmatter() const -> &Frontmatter;

    /// Returns the prompt's title, the text of its required H1 heading.
    title() const -> &str;

private:
    /// The parsed YAML frontmatter.
    frontmatter: Frontmatter,

    /// Ordered live Lua and prose blocks from the H1.
    h1_blocks: Vec<Block>,

    /// Top-level sections (H2s) in file order.
    sections: Vec<Section>,

    /// The required H1 title.
    title: String,

    static parse_body(
        frontmatter: Frontmatter,
        shared: &Arc<str>,
        frontmatter_lines: u32,
        emitter: &Emitter) -> Result<Prompt>;

    static parse_inner(input: &str, emitter: &Emitter) -> Result<Prompt>;

    /// The entry-point section: the first top-level section in file order;
    /// the Engine reaches it through crate::detail::entry.
    entry() const -> Option<&Section>;

    /// Returns the ordered live Lua and prose blocks from the H1; the Engine
    /// reaches them through crate::detail::h1_blocks.
    h1_blocks() const -> &[Block];

    /// Returns the top-level H2 sections in file order; the Engine reaches
    /// them through crate::detail::sections.
    sections() const -> &[Section];
}
```

<a id="level.2"></a>
## Level 2

<a id="type.runvil_parser.ParseError"></a>
### ParseError

```
/// A parse failure reported to callers.
struct ParseError;
```

<a id="type.runvil_parser.Frontmatter"></a>
### Frontmatter

```
/// The parsed YAML frontmatter.
struct Frontmatter;
```

<a id="type.runvil_parser.prompt.Section"></a>
### prompt::Section

**MACRO** derive: Clone, Debug, Eq, PartialEq

```
/// One section of a prompt: a heading, ordered blocks, and children.
struct Section
{
    /// Returns the ordered Lua and prose blocks of this section.
    blocks() const -> &[Block];

    /// Returns the child sections nested under this one.
    children() const -> &[Section];

    /// Classic trailing Lua fence when the last block is Lua and not the sole
    /// leading prologue (a section that is only one Lua block has no epilog).
    epilog() const -> Option<&LuaProgram>;

    /// Returns the pre-parsed bullet items for a list-only section.
    items() const -> &[BodyText];

    /// Returns the heading level (2 through 6).
    level() const -> u8;

    /// The section's own raw Markdown: from just after its heading line to the next
    /// heading of any level, leading blank lines and trailing whitespace trimmed.
    markdown() const -> &str;

    /// Returns the heading text (the section's address).
    name() const -> &str;

    /// Classic leading Lua fence when the first block is Lua.
    prologue() const -> Option<&LuaProgram>;

    /// Text of the last prose block, or "" when the section has none.
    prose() const -> &str;

private:
    /// Ordered lua/prose blocks for this section.
    blocks: Vec<Block>,

    /// Child sections nested under this one (deeper heading levels).
    children: Vec<Section>,

    /// Pre-parsed bullet items for list-only sections (no lua blocks).
    ///
    /// Empty for non-list sections.
    items: Vec<BodyText>,

    /// The heading level, 2 through 6.
    level: u8,

    /// The section's own raw Markdown, without its heading line or children.
    markdown: BodyText,

    /// The heading text (the section's address).
    name: String,
}
```

<a id="type.runvil_parser.Result"></a>
### Result

```
/// The crate's result type.
using Result<T> = std::result::Result<T, Error>;
```

<a id="level.3"></a>
## Level 3

<a id="type.runvil_parser.prompt.Block"></a>
### prompt::Block

**MACRO** derive: Clone, Debug, Eq, PartialEq

```
/// One executable block inside a section: a compiled Lua fence or prose.
enum Block
{
    /// An exact lua fence compiled at parse time.
    Lua(LuaProgram),

    /// Author prose: the pending Markdown accumulated since the nearest
    /// preceding heading, lua fence, or thematic break.
    ///
    /// The executor installs it as the following Lua block's lazy prose
    /// template.
    Prose
    {
        /// Captured Markdown, trimmed of surrounding blank lines.
        text: BodyText,
    },
}
```

<a id="type.runvil_parser.Error"></a>
### Error

```
/// The crate's internal error.
struct Error;
```

<a id="level.4"></a>
## Level 4

<a id="type.runvil_parser.BodyText"></a>
### BodyText

```
/// Shared Markdown text.
struct BodyText;
```

<a id="modules"></a>
## Modules

<a id="mod.runvil_parser"></a>
### runvil_parser

- [BodyText](#type.runvil_parser.BodyText)
- [Error](#type.runvil_parser.Error)
- [Frontmatter](#type.runvil_parser.Frontmatter)
- [ParseError](#type.runvil_parser.ParseError)
- [Result](#type.runvil_parser.Result)

```
using prompt::Block;

using runvil_lua::LuaProgram;

using prompt::Prompt;

using prompt::Section;
```

<a id="mod.runvil_parser.parse"></a>
### parse

```
//! Parses a prompt's source into a Prompt: frontmatter, H1, and sections.
```

<a id="mod.runvil_parser.prompt"></a>
### prompt

```
//! Holds the parsed prompt tree: Prompt, its Sections, and their Blocks.
//!
//! Code outside the Engine reads a prompt's title and frontmatter; the tree
//! below them is the Engine's, reached through crate::detail.
//!
//! Construction happens in the parsing modules; this module holds the value
//! types and the invariant-preserving operations on them.
```

- [prompt::Block](#type.runvil_parser.prompt.Block)
- [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
- [prompt::Section](#type.runvil_parser.prompt.Section)
