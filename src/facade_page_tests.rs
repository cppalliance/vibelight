//! Tests how a facade's page shows a declaration: only what a user of the
//! facade can name, under the name that user writes.

use std::fs;

use crate::layout::{Block, line_text};
use crate::layout_decl::public_block;
use crate::layout_items::item_block;
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
fn a_public_block_keeps_only_what_users_can_name_and_says_fields_are_hidden() {
    let (_tmp, krate) = krate(
        "pub struct Holder {\n    pub value: u8,\n    count: u32,\n}\n\
         impl Holder {\n    pub fn get(&self) -> u8 { self.value }\n    fn bump(&mut self) {}\n    \
         #[doc(hidden)]\n    pub fn secret(&self) {}\n}\n",
    );
    assert_eq!(
        public(&krate, "Holder", "Holder"),
        [
            "pub struct Holder",
            "{",
            "    /* private fields */",
            "",
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
fn a_struct_with_every_field_public_says_nothing_is_hidden() {
    let (_tmp, krate) = krate("pub struct Open {\n    pub a: u8,\n}\n");
    let lines = public(&krate, "Open", "Open");
    assert!(!lines.iter().any(|l| l.contains("private")), "{lines:#?}");
}

#[test]
fn a_hidden_tuple_field_keeps_its_place() {
    let (_tmp, krate) = krate("pub struct Pair(pub u8, u16);\n");
    assert_eq!(
        public(&krate, "Pair", "Pair"),
        ["pub struct Pair(pub u8, /* private field */)"]
    );
}

#[test]
fn a_trait_keeps_its_items_which_have_no_visibility_of_their_own() {
    let (_tmp, krate) = krate("pub trait Speak {\n    fn say(&self);\n}\n");
    assert_eq!(
        public(&krate, "Speak", "Speak"),
        ["pub trait Speak", "{", "    say()", "}"]
    );
}

#[test]
fn a_renamed_declaration_and_value_show_the_name_users_write() {
    let (_tmp, krate) = krate("pub struct Inner;\npub fn make() -> u8 {\n    0\n}\n");
    assert_eq!(public(&krate, "Inner", "Outer"), ["pub struct Outer"]);
    let item = (krate.modules[0].items.iter())
        .position(|item| item.name == "make")
        .expect("the function is collected");
    let at = ValueRef { module: 0, item };
    assert_eq!(
        text(&krate, &item_block(&krate, at, "build")),
        ["build() -> u8"]
    );
}
