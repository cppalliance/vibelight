//! The long-line rule: breaks a declaration that does not fit in `MAX_WIDTH`.

use crate::layout::{Line, Span, width};
use crate::model::TypeRef;

/// The widest a line may be, counting its indent and visible text.
pub(crate) const MAX_WIDTH: usize = 80;

/// A function declaration split where rule 16 may break it.
pub(crate) struct FnParts {
    /// Qualifiers, name, and generics.
    pub(crate) head: Vec<Span>,
    pub(crate) params: Vec<ParamParts>,
    /// What follows the closing parenthesis, such as ` const`.
    pub(crate) after: Vec<Span>,
    /// The return type, without the arrow.
    pub(crate) ret: Option<Vec<Span>>,
    pub(crate) where_clause: Option<Vec<Span>>,
    /// The closing punctuation, `;`.
    pub(crate) end: Vec<Span>,
}

/// One parameter: the comment lines above it, at indent 0, and its text.
pub(crate) struct ParamParts {
    pub(crate) notes: Vec<Line>,
    pub(crate) text: Vec<Span>,
}

fn s(text: &str) -> Span {
    Span::Code(text.to_owned())
}

fn line(indent: usize, spans: Vec<Span>) -> Line {
    Line {
        indent,
        spans,
        trailing: None,
    }
}

/// Lays out a function declaration at `indent` under rule 16. A parameter
/// with a comment forces one parameter per line.
pub(crate) fn function(
    parts: &FnParts,
    indent: usize,
    trailing: Option<String>,
    refs: &[TypeRef],
) -> Vec<Line> {
    let fits = |l: &Line| width(l.indent, &l.spans, refs) <= MAX_WIDTH;
    let forced = parts.params.iter().any(|p| !p.notes.is_empty());
    let pieces = Pieces::new(parts);

    let mut current: Vec<Line>;
    let mut first_fits = false;
    if parts.ret.is_some() {
        let whole = line(
            indent,
            concat(&[&pieces.one, &[s(" -> ")], &pieces.ret, &pieces.finish]),
        );
        if !forced && fits(&whole) {
            return finished(vec![whole], trailing);
        }
        current = vec![
            line(indent, concat(&[&pieces.one, &[s(" ->")]])),
            line(indent + 4, concat(&[&pieces.ret, &pieces.finish])),
        ];
        first_fits = fits(&current[0]);
        if !forced && current.iter().all(fits) {
            return finished(current, trailing);
        }
    } else {
        current = vec![line(indent, concat(&[&pieces.one, &pieces.finish]))];
        if !forced && fits(&current[0]) {
            return finished(current, trailing);
        }
    }
    if (forced || !first_fits) && !parts.params.is_empty() {
        current = one_per_line(parts, &pieces, indent, &fits);
        if current.iter().all(fits) {
            return finished(current, trailing);
        }
    }
    if let Some(clause) = &parts.where_clause {
        let last = current.pop().unwrap_or_default();
        if fits(&last) {
            current.push(last);
        } else {
            let keep = last.spans.len().saturating_sub(pieces.finish.len());
            current.push(line(last.indent, last.spans[..keep].to_vec()));
            current.push(line(indent + 4, concat(&[clause, &parts.end])));
        }
    }
    finished(current, trailing)
}

/// The runs a function declaration is assembled from.
struct Pieces {
    /// The head and parameters on one line, through the closing parenthesis
    /// and anything after it.
    one: Vec<Span>,
    /// The closing parenthesis and anything after it.
    tail: Vec<Span>,
    /// The return type.
    ret: Vec<Span>,
    /// The `where` clause, if any, and the closing punctuation.
    finish: Vec<Span>,
}

impl Pieces {
    fn new(parts: &FnParts) -> Pieces {
        let tail = concat(&[&[s(")")], &parts.after]);
        let mut finish = Vec::new();
        if let Some(clause) = &parts.where_clause {
            finish.push(s(" "));
            finish.extend(clause.iter().cloned());
        }
        finish.extend(parts.end.iter().cloned());
        let mut one = parts.head.clone();
        one.push(s("("));
        for (i, param) in parts.params.iter().enumerate() {
            if i > 0 {
                one.push(s(", "));
            }
            one.extend(param.text.iter().cloned());
        }
        one.extend(tail.iter().cloned());
        Pieces {
            one,
            tail,
            ret: parts.ret.clone().unwrap_or_default(),
            finish,
        }
    }
}

/// Lays out one parameter per line, comments above each, with the tail on
/// the last parameter when it fits and broken after `->` when not.
fn one_per_line(
    parts: &FnParts,
    pieces: &Pieces,
    indent: usize,
    fits: &impl Fn(&Line) -> bool,
) -> Vec<Line> {
    let mut lines = vec![line(indent, concat(&[&parts.head, &[s("(")]]))];
    let count = parts.params.len();
    for (i, param) in parts.params.iter().enumerate() {
        for note in &param.notes {
            lines.push(line(indent + 4 + note.indent, note.spans.clone()));
        }
        let mut spans = param.text.clone();
        if i + 1 < count {
            spans.push(s(","));
        }
        lines.push(line(indent + 4, spans));
    }
    let last = lines.pop().unwrap_or_default();
    let joined = if parts.ret.is_some() {
        line(
            indent + 4,
            concat(&[
                &last.spans,
                &pieces.tail,
                &[s(" -> ")],
                &pieces.ret,
                &pieces.finish,
            ]),
        )
    } else {
        line(
            indent + 4,
            concat(&[&last.spans, &pieces.tail, &pieces.finish]),
        )
    };
    if fits(&joined) || parts.ret.is_none() {
        lines.push(joined);
    } else {
        lines.push(line(
            indent + 4,
            concat(&[&last.spans, &pieces.tail, &[s(" ->")]]),
        ));
        lines.push(line(indent + 4, concat(&[&pieces.ret, &pieces.finish])));
    }
    lines
}

fn concat(parts: &[&[Span]]) -> Vec<Span> {
    parts.iter().flat_map(|p| p.iter().cloned()).collect()
}

/// Lays out a type header: a `where` clause that makes it too long moves to
/// its own line, indented 4 more.
pub(crate) fn header(
    spans: Vec<Span>,
    where_clause: Option<Vec<Span>>,
    refs: &[TypeRef],
) -> Vec<Line> {
    let Some(clause) = where_clause else {
        return vec![line(0, spans)];
    };
    let mut whole = spans.clone();
    whole.push(s(" "));
    whole.extend(clause.iter().cloned());
    if width(0, &whole, refs) <= MAX_WIDTH {
        return vec![line(0, whole)];
    }
    vec![line(0, spans), line(4, clause)]
}

fn finished(mut lines: Vec<Line>, trailing: Option<String>) -> Vec<Line> {
    if let Some(last) = lines.last_mut() {
        last.trailing = trailing;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|l| {
                let body: String = l
                    .spans
                    .iter()
                    .map(|s| match s {
                        Span::Code(t) | Span::Key(t) | Span::Comment(t) => t.clone(),
                        _ => String::new(),
                    })
                    .collect();
                format!("{}{body}", " ".repeat(l.indent))
            })
            .collect()
    }

    fn parts(name: &str, params: &[&str], ret: Option<&str>, clause: Option<&str>) -> FnParts {
        FnParts {
            head: vec![s(name)],
            params: params
                .iter()
                .map(|p| ParamParts {
                    notes: Vec::new(),
                    text: vec![s(p)],
                })
                .collect(),
            after: Vec::new(),
            ret: ret.map(|r| vec![s(r)]),
            where_clause: clause.map(|c| vec![s(c)]),
            end: vec![s(";")],
        }
    }

    #[test]
    fn exactly_max_width_does_not_break() {
        let name = format!("f{}", "x".repeat(71));
        let lines = function(&parts(&name, &["a: u8"], None, None), 0, None, &[]);
        assert_eq!(text(&lines), [format!("{name}(a: u8);")]);
        assert_eq!(text(&lines)[0].len(), 80);
    }

    #[test]
    fn one_column_more_breaks_after_the_arrow() {
        let name = format!("f{}", "x".repeat(61));
        let lines = function(&parts(&name, &["a: u8"], Some("u32"), None), 4, None, &[]);
        assert_eq!(
            text(&lines),
            [format!("    {name}(a: u8) ->"), "        u32;".to_owned()]
        );
    }

    #[test]
    fn long_first_line_goes_one_parameter_per_line() {
        let long = "a_parameter_with_a_long_name: u64";
        let lines = function(
            &parts("f", &[long, long, long], Some("u8"), None),
            0,
            None,
            &[],
        );
        assert_eq!(
            text(&lines),
            [
                "f(",
                &format!("    {long},"),
                &format!("    {long},"),
                &format!("    {long}) -> u8;")
            ]
        );
    }

    #[test]
    fn where_clause_moves_last() {
        let clause = format!("where T: {}", "Clone + ".repeat(8).trim_end_matches(" + "));
        let lines = function(
            &parts("f<T>", &["name: &str", "value: T"], None, Some(&clause)),
            4,
            None,
            &[],
        );
        assert_eq!(
            text(&lines),
            [
                "    f<T>(",
                "        name: &str,",
                "        value: T)",
                &format!("        {clause};")
            ]
        );
    }

    #[test]
    fn a_commented_parameter_list_stays_packed() {
        let mut commented = parts("f", &["a: u8", "b: u8"], None, None);
        commented.params[0].notes = vec![line(0, vec![s("// first")])];
        commented.params[1].notes = vec![line(0, vec![s("// second")])];
        let lines = function(&commented, 4, None, &[]);
        assert_eq!(
            text(&lines),
            [
                "    f(",
                "        // first",
                "        a: u8,",
                "        // second",
                "        b: u8);"
            ]
        );
    }

    #[test]
    fn a_line_that_cannot_fit_stays() {
        let huge = format!("p: {}", "x".repeat(90));
        let lines = function(&parts("f", &[&huge], None, None), 0, None, &[]);
        assert_eq!(text(&lines), ["f(".to_owned(), format!("    {huge});")]);
    }
}
