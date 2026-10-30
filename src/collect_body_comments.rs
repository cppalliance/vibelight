//! Pairs each ordinary comment inside a body with the statement it goes
//! with, for analysis. Body comments never become notes.

use std::cmp::Reverse;

use crate::collect_body_code::Statement;
use crate::collect_body_tokens::words;
use crate::comments::comment_lines;
use crate::model_body::{BodyCommentFact, Placement};
use crate::source::{Comment, SourceFile};

/// Builds the facts of a body's comments, given the body's statements in
/// source order.
pub(crate) fn comment_facts(
    file: &SourceFile,
    statements: &[Statement],
    comments: &[Comment],
) -> Vec<BodyCommentFact> {
    comments
        .iter()
        .map(|comment| {
            let (placement, statement) = place(file, statements, comment);
            let mut fingerprint = Vec::new();
            if let Some(at) = statement {
                fingerprint.extend(words(&file.text[statements[at].range.clone()]));
                if placement == Placement::Leading {
                    let end = statements[at].range.end;
                    for next in statements.iter().filter(|s| s.range.start >= end).take(2) {
                        fingerprint.extend(words(&file.text[next.range.clone()]));
                    }
                }
            }
            fingerprint.sort();
            fingerprint.dedup();
            BodyCommentFact {
                text: comment_lines(file, comment),
                line: file.line_of(comment.range.start),
                placement,
                stmt: statement,
                fingerprint,
            }
        })
        .collect()
}

/// Returns the line a statement ends on.
pub(crate) fn last_line(file: &SourceFile, statement: &Statement) -> u32 {
    let range = &statement.range;
    file.line_of(range.end.saturating_sub(1).max(range.start))
}

/// Places a comment. It trails the innermost statement that ends before
/// it on its line. Else it leads the first statement after it inside the
/// innermost statement that holds it, or anywhere in the body when none
/// holds it. Else it is inside that holding statement. Else it trails the
/// last statement before it, and with no statement at all it is inside the
/// body.
fn place(
    file: &SourceFile,
    statements: &[Statement],
    comment: &Comment,
) -> (Placement, Option<usize>) {
    let line = file.line_of(comment.range.start);
    let at = &comment.range;
    let trailing = statements
        .iter()
        .enumerate()
        .filter(|(_, s)| s.range.end <= at.start && last_line(file, s) == line)
        .max_by_key(|(i, s)| (s.depth, s.range.end, *i));
    if let Some((i, _)) = trailing {
        return (Placement::Trailing, Some(i));
    }
    let holder = statements
        .iter()
        .enumerate()
        .filter(|(_, s)| s.range.start <= at.start && at.end <= s.range.end)
        .max_by_key(|(i, s)| (s.depth, *i));
    let inside = holder.map_or(0..usize::MAX, |(_, s)| s.range.clone());
    let leading = statements.iter().position(|s| {
        s.range.start >= at.end && inside.start <= s.range.start && s.range.end <= inside.end
    });
    if let Some(i) = leading {
        return (Placement::Leading, Some(i));
    }
    if let Some((i, _)) = holder {
        return (Placement::Inner, Some(i));
    }
    let before = statements
        .iter()
        .enumerate()
        .filter(|(_, s)| s.range.end <= at.start)
        .max_by_key(|(i, s)| (s.range.end, Reverse(s.depth), *i));
    match before {
        Some((i, _)) => (Placement::Trailing, Some(i)),
        None => (Placement::Inner, None),
    }
}

#[cfg(test)]
mod tests {
    use crate::collect_body_facts::tests::collected;
    use crate::model_body::{BodyCommentFact, Placement};

    /// The comments of the first body, as (text, placement, the statement's
    /// first line, fingerprint).
    fn comments(lib: &str) -> Vec<(String, Placement, Option<u32>, Vec<String>)> {
        let (_tmp, krate) = collected(lib);
        let facts = &krate.bodies[0].facts;
        facts
            .comments
            .iter()
            .map(|c: &BodyCommentFact| {
                (
                    c.text.join("\n"),
                    c.placement,
                    c.stmt.map(|s| facts.stmt_lines[s].0),
                    c.fingerprint.clone(),
                )
            })
            .collect()
    }

    fn owned(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn comments_lead_trail_and_sit_inside_statements() {
        let found = comments(
            "pub fn run(items: &mut Vec<u8>, step: u8) {\n\
             // grow the list\n\
             items.push(step);\n\
             let total = items.len(); // count them\n\
             if total > 3 {\n\
             /* too many */\n\
             items.clear();\n\
             }\n\
             match total { 0 => {}\n\
             // nothing after this one\n\
             _ => {} }\n\
             // the end\n\
             }\n",
        );
        let lines: Vec<(&str, Placement, Option<u32>)> = found
            .iter()
            .map(|(text, placement, line, _)| (text.as_str(), *placement, *line))
            .collect();
        assert_eq!(
            lines,
            [
                ("// grow the list", Placement::Leading, Some(3)),
                ("// count them", Placement::Trailing, Some(4)),
                ("/* too many */", Placement::Leading, Some(7)),
                ("// nothing after this one", Placement::Inner, Some(9)),
                ("// the end", Placement::Trailing, Some(9)),
            ]
        );
        let leading = [
            ".", "=", ">", "clear", "if", "items", "len", "let", "push", "step", "total",
        ];
        assert_eq!(
            found[0].3,
            owned(&leading),
            "the statement and the next two"
        );
        assert_eq!(
            found[1].3,
            owned(&[".", "=", "items", "len", "let", "total"])
        );
    }

    #[test]
    fn a_comment_in_an_empty_body_goes_with_no_statement() {
        let found = comments("pub fn idle() {\n    // nothing yet\n}\n");
        assert_eq!(
            found,
            [(
                "// nothing yet".to_owned(),
                Placement::Inner,
                None,
                Vec::new()
            )]
        );
    }

    #[test]
    fn body_comments_never_become_notes() {
        let (_tmp, krate) =
            collected("/// Runs.\npub fn run() {\n    // inside\n    go();\n}\nfn go() {}\n");
        let item = &krate.modules[0].items[0];
        assert_eq!(item.meta.notes.len(), 1, "only the doc comment is a note");
        assert_eq!(krate.bodies[0].facts.comments[0].text, ["// inside"]);
    }
}
