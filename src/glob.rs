//! Matches `/`-separated paths against globs with `*`, `**`, and `?`.

/// Tells whether `path` matches `pattern`. Both use `/` between segments.
/// Inside one segment, `*` matches any run of characters and `?` matches
/// one character. A `**` segment matches any number of whole segments,
/// none included.
pub(crate) fn matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = parts(pattern);
    let path: Vec<&str> = parts(path);
    segments(&pattern, &path)
}

/// Tells whether a pattern holds a glob character, so it can match more
/// than the one path it spells.
pub(crate) fn is_glob(pattern: &str) -> bool {
    pattern.contains(['*', '?'])
}

/// Matches one segment against one pattern segment, with `*` and `?`.
pub(crate) fn segment(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    let (mut p, mut n) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while n < name.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == name[n]) {
            p += 1;
            n += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some((p, n));
            p += 1;
        } else if let Some((at, from)) = star {
            p = at + 1;
            n = from + 1;
            star = Some((at, from + 1));
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|&c| c == '*')
}

fn parts(text: &str) -> Vec<&str> {
    text.split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .collect()
}

fn segments(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| segments(rest, &path[skip..])),
        Some((first, rest)) => match path.split_first() {
            Some((name, tail)) => segment(first, name) && segments(rest, tail),
            None => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_star_matches_one_segment() {
        assert!(matches("crates/*", "crates/alpha"));
        assert!(matches(
            "crates/harness-internal/*",
            "crates/harness-internal/runner"
        ));
        assert!(!matches("crates/*", "crates/gateway/app"));
        assert!(!matches("crates/*", "crates"));
        assert!(matches("crates/a*a", "crates/alpha"));
        assert!(!matches("crates/a*b", "crates/alpha"));
    }

    #[test]
    fn a_double_star_matches_any_number_of_segments() {
        assert!(matches("crates/**", "crates"));
        assert!(matches("crates/**", "crates/gateway/stt/api"));
        assert!(matches("**/lua", "lua"));
        assert!(matches("**/lua", "crates/runvil-internal/lua"));
        assert!(matches("crates/**/api", "crates/gateway/stt/api"));
        assert!(!matches("crates/**/api", "crates/gateway/stt/engine"));
    }

    #[test]
    fn a_question_mark_matches_one_character_and_separators_are_ignored() {
        assert!(matches("crates/?eta", "crates/beta"));
        assert!(!matches("crates/?eta", "crates/zbeta"));
        assert!(matches("./crates//beta/", "crates\\beta"));
        assert!(is_glob("crates/*") && is_glob("a?") && !is_glob("crates/beta"));
    }
}
