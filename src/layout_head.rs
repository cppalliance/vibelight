//! Lays out a declaration's head as Rust writes it: the keyword line with
//! any tuple fields and a trait's supertraits, the `where` clause, and the
//! brace that opens the body or the `;` that ends the declaration.

use crate::layout::{Line, Span, code, plain, vis_spans, where_parts, width};
use crate::layout_members::param_lines;
use crate::model::{Decl, Shape, TypeRef};
use crate::wrap::{self, MAX_WIDTH};

fn line(indent: usize, spans: Vec<Span>) -> Line {
    Line {
        indent,
        spans,
        trailing: None,
    }
}

/// Finishes a header with any tuple fields, a trait's supertraits after a
/// colon, and the `where` clause, then ` {` when the declaration has a body
/// and `;` when it has none. A `where` clause that keeps the line within the
/// width stays on it; otherwise it takes lines of its own after the head,
/// and the brace or the `;` a line of its own after the clause.
pub(crate) fn header_lines(
    decl: &Decl,
    header: Vec<Span>,
    has_body: bool,
    refs: &[TypeRef],
) -> Vec<Line> {
    let mut lines = head_lines(decl, header);
    let mut last = lines.pop().unwrap_or_default();
    if let Shape::Trait { supertraits, .. } = &decl.shape {
        for (i, base) in supertraits.iter().enumerate() {
            last.spans.push(plain(if i == 0 { ": " } else { " + " }));
            last.spans.extend(code(&base.text));
        }
    }
    let end = if has_body { " {" } else { ";" };
    match decl.where_clause.as_ref().map(where_parts) {
        None => {
            last.spans.push(plain(end));
            lines.push(last);
        }
        Some(clause) => {
            let mut one = last.spans.clone();
            one.push(plain(" "));
            one.extend(clause.one.iter().cloned());
            one.push(plain(end));
            if width(last.indent, &one, refs) <= MAX_WIDTH {
                lines.push(line(last.indent, one));
            } else {
                lines.push(last);
                lines.extend(wrap::where_lines(&clause, &[], 0, refs));
                lines.push(line(0, vec![plain(end.trim_start())]));
            }
        }
    }
    if !has_body {
        if let Some(end) = lines.last_mut() {
            end.trailing.clone_from(&decl.meta.trailing);
        }
    }
    lines
}

/// Returns the head without its `where` clause: the header line, with a
/// tuple struct's fields on it, or one field per line when any field has a
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::layout::line_text;

    /// Returns the head of the declaration `name` in a one-file crate: its
    /// lines up to the one that opens its body or ends it, less the
    /// attribute lines above it.
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
        let lines: Vec<String> = crate::layout_decl::decl_block(&krate, id)
            .lines
            .iter()
            .map(|l| format!("{}{}", " ".repeat(l.indent), line_text(l, &krate.refs)))
            .filter(|l| !l.starts_with("#["))
            .collect();
        let end = lines
            .iter()
            .position(|l| l.ends_with('{') || l.ends_with(';'))
            .unwrap();
        lines[..=end].to_vec()
    }

    #[test]
    fn the_traits_a_type_implements_stay_off_the_head_line() {
        let source = "pub struct Run;\n\
                      impl std::fmt::Display for Run {}\n\
                      impl std::fmt::Debug for Run {}\n";
        assert_eq!(head(source, "Run"), ["pub struct Run;"]);
    }

    #[test]
    fn a_where_clause_that_fits_stays_on_the_head_line() {
        let source = "pub struct Holder<T> where T: Copy { pub value: T }\n";
        assert_eq!(
            head(source, "Holder"),
            ["pub struct Holder<T> where T: Copy {"]
        );
    }

    #[test]
    fn a_long_where_clause_takes_lines_of_its_own_before_the_brace() {
        let source = "pub struct Holder<T, U> where T: Copy + Clone + Default + \
                      std::fmt::Debug + std::fmt::Display, U: Iterator<Item = T> + Clone \
                      { pub value: T, pub other: U }\n";
        let lines = head(source, "Holder");
        assert_eq!(lines[0], "pub struct Holder<T, U>");
        assert!(lines[1].starts_with("where"), "{lines:?}");
        assert_eq!(lines.last().unwrap(), "{");
        assert!(lines.iter().all(|l| l.chars().count() <= MAX_WIDTH));
    }

    #[test]
    fn a_trait_lists_its_supertraits_after_a_colon() {
        let source = "pub trait Named: Clone + std::fmt::Debug { fn name(&self) -> String; }\n";
        assert_eq!(
            head(source, "Named"),
            ["pub trait Named: Clone + Debug {"],
            "a standard trait prints by its short name"
        );
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
