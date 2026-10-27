//! Tests that an HTML page gives a function, constant, or static name an id
//! and links names to those ids, in its own crate and in another.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::document::{Document, Node};
    use crate::layout::{Block, Line, Span};
    use crate::model::{CrossRef, Loc, Reach, TypeRef, ValueRef};
    use crate::render_html::{Anchors, render};

    /// A page holding one function name with an id, one name that links to
    /// that function, and one that links to a constant of crate 1.
    fn value_page() -> (Document, [Anchors; 2]) {
        let run_fn = ValueRef { module: 0, item: 0 };
        let limit = ValueRef { module: 0, item: 1 };
        let name = |text: &str, value: Option<ValueRef>, foreign: Option<CrossRef>| TypeRef {
            text: text.into(),
            path: vec![text.into()],
            global: false,
            module: 0,
            generics: Vec::new(),
            comment: true,
            may_be_value: true,
            target: None,
            value,
            canonical: None,
            written: None,
            foreign,
        };
        let reach = CrossRef {
            krate: 1,
            reach: Reach::Value(limit),
        };
        let line = Line {
            indent: 0,
            spans: vec![
                Span::Value("run".into(), Loc { file: 0, line: 1 }, run_fn),
                Span::Code("(); ".into()),
                Span::Ref(0),
                Span::Code(" ".into()),
                Span::Ref(1),
            ],
            trailing: None,
        };
        let doc = Document {
            title: "a".into(),
            id: "crate.a".into(),
            crate_docs: None,
            legend: Vec::new(),
            contents: Vec::new(),
            kill_summary: "none".into(),
            kill: Vec::new(),
            sections: vec![Node::Code(Block {
                lines: vec![line],
                repeats: Vec::new(),
            })],
            refs: vec![
                name("run", Some(run_fn), None),
                name("LIMIT", None, Some(reach)),
            ],
            decl_ids: Vec::new(),
            value_ids: BTreeMap::from([(run_fn, "fn.a.run".to_owned())]),
        };
        let page = |file: &str, values: BTreeMap<ValueRef, String>| Anchors {
            file: file.into(),
            ids: Vec::new(),
            values,
        };
        let other = BTreeMap::from([(limit, "const.b.LIMIT".to_owned())]);
        (
            doc,
            [page("a.html", BTreeMap::new()), page("b.html", other)],
        )
    }

    #[test]
    fn a_function_name_has_an_id_and_value_names_link_to_ids_in_either_crate() {
        let (doc, run) = value_page();
        let html = render(&doc, None, &run, crate::stamp::PLACEHOLDER);
        assert!(
            html.contains(
                "<span class=\"src\" id=\"fn.a.run\">run</span>(); \
                 <a href=\"#fn.a.run\">run</a> \
                 <a class=\"cross\" href=\"b.html#const.b.LIMIT\">LIMIT</a>"
            ),
            "{html}"
        );
    }

    #[test]
    fn with_source_links_on_the_id_sits_on_the_link_around_the_name() {
        let (doc, run) = value_page();
        let to = |loc: Loc| format!("src/lib.rs#L{}", loc.line);
        let html = render(&doc, Some(&to), &run, crate::stamp::PLACEHOLDER);
        assert!(
            html.contains("<a class=\"src\" href=\"src/lib.rs#L1\" id=\"fn.a.run\">run</a>"),
            "{html}"
        );
    }
}
