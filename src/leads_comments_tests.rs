//! Tests each detector of body comments on a small library where it finds
//! its lead, and on a close one where it finds none.

use crate::leads::Mode;
use crate::leads_tests::{MANIFEST, lib, with_leads};

/// Asserts what `id` finds in a library and that it finds nothing in a
/// close one.
fn finds(id: &str, positive: &str, expected: &[&str], negative: &str) {
    assert_eq!(lib(positive, id), expected, "{id} on its case");
    assert_eq!(
        lib(negative, id),
        Vec::<String>::new(),
        "{id} on the close case"
    );
}

/// Returns the mode and evidence of each lead with `id` in a library.
fn evidence(source: &str, id: &str) -> Vec<(Mode, String)> {
    let mut out = Vec::new();
    with_leads(&[MANIFEST, ("src/lib.rs", source)], |_, leads| {
        out = (leads.iter().filter(|lead| lead.id == id))
            .map(|lead| (lead.mode, lead.evidence.clone()))
            .collect();
    });
    out
}

/// A function that parses its input, with `comment` above its first
/// statement.
fn parser(comment: &str) -> String {
    format!(
        "pub fn run(input: &str) -> Vec<String> {{\n{comment}\n    let args = parse_args(input);\n    args\n}}\n\
         fn parse_args(text: &str) -> Vec<String> {{ vec![text.to_owned()] }}\n"
    )
}

#[test]
fn a_comment_whose_words_its_statement_writes_narrates_it() {
    let narrating = parser("    // Parse the args.");
    finds(
        "comment-statement-overlap",
        &narrating,
        &["run"],
        &format!(
            "{}{}",
            parser("    // Parse the args\n    // because the input may be long."),
            "pub fn idle(input: &str) -> usize {\n    // count the raw request headers\n    input.len()\n}\n"
        ),
    );
    assert_eq!(
        evidence(&narrating, "comment-statement-overlap"),
        [(
            Mode::Lexical,
            "2 of 2 words of \"// Parse the args.\" are in its statement: parse, args".to_owned()
        )]
    );
}

#[test]
fn a_comment_that_names_what_its_next_statements_do_not_write_is_a_lead() {
    let source =
        "pub fn run(items: &mut Vec<u8>) {\n    // Rebuild `index` first.\n    items.push(1);\n}\n";
    finds(
        "comment-name-absent-next-statement",
        source,
        &["run"],
        "pub fn run(items: &mut Vec<u8>) {\n    // Grow `items` by one.\n    items.push(1);\n    \
         items.push(2); // then `index`\n    // and items_len, the HashMap free\n    items.len();\n    \
         // `index` must wait, so skip it\n    items.clear();\n    \
         // read the chat_template key\n    let _ = \"chat_template\";\n}\n",
    );
    let found = evidence(source, "comment-name-absent-next-statement");
    assert_eq!(found[0].0, Mode::Lexical);
}

#[test]
fn a_comment_that_ties_code_to_code_elsewhere_is_a_sync_comment() {
    finds(
        "sync-comment",
        "pub fn run() -> u8 {\n    // Keep in sync with `render::TABLE`.\n    3\n}\n",
        &["run"],
        "pub fn run() -> u8 {\n    // The async queue must be matched later.\n    3\n}\n",
    );
}

#[test]
fn a_cost_or_an_invariant_that_no_statement_makes_is_a_claim() {
    let claimed = "pub fn run(keys: &mut Vec<u8>) {\n    // keys stay sorted, so lookups are O(log n)\n    keys.push(1);\n}\n";
    finds(
        "claim-comment",
        claimed,
        &["run"],
        "pub fn run(keys: &mut Vec<u8>) {\n    // keep the keys sorted\n    keys.sort();\n    \
         // unsorted is fine for a LOG(1) call\n    keys.push(1);\n}\n",
    );
    assert_eq!(
        evidence(claimed, "claim-comment"),
        [(
            Mode::DirectSyntax,
            "\"O(log n)\", \"sorted\" in \"// keys stay sorted, so lookups are O(log n)\""
                .to_owned()
        )]
    );
}

#[test]
fn a_papercut_in_a_body_comment_or_a_doc_is_a_lead_but_not_in_code_spans() {
    finds(
        "papercut",
        "pub fn run() -> u8 {\n    // TODO: handle errors\n    3\n}\n\
         /// Reads the whole file, for now.\npub fn read() {}\n\
         pub mod checks {\n    const _: () = {\n        // FIXME: a real check\n        ()\n    };\n}\n",
        &["checks", "read", "run"],
        "pub fn run() -> u8 {\n    // the hacker news feed\n    3\n}\n\
         /// Panics with `todo!()` when the `HACK` flag is unset.\npub fn read() {}\n",
    );
}
