//! Checks a markdown outline against its schema, for the tests: the stamp
//! line, then one record per line of at most [`LINE_CAP`] characters that
//! parses as a [`Record`] with no unknown or missing key and prints back
//! the same. Each `more` record follows its owner or the owner's last
//! part, with the next part number, and adds to a field the owner has; the
//! owner joined with its parts must meet the schema too. The joined records
//! must hold their kinds in order, sorted lists, leads grouped by id with
//! each subject once, a `doc` record only after the record it belongs to,
//! and module ranges that tile the catalog's lines, each starting at its
//! module's own record. No record has a `url`, and every location is a
//! string in its printed form. In `Raw` mode every source line is positive
//! and the header has its `root`; in `NormalizedGolden` mode, as the golden
//! files hold them, every source line is 0 and the header has no `root`.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde_json::Value;

use crate::model::{name_order, path_order};
use crate::render_slop_schema::{Loc, MoreRecord, Record, SCHEMA};
use crate::render_slop_split::{LINE_CAP, Node, owner};

/// How an outline's locations are checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// As a run writes it: positive lines, and the header's `root`.
    Raw,
    /// As a golden holds it: every line 0, and no `root`.
    NormalizedGolden,
}

/// The lists whose entries sort with [`name_order`].
const NAMED: [&str; 6] = ["attributes", "attrs", "cfgs", "derives", "lints", "limits"];
/// The lists whose entries are paths, sorted segment by segment.
const PATHS: [&str; 4] = ["named_by", "reexported_by", "direct_callers", "users"];
/// The lists whose entries sort as plain text.
const PLAIN: [&str; 4] = ["crates", "tests", "ops", "keys"];

/// A record with the parts that continue it, joined.
struct Joined {
    node: Node,
    /// The output line the record starts on.
    start: usize,
    /// The number of its last part, the record itself being part 1.
    part: usize,
}

/// Adds a part to the record it continues: the part's items to the array,
/// or its text to the string, that its `field` names.
///
/// # Errors
/// Names the field when the record has no such array or string, or the
/// part carries no items or text for it.
pub(crate) fn join(record: &mut Node, more: &MoreRecord) -> Result<(), String> {
    let field = &more.field;
    let mut at = record;
    for step in field.split('.') {
        let next = match at {
            Node::Map(entries) => (entries.iter_mut())
                .find(|(key, _)| key == step)
                .map(|(_, value)| value),
            Node::List(items) => step.parse().ok().and_then(|i: usize| items.get_mut(i)),
            Node::Text(_) | Node::Scalar(_) => None,
        };
        at = next.ok_or_else(|| format!("`{field}` names no field of the record"))?;
    }
    match (at, &more.items, &more.text) {
        (Node::List(list), Some(items), None) if !items.is_empty() => list.extend(items.clone()),
        (Node::Text(text), None, Some(more)) if !more.is_empty() => text.push_str(more),
        _ => return Err(format!("the part does not continue `{field}`")),
    }
    Ok(())
}

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
    let mut joined: Vec<Joined> = Vec::new();
    for (n, line) in lines.iter().enumerate().skip(1) {
        let at = n + 1;
        if line.chars().count() > LINE_CAP {
            return Err(format!("line {at} is wider than {LINE_CAP} characters"));
        }
        let record = parse(line).map_err(|why| format!("line {at} {why}"))?;
        let node: Node = serde_json::from_str(line).map_err(|e| e.to_string())?;
        let Record::More(next) = record else {
            joined.push(Joined {
                node,
                start: at,
                part: 1,
            });
            continue;
        };
        let last = (joined.last_mut()).ok_or_else(|| format!("line {at} continues nothing"))?;
        if owner(&last.node) != (next.path.clone(), next.id.clone()) || next.part != last.part + 1 {
            return Err(format!("line {at} is no next part of the record above it"));
        }
        join(&mut last.node, &next).map_err(|why| format!("line {at}: {why}"))?;
        last.part += 1;
    }
    let mut records = Vec::new();
    let mut starts = Vec::new();
    for part in &joined {
        let line = serde_json::to_string(&part.node).map_err(|e| e.to_string())?;
        let at = part.start;
        let record =
            parse(&line).map_err(|why| format!("the record on line {at}, joined, {why}"))?;
        let value: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        walk(&value, mode).map_err(|why| format!("line {at}: {why}"))?;
        records.push(record);
        starts.push(at);
    }
    order(&records, &starts, mode)?;
    leads(&records)?;
    docs(&records, &starts)?;
    ranges(&records, &starts, lines.len())
}

/// Reads one line as a record that prints back the same.
fn parse(line: &str) -> Result<Record, String> {
    let record: Record =
        serde_json::from_str(line).map_err(|e| format!("breaks the schema: {e}"))?;
    let again = serde_json::to_string(&record).map_err(|e| e.to_string())?;
    if again == line {
        Ok(record)
    } else {
        Err(format!("is not in the schema's form:\n{line}\n{again}"))
    }
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
fn order(records: &[Record], starts: &[usize], mode: Mode) -> Result<(), String> {
    match records.first() {
        Some(Record::Header(header)) => {
            if header.schema != SCHEMA || header.dir.as_deref() == Some(".") {
                return Err("line 2 has another schema or a `.` folder".to_owned());
            }
            let root = header.root.as_deref();
            let fine = match mode {
                Mode::Raw => root.is_some_and(|r| !r.is_empty() && !r.contains('\\')),
                Mode::NormalizedGolden => root.is_none(),
            };
            if !fine {
                return Err(format!("line 2 breaks the {mode:?} rules for `root`"));
            }
        }
        _ => return Err("line 2 is no header".to_owned()),
    }
    if !matches!(records.get(1), Some(Record::Modules(_))) || starts.get(1) != Some(&3) {
        return Err("line 3 is no modules record".to_owned());
    }
    for (n, pair) in records.windows(2).enumerate() {
        let (a, b) = (section(&pair[0]), section(&pair[1]));
        if b < a || (b == a && b < 2) {
            return Err(format!("line {} is out of the kinds' order", starts[n + 1]));
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
fn docs(records: &[Record], starts: &[usize]) -> Result<(), String> {
    for (n, pair) in records.windows(2).enumerate() {
        if let Record::Doc(doc) = &pair[1] {
            let owner = path_of(&pair[0]).filter(|_| !matches!(pair[0], Record::Unsupported(_)));
            if owner != Some(doc.path.as_str()) || doc.text.is_empty() {
                return Err(format!(
                    "line {} is a doc after another record",
                    starts[n + 1]
                ));
            }
        }
    }
    Ok(())
}

/// Checks that the module ranges are in path order and tile the catalog's
/// lines, `more` records included, up to the last line, each starting at
/// its module's own record, and that every module record starts a range.
fn ranges(records: &[Record], starts: &[usize], last: usize) -> Result<(), String> {
    let Some(Record::Modules(modules)) = records.get(1) else {
        return Err("line 3 is no modules record".to_owned());
    };
    let first = records.iter().position(|r| section(r) == 7);
    let mut next = first.map_or(last + 1, |i| starts[i]);
    let mut previous: Option<Vec<String>> = None;
    let mut begun = BTreeSet::new();
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
        if from != next || to < from || to > last {
            return Err(format!("module {} has the range {from}..{to}", item.path));
        }
        match starts.binary_search(&from).map(|i| &records[i]) {
            Ok(Record::Mod(own)) if own.path == item.path => {}
            _ => return Err(format!("module {}'s range starts elsewhere", item.path)),
        }
        begun.insert(from);
        next = to + 1;
    }
    if next != last + 1 {
        return Err(format!("the module ranges end at line {}", next - 1));
    }
    for (record, &at) in records.iter().zip(starts) {
        if matches!(record, Record::Mod(_)) && !begun.contains(&at) {
            return Err(format!("the module record on line {at} starts no range"));
        }
    }
    Ok(())
}

/// Checks a location string against the mode's rule for its line.
fn location(text: Option<&str>, mode: Mode) -> Result<(), String> {
    let loc = text
        .and_then(Loc::parse)
        .ok_or("a location is no location string")?;
    let fine = match mode {
        Mode::Raw => loc.line > 0,
        Mode::NormalizedGolden => loc.line == 0,
    };
    if fine {
        Ok(())
    } else {
        Err(format!("the location {loc} breaks the {mode:?} rules"))
    }
}

/// Checks every location and list of a record's value.
fn walk(value: &Value, mode: Mode) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            if map.contains_key("url") {
                return Err("a record has a `url`".to_owned());
            }
            if let Some(line) = map.get("line") {
                let line = line.as_u64().ok_or("a line is no number")?;
                if (mode == Mode::Raw) != (line > 0) {
                    return Err(format!("a comment line breaks the {mode:?} rules"));
                }
            }
            if let Some(loc) = map.get("loc") {
                location(loc.as_str(), mode)?;
            }
            if let Some(locs) = map.get("locs") {
                let locs = locs.as_array().ok_or("`locs` is no list")?;
                locs.iter()
                    .try_for_each(|loc| location(loc.as_str(), mode))?;
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
