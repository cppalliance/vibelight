//! Checks a markdown outline against its schema, for the tests: the stamp
//! line, then one record per line that parses as a [`Record`] with no
//! unknown or missing key and prints back the same, the kinds in their
//! order, sorted lists, leads grouped by id with each subject once, a `doc`
//! record only after the record it belongs to, and module ranges that tile
//! the catalog, each starting at its module's own record. In `Raw` mode
//! every source line is positive; in `NormalizedGolden` mode, as the golden
//! files hold them, every source line is 0 and no location has a link.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde_json::Value;

use crate::model::{name_order, path_order};
use crate::render_slop_schema::{Record, SCHEMA};

/// How an outline's locations are checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// As a run writes it: positive lines, with or without links.
    Raw,
    /// As a golden holds it: every line 0, and no link.
    NormalizedGolden,
}

/// The lists whose entries sort with [`name_order`].
const NAMED: [&str; 6] = ["attributes", "attrs", "cfgs", "derives", "lints", "limits"];
/// The lists whose entries are paths, sorted segment by segment.
const PATHS: [&str; 4] = ["named_by", "reexported_by", "direct_callers", "users"];
/// The lists whose entries sort as plain text.
const PLAIN: [&str; 4] = ["crates", "tests", "ops", "keys"];

/// Checks a whole outline, naming the first line that breaks the schema.
pub(crate) fn check(text: &str, mode: Mode) -> Result<(), String> {
    if text.contains('\r') {
        return Err("the outline holds a CR".to_owned());
    }
    let Some(body) = text.strip_suffix('\n') else {
        return Err("the outline does not end in a line break".to_owned());
    };
    let lines: Vec<&str> = body.split('\n').collect();
    if let Some(n) = lines.iter().position(|line| line.is_empty()) {
        return Err(format!("line {} is blank", n + 1));
    }
    let stamp = lines[0];
    if !(stamp.starts_with("*Updated ") && stamp.ends_with('*')) {
        return Err("line 1 is no stamp line".to_owned());
    }
    let mut records = Vec::new();
    for (n, line) in lines.iter().enumerate().skip(1) {
        let at = n + 1;
        let record: Record =
            serde_json::from_str(line).map_err(|e| format!("line {at} breaks the schema: {e}"))?;
        let again = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        if again != *line {
            return Err(format!(
                "line {at} is not in the schema's form:\n{line}\n{again}"
            ));
        }
        let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        walk(&value, mode).map_err(|why| format!("line {at}: {why}"))?;
        records.push(record);
    }
    order(&records)?;
    leads(&records)?;
    docs(&records)?;
    ranges(&records)
}

/// Returns where a kind prints: its section, with the catalog last.
fn section(record: &Record) -> usize {
    match record {
        Record::Header(_) => 0,
        Record::Modules(_) => 1,
        Record::Lead(_) => 2,
        Record::Surface(_) => 3,
        Record::External(_) => 4,
        Record::Spelling(_) => 5,
        Record::WireKeys(_) => 6,
        _ => 7,
    }
}

/// Checks the header, the modules record, and the order of the sections.
fn order(records: &[Record]) -> Result<(), String> {
    match records.first() {
        Some(Record::Header(header)) => {
            if header.schema != SCHEMA || header.dir.as_deref() == Some(".") {
                return Err("line 2 has another schema or a `.` folder".to_owned());
            }
        }
        _ => return Err("line 2 is no header".to_owned()),
    }
    if !matches!(records.get(1), Some(Record::Modules(_))) {
        return Err("line 3 is no modules record".to_owned());
    }
    for (n, pair) in records.windows(2).enumerate() {
        let (a, b) = (section(&pair[0]), section(&pair[1]));
        if b < a || (b == a && b < 2) {
            return Err(format!("line {} is out of the kinds' order", n + 3));
        }
    }
    Ok(())
}

/// Checks that each lead id's leads are together and that every lead names
/// each subject once and gives a summary's module and count together.
fn leads(records: &[Record]) -> Result<(), String> {
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut last: Option<&str> = None;
    for record in records {
        let Record::Lead(lead) = record else {
            continue;
        };
        if last != Some(lead.id.as_str()) && !done.insert(&lead.id) {
            return Err(format!("the {} leads are not together", lead.id));
        }
        last = Some(&lead.id);
        let unique: BTreeSet<&String> = lead.subjects.iter().collect();
        if unique.len() != lead.subjects.len() || lead.subjects.is_empty() {
            return Err(format!("a {} lead repeats a subject or has none", lead.id));
        }
        if lead.module.is_some() != lead.count.is_some() || lead.count.is_some_and(|c| c <= 5) {
            return Err(format!("a {} summary is malformed", lead.id));
        }
    }
    Ok(())
}

/// Returns a catalog record's path; `None` for any other record.
fn path_of(record: &Record) -> Option<&str> {
    Some(match record {
        Record::Mod(r) => &r.path,
        Record::Struct(r) | Record::Enum(r) | Record::Union(r) => &r.path,
        Record::Trait(r) => &r.path,
        Record::Alias(r) => &r.path,
        Record::Field(r) => &r.path,
        Record::Variant(r) => &r.path,
        Record::Fn(r) => &r.path,
        Record::Const(r) | Record::Static(r) => &r.path,
        Record::Extern(r) => &r.path,
        Record::Use(r) => &r.path,
        Record::Impl(r) => &r.path,
        Record::Macro(r) => &r.path,
        Record::Unsupported(r) => &r.path,
        _ => return None,
    })
}

/// Checks that each `doc` record follows a catalog record of its path.
fn docs(records: &[Record]) -> Result<(), String> {
    for (n, pair) in records.windows(2).enumerate() {
        if let Record::Doc(doc) = &pair[1] {
            let owner = path_of(&pair[0]).filter(|_| !matches!(pair[0], Record::Unsupported(_)));
            if owner != Some(doc.path.as_str()) || doc.text.is_empty() {
                return Err(format!("line {} is a doc after another record", n + 3));
            }
        }
    }
    Ok(())
}

/// Checks that the module ranges are in path order and tile the catalog,
/// each starting at its module's own record, and that every module record
/// starts a range.
fn ranges(records: &[Record]) -> Result<(), String> {
    let Some(Record::Modules(modules)) = records.get(1) else {
        return Err("line 3 is no modules record".to_owned());
    };
    let first = records
        .iter()
        .position(|r| section(r) == 7)
        .unwrap_or(records.len());
    let line = |index: usize| index + 2;
    let mut next = line(first);
    let mut previous: Option<Vec<String>> = None;
    let mut starts = BTreeSet::new();
    for item in &modules.items {
        let path: Vec<String> = item.path.split("::").map(str::to_owned).collect();
        let path = if item.path == "crate" {
            Vec::new()
        } else {
            path
        };
        if previous
            .as_ref()
            .is_some_and(|p| path_order(p, &path) == Ordering::Greater)
        {
            return Err(format!("module {} is out of path order", item.path));
        }
        previous = Some(path);
        let (Some(from), Some(to)) = (item.from, item.to) else {
            if item.from.is_some() || item.to.is_some() {
                return Err(format!("module {} has half a range", item.path));
            }
            continue;
        };
        if from != next || to < from || to > line(records.len() - 1) {
            return Err(format!("module {} has the range {from}..{to}", item.path));
        }
        match &records[from - 2] {
            Record::Mod(own) if own.path == item.path => {}
            _ => return Err(format!("module {}'s range starts elsewhere", item.path)),
        }
        starts.insert(from);
        next = to + 1;
    }
    if next != line(records.len()) {
        return Err(format!("the module ranges end at line {}", next - 1));
    }
    for (index, record) in records.iter().enumerate() {
        if matches!(record, Record::Mod(_)) && !starts.contains(&line(index)) {
            return Err(format!(
                "the module record on line {} starts no range",
                line(index)
            ));
        }
    }
    Ok(())
}

/// Checks every location and list of a record's value.
fn walk(value: &Value, mode: Mode) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            if let Some(line) = map.get("line") {
                let line = line.as_u64().ok_or("a line is no number")?;
                let fine = match mode {
                    Mode::Raw => line > 0,
                    Mode::NormalizedGolden => line == 0 && !map.contains_key("url"),
                };
                if !fine {
                    return Err(format!("a location breaks the {mode:?} rules"));
                }
            }
            for (key, inner) in map {
                if let Value::Array(items) = inner {
                    if items.is_empty() {
                        return Err(format!("`{key}` is an empty list"));
                    }
                    sorted(key, items)?;
                }
                walk(inner, mode)?;
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(|item| walk(item, mode)),
        _ => Ok(()),
    }
}

/// Checks that a list of names is sorted, each entry once.
fn sorted(key: &str, items: &[Value]) -> Result<(), String> {
    let texts: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
    if texts.len() != items.len() {
        return Ok(());
    }
    let segments = |s: &str| -> Vec<String> { s.split("::").map(str::to_owned).collect() };
    let less = |a: &str, b: &str| {
        if NAMED.contains(&key) {
            name_order(a, b) == Ordering::Less
        } else if PATHS.contains(&key) {
            path_order(&segments(a), &segments(b)).then_with(|| a.cmp(b)) == Ordering::Less
        } else if PLAIN.contains(&key) {
            a < b
        } else {
            true
        }
    };
    if texts.windows(2).all(|w| less(w[0], w[1])) {
        Ok(())
    } else {
        Err(format!("`{key}` is not sorted, each entry once"))
    }
}
