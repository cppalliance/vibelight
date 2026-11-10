//! Lays out a declaration's head: the keyword line with any tuple fields,
//! the base list, and the `where` clause.

use crate::html::layout::marker::Marker;
use crate::html::layout::members::param_lines;
use crate::html::layout::{Line, Span, code, note_lines, plain, vis_spans, where_parts, width};
use crate::html::wrap::{self, MAX_WIDTH};
use crate::model::{Base, Decl, Shape, TypeRef, name_order};

/// The column a base's name starts on, after the `: ` or `, ` that opens
/// its line, which the comment of its impl starts on too.
const BASE_NAME_INDENT: usize = 6;

fn line(indent: usize, spans: Vec<Span>) -> Line {
    Line {
        indent,
        spans,
        trailing: None,
    }
}

/// A base that takes a line of its own: its entry, and the comment lines of
/// its impl, which go above it.
struct Own {
    spans: Vec<Span>,
    notes: Vec<Line>,
}

/// Finishes a header with any tuple fields, the base list, and the `where`
/// clause; a type with no body ends in `;`. The base list stays on the head
/// line while it holds only short entries, no impl of theirs has a comment,
/// and the line fits. Otherwise each base line starts with `: ` or `, `:
/// the short names first, wrapping to fit, then each base whose impl has a
/// comment, under that comment, then one long path per line, then the
/// `where` clause on its own line.
pub(crate) fn header_lines(
    decl: &Decl,
    header: Vec<Span>,
    has_body: bool,
    refs: &[TypeRef],
) -> Vec<Line> {
    let mut lines = head_lines(decl, header);
    let (short, own) = lists(decl, refs);
    let where_clause = decl.where_clause.as_ref().map(where_parts);
    let last = lines.pop().unwrap_or_default();
    let mut one = last.spans.clone();
    if !short.is_empty() {
        one.push(plain(" : "));
        for (i, entry) in short.iter().enumerate() {
            if i > 0 {
                one.push(plain(", "));
            }
            one.extend(entry.iter().cloned());
        }
    }
    if own.is_empty() && width(last.indent, &one, refs) <= MAX_WIDTH {
        match &where_clause {
            Some(clause) if has_body && last.indent == 0 => {
                lines.extend(wrap::header(one, Some(clause), refs));
            }
            Some(clause) if has_body => {
                lines.push(line(last.indent, one));
                lines.extend(wrap::where_lines(clause, &[], 4, refs));
            }
            Some(clause) => {
                one.push(plain(" "));
                one.extend(clause.one.iter().cloned());
                lines.push(line(last.indent, one));
            }
            None => lines.push(line(last.indent, one)),
        }
    } else {
        lines.push(last);
        lines.extend(comma_lines(&short, own, refs));
        if let Some(clause) = &where_clause {
            lines.extend(wrap::where_lines(clause, &[], 4, refs));
        }
    }
    if !has_body {
        if let Some(end) = lines.last_mut() {
            end.trailing.clone_from(&decl.meta.trailing);
        }
    }
    lines
}

/// Returns the head without its base list: the header line, with a tuple
/// struct's fields on it, or one field per line when any field has a
/// comment.
fn head_lines(decl: &Decl, mut header: Vec<Span>) -> Vec<Line> {
    let Shape::Tuple(fields) = &decl.shape else {
        return vec![line(0, header)];
    };
    header.push(plain("("));
    if fields.iter().any(|f| !f.meta.notes.is_empty()) {
        let mut lines = vec![line(0, header)];
        let mut fields = param_lines(fields, 4);
        let mut last = fields.pop().unwrap_or_default();
        last.spans.push(plain(")"));
        lines.extend(fields);
        lines.push(last);
        return lines;
    }
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            header.push(plain(", "));
        }
        header.extend(vis_spans(&field.meta.vis));
        header.extend(code(&field.text));
    }
    header.push(plain(")"));
    vec![line(0, header)]
}

/// Returns the bases in print order: the short entries that share lines,
/// sorted by name, and then the entries that each take a line, the short
/// ones whose impl has a comment, sorted by name, before the long ones,
/// sorted by path. A trait's bases are its supertraits, which no impl
/// writes, so none has a comment.
fn lists(decl: &Decl, refs: &[TypeRef]) -> (Vec<Vec<Span>>, Vec<Own>) {
    let (bases, of_impls) = match &decl.shape {
        Shape::Trait { supertraits, .. } => (supertraits.as_slice(), false),
        _ => (decl.bases.as_slice(), true),
    };
    let comment = |index: usize| -> Vec<Line> {
        (decl.impl_notes.iter())
            .filter(|group| of_impls && group.base == Some(index))
            .flat_map(|group| note_lines(&group.notes, BASE_NAME_INDENT, Marker::DOC))
            .collect()
    };
    let by_name = |a: &&Base, b: &&Base| {
        name_order(&a.name, &b.name).then_with(|| a.text.visible(refs).cmp(&b.text.visible(refs)))
    };
    let by_path = |a: &&Base, b: &&Base| name_order(&a.text.visible(refs), &b.text.visible(refs));
    let mut short = Vec::new();
    let mut marked = Vec::new();
    let mut long = Vec::new();
    for (index, base) in bases.iter().enumerate() {
        let lines = comment(index);
        match (base.long, lines.is_empty()) {
            (false, true) => short.push(base),
            (false, false) => marked.push((base, lines)),
            (true, _) => long.push((base, lines)),
        }
    }
    short.sort_by(by_name);
    marked.sort_by(|a, b| by_name(&a.0, &b.0));
    long.sort_by(|a, b| by_path(&a.0, &b.0));
    let own = (marked.into_iter().chain(long))
        .map(|(base, notes)| Own {
            spans: code(&base.text),
            notes,
        })
        .collect();
    (short.into_iter().map(|b| code(&b.text)).collect(), own)
}

/// Returns the base lines of the leading-comma form.
fn comma_lines(short: &[Vec<Span>], own: Vec<Own>, refs: &[TypeRef]) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut current: Vec<Span> = Vec::new();
    for entry in short {
        if current.is_empty() {
            current.push(plain(": "));
        } else {
            let mut longer = current.clone();
            longer.push(plain(", "));
            longer.extend(entry.iter().cloned());
            if width(4, &longer, refs) <= MAX_WIDTH {
                current = longer;
                continue;
            }
            lines.push(line(4, std::mem::take(&mut current)));
            current.push(plain(", "));
        }
        current.extend(entry.iter().cloned());
    }
    if !current.is_empty() {
        lines.push(line(4, current));
    }
    let mut first = lines.is_empty();
    for entry in own {
        lines.extend(entry.notes);
        let mut spans = vec![plain(if first { ": " } else { ", " })];
        spans.extend(entry.spans);
        lines.push(line(4, spans));
        first = false;
    }
    lines
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::html::layout::line_text;

    /// Returns the lines of the declaration `name` in a one-file crate.
    fn block(source: &str, name: &str) -> Vec<String> {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), source).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        crate::resolve::resolve(&mut krate);
        crate::attach::attach(&mut krate);
        let id = krate.decls.iter().position(|d| d.name == name).unwrap();
        let block = crate::html::layout::decl::decl_block(&krate, id);
        (block.lines.iter())
            .map(|l| format!("{}{}", " ".repeat(l.indent), line_text(l, &krate.refs)))
            .collect()
    }

    /// Returns the head of the declaration `name` in a one-file crate: its
    /// lines before the opening brace.
    fn head(source: &str, name: &str) -> Vec<String> {
        let lines = block(source, name);
        lines.into_iter().take_while(|l| l != "{").collect()
    }

    #[test]
    fn standard_traits_that_fit_stay_on_the_head_line() {
        let source = "pub struct Run;\n\
                      impl std::fmt::Display for Run {}\n\
                      impl std::fmt::Debug for Run {}\n";
        assert_eq!(head(source, "Run"), ["pub struct Run : Debug, Display"]);
    }

    #[test]
    fn a_long_entry_puts_each_base_on_its_own_line_before_the_where_clause() {
        let source = "pub trait Named {}\n\
                      pub struct Holder<T> where T: Copy { pub value: T }\n\
                      impl<T> Named for Holder<T> where T: Copy {}\n\
                      impl<T: Copy> std::fmt::Debug for Holder<T> {}\n";
        assert_eq!(
            head(source, "Holder"),
            [
                "pub struct Holder<T>",
                "    : Debug",
                "    , Named",
                "    where T: Copy"
            ]
        );
    }

    #[test]
    fn a_base_whose_impl_has_a_comment_takes_its_own_line_under_the_comment() {
        let source = "pub struct Run;\n\
                      impl std::fmt::Display for Run {}\n\
                      /// Shows the run\n/// for a log.\n\
                      impl std::fmt::Debug for Run {}\n\
                      impl std::fmt::Binary for Run {}\n";
        assert_eq!(
            block(source, "Run"),
            [
                "pub struct Run",
                "    : Binary, Display",
                "      /// Shows the run",
                "      /// for a log.",
                "    , Debug",
            ]
        );
    }

    #[test]
    fn the_first_base_opens_the_list_whether_or_not_it_has_a_comment() {
        let source = "pub trait Named {}\n\
                      pub struct Run;\n\
                      // Shows the run.\n\
                      impl std::fmt::Debug for Run {}\n\
                      // Names the run.\n\
                      impl Named for Run {}\n";
        assert_eq!(
            head(source, "Run"),
            [
                "pub struct Run",
                "      // Shows the run.",
                "    : Debug",
                "      // Names the run.",
                "    , Named",
            ]
        );
    }

    #[test]
    fn the_comment_of_an_inherent_impl_opens_the_body() {
        let source = "pub struct Run;\n\
                      // Starting.\n\
                      impl Run {\n    pub fn start(&self) {}\n}\n\
                      // Stopping.\n\
                      impl Run {\n    pub fn halt(&self) {}\n}\n";
        assert_eq!(
            block(source, "Run"),
            [
                "pub struct Run",
                "{",
                "    // Stopping.",
                "",
                "    // Starting.",
                "",
                "    halt()",
                "",
                "    start()",
                "}",
            ]
        );
    }

    #[test]
    fn a_struct_whose_first_member_shown_is_pub_stays_a_struct_and_drops_that_label() {
        let source = "pub struct Point {\n    hidden: u8,\n    pub x: i32,\n}\n\
                      impl Point {\n    fn check(&self) {}\n    pub fn new() -> Self { todo!() }\n}\n";
        assert_eq!(
            block(source, "Point"),
            [
                "pub struct Point",
                "{",
                "    x: i32,",
                "",
                "pub(self):",
                "    hidden: u8,",
                "",
                "pub:",
                "    static new() -> Self",
                "",
                "pub(self):",
                "    check()",
                "}",
            ],
            "the first member shown is the public field, which the source writes second; a \
             label prints where the visibility changes"
        );
        let plain = "pub struct Plain {\n    pub x: i32,\n}\n\
                     impl Plain {\n    pub fn get(&self) -> i32 { self.x }\n}\n";
        assert_eq!(
            block(plain, "Plain"),
            [
                "pub struct Plain",
                "{",
                "    x: i32,",
                "",
                "    get() -> i32",
                "}"
            ],
            "public data and public members: no label at all"
        );
        assert_eq!(block("pub struct Unit;\n", "Unit"), ["pub struct Unit"]);
    }

    #[test]
    fn a_struct_whose_first_member_shown_is_not_pub_is_a_class() {
        let source = "pub struct Counter {\n    count: u32,\n}\n\
                      impl Counter {\n    pub fn count(&self) -> u32 { self.count }\n}\n\
                      pub struct Gauge;\n\
                      impl Gauge {\n    pub(crate) fn level(&self) -> u8 { 0 }\n    \
                      fn private_level(&self) -> u8 { 0 }\n}\n";
        assert_eq!(
            block(source, "Counter"),
            [
                "pub class Counter",
                "{",
                "    count: u32,",
                "",
                "pub:",
                "    count() -> u32",
                "}",
            ],
            "a class starts private, so the private data has no label"
        );
        assert_eq!(
            block(source, "Gauge"),
            [
                "pub class Gauge",
                "{",
                "pub(crate):",
                "    level() -> u8",
                "",
                "pub(self):",
                "    private_level() -> u8",
                "}",
            ],
            "no C++ keyword starts at `pub(crate)`, so the label stays"
        );
    }

    #[test]
    fn the_first_field_of_a_tuple_struct_decides_its_keyword() {
        let source = "pub struct Id(i64);\n\
                      impl Id {\n    pub fn get(self) -> i64 { self.0 }\n}\n\
                      pub struct Pair(pub u8, u16);\n\
                      impl Pair {\n    pub fn first(&self) -> u8 { self.0 }\n}\n";
        assert_eq!(
            block(source, "Id"),
            [
                "pub class Id(i64)",
                "{",
                "pub:",
                "    get(self) -> i64",
                "}"
            ]
        );
        assert_eq!(
            block(source, "Pair"),
            [
                "pub struct Pair(pub u8, u16)",
                "{",
                "    first() -> u8",
                "}"
            ]
        );
    }

    #[test]
    fn an_enum_a_union_and_a_trait_keep_their_keyword_and_their_first_label() {
        let source = "pub enum Mode {\n    On,\n}\n\
                      impl Mode {\n    pub fn is_on(&self) -> bool { true }\n}\n\
                      pub union Bits {\n    pub word: u32,\n}\n\
                      impl Bits {\n    pub fn zero() -> Self { Bits { word: 0 } }\n}\n\
                      pub trait Speak {\n    fn say(&self);\n}\n";
        assert_eq!(
            block(source, "Mode"),
            [
                "pub enum Mode",
                "{",
                "    On,",
                "",
                "pub:",
                "    is_on() -> bool",
                "}",
            ]
        );
        assert_eq!(
            block(source, "Bits"),
            [
                "pub union Bits",
                "{",
                "pub:",
                "    word: u32,",
                "",
                "    static zero() -> Self",
                "}",
            ],
            "the label prints once for the data and the members that share it"
        );
        assert_eq!(
            block(source, "Speak"),
            ["pub trait Speak", "{", "    say()", "}"]
        );
    }

    #[test]
    fn a_type_whose_only_body_is_the_comment_of_an_inherent_impl_has_a_body() {
        let source = "pub struct Run;\n// Nothing yet.\nimpl Run {}\n";
        assert_eq!(
            block(source, "Run"),
            ["pub struct Run", "{", "    // Nothing yet.", "}"]
        );
    }

    #[test]
    fn a_short_list_past_the_width_wraps_with_a_leading_comma() {
        let traits = [
            "Binary", "Debug", "Display", "LowerExp", "LowerHex", "Octal", "Pointer", "UpperExp",
            "UpperHex", "Write",
        ];
        let mut source = String::from("pub struct Wide;\n");
        for name in traits {
            source.push_str("impl std::fmt::");
            source.push_str(name);
            source.push_str(" for Wide {}\n");
        }
        let lines = head(&source, "Wide");
        assert_eq!(
            lines,
            [
                "pub struct Wide",
                "    : Binary, Debug, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp",
                "    , UpperHex, Write"
            ]
        );
        assert!(lines.iter().all(|l| l.chars().count() <= MAX_WIDTH));
    }

    #[test]
    fn a_tuple_struct_with_commented_fields_keeps_its_where_clause() {
        let source = "pub struct Pair<T>(\n    /// The first.\n    pub T,\n    pub T,\n)\n\
                      where\n    T: Copy;\n";
        let lines = head(source, "Pair");
        assert!(
            lines.last().unwrap().ends_with("where T: Copy"),
            "{lines:?}"
        );
    }
}
