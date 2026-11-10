//! The compact view of a module's items, which the HTML prints above the
//! documented view of the same items: every declaration and item in the
//! same C++-style blocks, without comments, attributes, or blank lines, so
//! each type's labels and members stack one to a line.

use std::collections::BTreeMap;

use crate::html::layout::{Anchor, Line, Span};
use crate::model::ValueRef;

/// Returns the compact lines of a block: its code lines less every comment
/// line, every attribute and `reaches` line, every blank line, and every
/// trailing comment.
pub(crate) fn compact(lines: &[Line]) -> Vec<Line> {
    lines
        .iter()
        .filter(|line| kept(line))
        .map(|line| Line {
            indent: line.indent,
            spans: line.spans.iter().map(without_id).collect(),
            trailing: None,
        })
        .collect()
}

/// Returns a span as the compact view prints it: the name of a function,
/// constant, or static links to its source like any member's name, and
/// leaves its id to the documented view, so no id repeats on the page.
fn without_id(span: &Span) -> Span {
    match span {
        Span::Value(text, loc, _) => Span::Name(text.clone(), *loc),
        other => other.clone(),
    }
}

/// Gives each function name of a declaration's block the id it has on the
/// page, `<prefix>.<name>`, with `-2`, `-3`, and so on for a name that
/// repeats in the block. The documented view puts the id on the name, and
/// the compact view, when `link` is set, links the name to it; both number
/// the names of the same block, so the two always agree.
pub(crate) fn anchored(lines: &[Line], prefix: &str, link: bool) -> Vec<Line> {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    lines
        .iter()
        .map(|line| Line {
            spans: (line.spans.iter())
                .map(|span| match span {
                    Span::Fn(text, loc, _) => {
                        let count = seen.entry(text.as_str()).or_insert(0);
                        *count += 1;
                        let id = match *count {
                            1 => format!("{prefix}.{text}"),
                            n => format!("{prefix}.{text}-{n}"),
                        };
                        let anchor = if link {
                            Anchor::Link(id)
                        } else {
                            Anchor::Id(id)
                        };
                        Span::Fn(text.clone(), *loc, anchor)
                    }
                    other => other.clone(),
                })
                .collect(),
            ..line.clone()
        })
        .collect()
}

/// Returns the prefix of a declaration's function ids from the id of its
/// heading: `type.a.m.T` gives `fn.a.m.T`, and any other id stays as it is.
pub(crate) fn fn_prefix(heading: &str) -> String {
    heading
        .strip_prefix("type.")
        .map_or_else(|| heading.to_owned(), |rest| format!("fn.{rest}"))
}

/// Links each module-level name that `link` gives an id to that id, for the
/// compact view, as a function's name links to where the page documents it.
pub(crate) fn linked(lines: &[Line], link: impl Fn(ValueRef) -> Option<String>) -> Vec<Line> {
    lines
        .iter()
        .map(|line| Line {
            spans: (line.spans.iter())
                .map(|span| match span {
                    Span::Value(text, loc, at) => link(*at).map_or_else(
                        || span.clone(),
                        |id| Span::Fn(text.clone(), *loc, Anchor::Link(id)),
                    ),
                    other => other.clone(),
                })
                .collect(),
            ..line.clone()
        })
        .collect()
}

/// Joins the compact lines of several blocks, one blank line between two
/// blocks.
pub(crate) fn joined<'a>(blocks: impl IntoIterator<Item = &'a [Line]>) -> Vec<Line> {
    let mut out = Vec::new();
    for lines in blocks {
        let lines = compact(lines);
        if lines.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(Line::default());
        }
        out.extend(lines);
    }
    out
}

/// Whether a line stays in the compact view: a comment line opens with its
/// marker, and an attribute or `reaches` line is dim.
fn kept(line: &Line) -> bool {
    !matches!(
        line.spans.first(),
        None | Some(Span::Dim(_) | Span::Marker(_))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::layout::marker::Marker;
    use crate::html::layout::{key, line_text, plain};
    use crate::model::{Loc, ValueRef};

    fn line(indent: usize, spans: Vec<Span>) -> Line {
        Line {
            indent,
            spans,
            trailing: None,
        }
    }

    fn text(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|l| format!("{}{}", " ".repeat(l.indent), line_text(l, &[])))
            .collect()
    }

    fn comment(indent: usize, marker: Marker, text: &str) -> Line {
        line(
            indent,
            vec![Span::Marker(marker.spaced), Span::Comment(text.to_owned())],
        )
    }

    #[test]
    fn comments_attributes_and_blank_lines_go_and_labels_and_members_stay() {
        let lines = vec![
            comment(0, Marker::DOC, "A point."),
            line(0, vec![Span::Dim("[[non_exhaustive]]".to_owned())]),
            line(0, vec![key("pub struct"), plain(" Point")]),
            line(0, vec![plain("{")]),
            line(0, vec![plain("pub:")]),
            comment(4, Marker::DOC, "Across."),
            line(4, vec![plain("x: i32,")]),
            Line::default(),
            line(4, vec![plain("y: i32,")]),
            comment(4, Marker::CFG, "test"),
            line(0, vec![plain("pub(self):")]),
            line(4, vec![key("static"), plain(" new() -> Point")]),
            line(0, vec![plain("}")]),
        ];
        assert_eq!(
            text(&compact(&lines)),
            [
                "pub struct Point",
                "{",
                "pub:",
                "    x: i32,",
                "    y: i32,",
                "pub(self):",
                "    static new() -> Point",
                "}",
            ]
        );
    }

    #[test]
    fn a_trailing_comment_goes_and_a_value_name_leaves_its_id_behind() {
        let loc = Loc { file: 0, line: 1 };
        let at = ValueRef { module: 0, item: 0 };
        let lines = vec![Line {
            indent: 0,
            spans: vec![
                key("constexpr"),
                plain(" "),
                Span::Value("MAX".to_owned(), loc, at),
            ],
            trailing: Some("the most".to_owned()),
        }];
        let compacted = compact(&lines);
        assert_eq!(compacted[0].trailing, None);
        assert!(matches!(&compacted[0].spans[2], Span::Name(name, _) if name == "MAX"));
    }

    #[test]
    fn both_views_give_a_function_the_same_id_and_a_repeated_name_the_next_number() {
        let loc = Loc { file: 0, line: 1 };
        let function = |name: &str| line(4, vec![Span::Fn(name.to_owned(), loc, Anchor::None)]);
        let lines = vec![function("new"), function("len"), function("new")];
        let anchors = |link: bool| -> Vec<Anchor> {
            anchored(&lines, "fn.a.T", link)
                .into_iter()
                .map(|line| match &line.spans[0] {
                    Span::Fn(_, _, anchor) => anchor.clone(),
                    other => panic!("not a function name: {other:?}"),
                })
                .collect()
        };
        let ids = ["fn.a.T.new", "fn.a.T.len", "fn.a.T.new-2"].map(str::to_owned);
        assert_eq!(anchors(false), ids.clone().map(Anchor::Id));
        assert_eq!(anchors(true), ids.map(Anchor::Link));
        assert_eq!(fn_prefix("type.a.m.T"), "fn.a.m.T");
        assert_eq!(fn_prefix("api.T"), "api.T");
    }

    #[test]
    fn a_value_name_links_only_when_given_an_id() {
        let loc = Loc { file: 0, line: 1 };
        let value =
            |item: usize| Span::Value(format!("v{item}"), loc, ValueRef { module: 0, item });
        let lines = vec![line(0, vec![value(0), value(1)])];
        let linked = linked(&lines, |at| (at.item == 0).then(|| "fn.a.v0".to_owned()));
        assert_eq!(
            linked[0].spans[0],
            Span::Fn("v0".to_owned(), loc, Anchor::Link("fn.a.v0".to_owned()))
        );
        assert_eq!(linked[0].spans[1], value(1));
    }

    #[test]
    fn blocks_join_with_one_blank_line_between() {
        let a = vec![line(0, vec![plain("a")])];
        let b = vec![comment(0, Marker::DOC, "only a comment")];
        let c = vec![line(0, vec![plain("c")])];
        let joined = joined([a.as_slice(), b.as_slice(), c.as_slice()]);
        assert_eq!(text(&joined), ["a", "", "c"]);
    }
}
