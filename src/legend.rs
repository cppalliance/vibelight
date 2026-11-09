//! The legend of the HTML: what the overview and the pages' notation mean,
//! so a reader of the HTML alone can follow it. The overview prints it once.

use crate::render_html::escape;

/// The lines of the legend, in order; backticks mark code.
const LINES: &[&str] = &[
    "A facade is a crate whose public surface is mostly re-exports of other crates here: the API that users of a crate family write against. Its implementation crates are its family: the crates its code reaches, short of other facades. A warning marks a crate outside a family that uses one of its implementation crates directly, past the facade.",
    "A facade's page shows only what its users can name: each item under the path they write, with its `pub` members alone and `/* private fields */` where it hides some, pulled from the crate that defines it, under a dim line that says where that crate writes it.",
    "Each module's section opens with its items `At a glance`, as Rust declares them without comments, then shows them again `Documented`, each with its docs.",
    "Each module has a section: its path, its anatomy line of what it declares and by which visibility, its types, then its items. The sidebar is the module tree, and a bold name is on the public surface.",
    "Under a module's anatomy line, `Used by` names the modules that use its items and `Uses` the modules whose items it uses, each with the number of items used. `Instability` runs from 0 for a module that others use and that uses none to 1 for one that uses others and that none use.",
    "A type prints as Rust declares it, without bodies: its `#[derive]` and other attributes, its head and its fields, variants, or trait items, then one `impl` that holds the members of all its inherent impls, then a `// implements` line that names the other traits it implements.",
    "Members print from the widest visibility to the narrowest, `pub` first and private last, each set by name, with a blank line between sets. A module's items go the same way, as re-exports, constants, statics, and functions.",
    "A dim `reaches pub(crate)` line says that a node reaches less far than its visibility gives, because a module around it is narrower; a member that reaches as far as its type leaves it unsaid.",
    "A dim `#[...]` line is an attribute that shapes the API or its wire format, such as `#[must_use]` or `#[serde(rename = \"id\")]`, or a `#[cfg(...)]` the node depends on.",
    "`impl Trait for Type {}` under Other impls is an impl with no declared type to attach to, such as a blanket impl.",
    "`MACRO` marks a macro definition, an invocation, a derive from another crate, or an attribute macro.",
    "A type's or a module's doc comment prints as prose, and `backticked` text prints in the code font; a member's comment prints above it as `///` lines.",
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
        assert!(html.contains("<code>#[derive]</code>"));
        assert!(html.contains("<code>#[serde(rename = \"id\")]</code>"));
        assert!(!html.contains('`'));
    }
}
