//! Lays out the C++ special members that standard traits become, as one
//! packed group.

use crate::layout::{Entry, Line, Span, code, key, note_lines, plain};
use crate::model::{Special, SpecialKind, TypeRef, name_order};

/// Builds the special members of a type named `name` as one entry. No blank
/// line separates two members, except around a member with comment lines
/// above it, so a comment stays with its own line.
pub(crate) fn special_entry(specials: &[Special], name: &str, refs: &[TypeRef]) -> Option<Entry> {
    if specials.is_empty() {
        return None;
    }
    let mut sorted: Vec<&Special> = specials.iter().collect();
    let argument = |s: &Special| s.arg.as_ref().map(|a| a.visible(refs)).unwrap_or_default();
    sorted.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| name_order(&argument(a), &argument(b)))
    });
    let mut lines = Vec::new();
    let mut previous: Option<bool> = None;
    for special in sorted {
        let mut member = note_lines(&special.notes, 4, "///");
        let commented = !member.is_empty();
        if previous.is_some_and(|was| was || commented) {
            lines.push(Line::default());
        }
        previous = Some(commented);
        member.push(special_line(special, name));
        lines.extend(member);
    }
    Some(Entry {
        name: String::new(),
        lines,
        repeats: Vec::new(),
    })
}

/// Returns one member line, such as `Type(Type&) = default;`.
fn special_line(special: &Special, name: &str) -> Line {
    let named = |text: String| Span::Name(text, special.loc);
    let argument = || {
        special
            .arg
            .as_ref()
            .map_or_else(|| vec![plain(&format!("{name}&"))], code)
    };
    let mut spans = match special.kind {
        SpecialKind::Default => vec![named(name.to_owned()), plain("()")],
        SpecialKind::Clone | SpecialKind::From => {
            let mut spans = vec![named(name.to_owned()), plain("(")];
            spans.extend(argument());
            spans.push(plain(")"));
            spans
        }
        SpecialKind::Drop => vec![named(format!("~{name}")), plain("()")],
        SpecialKind::PartialEq | SpecialKind::PartialOrd => {
            let (result, operator) = if special.kind == SpecialKind::PartialEq {
                ("bool", "operator==")
            } else {
                ("auto", "operator<=>")
            };
            let mut spans = vec![
                key(result),
                plain(" "),
                named(operator.to_owned()),
                plain("("),
            ];
            spans.extend(argument());
            spans.push(plain(")"));
            spans
        }
    };
    if special.derived {
        spans.extend([plain(" = "), key("default")]);
    }
    spans.push(plain(";"));
    Line {
        indent: 4,
        spans,
        trailing: None,
    }
}
