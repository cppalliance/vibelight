//! The legend of the HTML: what the overview and the pages' notation mean,
//! so a reader of the HTML alone can follow it. The overview prints it once.
//! It opens with a sample: one type with a member of every visibility and
//! of every kind, laid out by the code that lays out every page.

use std::collections::BTreeMap;
use std::path::Path;

use crate::html::document::Document;
use crate::html::layout::decl::decl_block;
use crate::html::render_html::{Page, escape};

/// The Rust source of the legend's sample: the type `Sample`, which has a
/// field and a function of every visibility, and a member of every kind the
/// notation tells apart. It sits two modules deep so that `pub(super)` and
/// `pub(in crate::shapes)` name two places.
const SAMPLE: &str = "\
pub mod shapes {
    pub mod solid {
        #[derive(Clone, Debug, Default)]
        pub struct Sample {
            pub name: String,
            pub(crate) count: u32,
            pub(in crate::shapes) scope: u8,
            pub(super) parent: u8,
            secret: u64,
        }

        impl From<u32> for Sample {
            fn from(count: u32) -> Self {
                let mut sample = Self::default();
                sample.count = count;
                sample
            }
        }

        impl Drop for Sample {
            fn drop(&mut self) {}
        }

        impl Sample {
            pub const LIMIT: usize = 8;

            pub fn new(name: &str) -> Self {
                let mut sample = Self::default();
                sample.name = name.to_owned();
                sample
            }

            #[must_use]
            pub const fn count(&self) -> u32 {
                self.count
            }

            pub fn name(&self) -> &str {
                &self.name
            }

            pub fn rename(&mut self, name: String) {
                self.name = name;
            }

            pub fn total(self) -> u64 {
                self.secret
            }

            pub(crate) async fn load(path: &str) -> Option<Self> {
                (!path.is_empty()).then(Self::default)
            }

            fn check(&self) -> bool {
                self.secret > 0
            }
        }
    }
}
";

/// The lines of the legend, in order; backticks mark code.
const LINES: &[&str] = &[
    "The sample above is one type with a member of each visibility and of each kind, laid out as a crate's page lays out every type.",
    "A facade is a crate whose public surface is mostly re-exports of other crates here: the API that users of a crate family write against. Its implementation crates are its family: the crates its code reaches, short of other facades. A warning marks a crate outside a family that uses one of its implementation crates directly, past the facade.",
    "A facade's page shows what its users can name: each item under the path they write, with its `pub` members, pulled from the crate that defines it, under a dim line that says where that crate writes it. A type there also shows its private data, dim and with no comments, so a reader can see what a value holds. A type's header there leaves out `pub`, since every item of the page is public.",
    "A facade's sidebar lists its symbols twice: under each module that a user names them through, then all of them by name. A bold type there is one that no other item of its API names in its code: usually where reading starts. `Named, not re-exported`, at the end of its page, lists what that code names and the facade does not re-export, under the crate a user names each through.",
    "`2 of 4 use it`, after where a facade's item is defined, says that 2 of the 4 crates here that use the facade use the item. `unused` there says that none of them does, and the line says nothing when all of them do. A crate uses an item when its production code writes the item's name or a member's. It then also uses each item that the code of that item names, and so on: the type of a public field goes with the type that holds it. The sidebar's list by module prints the name of an unused item dim. Names are counted and calls are not, so the count is a place to look, not proof.",
    "Each module has a section: its path, its anatomy line of what it declares and by which visibility, its types, then its items. The sidebar opens with the module tree, each module with the number of symbols it declares. Under a rule, each module follows with its symbols by name. A bold name there is on the public surface.",
    "Under a module's anatomy line, `Used by` names the modules that use its items and `Uses` the modules whose items it uses, each with the number of items used. `Instability` is the number of modules it uses, divided by that plus the number of modules that use it: near 0 others lean on it, near 1 it leans on others.",
    "In `Layers`, a use counts for the crate that declares the item when the using crate depends on it, else for the first of its dependencies in the list that re-exports the item by name, else for the crate that declares it. Only production code counts, and doc links do not.",
    "A type prints as a C++-style declaration with its visibility as written, such as `pub(crate) struct`, with the members of all its impls merged in. A struct reads `struct` when the first member shown is `pub`, and `class` when it is not.",
    "`static` marks a function with no receiver; `() mut` marks one that takes `&mut self`, and a method with no marker takes `&self`.",
    "`: Clone, Debug` is the base list: the standard traits a type derives or implements, by short name. A comment written above an impl prints over its trait there, and the comment of an inherent impl opens the type's body.",
    "A type lists first what has no visibility of its own: an enum's variants, a trait's items, and its macro marks. Then come its fields, then its other members, each set grouped by visibility, the widest first. A label such as `pub:`, `pub(crate):`, or `pub(self):` for no keyword opens a group where the visibility changes. A `struct` starts at `pub` and a `class` at `pub(self)`, so neither prints that label first. On a facade's page `private:` stands for everything its users cannot name.",
    "A module's items are grouped under the same labels, each one printed, after its macro marks and its extern blocks.",
    "A dim `reaches pub(crate)` line says that a node reaches less far than its visibility gives, because a module around it is narrower; a member that reaches as far as its type leaves it unsaid.",
    "`[[...]]` is an attribute that shapes the API or its wire format, such as `[[must_use]]` or `[[serde(rename = \"id\")]]`.",
    "`implements: Trait for Type` is an impl with no declared type to attach to, such as a blanket impl.",
    "`MACRO` marks a macro definition, an invocation, a derive from another crate, or an attribute macro.",
    "A doc comment prints as prose, and `backticked` text prints in the code font; a member's comment reflows as one paragraph.",
    "A gold link goes to another crate's page. `Ctrl K` searches every declaration, a click with `Ctrl` opens a link in a new tab, and a click on a source link copies its file and line.",
];

/// Returns the legend's sample as the block that a crate's page prints for
/// it. The source is walked, collected, and laid out by the code that lays
/// out every page, so the sample shows what the pages show and nothing
/// else. No name in it links anywhere.
fn sample() -> String {
    let dir = Path::new("");
    let tree = crate::modtree::walk_text(dir, "lib.rs", SAMPLE);
    let mut krate = crate::collect::collect(tree, dir, "legend", "legend");
    crate::resolve::resolve(&mut krate);
    crate::attach::attach(&mut krate);
    let Some(id) = krate.decls.iter().position(|decl| decl.name == "Sample") else {
        return String::new();
    };
    let block = decl_block(&krate, id);
    let doc = Document {
        title: String::new(),
        id: String::new(),
        crate_docs: None,
        contents: Vec::new(),
        symbols: Vec::new(),
        kill_summary: String::new(),
        kill: Vec::new(),
        sections: Vec::new(),
        decl_ids: vec![String::new(); krate.decls.len()],
        refs: krate.refs,
        value_ids: BTreeMap::new(),
    };
    let page = Page {
        doc: &doc,
        member_links: None,
        run: &[],
        foreign: None,
        ids: false,
        api: None,
    };
    let mut out = String::new();
    page.pre(&mut out, &block);
    out
}

/// Returns the legend: its sample, then its lines as an HTML list.
pub(crate) fn html() -> String {
    let mut out = String::from("<div class=\"legend\">\n");
    out.push_str(&sample());
    out.push_str("<ul>\n");
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
    out.push_str("</ul>\n</div>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the sample's lines as a reader sees them: the tags left out
    /// and the escapes undone, a blank line for each gap between members.
    fn sample_lines() -> Vec<String> {
        let mut text = String::new();
        let html = sample().replace("<span class=\"gap\"></span>", "\n");
        let mut rest = html.as_str();
        while let Some(at) = rest.find('<') {
            text.push_str(&rest[..at]);
            let end = rest[at..].find('>').expect("a tag closes");
            rest = &rest[at + end + 1..];
        }
        text.push_str(rest);
        let text = text
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        text.trim_end().lines().map(str::to_owned).collect()
    }

    #[test]
    fn the_sample_has_a_member_of_every_visibility_and_kind_as_a_page_lays_it_out() {
        assert_eq!(
            sample_lines(),
            [
                "pub struct Sample : Clone, Debug",
                "{",
                "    name: String,",
                "",
                "pub(crate):",
                "    count: u32,",
                "",
                "pub(in crate::shapes):",
                "    scope: u8,",
                "",
                "pub(super):",
                "    parent: u8,",
                "",
                "pub(self):",
                "    secret: u64,",
                "",
                "pub:",
                "    static constexpr LIMIT: usize = 8",
                "",
                "    Sample() = default",
                "    Sample(u32&)",
                "    ~Sample()",
                "",
                "    static new(name: str&) -> Self",
                "",
                "    constexpr count() -> u32 [[must_use]]",
                "",
                "    name() -> str&",
                "",
                "    rename(name: String) mut",
                "",
                "    total(self) -> u64",
                "",
                "pub(crate):",
                "    static async load(path: str&) -> Option<Self>",
                "",
                "pub(self):",
                "    check() -> bool",
                "}",
            ],
            "the first member shown is public, so it is a struct and its first label is left \
             out; then each visibility, the widest first, for the data and again for the rest"
        );
    }

    #[test]
    fn the_legend_is_one_box_with_the_sample_above_its_lines() {
        let html = html();
        assert!(
            html.starts_with(
                "<div class=\"legend\">\n<pre><code><span class=\"k\">pub</span> \
                 <span class=\"k\">struct</span> Sample : Clone, Debug\n{\n"
            ),
            "{html}"
        );
        assert!(html.contains("}</code></pre>\n<ul>\n<li>The sample above is "));
        assert!(html.ends_with("</li>\n</ul>\n</div>\n"));
        assert!(
            !html.contains("href="),
            "no name of the sample links anywhere"
        );
        assert!(!html.contains(" id="), "and none takes an id");
    }

    #[test]
    fn backticks_turn_into_code_and_text_is_escaped() {
        let html = html();
        assert!(html.contains("<code>[[must_use]]</code>"));
        assert!(html.contains("<code>[[serde(rename = \"id\")]]</code>"));
        assert!(html.contains("<code>() mut</code>"));
        assert!(!html.contains('`'));
    }
}
