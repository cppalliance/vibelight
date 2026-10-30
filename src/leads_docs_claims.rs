//! The doc detectors of explicit claims that the written declarations
//! settle: the parameters an `# Arguments` section names, the return
//! values, errors, and parameter states a function's doc states, and the
//! numbers a doc ties to a constant.

use std::collections::BTreeMap;

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads::{Lead, Mode};
use crate::leads_docs::{Doc, Docs, Node};
use crate::leads_docs_refs::intra_path;
use crate::leads_docs_text::{Kind, Prose, identifiers};
use crate::leads_items::Func;
use crate::leads_words::words;
use crate::model::{ItemKind, MemberKind, ValueRef};
use crate::typerefs::strip_raw;

/// The headings of the sections that list a function's parameters.
const PARAM_SECTIONS: [&str; 4] = ["args", "arguments", "params", "parameters"];

/// The words that open a sentence about what a function returns.
const RETURN_VERBS: [&str; 3] = ["return", "returns", "yields"];

/// The most tokens after a return verb that the returned literal may be,
/// as in "Returns either `true` or `false`".
const RETURN_REACH: usize = 3;

/// The words after a literal that open a sentence about when a function
/// returns it, as in "`None` if the key is missing".
const CONDITION_WORDS: [&str; 6] = ["for", "if", "on", "otherwise", "unless", "when"];

/// The literals a return claim names, each with the names of the types
/// that may hold it.
const LITERALS: [(&str, &[&str]); 4] = [
    ("None", &["Option"]),
    ("Err", &FALLIBLE),
    ("true", &["bool"]),
    ("false", &["bool"]),
];

/// What the names of a type that may carry an error hold.
const FALLIBLE: [&str; 3] = ["Error", "Fallible", "Result"];

/// The words that may stand between a constant's name and the number a
/// doc ties to it, as in "`LIMIT` is 5".
const TIE_WORDS: [&str; 7] = ["at", "currently", "default", "defaults", "is", "set", "to"];

/// The most tokens after a constant's name that the tied number may be.
const TIE_STEPS: usize = 4;

/// The units that scale a number, so that it may state a constant in
/// another unit than the constant's own.
const UNITS: [&str; 30] = [
    "b",
    "billion",
    "bytes",
    "day",
    "days",
    "gb",
    "gib",
    "h",
    "hour",
    "hours",
    "k",
    "kb",
    "kib",
    "m",
    "mb",
    "mib",
    "million",
    "millisecond",
    "milliseconds",
    "min",
    "minute",
    "minutes",
    "ms",
    "ns",
    "s",
    "sec",
    "second",
    "seconds",
    "thousand",
    "us",
];

/// The suffixes a number literal may carry.
const NUMBER_SUFFIXES: [&str; 14] = [
    "f32", "f64", "i128", "i16", "i32", "i64", "i8", "isize", "u128", "u16", "u32", "u64", "u8",
    "usize",
];

/// Finds the leads of explicit claims.
pub(crate) fn find(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    let consts = constants(run);
    for doc in &docs.all {
        if let Node::Func(f) = doc.node {
            let func = &docs.catalog.funcs[f];
            doc_unknown_param(run, func, doc, out);
            doc_signature_mismatch(run, docs, func, doc, out);
        }
        doc_const_mismatch(run, &consts, doc, out);
    }
}

/// `doc-unknown-param`: code spans that open a list item or a sentence of
/// a function's `# Arguments` or `# Parameters` section and name no
/// parameter it writes, a leading underscore aside. A function with a
/// parameter whose pattern binds several names is left out.
fn doc_unknown_param(run: &AnalysisRun<'_>, func: &Func<'_>, doc: &Doc<'_>, out: &mut Vec<Lead>) {
    if func.args.iter().any(|arg| arg.name.is_none()) {
        return;
    }
    let bare = |name: &str| name.trim_start_matches('_').to_owned();
    let mut params: Vec<String> = func
        .args
        .iter()
        .filter_map(|arg| arg.name.clone())
        .collect();
    if func.has_receiver() {
        params.push("self".to_owned());
    }
    let prose = &doc.prose;
    let mut unknown: Vec<String> = Vec::new();
    for (i, tok) in prose.toks.iter().enumerate() {
        let listed = tok.kind == Kind::Code
            && prose.starts(i)
            && prose
                .section(i)
                .is_some_and(|s| PARAM_SECTIONS.contains(&s));
        let name = tok.text.trim();
        let ident = name.starts_with(|c: char| c.is_lowercase() || c == '_')
            && name.chars().all(|c| c.is_alphanumeric() || c == '_');
        let named = params.iter().any(|param| bare(param) == bare(name));
        if listed && ident && !named && !unknown.iter().any(|u| u == name) {
            unknown.push(name.to_owned());
        }
    }
    if !unknown.is_empty() {
        let shown: Vec<String> = unknown.iter().map(|name| format!("`{name}`")).collect();
        let evidence = format!(
            "the parameter section names {}, but `{}` takes `{}`",
            shown.join(", "),
            func.name,
            func.shown()
        );
        let lead = Lead::new("doc-unknown-param", Mode::DirectSyntax, evidence);
        out.push(doc.add(run, lead));
    }
}

/// Returns the literal of [`LITERALS`] that token `i` names, as a code span
/// or as a word; `None` and `Err` count as words only as written.
fn literal(prose: &Prose<'_>, i: usize) -> Option<(&'static str, &'static [&'static str])> {
    let tok = &prose.toks[i];
    let text = match tok.kind {
        Kind::Code => tok.text.as_str(),
        Kind::Word => &prose.doc[tok.span.clone()],
        Kind::Punct => return None,
    };
    let lowered = tok.kind == Kind::Word && text.starts_with(char::is_lowercase);
    LITERALS.iter().copied().find(|(lit, _)| {
        let bool_word = lowered && text == *lit;
        bool_word || (!lowered && identifiers(text).any(|id| id == *lit))
    })
}

/// `doc-signature-mismatch`: an `# Errors` section on a function whose
/// return type cannot carry an error, a sentence that names `None`, `Err`,
/// `true`, or `false` that the return type cannot hold, within
/// [`RETURN_REACH`] tokens after an opening "Returns" or as its opening
/// token before a condition, and a parameter said to be
/// `None` whose type cannot hold it. Opaque and generic types and type
/// aliases may hold anything.
fn doc_signature_mismatch(
    run: &AnalysisRun<'_>,
    docs: &Docs<'_>,
    func: &Func<'_>,
    doc: &Doc<'_>,
    out: &mut Vec<Lead>,
) {
    let prose = &doc.prose;
    let ret = func.ret.as_ref();
    let shown = ret.map_or_else(|| "nothing".to_owned(), |ret| format!("`{}`", ret.shown()));
    let holds = |heads: &[&str]| ret.is_some_and(|ret| docs.may_hold(run, ret, heads));
    let mut found = Vec::new();
    if prose.headings.iter().any(|h| h == "errors") && !holds(&FALLIBLE) {
        found.push(format!("an `# Errors` section, but it returns {shown}"));
    }
    let mut start = 0;
    while start < prose.toks.len() {
        let sentence = prose.toks[start].sentence;
        let end = start
            + (prose.toks[start..].iter())
                .take_while(|t| t.sentence == sentence)
                .count();
        let first = &prose.toks[start];
        let next = prose.toks.get(start + 1).filter(|t| t.sentence == sentence);
        let verb = first.kind == Kind::Word && RETURN_VERBS.contains(&first.text.as_str());
        let led = literal(prose, start).is_some()
            && next.is_some_and(|t| CONDITION_WORDS.contains(&t.text.as_str()));
        let claimed = if verb {
            start + 1..(start + 1 + RETURN_REACH).min(end)
        } else if led {
            start..start + 1
        } else {
            start..start
        };
        let impossible = claimed
            .filter_map(|i| literal(prose, i))
            .find(|(_, heads)| !holds(heads));
        if let Some((lit, _)) = impossible {
            let quoted = prose.sentence(sentence);
            found.push(format!(
                "\"{quoted}\" names `{lit}`, but it returns {shown}"
            ));
        }
        start = end;
    }
    for (i, tok) in prose.toks.iter().enumerate() {
        let arg = (tok.kind == Kind::Code)
            .then(|| {
                func.args
                    .iter()
                    .find(|a| a.name.as_deref() == Some(tok.text.as_str()))
            })
            .flatten();
        let Some(arg) = arg.filter(|arg| !docs.may_hold(run, &arg.ty, &["Option"])) else {
            continue;
        };
        let after: Vec<&str> = (prose.toks[i + 1..].iter())
            .take_while(|t| t.sentence == tok.sentence)
            .take(3)
            .map(|t| t.text.as_str())
            .collect();
        let none = |k: usize| literal(prose, i + 1 + k).is_some_and(|(lit, _)| lit == "None");
        let said = match after.as_slice() {
            ["is" | "being", ..] => none(1),
            ["may" | "can" | "might", "be", ..] => none(2),
            _ => false,
        };
        if said {
            found.push(format!(
                "`{}` is said to be `None`, but it is `{}`",
                tok.text,
                arg.ty.shown()
            ));
        }
    }
    if !found.is_empty() {
        let lead = Lead::new(
            "doc-signature-mismatch",
            Mode::DirectSyntax,
            found.join("; "),
        );
        out.push(doc.add(run, lead));
    }
}

/// Returns a number literal as base 10 digits, without separators or a
/// suffix; `None` for text that is not one number literal.
fn number(text: &str) -> Option<String> {
    let text: String = text.trim().chars().filter(|&c| c != '_').collect();
    let text = text.to_ascii_lowercase();
    let hex = text.starts_with("0x");
    let body = (NUMBER_SUFFIXES.iter())
        .filter(|suffix| !(hex && suffix.starts_with('f')))
        .find_map(|suffix| {
            let rest = text.strip_suffix(suffix)?;
            rest.ends_with(|c: char| c.is_ascii_hexdigit())
                .then_some(rest)
        })
        .unwrap_or(&text);
    for (prefix, radix) in [("0x", 16), ("0o", 8), ("0b", 2)] {
        if let Some(digits) = body.strip_prefix(prefix) {
            return u128::from_str_radix(digits, radix)
                .ok()
                .map(|n| n.to_string());
        }
    }
    if !body.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    if let Ok(n) = body.parse::<u128>() {
        return Some(n.to_string());
    }
    let float = body.parse::<f64>().ok().filter(|f| f.is_finite())?;
    Some(float.to_string())
}

/// Returns each constant and static of the run whose initializer is one
/// number literal, by target and name, with that number.
fn constants(run: &AnalysisRun<'_>) -> BTreeMap<(usize, String), Vec<(ItemId, String)>> {
    let mut out: BTreeMap<(usize, String), Vec<(ItemId, String)>> = BTreeMap::new();
    for (t, krate) in run.krates.iter().enumerate() {
        let mut add = |item, name: &str, value: &Option<String>| {
            if let Some(n) = value.as_deref().and_then(number) {
                let at = ItemId { target: t, item };
                out.entry((t, strip_raw(name))).or_default().push((at, n));
            }
        };
        for (m, module) in krate.modules.iter().enumerate() {
            for (k, item) in module.items.iter().enumerate() {
                if let ItemKind::Const { value, .. } | ItemKind::Static { value, .. } = &item.kind {
                    add(
                        ItemRef::Value(ValueRef { module: m, item: k }),
                        &item.name,
                        value,
                    );
                }
            }
        }
        for (d, decl) in krate.decls.iter().enumerate() {
            for (k, member) in decl.members.iter().enumerate() {
                if let MemberKind::Const { value, .. } = &member.kind {
                    add(ItemRef::Own(d, k), &member.name, value);
                }
            }
        }
        for (i, imp) in krate.impls.iter().enumerate() {
            for (k, member) in imp.members.iter().enumerate() {
                if let MemberKind::Const { value, .. } = &member.kind {
                    add(ItemRef::Inherent(i, k), &member.name, value);
                }
            }
        }
    }
    out
}

/// Returns the number a doc ties to the code span at `i` that names the
/// constant `name`: one after it past connectors, as in "`LIMIT` (5)",
/// "`LIMIT` = 5", or "`LIMIT` is 5", or one just before it, as in "5
/// (`LIMIT`)". A number before a unit that the constant's name does not
/// have is no tie.
fn tied(prose: &Prose<'_>, i: usize, name: &str) -> Option<String> {
    let toks = &prose.toks;
    let sentence = toks[i].sentence;
    let is_number = |k: usize| {
        (toks.get(k)).is_some_and(|t| {
            t.kind == Kind::Word && t.sentence == sentence && number(&t.text).is_some()
        })
    };
    let mut at = None;
    for (k, tok) in toks.iter().enumerate().skip(i + 1).take(TIE_STEPS) {
        if is_number(k) {
            at = Some(k);
            break;
        }
        let connector = match tok.kind {
            Kind::Punct => matches!(tok.text.as_str(), "(" | "=" | ":"),
            Kind::Word => TIE_WORDS.contains(&tok.text.as_str()),
            Kind::Code => false,
        };
        if !connector || tok.sentence != sentence {
            break;
        }
    }
    if at.is_none() && i >= 2 && toks[i - 1].text == "(" && is_number(i - 2) {
        at = Some(i - 2);
    }
    let k = at?;
    let unit = (toks.get(k + 1).filter(|t| t.sentence == sentence)).and_then(|t| match t.kind {
        Kind::Word if UNITS.contains(&t.text.as_str()) => Some(t.text.clone()),
        Kind::Punct if t.text == "%" => Some(t.text.clone()),
        _ => None,
    });
    if unit.is_some_and(|unit| !words(name).contains(&unit)) {
        return None;
    }
    number(&toks[k].text)
}

/// `doc-const-mismatch`: a number that a doc ties to a constant of its
/// target, by a code span of the constant's path, that differs from the
/// one number literal every constant of that name is initialized to.
fn doc_const_mismatch(
    run: &AnalysisRun<'_>,
    consts: &BTreeMap<(usize, String), Vec<(ItemId, String)>>,
    doc: &Doc<'_>,
    out: &mut Vec<Lead>,
) {
    let t = doc.item.target;
    let prose = &doc.prose;
    let mut found = Vec::new();
    let mut named = Vec::new();
    for (i, tok) in prose.toks.iter().enumerate() {
        let path = (tok.kind == Kind::Code)
            .then(|| intra_path(&tok.text))
            .flatten();
        let Some(path) = path else {
            continue;
        };
        let Some(list) = path.last().and_then(|name| consts.get(&(t, name.clone()))) else {
            continue;
        };
        let (item, value) = &list[0];
        if list.iter().any(|(_, v)| v != value) {
            continue;
        }
        let Some(said) = tied(prose, i, &path[path.len() - 1]) else {
            continue;
        };
        if said != *value {
            let shown = path.join("::");
            found.push(format!("`{shown}` ({said}), but it is {value}"));
            named.push(*item);
        }
    }
    if !found.is_empty() {
        let mut lead = Lead::new("doc-const-mismatch", Mode::DirectSyntax, found.join("; "));
        for item in named {
            lead = lead.item(run, item);
        }
        out.push(doc.add(run, lead));
    }
}
