//! Tests how a facade's page shows a declaration: only what a user of the
//! facade can name, under the name that user writes.

use std::fs;

use crate::html::layout::decl::public_block;
use crate::html::layout::items::item_block;
use crate::html::layout::{Block, line_text};
use crate::model::{Crate, ValueRef};

/// Collects and resolves a one-file crate.
fn krate(source: &str) -> (tempfile::TempDir, Crate) {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let src = tmp.path().join("src");
    fs::create_dir_all(&src).expect("a source folder can be made");
    fs::write(src.join("lib.rs"), source).expect("a source can be written");
    let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").expect("the crate walks");
    let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
    crate::resolve::resolve(&mut krate);
    crate::attach::attach(&mut krate);
    (tmp, krate)
}

fn text(krate: &Crate, block: &Block) -> Vec<String> {
    (block.lines.iter())
        .map(|l| format!("{}{}", " ".repeat(l.indent), line_text(l, &krate.refs)))
        .collect()
}

/// Returns the public block of the declaration `name`, shown as `shown`.
fn public(krate: &Crate, name: &str, shown: &str) -> Vec<String> {
    let id = (krate.decls.iter())
        .position(|d| d.name == name)
        .expect("the declaration is collected");
    text(krate, &public_block(krate, id, shown))
}

#[test]
fn a_public_block_keeps_only_what_users_can_name() {
    let (_tmp, krate) = krate(
        "pub struct Holder {\n    pub value: u8,\n    count: u32,\n}\n\
         impl Holder {\n    pub fn get(&self) -> u8 { self.value }\n    fn bump(&mut self) {}\n    \
         #[doc(hidden)]\n    pub fn secret(&self) {}\n}\n",
    );
    assert_eq!(
        public(&krate, "Holder", "Holder"),
        [
            "struct Holder",
            "{",
            "pub:",
            "    value: u8,",
            "",
            "pub:",
            "    get() -> u8",
            "}",
        ]
    );
}

#[test]
fn a_struct_with_no_public_member_shows_an_empty_body() {
    let (_tmp, krate) = krate("pub struct Secret {\n    a: u8,\n}\n");
    assert_eq!(
        public(&krate, "Secret", "Secret"),
        ["struct Secret", "{", "}"]
    );
}

#[test]
fn a_hidden_tuple_field_keeps_its_place() {
    let (_tmp, krate) = krate("pub struct Pair(pub u8, u16);\n");
    assert_eq!(
        public(&krate, "Pair", "Pair"),
        ["struct Pair(pub u8, /* private field */)"]
    );
}

#[test]
fn a_trait_keeps_its_items_which_have_no_visibility_of_their_own() {
    let (_tmp, krate) = krate("pub trait Speak {\n    fn say(&self);\n}\n");
    assert_eq!(
        public(&krate, "Speak", "Speak"),
        ["trait Speak", "{", "    say()", "}"]
    );
}

#[test]
fn a_header_leaves_out_pub_and_keeps_any_other_visibility() {
    let (_tmp, krate) = krate(
        "pub enum Mode {\n    On,\n}\n\
         pub union Bits {\n    pub word: u32,\n}\n\
         pub unsafe trait Raw {}\n\
         pub type Id = u32;\n\
         pub(crate) struct Inner;\n",
    );
    assert_eq!(
        public(&krate, "Mode", "Mode"),
        ["enum Mode", "{", "    On,", "}"]
    );
    assert_eq!(
        public(&krate, "Bits", "Bits"),
        ["union Bits", "{", "pub:", "    word: u32,", "}"]
    );
    assert_eq!(public(&krate, "Raw", "Raw"), ["unsafe trait Raw", "{", "}"]);
    assert_eq!(public(&krate, "Id", "Id"), ["using Id = u32"]);
    assert_eq!(
        public(&krate, "Inner", "Inner"),
        ["pub(crate) struct Inner"],
        "only `pub` goes: a narrower visibility says something"
    );
}

/// Returns the sidebar's API entry of a crate `runvil` for `counts`, each
/// a module path with `.` between its names, empty for the crate root.
fn tree(counts: &[(&str, usize)]) -> String {
    let paths: Vec<Vec<String>> = (counts.iter())
        .map(|(path, _)| {
            path.split('.')
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .collect();
    let counts: Vec<(&[String], usize)> = (paths.iter().zip(counts))
        .map(|(path, (_, count))| (path.as_slice(), *count))
        .collect();
    let mut out = String::new();
    super::api_tree(&mut out, "runvil", &counts);
    out
}

#[test]
fn the_sidebar_puts_the_modules_of_the_api_below_the_crate_root() {
    assert_eq!(
        tree(&[("", 18), ("effect", 3), ("vfs.path", 2)]),
        "<li><a href=\"#api\">API</a>\n<ul>\n\
         <li><a href=\"#api.mod\">runvil</a> <span class=\"dim\">18</span>\n<ul>\n\
         <li><a href=\"#api.mod.effect\">effect</a> <span class=\"dim\">3</span></li>\n\
         <li><a href=\"#api.mod.vfs.path\">vfs::path</a> <span class=\"dim\">2</span></li>\n\
         </ul>\n</li>\n\
         </ul>\n</li>\n",
        "a module at the crate's level would read as `runvil::runvil`"
    );
}

#[test]
fn a_crate_root_with_no_module_below_it_is_one_line() {
    assert_eq!(
        tree(&[("", 5)]),
        "<li><a href=\"#api\">API</a>\n<ul>\n\
         <li><a href=\"#api.mod\">runvil</a> <span class=\"dim\">5</span></li>\n\
         </ul>\n</li>\n"
    );
    assert_eq!(
        tree(&[]),
        "<li><a href=\"#api\">API</a>\n<ul>\n</ul>\n</li>\n"
    );
}

#[test]
fn a_crate_root_that_holds_no_item_is_a_plain_name_above_its_modules() {
    assert_eq!(
        tree(&[("effect", 3)]),
        "<li><a href=\"#api\">API</a>\n<ul>\n\
         <li>runvil\n<ul>\n\
         <li><a href=\"#api.mod.effect\">effect</a> <span class=\"dim\">3</span></li>\n\
         </ul>\n</li>\n\
         </ul>\n</li>\n",
        "the page has no section for a root with no item, so there is nothing to link to"
    );
}

#[test]
fn a_renamed_declaration_and_value_show_the_name_users_write() {
    let (_tmp, krate) = krate("pub struct Inner;\npub fn make() -> u8 {\n    0\n}\n");
    assert_eq!(public(&krate, "Inner", "Outer"), ["struct Outer"]);
    let item = (krate.modules[0].items.iter())
        .position(|item| item.name == "make")
        .expect("the function is collected");
    let at = ValueRef { module: 0, item };
    assert_eq!(
        text(&krate, &item_block(&krate, at, "build")),
        ["build() -> u8"]
    );
}
