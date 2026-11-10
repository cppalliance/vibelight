//! Tests how a facade's page shows a declaration: what a user of the
//! facade can name, under the name that user writes, and a type's private
//! data, dim.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use super::{
    Facade, Groups, Named, module_symbols, plain_named, public_refs, symbol_link, unused_by, usage,
};
use crate::html::document::Document;
use crate::html::layout::decl::public_block;
use crate::html::layout::items::item_block;
use crate::html::layout::{Block, Line, Span, line_text, span_text};
use crate::html::render_html::{ApiIds, Page};
use crate::model::{Crate, Reach, ValueRef};
use crate::module_graph::Users;

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

/// Returns the id of the declaration `name`.
fn decl(krate: &Crate, name: &str) -> usize {
    (krate.decls.iter())
        .position(|d| d.name == name)
        .expect("the declaration is collected")
}

/// Returns the public block of the declaration `name`, shown as `shown`,
/// with each dim run between `<` and `>`.
fn public(krate: &Crate, name: &str, shown: &str) -> Vec<String> {
    let block = public_block(krate, decl(krate, name), shown);
    (block.lines.iter())
        .map(|line| {
            let text: String = (line.spans.iter())
                .map(|span| match span {
                    Span::DimOpen => "<",
                    Span::DimClose => ">",
                    other => span_text(other, &krate.refs),
                })
                .collect();
            format!("{}{text}", " ".repeat(line.indent))
        })
        .collect()
}

#[test]
fn a_public_block_keeps_what_users_can_name_and_the_private_data_dim() {
    let (_tmp, krate) = krate(
        "pub struct Holder {\n    /// Counts the reads.\n    count: u32,\n    pub value: u8,\n    \
         #[doc(hidden)]\n    pub raw: u8,\n}\n\
         impl Holder {\n    pub fn get(&self) -> u8 { self.value }\n    fn bump(&mut self) {}\n    \
         #[doc(hidden)]\n    pub fn secret(&self) {}\n}\n",
    );
    assert_eq!(
        public(&krate, "Holder", "Holder"),
        [
            "struct Holder",
            "{",
            "    value: u8,",
            "",
            "private:",
            "    <count: u32,>",
            "    <raw: u8,>",
            "",
            "pub:",
            "    get() -> u8",
            "}",
        ],
        "the first member shown is public, so it is a struct; the data its users cannot name \
         follows under one label, with no comment and no blank line; a private function and \
         one hidden from the docs stay out"
    );
}

#[test]
fn a_struct_with_only_private_data_is_a_class_that_shows_it_with_no_label() {
    let (_tmp, krate) = krate(
        "pub struct Secret {\n    a: u8,\n    pub(crate) b: u16,\n}\n\
         impl Secret {\n    pub fn open(&self) -> u8 { self.a }\n}\n\
         pub struct Sealed {\n    a: u8,\n}\n",
    );
    assert_eq!(
        public(&krate, "Secret", "Secret"),
        [
            "class Secret",
            "{",
            "    <a: u8,>",
            "    <b: u16,>",
            "",
            "pub:",
            "    open() -> u8",
            "}",
        ],
        "a class starts private, and on this page `pub(crate)` is private too"
    );
    assert_eq!(
        public(&krate, "Sealed", "Sealed"),
        ["class Sealed", "{", "    <a: u8,>", "}"]
    );
}

#[test]
fn a_private_tuple_field_is_dim_in_the_header_and_the_first_field_decides_the_keyword() {
    let (_tmp, krate) = krate(
        "pub struct Id(i64);\n\
         impl Id {\n    pub fn get(self) -> i64 { self.0 }\n}\n\
         pub struct Pair(u16, pub u8);\n\
         pub struct Open(pub u8, u16);\n\
         pub struct Empty();\n",
    );
    assert_eq!(
        public(&krate, "Id", "Id"),
        ["class Id(<i64>)", "{", "pub:", "    get(self) -> i64", "}"]
    );
    assert_eq!(
        public(&krate, "Pair", "Pair"),
        ["class Pair(<u16>, pub u8)"]
    );
    assert_eq!(
        public(&krate, "Open", "Open"),
        ["struct Open(pub u8, <u16>)"]
    );
    assert_eq!(public(&krate, "Empty", "Empty"), ["struct Empty()"]);
}

#[test]
fn a_type_in_private_data_links_only_to_a_heading_of_the_page_and_counts_for_nothing() {
    let (_tmp, krate) = krate(
        "pub struct Inner;\npub struct Holder {\n    inner: Inner,\n    pub open: Inner,\n}\n",
    );
    let block = public_block(&krate, decl(&krate, "Holder"), "Holder");
    let line = |text: &str| {
        (block.lines.iter())
            .find(|line| line_text(line, &krate.refs) == text)
            .expect("the block has the line")
    };
    let (private, open) = (line("inner: Inner,"), line("open: Inner,"));
    assert_eq!(private.spans.first(), Some(&Span::DimOpen));
    assert_eq!(
        public_refs(private),
        Vec::<usize>::new(),
        "no user writes it"
    );
    assert_eq!(public_refs(open).len(), 1, "a user writes a public field");
    let doc = Document {
        title: String::new(),
        id: String::new(),
        crate_docs: None,
        contents: Vec::new(),
        kill_summary: String::new(),
        kill: Vec::new(),
        sections: Vec::new(),
        refs: krate.refs.clone(),
        decl_ids: vec!["type.t.Inner".to_owned(); krate.decls.len()],
        value_ids: BTreeMap::new(),
    };
    let render = |api: &ApiIds| {
        let page = Page {
            doc: &doc,
            member_links: None,
            run: &[],
            foreign: None,
            ids: false,
            api: Some((api, 0)),
        };
        let mut out = String::new();
        page.line(&mut out, private);
        out
    };
    assert_eq!(
        render(&ApiIds::new()),
        "    <span class=\"dim\"><span class=\"src\">inner</span>: Inner,</span>",
        "the page does not show Inner, so the name is plain text though its crate's page has it"
    );
    let shown = ApiIds::from([(
        (0, Reach::Decl(decl(&krate, "Inner"))),
        "api.Inner".to_owned(),
    )]);
    assert_eq!(
        render(&shown),
        "    <span class=\"dim\"><span class=\"src\">inner</span>: \
         <a href=\"#api.Inner\">Inner</a>,</span>"
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

/// Returns the sidebar's API entries of a crate `runvil` for `counts`, each
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
        "<li><a href=\"#api\">API</a></li>\n\
         <li><a href=\"#api.mod\">runvil</a> <span class=\"dim\">18</span>\n<ul>\n\
         <li><a href=\"#api.mod.effect\">effect</a> <span class=\"dim\">3</span></li>\n\
         <li><a href=\"#api.mod.vfs.path\">vfs::path</a> <span class=\"dim\">2</span></li>\n\
         </ul>\n</li>\n",
        "a module at the crate's level would read as `runvil::runvil`, and the crate is at \
         the level of `API`, which holds nothing else"
    );
}

#[test]
fn a_crate_root_with_no_module_below_it_is_one_line() {
    assert_eq!(
        tree(&[("", 5)]),
        "<li><a href=\"#api\">API</a></li>\n\
         <li><a href=\"#api.mod\">runvil</a> <span class=\"dim\">5</span></li>\n"
    );
    assert_eq!(tree(&[]), "<li><a href=\"#api\">API</a></li>\n");
}

#[test]
fn a_crate_root_that_holds_no_item_is_a_plain_name_above_its_modules() {
    assert_eq!(
        tree(&[("effect", 3)]),
        "<li><a href=\"#api\">API</a></li>\n\
         <li>runvil\n<ul>\n\
         <li><a href=\"#api.mod.effect\">effect</a> <span class=\"dim\">3</span></li>\n\
         </ul>\n</li>\n",
        "the page has no section for a root with no item, so there is nothing to link to"
    );
}

/// Returns an item of the API with the heading id `id`, which declaration
/// `decl` of target 0 defines when given.
fn named(name: &str, suffix: &'static str, id: &str, decl: Option<usize>) -> Named {
    let mut named = plain_named(name, suffix, 0, Line::default());
    id.clone_into(&mut named.id);
    named.item = decl.map(Reach::Decl);
    named
}

/// Returns the sidebar's list of symbols by module of a crate `runvil`,
/// when no item names another but declaration 0.
fn by_module(groups: Vec<(&[&str], Vec<Named>)>) -> String {
    let groups: Groups = (groups.into_iter())
        .map(|(path, items)| {
            let path = path.iter().map(|&name| name.to_owned()).collect();
            (path, (None, items))
        })
        .collect();
    let used = BTreeSet::from([(0, Reach::Decl(0))]);
    let mut out = String::new();
    module_symbols(&mut out, "runvil", &groups, &used);
    out
}

#[test]
fn the_sidebar_lists_the_symbols_of_each_module_under_the_module() {
    let groups: Vec<(&[&str], Vec<Named>)> = vec![
        (
            &[],
            vec![
                named("Error", "", "api.Error", Some(0)),
                named("make", "()", "fn.runvil.make", None),
            ],
        ),
        (
            &["effect"],
            vec![named("Round", "", "api.effect.Round", Some(1))],
        ),
        (
            &["vfs", "path"],
            vec![named("join", "!", "api.vfs.path.join", None)],
        ),
    ];
    assert_eq!(
        by_module(groups),
        "<hr>\n<ul class=\"modules\">\n\
         <li><a href=\"#api.mod\">runvil</a>\n<ul>\n\
         <li><a href=\"#api.Error\">Error</a></li>\n\
         <li><a href=\"#fn.runvil.make\">make()</a></li>\n\
         </ul>\n</li>\n\
         <li><a href=\"#api.mod.effect\">effect</a>\n<ul>\n\
         <li><a href=\"#api.effect.Round\"><b>Round</b></a></li>\n\
         </ul>\n</li>\n\
         <li><a href=\"#api.mod.vfs.path\">vfs::path</a>\n<ul>\n\
         <li><a href=\"#api.vfs.path.join\">join!</a></li>\n\
         </ul>\n</li>\n\
         </ul>\n",
        "the crate is a top-level entry like each module, and a type that nothing names is bold"
    );
}

#[test]
fn an_api_in_one_module_has_no_list_by_module() {
    let root: Vec<(&[&str], Vec<Named>)> =
        vec![(&[], vec![named("Error", "", "api.Error", Some(0))])];
    assert_eq!(
        by_module(root),
        "",
        "it would repeat the list of every symbol"
    );
    assert_eq!(by_module(Vec::new()), "");
}

#[test]
fn a_user_of_the_facade_that_does_not_name_an_item_counts_against_it() {
    let users = Users::from([
        ((0, Reach::Decl(0)), BTreeSet::from([1, 3])),
        ((0, Reach::Decl(1)), BTreeSet::from([3])),
    ]);
    let facade = |used_by: Vec<usize>| Facade {
        index: 9,
        owners: &[],
        surface: &[],
        run: &[],
        used_by,
        users: &users,
    };
    let both = facade(vec![1, 2]);
    assert_eq!(
        unused_by(&both, 0, Reach::Decl(0)),
        Some((1, 2)),
        "target 1 names it and target 2 does not"
    );
    assert_eq!(
        unused_by(&both, 0, Reach::Decl(1)),
        Some((2, 2)),
        "target 3 names it, and target 3 does not use the facade"
    );
    assert_eq!(
        unused_by(&both, 0, Reach::Decl(2)),
        Some((2, 2)),
        "no target names it"
    );
    assert_eq!(
        unused_by(&facade(Vec::new()), 0, Reach::Decl(0)),
        None,
        "no target uses the facade, so there is nothing to count"
    );
}

#[test]
fn an_item_ends_its_line_with_the_users_that_do_not_use_it_and_is_dim_when_none_does() {
    let with = |unused: Option<(usize, usize)>| {
        let mut item = named("Round", "", "api.Round", Some(1));
        item.unused = unused;
        item
    };
    let used = BTreeSet::new();
    let link = |unused| symbol_link(&with(unused), &used);
    assert_eq!(with(Some((0, 4))).unused_note(), "", "each user uses it");
    assert_eq!(
        with(Some((3, 4))).unused_note(),
        " &middot; not used by 3 of 4"
    );
    assert_eq!(with(None).unused_note(), "", "the facade has no user");
    assert_eq!(
        link(Some((4, 4))),
        "<a class=\"dim\" href=\"#api.Round\"><b>Round</b></a>",
        "dim, since none of the four uses it, and bold, since no item names it"
    );
    assert_eq!(
        link(Some((3, 4))),
        "<a href=\"#api.Round\"><b>Round</b></a>"
    );
    assert_eq!(link(None), "<a href=\"#api.Round\"><b>Round</b></a>");
}

#[test]
fn the_opening_sentence_counts_the_users_and_the_items_none_of_them_uses() {
    assert_eq!(usage("runvil", 0, 0), "", "no crate of the run uses it");
    assert_eq!(usage("runvil", 4, 0), " 4 crates here use runvil.");
    assert_eq!(
        usage("runvil", 4, 56),
        " 4 crates here use runvil, and none of them uses 56 of the items."
    );
    assert_eq!(
        usage("relay", 1, 1),
        " 1 crate here uses relay, and it does not use 1 of the items."
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
