//! The legend of the HTML: what the overview and the pages' notation mean,
//! so a reader of the HTML alone can follow it. The overview prints it once.

use crate::html::render_html::escape;

/// The lines of the legend, in order; backticks mark code.
const LINES: &[&str] = &[
    "A facade is a crate whose public surface is mostly re-exports of other crates here: the API that users of a crate family write against. Its implementation crates are its family: the crates its code reaches, short of other facades. A warning marks a crate outside a family that uses one of its implementation crates directly, past the facade.",
    "A facade's page shows only what its users can name: each item under the path they write, with its `pub` members alone, pulled from the crate that defines it, under a dim line that says where that crate writes it. A type's header there leaves out `pub`, as in `struct Run`, since every item of the page is public.",
    "In a facade's list of symbols, a bold type is one that no other item of its API names in its code: usually where reading starts. `Named, not re-exported`, at the end of its page, lists what that code names and the facade does not re-export, under the crate a user names each through.",
    "Each module has a section: its path, its anatomy line of what it declares and by which visibility, its types, then its items. The sidebar is the module tree, and a bold name is on the public surface.",
    "Under a module's anatomy line, `Used by` names the modules that use its items and `Uses` the modules whose items it uses, each with the number of items used. `Instability` runs from 0 for a module that others use and that uses none to 1 for one that uses others and that none use.",
    "A type prints as a C++-style declaration with Rust's item keyword and its visibility as written, such as `pub(crate) struct`, with the members of all its impls merged in.",
    "`static` marks a function with no receiver; `() mut` marks one that takes `&mut self`, and a method with no marker takes `&self`.",
    "`: Clone, Debug` is the base list: the standard traits a type derives or implements, by short name. A comment written above an impl prints over its trait there, and the comment of an inherent impl opens the type's body.",
    "A type lists first what has no visibility of its own: an enum's variants, a trait's items, and its macro marks. Then come its fields, then its other members, each set grouped by visibility under a label such as `pub:`, `pub(crate):`, or `pub(self):` for no keyword, the widest first.",
    "A module's items are grouped under the same labels, after its macro marks and its extern blocks.",
    "A dim `reaches pub(crate)` line says that a node reaches less far than its visibility gives, because a module around it is narrower; a member that reaches as far as its type leaves it unsaid.",
    "`[[...]]` is an attribute that shapes the API or its wire format, such as `[[must_use]]` or `[[serde(rename = \"id\")]]`.",
    "`implements: Trait for Type` is an impl with no declared type to attach to, such as a blanket impl.",
    "`MACRO` marks a macro definition, an invocation, a derive from another crate, or an attribute macro.",
    "A doc comment prints as prose, and `backticked` text prints in the code font; a member's comment reflows as one paragraph.",
    "A gold link goes to another crate's page. `Ctrl K` searches every declaration, a click with `Ctrl` opens a link in a new tab, and a click on a source link copies its file and line.",
];

/// Returns the legend as an HTML list.
pub(crate) fn html() -> String {
    let mut out = String::from("<ul class=\"legend\">\n");
    for line in LINES {
        out.push_str("<li>");
        for (index, part) in line.split('`').enumerate() {
            if index % 2 == 1 {
                out.push_str("<code>");
                out.push_str(&escape(part));
                out.push_str("</code>");
            } else {
                out.push_str(&escape(part));
            }
        }
        out.push_str("</li>\n");
    }
    out.push_str("</ul>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backticks_turn_into_code_and_text_is_escaped() {
        let html = html();
        assert!(html.starts_with("<ul class=\"legend\">\n<li>A facade is "));
        assert!(html.contains("<code>[[must_use]]</code>"));
        assert!(html.contains("<code>[[serde(rename = \"id\")]]</code>"));
        assert!(html.contains("<code>() mut</code>"));
        assert!(!html.contains('`'));
    }
}
