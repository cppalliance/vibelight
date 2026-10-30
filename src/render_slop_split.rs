//! Keeps every line of an outline within [`LINE_CAP`] characters, since
//! some tools cut longer lines short. A record that would be wider keeps
//! the fields that name it and as much of its widest field as fits, and
//! `more` records right after it carry the rest. Each names the owner by
//! its `path`, or a lead by its first subject and its `id`, names the field
//! it continues in `field`, counts its `part` from 2, the owner being part
//! 1, and carries either `items`, whole elements of an array, or `text`, a
//! piece of a string. An element too wide for a line of its own is cut the
//! same way, so `field` may name a field of an element, as in
//! `spellings.3.locs`, or a string element too wide to share a line, as in
//! `ops.2`. A location and a lead's subject are never cut.

use std::cmp::Reverse;
use std::fmt;
use std::mem;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::render_slop_schema::{MoreRecord, Record};

/// The widest line an outline writes, in characters.
pub(crate) const LINE_CAP: usize = 900;

/// How many digits a source line counts as, at least, when a record is
/// measured, so where a record is cut never depends on where the source
/// puts a declaration.
const LINE_DIGITS: usize = 6;

/// The keys no cut touches: those that name a record or a part, and a
/// declaration's location.
const FIXED: [&str; 6] = ["k", "path", "id", "field", "part", "loc"];

/// A JSON value that keeps its keys in the order they print.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Node {
    Text(String),
    /// A number, a boolean, or null.
    Scalar(Value),
    List(Vec<Node>),
    Map(Vec<(String, Node)>),
}

impl Node {
    /// Returns the value of `key`, when this is a map that has it.
    pub(crate) fn get(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Returns the text of a string.
    pub(crate) fn text(&self) -> Option<&str> {
        match self {
            Node::Text(text) => Some(text),
            _ => None,
        }
    }
}

impl Serialize for Node {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Node::Text(text) => serializer.serialize_str(text),
            Node::Scalar(value) => value.serialize(serializer),
            Node::List(items) => serializer.collect_seq(items),
            Node::Map(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Node, D::Error> {
        deserializer.deserialize_any(NodeVisitor)
    }
}

/// Reads a [`Node`], keeping its keys in the order they come.
struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Node, E> {
        Ok(Node::Scalar(Value::Bool(v)))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Node, E> {
        Ok(Node::Scalar(Value::from(v)))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Node, E> {
        Ok(Node::Scalar(Value::from(v)))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Node, E> {
        Ok(Node::Scalar(Value::from(v)))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Node, E> {
        Ok(Node::Text(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Node, E> {
        Ok(Node::Text(v))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Node, E> {
        Ok(Node::Scalar(Value::Null))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Node, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Node::List(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Node, A::Error> {
        let mut entries = Vec::new();
        while let Some(entry) = map.next_entry()? {
            entries.push(entry);
        }
        Ok(Node::Map(entries))
    }
}

/// Returns what names a record's `more` records: its `path`, a lead's
/// first subject, or else its kind; and its `id`, when it has one.
pub(crate) fn owner(record: &Node) -> (String, Option<String>) {
    let text = |key: &str| record.get(key).and_then(Node::text).map(str::to_owned);
    let subject = match record.get("subjects") {
        Some(Node::List(items)) => items.first().and_then(Node::text).map(str::to_owned),
        _ => None,
    };
    let path = text("path").or(subject).or_else(|| text("k"));
    (path.unwrap_or_default(), text("id"))
}

/// Returns how many characters one character of a string prints in.
fn escaped(c: char) -> usize {
    match c {
        '"' | '\\' | '\u{8}' | '\t' | '\n' | '\u{c}' | '\r' => 2,
        c if c < ' ' => 6,
        _ => 1,
    }
}

/// Returns how many characters a string prints in, with its quotes.
fn quoted(text: &str) -> usize {
    2 + text.chars().map(escaped).sum::<usize>()
}

/// Returns how many characters `node` prints in as the value of `key`, or
/// as an element of the array under `key`, with each source line counted
/// as at least [`LINE_DIGITS`] digits.
fn width(node: &Node, key: &str) -> usize {
    match node {
        Node::Text(text) if key == "loc" || key == "locs" => {
            let digits = text.rsplit(':').next().map_or(0, str::len);
            quoted(text) + LINE_DIGITS.saturating_sub(digits)
        }
        Node::Text(text) => quoted(text),
        Node::Scalar(value) if key == "line" => value.to_string().len().max(LINE_DIGITS),
        Node::Scalar(value) => value.to_string().len(),
        Node::List(items) => {
            1 + items
                .iter()
                .map(|item| width(item, key) + 1)
                .sum::<usize>()
                .max(1)
        }
        Node::Map(entries) => map_width(entries),
    }
}

/// Returns how many characters a map prints in.
fn map_width(entries: &[(String, Node)]) -> usize {
    let inside: usize = (entries.iter())
        .map(|(key, value)| quoted(key) + 1 + width(value, key) + 1)
        .sum();
    1 + inside.max(1)
}

/// Tells whether a cut may shorten the value of `key` in a map.
fn cuttable(key: &str, value: &Node) -> bool {
    !FIXED.contains(&key) && !matches!(value, Node::Scalar(_))
}

/// Tells whether a cut may shorten an element of the array under `key`: a
/// map, or a string too wide to share a line, but never a location or a
/// subject, since a lead's first subject names its parts.
fn divisible(item: &Node, key: &str) -> bool {
    match item {
        Node::Map(_) => true,
        Node::Text(_) => !["locs", "subjects"].contains(&key) && width(item, key) > LINE_CAP / 2,
        Node::Scalar(_) | Node::List(_) => false,
    }
}

/// Returns how few characters `node`, the value of `key`, prints in once
/// cut as far as [`shrink`] cuts.
fn narrowest(node: &Node, key: &str) -> usize {
    match node {
        Node::Text(text) => 2 + text.chars().next().map_or(0, escaped),
        Node::List(items) => match items.first() {
            Some(first) if divisible(first, key) => 2 + narrowest(first, key),
            Some(first) => 2 + width(first, key),
            None => 2,
        },
        Node::Map(entries) => {
            let inside: usize = (entries.iter())
                .map(|(k, v)| {
                    let value = if cuttable(k, v) {
                        narrowest(v, k)
                    } else {
                        width(v, k)
                    };
                    quoted(k) + 1 + value + 1
                })
                .sum();
            1 + inside.max(1)
        }
        Node::Scalar(_) => width(node, key),
    }
}

/// Returns where to cut `text` so that its head prints in `room`
/// characters, quotes left out: after the last whitespace in the head's
/// second half when it has one. The head keeps at least one character.
fn cut(text: &str, room: usize) -> usize {
    let mut used = 0;
    let mut end = 0;
    let mut space = 0;
    for (at, c) in text.char_indices() {
        used += escaped(c);
        if used > room && end > 0 {
            return if space * 2 > end { space } else { end };
        }
        end = at + c.len_utf8();
        if c.is_whitespace() {
            space = end;
        }
    }
    end
}

/// What a cut took off a field: the rest of an array or of a string.
struct Rest {
    /// The field's path from the owner, its steps joined with dots.
    field: String,
    /// Where `Body::Items` starts in the whole array.
    start: usize,
    body: Body,
}

enum Body {
    Items(Vec<Node>),
    Text(String),
}

/// Joins a field path and one more step.
fn step(field: &str, next: &str) -> String {
    if field.is_empty() {
        next.to_owned()
    } else {
        format!("{field}.{next}")
    }
}

/// Cuts `node`, the value of `key` at `field` below the owner, until it
/// prints in `room` characters or nothing is left to cut, and pushes what
/// it takes off to `rests`, field by field in the map's order. A map cuts
/// its widest field first, which keeps as much as fits beside the others,
/// or half the room when cutting the others makes that much; then its next
/// widest fields, as far as they must. A string keeps at least one
/// character and an array at least one element, which is cut itself when
/// it alone is too wide and [`divisible`].
fn shrink(node: &mut Node, key: &str, room: usize, field: &str, rests: &mut Vec<Rest>) {
    if width(node, key) <= room {
        return;
    }
    match node {
        Node::Text(text) => {
            let rest = text.split_off(cut(text, room.saturating_sub(2)));
            if !rest.is_empty() {
                let field = field.to_owned();
                rests.push(Rest {
                    field,
                    start: 0,
                    body: Body::Text(rest),
                });
            }
        }
        Node::List(items) => {
            let mut used = 2;
            let mut keep = 0;
            for item in items.iter() {
                let more = width(item, key) + usize::from(keep > 0);
                if keep > 0 && used + more > room {
                    break;
                }
                used += more;
                keep += 1;
            }
            let tail = items.split_off(keep);
            let first = items.first_mut().filter(|first| divisible(first, key));
            if let Some(first) = first.filter(|_| used > room) {
                shrink(first, key, room.saturating_sub(2), &step(field, "0"), rests);
            }
            if !tail.is_empty() {
                let field = field.to_owned();
                rests.push(Rest {
                    field,
                    start: keep,
                    body: Body::Items(tail),
                });
            }
        }
        Node::Map(entries) => {
            let mut done: Vec<(usize, Vec<Rest>)> = Vec::new();
            loop {
                let total = map_width(entries);
                if total <= room {
                    break;
                }
                let need = total - room;
                let open: Vec<usize> = (0..entries.len())
                    .filter(|&i| done.iter().all(|(j, _)| *j != i))
                    .filter(|&i| cuttable(&entries[i].0, &entries[i].1))
                    .collect();
                let wide = |i: usize| width(&entries[i].1, &entries[i].0);
                let Some(&i) = open.iter().max_by_key(|&&i| (wide(i), Reverse(i))) else {
                    break;
                };
                let mut keep = wide(i).saturating_sub(need);
                if done.is_empty() {
                    let spare: usize = (open.iter().filter(|&&j| j != i))
                        .map(|&j| wide(j).saturating_sub(narrowest(&entries[j].1, &entries[j].0)))
                        .sum();
                    let fair = wide(i).saturating_sub(need.saturating_sub(spare));
                    keep = keep.max(fair.min(room / 2));
                }
                let (k, v) = &mut entries[i];
                let mut taken = Vec::new();
                shrink(v, k, keep, &step(field, k), &mut taken);
                done.push((i, taken));
            }
            done.sort_by_key(|(i, _)| *i);
            rests.extend(done.into_iter().flat_map(|(_, taken)| taken));
        }
        Node::Scalar(_) => {}
    }
}

/// The lines of one record as they are written: the owner, then its parts.
struct Parts {
    path: String,
    id: Option<String>,
    /// The number of the last part written, the owner being part 1.
    part: usize,
    lines: Vec<String>,
}

impl Parts {
    /// Returns the next part, which continues `field` with `body`.
    fn record(&self, field: &str, body: Body) -> Record {
        let (items, text) = match body {
            Body::Items(items) => (Some(items), None),
            Body::Text(text) => (None, Some(text)),
        };
        Record::More(MoreRecord {
            path: self.path.clone(),
            id: self.id.clone(),
            field: field.to_owned(),
            part: self.part + 1,
            items,
            text,
        })
    }

    /// Returns how wide the next part is with nothing in it.
    fn frame(&self, field: &str, text: bool) -> serde_json::Result<usize> {
        let empty = if text {
            Body::Text(String::new())
        } else {
            Body::Items(Vec::new())
        };
        Ok(serde_json::to_string(&self.record(field, empty))?
            .chars()
            .count())
    }

    /// Writes the next part, which continues `field` with `body`.
    fn push(&mut self, field: &str, body: Body) -> serde_json::Result<()> {
        let line = serde_json::to_string(&self.record(field, body))?;
        self.lines.push(line);
        self.part += 1;
        Ok(())
    }

    /// Writes `rest` in as few parts as fit, cutting each element that is
    /// too wide for a part of its own and writing its rest right after it.
    fn write(&mut self, rest: Rest) -> serde_json::Result<()> {
        let field = rest.field;
        match rest.body {
            Body::Text(text) => {
                let mut left = text.as_str();
                while !left.is_empty() {
                    let room = LINE_CAP.saturating_sub(self.frame(&field, true)?);
                    let (head, tail) = left.split_at(cut(left, room));
                    self.push(&field, Body::Text(head.to_owned()))?;
                    left = tail;
                }
            }
            Body::Items(items) => {
                let key = field.rsplit('.').next().unwrap_or_default().to_owned();
                let mut chunk: Vec<Node> = Vec::new();
                let mut used = 0;
                for (index, mut item) in (rest.start..).zip(items) {
                    let wide = width(&item, &key);
                    if !chunk.is_empty() && self.frame(&field, false)? + used + 1 + wide > LINE_CAP
                    {
                        self.push(&field, Body::Items(mem::take(&mut chunk)))?;
                        used = 0;
                    }
                    let room = LINE_CAP.saturating_sub(self.frame(&field, false)?);
                    if chunk.is_empty() && wide > room {
                        let mut nested = Vec::new();
                        if divisible(&item, &key) {
                            let at = step(&field, &index.to_string());
                            shrink(&mut item, &key, room, &at, &mut nested);
                        }
                        self.push(&field, Body::Items(vec![item]))?;
                        for rest in nested {
                            self.write(rest)?;
                        }
                    } else {
                        used += wide + usize::from(!chunk.is_empty());
                        chunk.push(item);
                    }
                }
                if !chunk.is_empty() {
                    self.push(&field, Body::Items(chunk))?;
                }
            }
        }
        Ok(())
    }
}

/// Writes a record as lines of at most [`LINE_CAP`] characters: the
/// record itself when it fits, and otherwise the record cut down, then the
/// `more` records that carry the rest.
///
/// # Errors
/// Returns the error of a record that cannot be written as JSON.
pub(crate) fn lines(record: &Record) -> serde_json::Result<Vec<String>> {
    let line = serde_json::to_string(record)?;
    // Counting source lines as wider at most doubles a line's width.
    if line.len() * 2 <= LINE_CAP {
        return Ok(vec![line]);
    }
    let mut node: Node = serde_json::from_str(&line)?;
    if width(&node, "") <= LINE_CAP {
        return Ok(vec![line]);
    }
    let (path, id) = owner(&node);
    let mut rests = Vec::new();
    shrink(&mut node, "", LINE_CAP, "", &mut rests);
    let mut parts = Parts {
        path,
        id,
        part: 1,
        lines: vec![serde_json::to_string(&node)?],
    };
    for rest in rests {
        parts.write(rest)?;
    }
    Ok(parts.lines)
}
