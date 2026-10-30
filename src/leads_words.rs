//! The names that detectors compare: an identifier as its lowercase words,
//! a parameter's pattern and type, and the one name a pattern binds.

use crate::leads_types::is_word;
use crate::model::{Seg, Text};
use crate::typerefs::strip_raw;

/// Splits the Rust text of a parameter or of a named variant field at its
/// first colon: the pattern as written, and the type's segments.
pub(crate) fn split_param(text: &Text) -> Option<(String, Vec<Seg>)> {
    let segs = text.rust();
    let Some(Seg::Plain(first)) = segs.first() else {
        return None;
    };
    let bytes = first.as_bytes();
    let colon = (0..bytes.len()).find(|&i| {
        bytes[i] == b':' && bytes.get(i + 1) != Some(&b':') && (i == 0 || bytes[i - 1] != b':')
    })?;
    let mut rest = Vec::new();
    if !first[colon + 1..].trim().is_empty() {
        rest.push(Seg::Plain(first[colon + 1..].to_owned()));
    }
    rest.extend(segs[1..].iter().cloned());
    Some((first[..colon].trim().to_owned(), rest))
}

/// Returns the one name a pattern binds, without `r#`, such as `x` for
/// `mut x`; `None` for a pattern that binds several names or none.
pub(crate) fn bound_name(pattern: &str) -> Option<String> {
    let mut name = pattern.trim();
    for prefix in ["ref ", "mut "] {
        name = name.strip_prefix(prefix).unwrap_or(name).trim_start();
    }
    let bare = name.strip_prefix("r#").unwrap_or(name);
    let one = !bare.is_empty() && bare.chars().all(is_word) && !bare.starts_with(char::is_numeric);
    (one && bare != "_").then(|| strip_raw(name))
}

/// Returns an identifier's words in lowercase: split at underscores and at
/// each capital that starts a word, with digits kept beside the letters
/// before them, such as `parse`, `v2`, and `args` for `ParseV2Args`.
pub(crate) fn words(name: &str) -> Vec<String> {
    let name = name.strip_prefix("r#").unwrap_or(name);
    let mut out = Vec::new();
    for part in name.split('_').filter(|p| !p.is_empty()) {
        let chars: Vec<char> = part.chars().collect();
        let mut word = String::new();
        for (i, &c) in chars.iter().enumerate() {
            let prev = i.checked_sub(1).map(|j| chars[j]);
            let next = chars.get(i + 1).copied();
            let starts = c.is_uppercase()
                && prev.is_some_and(|p| {
                    p.is_lowercase()
                        || p.is_numeric()
                        || (p.is_uppercase() && next.is_some_and(char::is_lowercase))
                });
            if starts && !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            word.extend(c.to_lowercase());
        }
        if !word.is_empty() {
            out.push(word);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_split_into_words() {
        assert_eq!(words("ParseV2Args"), ["parse", "v2", "args"]);
        assert_eq!(words("HTTPServer"), ["http", "server"]);
        assert_eq!(words("r#type_name"), ["type", "name"]);
        assert_eq!(words("old_value_2"), ["old", "value", "2"]);
        assert_eq!(bound_name("mut r#type").as_deref(), Some("type"));
        assert_eq!(bound_name("(a, b)"), None);
        assert_eq!(bound_name("_"), None);
        assert_eq!(bound_name("_unused").as_deref(), Some("_unused"));
    }

    #[test]
    fn a_parameter_splits_at_its_first_colon_outside_a_path() {
        let text = Text::plain("Point::Origin(x): Point");
        let (pattern, rest) = split_param(&text).unwrap();
        assert_eq!(pattern, "Point::Origin(x)");
        assert_eq!(rest, [Seg::Plain(" Point".to_owned())]);
        assert_eq!(split_param(&Text::plain("self")), None);
    }
}
