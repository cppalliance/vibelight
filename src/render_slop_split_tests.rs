//! Tests the cut of records too wide for one line: a long lead, doc,
//! spelling, enum, function, and modules record each split into lines of
//! at most [`LINE_CAP`] characters, whose `more` records join back into
//! the record, and the cuts stay where they are when the source lines move.

use serde_json::Value;

use crate::render_slop_catalog::{BodyComment, FnRecord, TypeRecord};
use crate::render_slop_check::join;
use crate::render_slop_schema::{
    DocRecord, LeadRecord, Loc, Mode, ModuleRange, Modules, Record, Spelling, SpellingRecord,
};
use crate::render_slop_split::{LINE_CAP, Node, lines, owner};

/// Returns a location in one of a few files, on a line that grows with `n`
/// by `step`.
fn loc(n: u32, step: u32) -> Loc {
    Loc {
        stem: (n % 5 == 0).then(|| "other".to_owned()),
        file: format!("src/module_{}/file.rs", n % 7),
        line: n * step + 1,
    }
}

/// Returns `count` sorted paths.
fn paths(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|n| format!("{prefix}::Item{n:04}"))
        .collect()
}

fn lead(step: u32) -> Record {
    Record::Lead(LeadRecord {
        id: "same-shape".to_owned(),
        mode: Mode::DirectSyntax,
        subjects: paths("shapes", 60),
        evidence: "the same fields in the same order; ".repeat(40),
        locs: Some((0..120).map(|n| loc(n, step)).collect()),
        limits: Some(vec!["macros".to_owned()]),
        module: None,
        count: None,
    })
}

fn doc() -> Record {
    Record::Doc(DocRecord {
        path: "a::B".to_owned(),
        text: "Some \"quoted\" words, a\\b, \u{fc} \u{2713}\ttabs\nand lines. ".repeat(120),
    })
}

fn spelling(step: u32) -> Record {
    let spelled = |text: &str, count: u32| Spelling {
        text: text.to_owned(),
        count: usize::try_from(count).unwrap(),
        locs: (0..count).map(|n| loc(n, step)).collect(),
    };
    Record::Spelling(SpellingRecord {
        path: "std::option::Option".to_owned(),
        preferred: Some("Option".to_owned()),
        spellings: vec![
            spelled("Option", 150),
            spelled("option::Option", 3),
            spelled("std::option::Option", 90),
        ],
    })
}

fn enumeration() -> Record {
    let variants: Vec<String> = (0..200).map(|n| format!("Variant{n}(u32)")).collect();
    Record::Enum(TypeRecord {
        path: "Big".to_owned(),
        rust: format!("pub enum Big {{ {} }}", variants.join(", ")),
        loc: loc(1, 1),
        named_by: Some(paths("users", 60)),
        ..TypeRecord::default()
    })
}

fn function(step: u32) -> Record {
    let comment = |n: u32| BodyComment {
        text: format!("// step {n}: {}", "checks the input ".repeat(4)),
        line: n * step + 1,
    };
    Record::Fn(FnRecord {
        path: "hardening::harden".to_owned(),
        sig: "pub fn harden(lua: &Lua) -> Result<()>".to_owned(),
        loc: loc(1, step),
        ops: Some(vec![
            format!("lua.load(r#\"{}\"#).exec", "local x = 1\n".repeat(120)),
            "lua.set".to_owned(),
        ]),
        comments: Some((0..30).map(comment).collect()),
        ..FnRecord::default()
    })
}

fn modules() -> Record {
    Record::Modules(Modules {
        items: (0..300)
            .map(|n| ModuleRange {
                path: format!("module_{n:03}"),
                from: Some(10 * n + 4),
                to: Some(10 * n + 13),
            })
            .collect(),
    })
}

/// Splits a record, checks that every line fits and every `more` record
/// carries the next part, joins the parts, and returns the lines.
fn split(record: &Record) -> Vec<String> {
    let whole = serde_json::to_string(record).unwrap();
    let lines = lines(record).unwrap();
    assert!(lines.len() > 1, "{whole}");
    let mut joined: Node = serde_json::from_str(&lines[0]).unwrap();
    let named = owner(&joined);
    assert_eq!(named, owner(&serde_json::from_str(&whole).unwrap()));
    for (n, line) in lines.iter().enumerate() {
        assert!(line.chars().count() <= LINE_CAP, "{line}");
        let parsed: Record = serde_json::from_str(line).unwrap();
        match parsed {
            Record::More(more) => {
                assert!(n > 0, "the owner comes first");
                assert_eq!(more.part, n + 1, "{line}");
                assert_eq!((more.path.clone(), more.id.clone()), named, "{line}");
                join(&mut joined, &more).unwrap();
            }
            _ => assert_eq!(n, 0, "{line}"),
        }
    }
    assert_eq!(serde_json::to_string(&joined).unwrap(), whole);
    let original: Value = serde_json::from_str(&whole).unwrap();
    assert_eq!(serde_json::to_value(&joined).unwrap(), original);
    lines
}

/// Returns the parts' fields, in order.
fn fields(lines: &[String]) -> Vec<String> {
    let field = |line: &String| {
        let value: Value = serde_json::from_str(line).unwrap();
        value["field"].as_str().unwrap_or("").to_owned()
    };
    lines.iter().skip(1).map(field).collect()
}

#[test]
fn a_long_lead_continues_under_its_first_subject_and_id() {
    let lines = split(&lead(13));
    let open = "{\"k\":\"more\",\"path\":\"shapes::Item0000\",\"id\":\"same-shape\",\"field\":";
    assert!(
        lines[1..].iter().all(|line| line.starts_with(open)),
        "{lines:?}"
    );
    let fields = fields(&lines);
    let cut = ["subjects", "evidence", "locs"];
    for field in cut {
        assert!(
            fields.iter().any(|f| f == field),
            "no {field} in {fields:?}"
        );
    }
    assert!(
        fields.iter().all(|f| cut.contains(&f.as_str())),
        "{fields:?}"
    );
}

#[test]
fn a_long_doc_continues_as_text_cut_at_whitespace() {
    let lines = split(&doc());
    assert!(fields(&lines).iter().all(|f| f == "text"));
    let owner: Value = serde_json::from_str(&lines[0]).unwrap();
    let head = owner["text"].as_str().unwrap();
    assert!(head.ends_with(char::is_whitespace), "{head:?}");
}

#[test]
fn a_spelling_whose_element_is_too_wide_cuts_that_element() {
    let lines = split(&spelling(13));
    let fields = fields(&lines);
    for field in ["spellings.0.locs", "spellings", "spellings.2.locs"] {
        assert!(
            fields.iter().any(|f| f == field),
            "no {field} in {fields:?}"
        );
    }
    let first = fields.iter().position(|f| f == "spellings.0.locs");
    let element = fields.iter().position(|f| f == "spellings");
    assert!(first < element, "an element's rest comes right after it");
}

#[test]
fn an_enum_cuts_its_shape_and_its_lists_and_a_modules_record_its_ranges() {
    let lines = split(&enumeration());
    let fields = fields(&lines);
    assert!(fields.iter().any(|f| f == "rust"), "{fields:?}");
    assert!(fields.iter().any(|f| f == "named_by"), "{fields:?}");
    let lines = split(&modules());
    assert!(
        lines[1]
            .starts_with("{\"k\":\"more\",\"path\":\"modules\",\"field\":\"items\",\"part\":2,")
    );
}

#[test]
fn a_function_cuts_a_string_element_too_wide_for_a_line_and_its_comments() {
    let lines = split(&function(13));
    let fields = fields(&lines);
    for field in ["ops.0", "ops", "comments"] {
        assert!(
            fields.iter().any(|f| f == field),
            "no {field} in {fields:?}"
        );
    }
    let element = fields.iter().position(|f| f == "ops.0");
    let rest = fields.iter().position(|f| f == "ops");
    assert!(element < rest, "an element's rest comes right after it");
}

/// Returns each line's kind, field, and part, and how many items or
/// characters of text it carries.
fn shape(lines: &[String]) -> Vec<(String, String, u64, usize)> {
    let shape = |line: &String| {
        let value: Value = serde_json::from_str(line).unwrap();
        let text = |key: &str| value[key].as_str().unwrap_or("").to_owned();
        let size = value["items"]
            .as_array()
            .map_or_else(|| text("text").chars().count(), Vec::len);
        (
            text("k"),
            text("field"),
            value["part"].as_u64().unwrap_or(1),
            size,
        )
    };
    lines.iter().map(shape).collect()
}

#[test]
fn moving_the_source_lines_moves_no_cut() {
    let pairs = [
        (lead(1), lead(997)),
        (spelling(1), spelling(1999)),
        (function(1), function(9973)),
    ];
    for (near, far) in pairs {
        assert_eq!(shape(&split(&near)), shape(&split(&far)));
    }
}
