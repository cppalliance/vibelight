//! Splits a generic parameter list into its parameters, for the impl line
//! that names a type with its parameters.

/// Splits a generic parameter list, such as `<'a, T: Copy = u8, const N:
/// usize>`, into each parameter without its default and its name.
pub(crate) fn generic_params(text: &str) -> Vec<(String, String)> {
    let inner = text
        .trim()
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
        .unwrap_or_default();
    let mut params = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    let mut push = |part: &str| {
        let part = part.trim();
        if part.is_empty() {
            return;
        }
        let without_default = top_level_split(part, '=').0.trim().to_owned();
        let named = without_default
            .strip_prefix("const ")
            .unwrap_or(&without_default);
        let name = top_level_split(named, ':').0.trim().to_owned();
        params.push((without_default, name));
    };
    for (at, c) in inner.char_indices() {
        match c {
            ',' if depth == 0 => {
                push(&inner[start..at]);
                start = at + 1;
            }
            _ => depth = nesting(depth, c, inner[..at].ends_with('-')),
        }
    }
    push(&inner[start..]);
    params
}

/// Returns the depth of brackets after `c`: an angle bracket, a
/// parenthesis, or a square bracket opens or closes one, and the `>` of an
/// arrow, which `after_dash` tells, does neither.
fn nesting(depth: usize, c: char, after_dash: bool) -> usize {
    match c {
        '<' | '(' | '[' => depth + 1,
        '>' if after_dash => depth,
        '>' | ')' | ']' => depth.saturating_sub(1),
        _ => depth,
    }
}

/// Splits text at the first `stop` outside angle brackets, parentheses, and
/// square brackets.
fn top_level_split(text: &str, stop: char) -> (&str, &str) {
    let mut depth = 0usize;
    for (at, c) in text.char_indices() {
        if c == stop && depth == 0 {
            return (&text[..at], &text[at + 1..]);
        }
        depth = nesting(depth, c, text[..at].ends_with('-'));
    }
    (text, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_params_drop_defaults_and_keep_each_name() {
        assert_eq!(
            generic_params("<'a, T: Copy = u8, const N: usize, F: Fn(u8) -> u8>"),
            [
                ("'a".to_owned(), "'a".to_owned()),
                ("T: Copy".to_owned(), "T".to_owned()),
                ("const N: usize".to_owned(), "N".to_owned()),
                ("F: Fn(u8) -> u8".to_owned(), "F".to_owned()),
            ]
        );
        assert_eq!(
            generic_params("<T: Into<Box<dyn Fn() -> u8>>, U>"),
            [
                ("T: Into<Box<dyn Fn() -> u8>>".to_owned(), "T".to_owned()),
                ("U".to_owned(), "U".to_owned()),
            ]
        );
        assert_eq!(generic_params(""), Vec::<(String, String)>::new());
    }
}
