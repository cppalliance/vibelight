//! Lays out a declaration's head: the keyword line with any tuple fields,
//! the base list, and the `where` clause.

use crate::layout::{Line, Span, code, plain, where_parts, width};
use crate::layout_members::param_lines;
use crate::model::{Base, Decl, Shape, TypeRef, name_order};
use crate::wrap::{self, MAX_WIDTH};

fn line(indent: usize, spans: Vec<Span>) -> Line {
    Line {
        indent,
        spans,
        trailing: None,
    }
}

/// Finishes a header with any tuple fields, the base list, and the `where`
/// clause; a type with no body ends in `;`. The base list stays on the head
/// line while it holds only short entries and the line fits. Otherwise each
/// base line starts with `: ` or `, `: the short names first, wrapping to
/// fit, then one long path per line, then the `where` clause on its own line.
pub(crate) fn header_lines(
    decl: &Decl,
    header: Vec<Span>,
    has_body: bool,
    refs: &[TypeRef],
) -> Vec<Line> {
    let mut lines = head_lines(decl, header);
    let bases = match &decl.shape {
        Shape::Trait { supertraits, .. } => supertraits.as_slice(),
        _ => decl.bases.as_slice(),
    };
    let (short, long) = lists(bases, refs);
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
    if long.is_empty() && width(last.indent, &one, refs) <= MAX_WIDTH {
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
        lines.extend(comma_lines(&short, &long, refs));
        if let Some(clause) = &where_clause {
            lines.extend(wrap::where_lines(clause, &[], 4, refs));
        }
    }
    if !has_body {
        if let Some(end) = lines.last_mut() {
            end.spans.push(plain(";"));
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
        header.extend(code(&field.text));
    }
    header.push(plain(")"));
    vec![line(0, header)]
}

/// Returns the short entries, sorted by name, and the long entries, sorted
/// by path.
fn lists(bases: &[Base], refs: &[TypeRef]) -> (Vec<Vec<Span>>, Vec<Vec<Span>>) {
    let mut short: Vec<&Base> = bases.iter().filter(|b| !b.long).collect();
    short.sort_by(|a, b| {
        name_order(&a.name, &b.name).then_with(|| a.text.visible(refs).cmp(&b.text.visible(refs)))
    });
    let mut long: Vec<&Base> = bases.iter().filter(|b| b.long).collect();
    long.sort_by(|a, b| name_order(&a.text.visible(refs), &b.text.visible(refs)));
    let spans = |list: Vec<&Base>| list.into_iter().map(|b| code(&b.text)).collect();
    (spans(short), spans(long))
}

/// Returns the base lines of the leading-comma form.
fn comma_lines(short: &[Vec<Span>], long: &[Vec<Span>], refs: &[TypeRef]) -> Vec<Line> {
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
    for entry in long {
        let mut spans = vec![plain(if lines.is_empty() { ": " } else { ", " })];
        spans.extend(entry.iter().cloned());
        lines.push(line(4, spans));
    }
    lines
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::layout::line_text;

    /// Returns the head of the declaration `name` in a one-file crate: its
    /// lines before the opening brace.
    fn head(source: &str, name: &str) -> Vec<String> {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), source).unwrap();
        let tree = crate::modtree::walk(tmp.path(), "src/lib.rs").unwrap();
        let mut krate = crate::collect::collect(tree, tmp.path(), "t", "t");
        crate::resolve::resolve(&mut krate);
        crate::attach::attach(&mut krate);
        let id = krate.decls.iter().position(|d| d.name == name).unwrap();
        crate::layout_decl::decl_block(&krate, id)
            .lines
            .iter()
            .map(|l| format!("{}{}", " ".repeat(l.indent), line_text(l, &krate.refs)))
            .take_while(|l| l != "{")
            .collect()
    }

    #[test]
    fn standard_traits_that_fit_stay_on_the_head_line() {
        let source = "pub struct Run;\n\
                      impl std::fmt::Display for Run {}\n\
                      impl std::fmt::Debug for Run {}\n";
        assert_eq!(head(source, "Run"), ["struct Run : Debug, Display;"]);
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
                "struct Holder<T>",
                "    : Debug",
                "    , Named",
                "    where T: Copy"
            ]
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
                "struct Wide",
                "    : Binary, Debug, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp",
                "    , UpperHex, Write;"
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
            lines.last().unwrap().ends_with("where T: Copy;"),
            "{lines:?}"
        );
    }
}
