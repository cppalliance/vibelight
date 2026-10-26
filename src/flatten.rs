//! Flattens the markdown in doc comments: link syntax goes, and backticks
//! stay, so a backticked name still reads as code.

/// One piece of a flattened doc line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Piece {
    Text(String),
    /// A name from a backticked span or an intra-doc link: the text shown
    /// and the path it may resolve to.
    Name {
        text: String,
        path: String,
    },
}

/// Flattens doc lines. Code fence lines and the lines inside a fence stay
/// as written; reference definitions are dropped, and so are trailing
/// empty lines.
pub(crate) fn flatten(lines: &[String]) -> Vec<Vec<Piece>> {
    let mut out: Vec<Vec<Piece>> = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in lines {
        let trimmed = line.trim_start();
        if let Some((mark, count)) = fence {
            out.push(vec![Piece::Text(line.clone())]);
            let run = trimmed.chars().take_while(|&c| c == mark).count();
            if run >= count && trimmed[run * mark.len_utf8()..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if let Some(opening) = fence_open(trimmed) {
            out.extend(inline_lines(&paragraph));
            paragraph.clear();
            out.push(vec![Piece::Text(line.clone())]);
            fence = Some(opening);
            continue;
        }
        if is_reference_definition(line) {
            out.extend(inline_lines(&paragraph));
            paragraph.clear();
            continue;
        }
        paragraph.push(line);
    }
    out.extend(inline_lines(&paragraph));
    while out.last().is_some_and(|line| {
        line.iter()
            .all(|p| matches!(p, Piece::Text(t) if t.trim().is_empty()))
    }) {
        out.pop();
    }
    out
}

fn fence_open(trimmed: &str) -> Option<(char, usize)> {
    let mark = trimmed.chars().next().filter(|&c| c == '`' || c == '~')?;
    let count = trimmed.chars().take_while(|&c| c == mark).count();
    (count >= 3).then_some((mark, count))
}

fn is_reference_definition(line: &str) -> bool {
    let trimmed = line.trim_start();
    if line.len() - trimmed.len() > 3 || !trimmed.starts_with('[') {
        return false;
    }
    let Some(close) = trimmed.find("]:") else {
        return false;
    };
    close > 1 && !trimmed[1..close].contains(['[', ']']) && !trimmed[close + 2..].trim().is_empty()
}

/// Processes the inline markup of consecutive lines, which may share a code
/// span or a link, and splits the result back into lines.
fn inline_lines(lines: &[&str]) -> Vec<Vec<Piece>> {
    if lines.is_empty() {
        return Vec::new();
    }
    let joined = lines.join("\n");
    let chars: Vec<char> = joined.chars().collect();
    let mut out = vec![Vec::new()];
    for piece in inline(&chars) {
        match piece {
            Piece::Text(text) => {
                let mut parts = text.split('\n');
                if let Some(first) = parts.next() {
                    push_text(out.last_mut().expect("out starts with one line"), first);
                }
                for part in parts {
                    out.push(Vec::new());
                    push_text(out.last_mut().expect("a line was just pushed"), part);
                }
            }
            Piece::Name { text, path } => {
                if text.contains('\n') {
                    let mut parts = text.split('\n');
                    if let Some(first) = parts.next() {
                        push_text(out.last_mut().expect("out starts with one line"), first);
                    }
                    for part in parts {
                        out.push(vec![Piece::Text(part.to_owned())]);
                    }
                } else {
                    out.last_mut()
                        .expect("out starts with one line")
                        .push(Piece::Name { text, path });
                }
            }
        }
    }
    out
}

fn push_text(line: &mut Vec<Piece>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(Piece::Text(last)) = line.last_mut() {
        last.push_str(text);
    } else {
        line.push(Piece::Text(text.to_owned()));
    }
}

/// Flattens inline markup: links lose their syntax, and code spans keep
/// their backticks around the name they hold.
fn inline(chars: &[char]) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && chars.get(i + 1).is_some_and(char::is_ascii_punctuation) {
            text.push(c);
            text.push(chars[i + 1]);
            i += 2;
        } else if c == '`' {
            let n = run(chars, i, '`');
            if let Some(close) = find_run(chars, i + n, n) {
                let raw: String = chars[i + n..close].iter().collect();
                let content = code_content(&raw);
                let (lead, tail) = if content.len() < raw.len() {
                    (&raw[..1], &raw[raw.len() - 1..])
                } else {
                    ("", "")
                };
                let ticks = "`".repeat(n);
                text.push_str(&ticks);
                text.push_str(lead);
                match path_of(&content) {
                    Some(path) => {
                        flush(&mut out, &mut text);
                        out.push(Piece::Name {
                            text: content,
                            path,
                        });
                    }
                    None => text.push_str(&content),
                }
                text.push_str(tail);
                text.push_str(&ticks);
                i = close + n;
            } else {
                text.extend(&chars[i..i + n]);
                i += n;
            }
        } else if c == '[' {
            if let Some((consumed, pieces)) = link(chars, i) {
                flush(&mut out, &mut text);
                out.extend(pieces);
                i += consumed;
            } else {
                text.push(c);
                i += 1;
            }
        } else {
            text.push(c);
            i += 1;
        }
    }
    flush(&mut out, &mut text);
    out
}

fn flush(out: &mut Vec<Piece>, text: &mut String) {
    if !text.is_empty() {
        out.push(Piece::Text(std::mem::take(text)));
    }
}

/// Parses a link starting at the `[` at `start`, returning how many
/// characters it spans and its flattened pieces.
fn link(chars: &[char], start: usize) -> Option<(usize, Vec<Piece>)> {
    let close = matching(chars, start, '[', ']')?;
    let label = &chars[start + 1..close];
    let pieces = inline(label);
    let shown: String = pieces
        .iter()
        .map(|p| match p {
            Piece::Text(t) | Piece::Name { text: t, .. } => t.as_str(),
        })
        .collect();
    let next = chars.get(close + 1).copied();
    let (end, target) = match next {
        Some('(') => {
            let paren = matching(chars, close + 1, '(', ')')?;
            let inner: String = chars[close + 2..paren].iter().collect();
            let dest = inner.split_whitespace().next().unwrap_or("").to_owned();
            (paren + 1, Some(dest))
        }
        Some('[') => {
            let bracket = matching(chars, close + 1, '[', ']')?;
            let reference: String = chars[close + 2..bracket].iter().collect();
            let target = if reference.trim().is_empty() {
                label.iter().collect()
            } else {
                reference
            };
            (bracket + 1, Some(target))
        }
        _ => {
            let raw: String = label.iter().collect();
            path_of(raw.trim_matches('`'))?;
            (close + 1, Some(raw))
        }
    };
    let target = target.unwrap_or_default();
    let target = target.trim().trim_matches('`');
    let result = match path_of(target) {
        Some(path) if !shown.is_empty() && !shown.contains('\n') => named(&shown, path),
        _ => pieces,
    };
    Some((end - start, result))
}

/// Returns a link's name. Backticks around the label stay outside the name,
/// so `` [`items`](Self::items) `` shows `` `items` `` and only `items` is
/// the name.
fn named(shown: &str, path: String) -> Vec<Piece> {
    let inner = shown.trim_start_matches('`');
    let ticks = shown.len() - inner.len();
    let inner = inner.trim_end_matches('`');
    if ticks == 0 || inner.is_empty() || shown.len() - inner.len() != 2 * ticks {
        return vec![Piece::Name {
            text: shown.to_owned(),
            path,
        }];
    }
    let ticks = "`".repeat(ticks);
    vec![
        Piece::Text(ticks.clone()),
        Piece::Name {
            text: inner.to_owned(),
            path,
        },
        Piece::Text(ticks),
    ]
}

/// Finds the bracket that closes the one at `open`, skipping code spans.
fn matching(chars: &[char], open: usize, left: char, right: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = open;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            i += 2;
            continue;
        }
        if c == '`' {
            let n = run(chars, i, '`');
            match find_run(chars, i + n, n) {
                Some(close) => i = close + n,
                None => i += n,
            }
            continue;
        }
        if c == left {
            depth += 1;
        } else if c == right {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn run(chars: &[char], at: usize, c: char) -> usize {
    chars[at..].iter().take_while(|&&x| x == c).count()
}

/// Finds the next run of exactly `n` backticks at or after `from`.
fn find_run(chars: &[char], from: usize, n: usize) -> Option<usize> {
    let mut i = from;
    while i < chars.len() {
        if chars[i] == '`' {
            let len = run(chars, i, '`');
            if len == n {
                return Some(i);
            }
            i += len;
        } else {
            i += 1;
        }
    }
    None
}

/// Returns a code span's content, with one padding space removed from each
/// side when both are present, as markdown does.
fn code_content(content: &str) -> String {
    let padded = content.len() >= 2
        && content.starts_with([' ', '\n'])
        && content.ends_with([' ', '\n'])
        && !content.trim().is_empty();
    if padded {
        content[1..content.len() - 1].to_owned()
    } else {
        content.to_owned()
    }
}

/// Returns the path a link target or code span names, without a
/// disambiguator prefix or a trailing `()` or `!`, when it looks like one.
fn path_of(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let raw = raw.split_once('@').map_or(raw, |(kind, rest)| {
        if kind.chars().all(|c| c.is_ascii_lowercase()) {
            rest
        } else {
            raw
        }
    });
    let raw = raw
        .strip_suffix("()")
        .or_else(|| raw.strip_suffix('!'))
        .unwrap_or(raw);
    let body = raw.strip_prefix("::").unwrap_or(raw);
    let valid = !body.is_empty()
        && body.split("::").all(|segment| {
            let segment = segment.strip_prefix("r#").unwrap_or(segment);
            segment.starts_with(|c: char| c.is_alphabetic() || c == '_')
                && segment.chars().all(|c| c.is_alphanumeric() || c == '_')
        });
    valid.then(|| raw.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(lines: &[&str]) -> Vec<String> {
        let lines: Vec<String> = lines.iter().map(|s| (*s).to_owned()).collect();
        flatten(&lines)
            .into_iter()
            .map(|line| {
                line.iter()
                    .map(|p| match p {
                        Piece::Text(t) => t.clone(),
                        Piece::Name { text, path } => format!("<{text}|{path}>"),
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn code_spans_and_intra_doc_links_keep_their_backticks() {
        assert_eq!(
            flat(&[" Links: [`DocFirst`], [the plain one](PlainFirst), and `inline code`."]),
            [" Links: `<DocFirst|DocFirst>`, <the plain one|PlainFirst>, and `inline code`."]
        );
        assert_eq!(
            flat(&[" Holds [`Prompt`], its [`Section`]s, and `[Lua, Prose, Lua]`."]),
            [" Holds `<Prompt|Prompt>`, its `<Section|Section>`s, and `[Lua, Prose, Lua]`."]
        );
        assert_eq!(
            flat(&[" through [`crate::detail::entry`]."]),
            [" through `<crate::detail::entry|crate::detail::entry>`."]
        );
        assert_eq!(
            flat(&[" Builds `prompt` with ``args`` and [`items`](Self::items)."]),
            [" Builds `<prompt|prompt>` with ``<args|args>`` and `<items|Self::items>`."]
        );
        assert_eq!(flat(&[" Pads ` x `."]), [" Pads ` <x|x> `."]);
    }

    #[test]
    fn urls_and_non_paths_stay_text() {
        assert_eq!(
            flat(&[" See [the docs](https://x.org) and [0]."]),
            [" See the docs and [0]."]
        );
        assert_eq!(flat(&[" Or ``"]), [" Or ``"]);
        assert_eq!(flat(&[" A ```` run."]), [" A ```` run."]);
        assert_eq!(flat(&[" The `\"\"` case."]), [" The `\"\"` case."]);
    }

    #[test]
    fn fences_stay_verbatim_and_references_drop() {
        assert_eq!(
            flat(&[
                " Before `x`:",
                "",
                " ```",
                " let x = [1, 2];",
                " ```",
                "",
                " [x]: crate::X",
                ""
            ]),
            [" Before `<x|x>`:", "", " ```", " let x = [1, 2];", " ```"]
        );
    }

    #[test]
    fn spans_cross_lines() {
        assert_eq!(flat(&[" a `long", " span` b"]), [" a `long", " span` b"]);
        assert_eq!(
            flat(&[" see [`Foo`](", " Foo) ok"]),
            [" see `<Foo|Foo>` ok"]
        );
    }
}
