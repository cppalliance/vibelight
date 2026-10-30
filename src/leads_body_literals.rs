//! The detectors of literals and doc weight: string and number literals
//! that several bodies repeat, macro arguments included, and functions
//! whose doc is far longer than their code. Literals that one set of
//! bodies shares make one lead, so a table that two functions both spell
//! out is one lead rather than one per entry.

use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::ItemId;
use crate::leads::{Lead, Mode};
use crate::leads_docs_text::{boilerplate, without};
use crate::model_body::{Body, BodyKind};

/// The fewest characters of a string literal that `repeated-literal`
/// compares, format placeholders such as `{name:?}` left out.
const LITERAL_CHARS: usize = 8;

/// The fewest bodies that one string literal is in, or the fewest targets.
const LITERAL_BODIES: usize = 3;
const LITERAL_TARGETS: usize = 2;

/// The most characters of a literal that evidence shows.
const LITERAL_SHOWN: usize = 60;

/// The most literals of one lead that evidence shows.
const LITERALS_SHOWN: usize = 4;

/// The fewest function bodies that one number is in.
const NUMBER_BODIES: usize = 3;

/// The magnitude below which `repeated-number` leaves a number out, since
/// small numbers such as a count of three repeat by chance.
const NUMBER_FROM: f64 = 10.0;

/// The round numbers that `repeated-number` leaves out, such as a buffer
/// size or a percent base; a sign is not kept.
const ROUND_NUMBERS: [&str; 9] = ["16", "32", "64", "100", "128", "256", "512", "1000", "1024"];

/// The fewest doc words, outside code blocks, of a `doc-outweighs-body`
/// lead, and how many times the body's canonical tokens they exceed.
const DOC_WORDS: usize = 100;
const DOC_TIMES: usize = 3;

/// The items whose bodies share some literals, each once.
type Sites = BTreeSet<ItemId>;

pub(crate) fn find(run: &AnalysisRun<'_>, out: &mut Vec<Lead>) {
    let bodies = bodies(run);
    repeated_literal(run, &bodies, out);
    repeated_number(run, &bodies, out);
    doc_outweighs_body(run, &bodies, out);
}

/// Returns each body of an item, with the item, in target and body order.
fn bodies<'a>(run: &AnalysisRun<'a>) -> Vec<(ItemId, &'a Body)> {
    let mut out = Vec::new();
    for (t, krate) in run.krates.iter().enumerate() {
        for (b, body) in krate.bodies.iter().enumerate() {
            if let Some(item) = run.body_items[t][b] {
                out.push((ItemId { target: t, item }, body));
            }
        }
    }
    out
}

/// Tells whether a body is a function's.
fn is_function(body: &Body) -> bool {
    matches!(
        body.kind,
        BodyKind::Fn | BodyKind::Method | BodyKind::TraitImplMethod | BodyKind::TraitDefault
    )
}

/// Returns, for each set of items whose bodies share literals, those
/// literals in order.
fn by_sites<'a>(sites: BTreeMap<&'a str, Sites>) -> BTreeMap<Sites, Vec<&'a str>> {
    let mut out: BTreeMap<Sites, Vec<&'a str>> = BTreeMap::new();
    for (literal, items) in sites {
        out.entry(items).or_default().push(literal);
    }
    out
}

/// Builds a lead whose subjects are `items`, with evidence that names the
/// first of `literals`, each as `show` writes it, and the bodies.
fn shared_lead(
    run: &AnalysisRun<'_>,
    id: &'static str,
    literals: &[&str],
    items: &Sites,
    show: impl Fn(&str) -> String,
) -> Lead {
    let shown: Vec<String> = literals
        .iter()
        .take(LITERALS_SHOWN)
        .map(|l| show(l))
        .collect();
    let more = literals.len().saturating_sub(LITERALS_SHOWN);
    let listed = if more == 0 {
        shown.join(", ")
    } else {
        format!("{} and {more} more", shown.join(", "))
    };
    let targets: BTreeSet<usize> = items.iter().map(|item| item.target).collect();
    let place = match targets.len() {
        1 => format!("{} bodies", items.len()),
        n => format!("{} bodies of {n} targets", items.len()),
    };
    let evidence = match literals.len() {
        1 => format!("{listed} in {place}"),
        n => format!("{n} literals in {place}: {listed}"),
    };
    let mut lead = Lead::new(id, Mode::DirectSyntax, evidence);
    for &item in items {
        lead = lead.item(run, item);
    }
    lead
}

/// Counts the characters of a string outside its format placeholders: a
/// brace pair around nothing but a name and a format spec.
fn plain_chars(text: &str) -> usize {
    let spec = |c: char| c.is_alphanumeric() || "_.:?#<>^+-$".contains(c);
    let mut count = 0;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let placeholder = rest.strip_prefix('{').and_then(|inner| {
            let end = inner.find(|c: char| !spec(c))?;
            inner[end..].starts_with('}').then_some(end + 2)
        });
        if let Some(len) = placeholder {
            rest = &rest[len..];
        } else {
            count += 1;
            rest = &rest[c.len_utf8()..];
        }
    }
    count
}

/// Shows a string literal as Rust writes it, cut at [`LITERAL_SHOWN`]
/// characters.
fn show_string(text: &str) -> String {
    let mut shown: String = text.chars().take(LITERAL_SHOWN).collect();
    let cut = shown.len() < text.len();
    if cut {
        shown.push_str("...");
    }
    format!("{shown:?}")
}

/// `repeated-literal`: a string literal of [`LITERAL_CHARS`] or more
/// characters outside its format placeholders that [`LITERAL_BODIES`] or
/// more bodies of items hold, or bodies in [`LITERAL_TARGETS`] or more
/// targets. Literals that the same bodies hold make one lead.
fn repeated_literal(run: &AnalysisRun<'_>, bodies: &[(ItemId, &Body)], out: &mut Vec<Lead>) {
    let mut sites: BTreeMap<&str, Sites> = BTreeMap::new();
    for &(item, body) in bodies {
        for text in &body.facts.strings {
            if plain_chars(text) >= LITERAL_CHARS {
                sites.entry(text).or_default().insert(item);
            }
        }
    }
    for (items, literals) in by_sites(sites) {
        let targets: BTreeSet<usize> = items.iter().map(|item| item.target).collect();
        if items.len() >= LITERAL_BODIES || targets.len() >= LITERAL_TARGETS {
            out.push(shared_lead(
                run,
                "repeated-literal",
                &literals,
                &items,
                show_string,
            ));
        }
    }
}

/// Returns a number's digits without a fraction of zeros, such as `2` for
/// `2.0`.
fn plain_value(digits: &str) -> &str {
    match digits.split_once('.') {
        Some((whole, fraction)) if fraction.bytes().all(|b| b == b'0') => whole,
        _ => digits,
    }
}

/// Tells whether `repeated-number` compares a number: one of magnitude
/// [`NUMBER_FROM`] or more that is none of [`ROUND_NUMBERS`].
fn telling(value: &str) -> bool {
    let large = value.parse::<f64>().is_ok_and(|n| n >= NUMBER_FROM);
    large && !ROUND_NUMBERS.contains(&value)
}

/// `repeated-number`: a number literal of magnitude [`NUMBER_FROM`] or
/// more, other than one of [`ROUND_NUMBERS`], whatever its sign, that
/// [`NUMBER_BODIES`] or more function bodies of one target hold. A
/// constant's initializer names its number, so it does not count, and one
/// number in several targets is mostly chance. Numbers that the same
/// bodies hold make one lead.
fn repeated_number(run: &AnalysisRun<'_>, bodies: &[(ItemId, &Body)], out: &mut Vec<Lead>) {
    let mut sites: BTreeMap<(usize, &str), Sites> = BTreeMap::new();
    for &(item, body) in bodies.iter().filter(|(_, body)| is_function(body)) {
        for digits in &body.facts.numbers {
            let value = plain_value(digits);
            if telling(value) {
                sites.entry((item.target, value)).or_default().insert(item);
            }
        }
    }
    let mut shared: BTreeMap<Sites, Vec<&str>> = BTreeMap::new();
    for ((_, value), items) in sites {
        shared.entry(items).or_default().push(value);
    }
    for (items, numbers) in shared {
        if items.len() >= NUMBER_BODIES {
            let show = |number: &str| format!("`{number}`");
            out.push(shared_lead(run, "repeated-number", &numbers, &items, show));
        }
    }
}

/// Counts the words of a doc outside its fenced code blocks.
fn doc_words(doc: &str) -> usize {
    let mut fenced = false;
    let mut words = 0;
    for line in doc.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
        } else if !fenced {
            words += line.split_whitespace().count();
        }
    }
    words
}

/// `doc-outweighs-body`: a function whose doc has [`DOC_WORDS`] or more
/// words outside code blocks and boilerplate passages, more than
/// [`DOC_TIMES`] times its body's canonical tokens.
fn doc_outweighs_body(run: &AnalysisRun<'_>, bodies: &[(ItemId, &Body)], out: &mut Vec<Lead>) {
    for &(item, body) in bodies.iter().filter(|(_, body)| is_function(body)) {
        let Some(doc) = run.meta(item).and_then(|meta| meta.raw_doc.as_deref()) else {
            continue;
        };
        let doc = without(doc, &boilerplate(doc, &run.boilerplate));
        let (words, tokens) = (doc_words(&doc), body.facts.fingerprint.tokens);
        if words >= DOC_WORDS && words > DOC_TIMES * tokens {
            let evidence = format!("{words} doc words beside {tokens} body tokens");
            out.push(Lead::new("doc-outweighs-body", Mode::DirectSyntax, evidence).item(run, item));
        }
    }
}
