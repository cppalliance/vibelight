//! The Legend section of the HTML: what the page's notation means, so a
//! reader of the page alone can follow it.

use crate::document::{Inline, Item};
use crate::layout::Span;

/// The lines of the HTML legend, in order; backticks mark code.
const HTML_LINES: &[&str] = &[
    "A type prints as a C++ declaration: `class` when its body starts with an internal or private member and `struct` when it starts public, with the members of all its impls merged in.",
    "`static` marks a function with no receiver; `() mut` marks one that takes `&mut self`, and a method with no marker takes `&self`.",
    "`: Clone, Debug` is the base list: the standard traits a type derives or implements, by short name.",
    "A type lists its data first, its fields or variants, then its other members. `internal:` opens the `pub(crate)` members, `private:` the rest, and `public:` goes back to public; a label prints only where the access changes from what the keyword gives, so a type can show one twice.",
    "A module's item that is not public has a line above it that reads `internal` or `private`.",
    "`[[...]]` is an attribute that shapes the API or its wire format, such as `[[must_use]]` or `[[serde(rename = \"id\")]]`.",
    "`implements: Trait for Type` is an impl with no declared type to attach to, such as a blanket impl.",
    "`MACRO` marks a macro definition, an invocation, a derive from another crate, or an attribute macro.",
    "A doc comment prints as prose, and `backticked` text prints in the code font; a member's comment reflows as one paragraph.",
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
