*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"runvil_parser","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":29,"to":44},{"path":"parse","from":45,"to":46},{"path":"prompt","from":47,"to":116}]}
{"k":"lead","id":"field-and-accessor","mode":"direct-syntax","subjects":["prompt::Prompt::h1_blocks"],"evidence":"`pub(crate) h1_blocks` and `fn h1_blocks`","locs":["src/prompt.rs:0","src/prompt.rs:0"]}
{"k":"lead","id":"field-and-accessor","mode":"direct-syntax","subjects":["prompt::Prompt::sections"],"evidence":"`pub(crate) sections` and `fn sections`","locs":["src/prompt.rs:0","src/prompt.rs:0"]}
{"k":"lead","id":"wrapper-name-family","mode":"direct-syntax","subjects":["prompt::Prompt::parse","prompt::Prompt::parse_body"],"evidence":"`parse` and `parse_body`","locs":["src/parse.rs:0","src/parse.rs:0"]}
{"k":"lead","id":"wrapper-name-family","mode":"direct-syntax","subjects":["prompt::Prompt::parse","prompt::Prompt::parse_inner"],"evidence":"`parse` and `parse_inner`","locs":["src/parse.rs:0","src/parse.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["parse"],"evidence":"holds only 1 impls","locs":["src/parse.rs:0"]}
{"k":"lead","id":"doc-unresolved-link","mode":"resolved","subjects":["prompt"],"evidence":"[`crate::detail`] reach no item, and no item of the run has the last name","locs":["src/prompt.rs:0"],"limits":["macros"]}
{"k":"lead","id":"impossible-error","mode":"direct-syntax","subjects":["prompt::Prompt::parse"],"evidence":"\"invariants\" in \"`Internal` when the parser fails through no fault of the prompt: one of its own invariants breaks, or the Lua compiler cannot start.\"","locs":["src/parse.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["prompt::Section::epilog"],"evidence":"no hit of `epilog` in the scanned sources beyond its 1 declaration","locs":["src/prompt.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["prompt::Section::prologue"],"evidence":"no hit of `prologue` in the scanned sources beyond its 1 declaration","locs":["src/prompt.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["prompt::Section::prose"],"evidence":"no hit of `prose` in the scanned sources beyond its 1 declaration","locs":["src/prompt.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["prompt::Prompt::parse"],"evidence":"4 production hits of `parse` outside macros, none a resolved or unique qualified path to it","locs":["src/parse.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["prompt::Block::Lua"],"evidence":"no observed path builds `Block::Lua`; 3 patterns or comparisons match it","locs":["src/prompt.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["prompt::Block::Prose"],"evidence":"no observed path builds `Block::Prose`; 1 patterns or comparisons match it","locs":["src/prompt.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"doc-outweighs-body","mode":"direct-syntax","subjects":["prompt::Prompt::parse"],"evidence":"401 doc words beside 98 body tokens","locs":["src/parse.rs:0"]}
{"k":"surface","path":"Block","kind":"enum","item":"prompt::Block"}
{"k":"surface","path":"BodyText","kind":"struct"}
{"k":"surface","path":"Error","kind":"struct"}
{"k":"surface","path":"Frontmatter","kind":"struct"}
{"k":"surface","path":"LuaProgram","kind":"external","item":"runvil_lua::LuaProgram"}
{"k":"surface","path":"ParseError","kind":"struct"}
{"k":"surface","path":"Prompt","kind":"struct","item":"prompt::Prompt"}
{"k":"surface","path":"Section","kind":"struct","item":"prompt::Section"}
{"k":"external","path":"runvil_lua::LuaProgram","uses":3,"named_by":["prompt::Block","prompt::Section"]}
{"k":"external","path":"runvil_types::emitter::Emitter","uses":2,"named_by":["prompt::Prompt"]}
{"k":"external","path":"runvil_types::event::Event","uses":1,"named_by":["prompt::Prompt"]}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":5}
{"k":"doc","path":"crate","text":"Prompt file parser.\n\nA prompt is one markdown file: YAML frontmatter, a required H1, optional H1\nblocks, then H2 sections.\nH1 and section content are alternating sequences of exact `lua` fences and\nprose ([`Block`]). Sections nest recursively (H3 under H2, H4 under H3, and\nso on through H6). Classic prologue/prose/epilog is exactly `[Lua, Prose, Lua]`.\n\nThe parser turns bytes into a [`Prompt`] tree without executing any of it."}
{"k":"use","path":"Block","rust":"pub use prompt::Block","loc":"src/lib.rs:0"}
{"k":"use","path":"Prompt","rust":"pub use prompt::Prompt","loc":"src/lib.rs:0"}
{"k":"use","path":"Section","rust":"pub use prompt::Section","loc":"src/lib.rs:0"}
{"k":"use","path":"LuaProgram","rust":"pub use runvil_lua::LuaProgram","loc":"src/lib.rs:0"}
{"k":"struct","path":"BodyText","rust":"pub struct BodyText","loc":"src/lib.rs:0","named_by":["prompt::Block","prompt::Section"],"token_hits":5}
{"k":"doc","path":"BodyText","text":"Shared Markdown text."}
{"k":"struct","path":"Frontmatter","rust":"pub struct Frontmatter","loc":"src/lib.rs:0","named_by":["prompt::Prompt"],"token_hits":7}
{"k":"doc","path":"Frontmatter","text":"The parsed YAML frontmatter."}
{"k":"struct","path":"ParseError","rust":"pub struct ParseError","loc":"src/lib.rs:0","named_by":["prompt::Prompt"],"token_hits":3}
{"k":"doc","path":"ParseError","text":"A parse failure reported to callers."}
{"k":"struct","path":"Error","rust":"pub struct Error","loc":"src/lib.rs:0","named_by":["Result"],"token_hits":8}
{"k":"doc","path":"Error","text":"The crate's internal error."}
{"k":"alias","path":"Result","rust":"pub(crate) type Result<T> = std::result::Result<T, Error>","loc":"src/lib.rs:0","named_by":["prompt::Prompt"],"token_hits":6}
{"k":"doc","path":"Result","text":"The crate's result type."}
{"k":"mod","path":"parse","loc":"src/parse.rs:0","vis":"private"}
{"k":"doc","path":"parse","text":"Parses a prompt's source into a [`Prompt`]: frontmatter, H1, and sections."}
{"k":"mod","path":"prompt","loc":"src/prompt.rs:0","vis":"private","types":3}
{"k":"doc","path":"prompt","text":"Holds the parsed prompt tree: [`Prompt`], its [`Section`]s, and their [`Block`]s.\n\nCode outside the Engine reads a prompt's title and frontmatter; the tree\nbelow them is the Engine's, reached through [`crate::detail`].\n\nConstruction happens in the parsing modules; this module holds the value\ntypes and the invariant-preserving operations on them."}
{"k":"impl","path":"prompt::Prompt","rust":"impl Prompt","locs":["src/parse.rs:0","src/prompt.rs:0"]}
{"k":"fn","path":"prompt::Prompt::parse","sig":"pub fn parse(input: &str, execution: &str) -> (std::result::Result<Prompt, ParseError>, Vec<Event>)","loc":"src/parse.rs:0","token_hits":4,"body":{"statements":6,"tokens":98},"ops":["execution","input"]}
{"k":"doc","path":"prompt::Prompt::parse","text":"Parses the full source text of a prompt file into a [`Prompt`] and\nreturns the events reported during parsing alongside the result.\n\nThe caller supplies `execution`, an identifier that every event\ncarries. Every event is reported under task `0` because parsing happens\nbefore any run starts.\n\nThe events come in this order: `ParseStarted`, then a\n`LuaCompilationStarted` event and a `LuaCompilationSucceeded` or\n`LuaCompilationFailed` event for each Lua block the parser compiles,\nthen `ParseSucceeded` or `ParseFailed`. They exist only for the caller\nto log.\n\n# Errors\nOn failure, the first element of the pair is a [`ParseError`] whose\nkind is:\n\n- `Frontmatter` when the `---` delimiters are missing, the frontmatter\n  fails to decode, a `uses:` entry is neither a Pack id nor\n  vendor/name, or two entries share a Pack id.\n- "}
{"k":"more","path":"prompt::Prompt::parse","field":"text","part":2,"text":"`Structure` when the H1 is missing, there is more than one H1, the H1\n  title is empty, a section heading skips a level (such as an H4\n  directly under an H2), a section heading is empty, or two sibling\n  sections have the same name.\n- `Fence` when a `lua` fence is left open, or anywhere in the body a\n  fence or line breaks the strict Lua fence rule. A `lua` fence opens\n  with a line that is exactly three backticks then `lua` and closes\n  with a line that is exactly three backticks. Label: a fence's label\n  starts with `lua` in any letter case without being exactly `lua`,\n  such as `Lua` or `lua extra`. Opener: a fence labeled `lua` opens\n  with any other line, such as one that is indented, uses tildes or\n  four backticks, or sits in a block quote or list item. Closer: a\n  fence with an exact "}
{"k":"more","path":"prompt::Prompt::parse","field":"text","part":3,"text":"opening line closes with any other line.\n  Lookalike: a line outside every fence looks like a `lua` opening\n  line, such as one in an indented code block. The first violation in\n  source order fails the parse at its line, before any Lua compiles.\n  Lines inside the body of another fenced code block are never\n  inspected, so a document can show a Lua fence as an example.\n- `List` when a section made up only of list items has an empty item.\n- `Lua` when a Lua block in the H1 or in a section fails to compile as\n  Lua.\n- `Internal` when the parser fails through no fault of the prompt: one\n  of its own invariants breaks, or the Lua compiler cannot start."}
{"k":"fn","path":"prompt::Prompt::parse_inner","sig":"fn parse_inner(input: &str, emitter: &Emitter) -> Result<Prompt>","loc":"src/parse.rs:0","direct_callers":["prompt::Prompt::parse"],"token_hits":1,"body":{"statements":6,"tokens":277,"tries":3,"nones":2},"ops":["emitter","input"],"comments":[{"text":"// The location is relative to the frontmatter block, which starts","line":0},{"text":"// on file line 2 (line 1 is the opening `---` delimiter).","line":0},{"text":"// Everything past the frontmatter postdates the prompt's name, so a","line":0},{"text":"// failure from here on is stamped with it (and its span's position).","line":0}]}
{"k":"fn","path":"prompt::Prompt::parse_body","sig":"fn parse_body(frontmatter: Frontmatter, shared: &Arc<str>, frontmatter_lines: u32, emitter: &Emitter) -> Result<Prompt>","loc":"src/parse.rs:0","direct_callers":["prompt::Prompt::parse_inner"],"token_hits":1,"body":{"statements":14,"tokens":419,"tries":5,"errs":4,"macros":["format"]},"ops":["emitter","frontmatter","frontmatter_lines","shared"],"comments":[{"text":"// Everything before the H1 is preface and has no prompt semantics.","line":0},{"text":"// Sections are headings after the H1 at level 2 or deeper.","line":0}]}
{"k":"enum","path":"prompt::Block","rust":"pub enum Block { Lua(LuaProgram), #[non_exhaustive] Prose { text: BodyText } }","loc":"src/prompt.rs:0","attrs":["non_exhaustive"],"derives":["Clone","Debug","Eq","PartialEq"],"named_by":["prompt::Prompt","prompt::Section"],"reexported_by":["crate"],"token_hits":9}
{"k":"doc","path":"prompt::Block","text":"One executable block inside a section: a compiled Lua fence or prose."}
{"k":"variant","path":"prompt::Block::Lua","rust":"Lua(LuaProgram)","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Block::Lua","text":"An exact `lua` fence compiled at parse time."}
{"k":"variant","path":"prompt::Block::Prose","rust":"#[non_exhaustive] Prose { text: BodyText }","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Block::Prose","text":"Author prose: the pending Markdown accumulated since the nearest\npreceding heading, `lua` fence, or thematic break.\n\nThe executor installs it as the following Lua block's lazy `prose`\ntemplate."}
{"k":"field","path":"prompt::Block::Prose::text","rust":"text: BodyText","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Block::Prose::text","text":"Captured Markdown, trimmed of surrounding blank lines."}
{"k":"struct","path":"prompt::Section","rust":"pub struct Section { pub(crate) name: String, pub(crate) level: u8, pub(crate) blocks: Vec<Block>, pub(crate) children: Vec<Section>, pub(crate) items: Vec<BodyText>, pub(crate) markdown: BodyText }","loc":"src/prompt.rs:0","attrs":["non_exhaustive"],"derives":["Clone","Debug","Eq","PartialEq"],"named_by":["prompt::Prompt"],"reexported_by":["crate"],"token_hits":7}
{"k":"doc","path":"prompt::Section","text":"One section of a prompt: a heading, ordered blocks, and children."}
{"k":"field","path":"prompt::Section::name","rust":"pub(crate) name: String","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Section::name","text":"The heading text (the section's address)."}
{"k":"field","path":"prompt::Section::level","rust":"pub(crate) level: u8","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Section::level","text":"The heading level, 2 through 6."}
{"k":"field","path":"prompt::Section::blocks","rust":"pub(crate) blocks: Vec<Block>","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Section::blocks","text":"Ordered lua/prose blocks for this section."}
{"k":"field","path":"prompt::Section::children","rust":"pub(crate) children: Vec<Section>","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Section::children","text":"Child sections nested under this one (deeper heading levels)."}
{"k":"field","path":"prompt::Section::items","rust":"pub(crate) items: Vec<BodyText>","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Section::items","text":"Pre-parsed bullet items for list-only sections (no lua blocks).\n\nEmpty for non-list sections."}
{"k":"field","path":"prompt::Section::markdown","rust":"pub(crate) markdown: BodyText","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Section::markdown","text":"The section's own raw Markdown, without its heading line or children."}
{"k":"impl","path":"prompt::Section","rust":"impl Section","locs":["src/prompt.rs:0"]}
{"k":"fn","path":"prompt::Section::name","sig":"pub fn name(&self) -> &str","loc":"src/prompt.rs:0","attrs":["must_use"],"method_name_hits":1,"token_hits":4,"body":{"statements":1,"tokens":6,"single":"field name"},"ops":["self.name"]}
{"k":"doc","path":"prompt::Section::name","text":"Returns the heading text (the section's address)."}
{"k":"fn","path":"prompt::Section::level","sig":"pub fn level(&self) -> u8","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":3,"body":{"statements":1,"tokens":5,"single":"field level"},"ops":["self.level"]}
{"k":"doc","path":"prompt::Section::level","text":"Returns the heading level (2 through 6)."}
{"k":"fn","path":"prompt::Section::blocks","sig":"pub fn blocks(&self) -> &[Block]","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":4,"body":{"statements":1,"tokens":6,"single":"field blocks"},"ops":["self.blocks"]}
{"k":"doc","path":"prompt::Section::blocks","text":"Returns the ordered Lua and prose blocks of this section."}
{"k":"fn","path":"prompt::Section::children","sig":"pub fn children(&self) -> &[Section]","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":1,"body":{"statements":1,"tokens":6,"single":"field children"},"ops":["self.children"]}
{"k":"doc","path":"prompt::Section::children","text":"Returns the child sections nested under this one."}
{"k":"fn","path":"prompt::Section::items","sig":"pub fn items(&self) -> &[BodyText]","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":2,"body":{"statements":1,"tokens":6,"single":"field items"},"ops":["self.items"]}
{"k":"doc","path":"prompt::Section::items","text":"Returns the pre-parsed bullet items for a list-only section."}
{"k":"fn","path":"prompt::Section::markdown","sig":"pub fn markdown(&self) -> &str","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":1,"body":{"statements":1,"tokens":9,"single":"method as_str"},"ops":["self.markdown.as_str"]}
{"k":"doc","path":"prompt::Section::markdown","text":"The section's own raw Markdown: from just after its heading line to the next\nheading of any level, leading blank lines and trailing whitespace trimmed."}
{"k":"fn","path":"prompt::Section::prologue","sig":"pub fn prologue(&self) -> Option<&LuaProgram>","loc":"src/prompt.rs:0","attrs":["must_use"],"body":{"statements":1,"tokens":32,"nones":1},"ops":["self.blocks.first"]}
{"k":"doc","path":"prompt::Section::prologue","text":"Classic leading Lua fence when the first block is Lua."}
{"k":"fn","path":"prompt::Section::prose","sig":"pub fn prose(&self) -> &str","loc":"src/prompt.rs:0","attrs":["must_use"],"body":{"statements":1,"tokens":50,"nones":1,"single":"method unwrap_or"},"ops":["self.blocks.iter.rev.find_map.unwrap_or"]}
{"k":"doc","path":"prompt::Section::prose","text":"Text of the last prose block, or `\"\"` when the section has none."}
{"k":"fn","path":"prompt::Section::epilog","sig":"pub fn epilog(&self) -> Option<&LuaProgram>","loc":"src/prompt.rs:0","attrs":["must_use"],"body":{"statements":1,"tokens":45,"nones":2},"ops":["self.blocks.as_slice"]}
{"k":"doc","path":"prompt::Section::epilog","text":"Classic trailing Lua fence when the last block is Lua and not the sole\nleading prologue (a section that is only one Lua block has no epilog)."}
{"k":"struct","path":"prompt::Prompt","rust":"pub struct Prompt { pub(crate) frontmatter: Frontmatter, pub(crate) title: String, pub(crate) h1_blocks: Vec<Block>, pub(crate) sections: Vec<Section> }","loc":"src/prompt.rs:0","attrs":["non_exhaustive"],"derives":["Clone","Debug","Eq","PartialEq"],"reexported_by":["crate"],"token_hits":8}
{"k":"doc","path":"prompt::Prompt","text":"A fully parsed prompt file."}
{"k":"field","path":"prompt::Prompt::frontmatter","rust":"pub(crate) frontmatter: Frontmatter","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Prompt::frontmatter","text":"The parsed YAML frontmatter."}
{"k":"field","path":"prompt::Prompt::title","rust":"pub(crate) title: String","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Prompt::title","text":"The required H1 title."}
{"k":"field","path":"prompt::Prompt::h1_blocks","rust":"pub(crate) h1_blocks: Vec<Block>","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Prompt::h1_blocks","text":"Ordered live Lua and prose blocks from the H1."}
{"k":"field","path":"prompt::Prompt::sections","rust":"pub(crate) sections: Vec<Section>","loc":"src/prompt.rs:0"}
{"k":"doc","path":"prompt::Prompt::sections","text":"Top-level sections (H2s) in file order."}
{"k":"fn","path":"prompt::Prompt::frontmatter","sig":"pub fn frontmatter(&self) -> &Frontmatter","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":10,"body":{"statements":1,"tokens":6,"single":"field frontmatter"},"ops":["self.frontmatter"]}
{"k":"doc","path":"prompt::Prompt::frontmatter","text":"Returns the parsed frontmatter."}
{"k":"fn","path":"prompt::Prompt::title","sig":"pub fn title(&self) -> &str","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":6,"body":{"statements":1,"tokens":6,"single":"field title"},"ops":["self.title"]}
{"k":"doc","path":"prompt::Prompt::title","text":"Returns the prompt's title, the text of its required H1 heading."}
{"k":"fn","path":"prompt::Prompt::h1_blocks","sig":"pub(crate) fn h1_blocks(&self) -> &[Block]","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":3,"body":{"statements":1,"tokens":6,"single":"field h1_blocks"},"ops":["self.h1_blocks"]}
{"k":"doc","path":"prompt::Prompt::h1_blocks","text":"Returns the ordered live Lua and prose blocks from the H1; the Engine\nreaches them through [`crate::detail::h1_blocks`]."}
{"k":"fn","path":"prompt::Prompt::sections","sig":"pub(crate) fn sections(&self) -> &[Section]","loc":"src/prompt.rs:0","attrs":["must_use"],"token_hits":9,"body":{"statements":1,"tokens":6,"single":"field sections"},"ops":["self.sections"]}
{"k":"doc","path":"prompt::Prompt::sections","text":"Returns the top-level H2 sections in file order; the Engine reaches\nthem through [`crate::detail::sections`]."}
{"k":"fn","path":"prompt::Prompt::entry","sig":"pub(crate) fn entry(&self) -> Option<&Section>","loc":"src/prompt.rs:0","attrs":["must_use"],"body":{"statements":1,"tokens":9,"single":"method first"},"ops":["self.sections.first"]}
{"k":"doc","path":"prompt::Prompt::entry","text":"The entry-point section: the first top-level section in file order;\nthe Engine reaches it through [`crate::detail::entry`]."}
