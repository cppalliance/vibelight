//! The Legend section of the HTML: what the page's notation means, so a
//! reader of the page alone can follow it.

use crate::document::{Inline, Item};
use crate::layout::Span;

/// The lines of the HTML legend, in order; backticks mark code.
const HTML_LINES: &[&str] = &[
    "Each module has a section: its path, its anatomy line of what it declares and by which visibility, its types, then its items. The sidebar is the module tree, and a bold name is on the public surface.",
    "Under a module's anatomy line, `Used by` names the modules that use its items and `Uses` the modules whose items it uses, each with the number of items used. `Instability` runs from 0 for a module that others use and that uses none to 1 for one that uses others and that none use.",
    "A type prints as a C++-style declaration with Rust's item keyword and its visibility as written, such as `pub(crate) struct`, with the members of all its impls merged in.",
    "`static` marks a function with no receiver; `() mut` marks one that takes `&mut self`, and a method with no marker takes `&self`.",
    "`: Clone, Debug` is the base list: the standard traits a type derives or implements, by short name.",
    "A type lists first what has no visibility of its own: an enum's variants, a trait's items, and its macro marks. Then come its fields, then its other members, each set grouped by visibility under a label such as `pub:`, `pub(crate):`, or `pub(self):` for no keyword, the widest first.",
    "A module's items are grouped under the same labels, after its macro marks and its extern blocks.",
    "A dim `reaches pub(crate)` line says that a node reaches less far than its visibility gives, because a module around it is narrower; a member that reaches as far as its type leaves it unsaid.",
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
