//! Tests each detector of doc text on a small library where it finds its
//! lead, and on a close one where it finds none.

use crate::leads::Mode;
use crate::leads_tests::{MANIFEST, found, lib, with_leads};

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

#[test]
fn an_optional_or_boolean_field_documented_with_a_condition_is_conditional() {
    finds(
        "conditional-field",
        "pub struct Run {\npub kind: u8,\n/// `None` unless `kind` is resumed.\n\
         pub resumed: Option<u32>,\n/// Empty for a fresh kind.\npub steps: Vec<u8>,\n}\n",
        &["Run::resumed", "Run::steps"],
        "pub struct Run {\npub kind: u8,\n/// `None` when the user gave none.\n\
         pub resumed: Option<u32>,\n/// Set only when `kind` is resumed.\npub name: String,\n\
         /// Empty for a run with no steps, whatever its kind may be.\npub steps: Vec<u8>,\n}\n",
    );
}

#[test]
fn a_sentence_that_lists_some_variants_of_an_enum_is_partial() {
    finds(
        "partial-doc-list",
        "/// A section: either `Lua` or `Prose`.\npub enum Section { Lua, Prose, Yaml }\n",
        &["Section"],
        "/// A section: `Lua`, `Prose`, or `Yaml`.\npub enum Section { Lua, Prose, Yaml }\n\
         /// A kind such as `Fast` or `Slow`.\npub enum Speed { Fast, Slow, Still }\n",
    );
    let field = |doc: &str| {
        format!(
            "pub enum Mode {{ Fast, Slow, Still }}\npub struct Car {{\n/// {doc}\npub mode: Mode,\n}}\n"
        )
    };
    let car = field("Either `Fast` or `Slow` while it moves.");
    assert_eq!(lib(&car, "partial-doc-list"), ["Car::mode, Mode"]);
    let bare = field("`Fast` or `Slow` while it moves.");
    assert_eq!(
        lib(&bare, "partial-doc-list"),
        Vec::<String>::new(),
        "two bare names"
    );
    let handled = "pub enum Mode { Fast, Slow, Still, Gone }\n\
                   /// Returns either `Fast` or `Slow`.\npub fn pick() -> Mode { Mode::Fast }\n";
    assert_eq!(
        lib(handled, "partial-doc-list"),
        Vec::<String>::new(),
        "a function's doc"
    );
}

#[test]
fn a_doc_that_tells_an_earlier_design_has_history_words() {
    finds(
        "history-words",
        "//! Formerly the parser crate.\n/// The flag is no longer used.\npub struct Flag;\n",
        &["crate", "Flag"],
        "/// Returns the previously stored value.\npub fn swap() {}\n\
         /// The key used to look up the entry, which no longer exists.\npub struct Key;\n\
         /// The legacy transport of the protocol.\npub struct Sse;\n",
    );
}

#[test]
fn a_doc_that_excuses_its_own_visibility_is_a_lead_unless_the_node_is_private() {
    finds(
        "visibility-excuse",
        "/// Public only so the engine can convert it back.\npub enum Error { A }\n",
        &["Error"],
        "/// Public only so the engine can convert it back.\nenum Error { A }\n\
         /// Valid only for unit variants.\npub struct Unit;\n",
    );
}

#[test]
fn a_doc_made_only_of_its_own_names_echoes_them() {
    finds(
        "doc-echo",
        "pub struct Model {\n/// The default model name.\npub default_model_name: String,\n}\n\
         /// Returns the parsed config file.\npub fn parsed_config(file: &str) -> u8 { 0 }\n",
        &["Model::default_model_name", "parsed_config"],
        "pub struct Model {\n/// The model to fall back on when none matches.\n\
         pub default_model_name: String,\n}\n/// The name.\npub fn name() {}\n\
         /// The `default` model name.\npub fn default_model_name() {}\n",
    );
}

#[test]
fn eight_shared_words_in_two_docs_repeat_text_with_every_site() {
    let docs = |second: &str| {
        format!(
            "/// A prompt without an args key gets the default declaration of one optional \
             string field named `prose`.\npub struct Prompt;\n/// {second}\npub fn args() {{}}\n"
        )
    };
    let shared = docs("Yields the default declaration: one optional string field named `prose`.");
    finds(
        "repeated-doc-text",
        &shared,
        &["args, Prompt"],
        &docs("Yields the default declaration: one optional text field named `prose`."),
    );
    with_leads(&[MANIFEST, ("src/lib.rs", &shared)], |_, leads| {
        let lead = leads.iter().find(|l| l.id == "repeated-doc-text").unwrap();
        assert_eq!(
            lead.evidence,
            "\"default declaration of one optional string field named `prose`\" in 2 docs",
            "the passage as the first site writes it"
        );
    });
}

#[test]
fn an_intra_doc_link_that_reaches_nothing_rests_on_absence() {
    finds(
        "doc-unresolved-link",
        "/// See [`Missing`] and [the walker](crate::walk::Gone).\npub fn run() {}\npub mod walk {}\n",
        &["run"],
        "/// See [`Present`], [`Present::go`], [`std::fmt::Display`], [`Vec`], and [`walk`].\n\
         /// A [default] value, not a link.\npub struct Present;\n\
         impl Present { pub fn go(&self) {} }\npub mod walk {}\n",
    );
    let lib = "/// See [`Missing`].\npub fn run() {}\n";
    with_leads(&[MANIFEST, ("src/lib.rs", lib)], |_, leads| {
        let lead = leads
            .iter()
            .find(|l| l.id == "doc-unresolved-link")
            .unwrap();
        assert_eq!(
            (lead.mode, lead.limits.as_slice()),
            (Mode::Resolved, &["macros"][..])
        );
    });
}

#[test]
fn a_backticked_name_that_no_code_writes_is_unresolved() {
    finds(
        "doc-unresolved-backtick",
        "/// Calls `vanished_helper` and `Gone::away` first.\npub fn run() {}\npub struct Gone;\n",
        &["run"],
        "/// Calls `helper` with `T`, `HashMap`, the `max_retries` key, `step_failed`, and \
         `local_copy`, as `displayName`, `RUST_LOG`, `Outside`, and `word` name them.\n\
         ///\n/// ```ignore\n/// let local_copy = 1;\n/// ```\n\
         ///\n/// A stem such as `made_up_stem`, and a getter named `get_x`.\n\
         pub fn run<T>() { helper(\"max_retries\") }\nfn helper(_key: &str) {}\n\
         pub enum Event { StepFailed }\n\
         #[derive(serde::Deserialize)]\npub struct Wire {\n/// The `total_tokens` key.\npub n: u8,\n}\n",
    );
}

#[test]
fn an_arguments_section_that_names_no_parameter_is_a_lead() {
    let doc = |listed: &str| {
        format!(
            "/// Runs it.\n///\n/// # Arguments\n///\n/// * `count` - how many, such as `Fast`.\n\
             /// * `{listed}` - the cap.\npub fn run(count: u8, _limit: u8) {{}}\n"
        )
    };
    finds("doc-unknown-param", &doc("cap"), &["run"], &doc("limit"));
}

#[test]
fn a_claim_the_written_signature_cannot_hold_is_a_mismatch() {
    finds(
        "doc-signature-mismatch",
        "/// Reads it.\n///\n/// # Errors\n///\n/// Fails when the file is gone.\npub fn read() -> u8 { 0 }\n\
         /// Returns `None` when empty.\npub fn first() -> u8 { 0 }\n\
         /// Greets them; `name` may be `None`.\npub fn greet(name: &str) {}\n",
        &["first", "greet", "read"],
        "pub type Res<T> = Result<T, ()>;\n\
         /// # Errors\n///\n/// Fails.\npub fn read() -> Res<u8> { Ok(0) }\n\
         /// Returns `None` when empty.\npub fn first() -> Option<u8> { None }\n\
         /// Returns `true` if `name` is `None`.\npub fn greet(name: Option<&str>) -> bool { true }\n",
    );
}

#[test]
fn a_written_count_of_variants_or_fields_that_differs_is_a_mismatch() {
    finds(
        "doc-count-mismatch",
        "/// The three section kinds.\npub enum SectionKind { Lua, Prose, Yaml, Toml }\n\
         /// A point of two fields.\npub struct Point { pub x: u8, pub y: u8, pub z: u8 }\n",
        &["Point", "SectionKind"],
        "/// The four section kinds.\npub enum SectionKind { Lua, Prose, Yaml, Toml }\n\
         /// One of at least two forms.\npub enum Form { A, B, C }\n",
    );
}

#[test]
fn a_number_tied_to_a_constant_that_differs_from_its_literal_is_a_mismatch() {
    finds(
        "doc-const-mismatch",
        "/// Retries up to `MAX_RETRIES` (5) times.\npub fn retry() {}\npub const MAX_RETRIES: u32 = 3;\n",
        &["MAX_RETRIES, retry"],
        "/// Retries up to `MAX_RETRIES` (3) times.\npub fn retry() {}\npub const MAX_RETRIES: u32 = 3;\n\
         /// Waits `TIMEOUT_MS` (5 seconds).\npub fn wait() {}\npub const TIMEOUT_MS: u64 = 5_000;\n",
    );
}

#[test]
fn a_collection_doc_that_claims_what_its_type_does_not_keep_is_a_lead() {
    finds(
        "container-invariant-claim",
        "pub struct Index {\n/// The names, sorted, each once.\npub names: Vec<String>,\n}\n",
        &["Index::names"],
        "pub struct Index {\n/// The names, sorted, each once.\n\
         pub names: std::collections::BTreeSet<String>,\n/// The names as written.\npub raw: Vec<String>,\n}\n",
    );
}

#[test]
fn an_error_documented_as_impossible_is_a_lead() {
    finds(
        "impossible-error",
        "pub enum Error {\n/// A state the parser rules out; it cannot occur.\nInternal,\n}\n\
         /// Parses it.\n///\n/// # Errors\n///\n/// Fails on a broken invariant.\npub fn parse() -> Result<(), Error> { Ok(()) }\n",
        &["Error::Internal", "parse"],
        "pub enum Error {\n/// The file cannot be read.\nRead,\n}\n\
         /// It cannot occur.\npub struct Quiet;\n/// Keeps an invariant.\npub fn parse() {}\n",
    );
}

#[test]
fn doc_leads_print_in_every_target_whose_docs_share_text() {
    let doc = "/// Opens the store, replays every pending write, and checks each index before \
               it serves reads.\n";
    let files = [
        MANIFEST,
        ("src/lib.rs", &*format!("{doc}pub fn open() {{}}\n")),
        (
            "src/main.rs",
            &*format!("{doc}fn open() {{}}\nfn main() {{}}\n"),
        ),
    ];
    assert_eq!(found(&files, "repeated-doc-text"), ["0:open, 1:open"]);
}
