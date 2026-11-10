//! The ids of the functions inside a declaration's block: each function's
//! name takes an id under the id of its declaration's heading, so a link
//! can land on one function of a type.

use std::collections::BTreeMap;

use crate::html::layout::{Line, Span};

/// Gives each function name of a declaration's block the id it has on the
/// page, `<prefix>.<name>`, with `-2`, `-3`, and so on for a name that
/// repeats in the block.
pub(crate) fn anchored(lines: &[Line], prefix: &str) -> Vec<Line> {
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
                        Span::Fn(text.clone(), *loc, Some(id))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Loc;

    #[test]
    fn a_function_takes_its_id_and_a_repeated_name_the_next_number() {
        let loc = Loc { file: 0, line: 1 };
        let function = |name: &str| Line {
            indent: 4,
            spans: vec![Span::Fn(name.to_owned(), loc, None)],
            trailing: None,
        };
        let lines = vec![function("new"), function("len"), function("new")];
        let ids: Vec<Option<String>> = anchored(&lines, "fn.a.T")
            .into_iter()
            .map(|line| match &line.spans[0] {
                Span::Fn(_, _, id) => id.clone(),
                other => panic!("not a function name: {other:?}"),
            })
            .collect();
        assert_eq!(
            ids,
            ["fn.a.T.new", "fn.a.T.len", "fn.a.T.new-2"].map(|id| Some(id.to_owned()))
        );
        assert_eq!(fn_prefix("type.a.m.T"), "fn.a.m.T");
        assert_eq!(fn_prefix("api.T"), "api.T");
    }
}
