//! The Legend section of the markdown: what the outline's notation means,
//! so a reader of the file alone, often a model, can follow it.

use crate::document::{Inline, Item};
use crate::layout::Span;

/// The lines of the Legend, in order; backticks mark code.
const LINES: &[&str] = &[
    "Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.",
    "Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.",
    "`#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.",
    "Each impl prints with its type, so a module that holds only impls shows nothing but its docs.",
    "`MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.",
    "Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.",
    "Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.",
    "Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.",
    "A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.",
    "A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.",
    "The first line is the UTC time the file's content last changed.",
];

/// The lines of the HTML legend: the notation a reader of the page meets
/// that the markdown legend does not explain, in the same `backtick` style.
const HTML_LINES: &[&str] = &[
    "A type prints as a C++ declaration: `class` when every member is private and `struct` when one is public, with the members of all its impls merged in.",
    "`static` marks a function with no receiver; `() mut` marks one that takes `&mut self`, and a method with no marker takes `&self`.",
    "`: Clone, Debug` is the base list: the standard traits a type derives or implements, by short name.",
    "`internal:` opens the `pub(crate)` members and `private:` the rest; a type that is not public gets `// internal` or `// private` above its header.",
    "`[[...]]` is an attribute that shapes the API or its wire format, such as `[[must_use]]` or `[[serde(rename = \"id\")]]`.",
    "`implements: Trait for Type;` is an impl with no declared type to attach to, such as a blanket impl.",
    "`MACRO` marks a macro definition, an invocation, a derive from another crate, or an attribute macro.",
    "A doc comment prints as prose, and `backticked` text prints in the code font; a member's comment keeps the fixed layout of its block.",
];

/// Returns the HTML legend as list items.
pub(crate) fn html_items() -> Vec<Item> {
    HTML_LINES
        .iter()
        .map(|line| Item {
            inlines: inlines(line),
            children: Vec::new(),
        })
        .collect()
}

/// Returns the legend as list items.
pub(crate) fn items() -> Vec<Item> {
    LINES
        .iter()
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
