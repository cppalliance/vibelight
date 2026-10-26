//! The Legend section: what the outline's notation means, so a reader of
//! the file alone can follow it. The text depends on the run's options,
//! never on source links.

use crate::document::{Inline, Item};
use crate::layout::Span;
use crate::settings::{KillList, Options};

/// The lines about how a block prints; backticks mark code.
const BLOCKS: &[&str] = &[
    "Each type is one C++-style block: a `struct` has a public member, and a `class` has none.",
    "Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.",
    "A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.",
    "References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.",
    "`constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.",
    "`Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.",
    "A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.",
];

/// The start of the line about macros; the rest says what the kill list
/// shows.
const MACROS: &str = "`MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro";

/// The lines about the sections.
const SECTIONS: &[&str] = &[
    "Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.",
    "Level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.",
    "Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.",
    "With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.",
];

/// The line about the facts, when they are on.
const FACTS: &str = "A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, special members, and functions; a `Used by:` line links what names it; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.";

/// The last line, about the stamp.
const STAMP: &str = "The first line is the UTC time the file's content last changed.";

/// Returns the legend as list items for a run's options.
pub(crate) fn items(options: Options) -> Vec<Item> {
    let macros = match options.kill_list {
        KillList::Full => format!("{MACROS}; the kill list near the top gathers them."),
        KillList::Counts => format!("{MACROS}; the kill list near the top counts them."),
        KillList::Off => format!("{MACROS}."),
    };
    let mut lines: Vec<&str> = BLOCKS.to_vec();
    lines.push(&macros);
    lines.extend(SECTIONS);
    if options.facts {
        lines.push(FACTS);
    }
    lines.push(STAMP);
    lines
        .into_iter()
        .map(|line| Item {
            inlines: inlines(line),
            children: Vec::new(),
        })
        .collect()
}

/// Splits a line at its backticks into text and code.
fn inlines(line: &str) -> Vec<Inline> {
    line.split('`')
        .enumerate()
        .filter(|(_, part)| !part.is_empty())
        .map(|(index, part)| {
            if index % 2 == 1 {
                Inline::Code(vec![Span::Code(part.to_owned())])
            } else {
                Inline::Text(part.to_owned())
            }
        })
        .collect()
}
