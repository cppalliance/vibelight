//! The long-line rule: breaks a declaration that does not fit in `MAX_WIDTH`.

use crate::layout::{Line, Span, width};
use crate::model::TypeRef;

/// The widest a line may be, counting its indent and visible text.
pub(crate) const MAX_WIDTH: usize = 80;

/// A function declaration split where the long-line rule may break it.
pub(crate) struct FnParts {
    /// Visibility, qualifiers, `fn`, name, and generics.
    pub(crate) head: Vec<Span>,
    pub(crate) params: Vec<ParamParts>,
    /// The return type, without the arrow.
    pub(crate) ret: Option<Vec<Span>>,
    pub(crate) where_clause: Option<WhereParts>,
    /// The closing punctuation, `;`.
    pub(crate) end: Vec<Span>,
}

/// A `where` clause laid out both ways the long-line rule prints it.
pub(crate) struct WhereParts {
    /// `where` and the predicates joined by `, `.
    pub(crate) one: Vec<Span>,
    pub(crate) predicates: Vec<Vec<Span>>,
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

/// Lays out a function declaration at `indent` the way rustfmt does: on
/// one line when it fits; else the signature on one line and its `where`
/// clause below it, when the signature fits; else one parameter per line,
/// each ending in a comma, and `)` with the return type on a line of their
/// own, the `where` clause below. A parameter with a comment forces one
/// parameter per line.
pub(crate) fn function(
    parts: &FnParts,
    indent: usize,
    trailing: Option<String>,
    refs: &[TypeRef],
) -> Vec<Line> {
    let fits = |l: &Line| width(l.indent, &l.spans, refs) <= MAX_WIDTH;
    let forced = parts.params.iter().any(|p| !p.notes.is_empty());
    let mut close = vec![s(")")];
    if let Some(ret) = &parts.ret {
        close.push(s(" -> "));
        close.extend(ret.iter().cloned());
    }
    let mut signature = concat(&[&parts.head, &[s("(")]]);
    for (i, param) in parts.params.iter().enumerate() {
        if i > 0 {
            signature.push(s(", "));
        }
        signature.extend(param.text.iter().cloned());
    }
    signature.extend(close.iter().cloned());
    if !forced {
        let mut whole = signature.clone();
        if let Some(clause) = &parts.where_clause {
            whole.push(s(" "));
            whole.extend(clause.one.iter().cloned());
        }
        whole.extend(parts.end.iter().cloned());
        let whole = line(indent, whole);
        if fits(&whole) {
            return finished(vec![whole], trailing);
        }
        let alone = line(indent, signature.clone());
        if parts.params.is_empty() || (parts.where_clause.is_some() && fits(&alone)) {
            return finished(below(alone, parts, indent, refs), trailing);
        }
    }
    let mut lines = vec![line(indent, concat(&[&parts.head, &[s("(")]]))];
    for param in &parts.params {
        for note in &param.notes {
            lines.push(line(indent + 4 + note.indent, note.spans.clone()));
        }
        lines.push(line(indent + 4, concat(&[&param.text, &[s(",")]])));
    }
    lines.extend(below(line(indent, close), parts, indent, refs));
    finished(lines, trailing)
}

/// Returns a declaration's last line, `last`, then its `where` clause below
/// it when it has one, each ending in the closing punctuation.
fn below(mut last: Line, parts: &FnParts, indent: usize, refs: &[TypeRef]) -> Vec<Line> {
    if let Some(clause) = &parts.where_clause {
        let mut lines = vec![last];
        lines.extend(where_lines(clause, &parts.end, indent, refs));
        return lines;
    }
    last.spans.extend(parts.end.iter().cloned());
    vec![last]
}

/// Lays out a `where` clause at `indent`, followed by `end`: on one line
/// when it fits, else `where` alone and one predicate per line, indented 4
/// more, every one but the last ending in a comma.
pub(crate) fn where_lines(
    clause: &WhereParts,
    end: &[Span],
    indent: usize,
    refs: &[TypeRef],
) -> Vec<Line> {
    let whole = line(indent, concat(&[&clause.one, end]));
    if width(indent, &whole.spans, refs) <= MAX_WIDTH {
        return vec![whole];
    }
    let mut lines = vec![line(indent, vec![Span::Key("where".to_owned())])];
    let count = clause.predicates.len();
    for (i, predicate) in clause.predicates.iter().enumerate() {
        let after = if i + 1 < count {
            vec![s(",")]
        } else {
            end.to_vec()
        };
        lines.push(line(indent + 4, concat(&[predicate, &after])));
    }
    lines
}

fn concat(parts: &[&[Span]]) -> Vec<Span> {
    parts.iter().flat_map(|p| p.iter().cloned()).collect()
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
            ret: ret.map(|r| vec![s(r)]),
            where_clause: clause.map(|c| WhereParts {
                one: vec![s(c)],
                predicates: c
                    .trim_start_matches("where ")
                    .split(", ")
                    .map(|p| vec![s(p)])
                    .collect(),
            }),
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
    fn one_column_more_goes_one_parameter_per_line() {
        let name = format!("f{}", "x".repeat(61));
        let lines = function(&parts(&name, &["a: u8"], Some("u32"), None), 4, None, &[]);
        assert_eq!(
            text(&lines),
            [
                format!("    {name}("),
                "        a: u8,".to_owned(),
                "    ) -> u32;".to_owned()
            ]
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
                &format!("    {long},"),
                ") -> u8;"
            ]
        );
    }

    #[test]
    fn a_where_clause_that_does_not_fit_on_the_line_goes_below_the_signature() {
        let clause = format!("where T: {}", "Clone + ".repeat(8).trim_end_matches(" + "));
        let lines = function(
            &parts("f<T>", &["name: &str", "value: T"], None, Some(&clause)),
            4,
            None,
            &[],
        );
        assert_eq!(
            text(&lines),
            ["    f<T>(name: &str, value: T)", &format!("    {clause};")]
        );
    }

    #[test]
    fn a_where_clause_that_still_does_not_fit_goes_one_predicate_per_line() {
        let first = format!("A: {}", "Clone + ".repeat(5).trim_end_matches(" + "));
        let second = format!("B: {}", "Send + ".repeat(5).trim_end_matches(" + "));
        let clause = format!("where {first}, {second}");
        let lines = function(
            &parts("f<A, B>", &["a: A", "b: B"], None, Some(&clause)),
            4,
            None,
            &[],
        );
        assert_eq!(
            text(&lines),
            [
                "    f<A, B>(a: A, b: B)",
                "    where",
                &format!("        {first},"),
                &format!("        {second};")
            ]
        );
    }

    #[test]
    fn a_commented_parameter_list_goes_one_parameter_per_line() {
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
                "        b: u8,",
                "    );"
            ]
        );
    }

    #[test]
    fn a_line_that_cannot_fit_stays() {
        let huge = format!("p: {}", "x".repeat(90));
        let lines = function(&parts("f", &[&huge], None, None), 0, None, &[]);
        assert_eq!(
            text(&lines),
            ["f(".to_owned(), format!("    {huge},"), ");".to_owned()]
        );
    }
}
