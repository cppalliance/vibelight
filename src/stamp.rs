//! The UTC stamp on the first line of every output file, which records when
//! the file's content last changed, and the mask that hides it when files
//! are compared.

use std::env::{self, VarError};

use anyhow::{Result, bail};

/// The stamp a check run renders. A check compares masked text, so the
/// value never shows; it has the length of a real stamp.
pub(crate) const PLACEHOLDER: &str = "0000-00-00T00:00:00Z";

/// The largest epoch that still has a four-digit year: 9999-12-31T23:59:59Z.
const MAX_EPOCH: u64 = 253_402_300_799;

const SECONDS_PER_DAY: u64 = 86_400;

/// Returns the time to stamp, in UTC, as RFC 3339 to the second: from
/// `SOURCE_DATE_EPOCH` when it is set, and otherwise from the clock.
///
/// # Errors
/// Returns an error when `SOURCE_DATE_EPOCH` is not a whole number of
/// seconds from 0 to `MAX_EPOCH`.
pub(crate) fn now() -> Result<String> {
    match env::var("SOURCE_DATE_EPOCH") {
        Ok(value) => Ok(format(parse(&value)?)),
        Err(VarError::NotPresent) => Ok(format(clock())),
        Err(VarError::NotUnicode(_)) => {
            bail!(
                "SOURCE_DATE_EPOCH must be a whole number of seconds since 1970, but it is not text"
            )
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the stamp is the one part of the output that reads the clock"
)]
fn clock() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs().min(MAX_EPOCH))
}

/// Reads a `SOURCE_DATE_EPOCH` value.
fn parse(value: &str) -> Result<u64> {
    match value.trim().parse::<u64>() {
        Ok(seconds) if seconds <= MAX_EPOCH => Ok(seconds),
        _ => bail!(
            "SOURCE_DATE_EPOCH must be a whole number of seconds from 0 to {MAX_EPOCH}, but it is `{value}`"
        ),
    }
}

/// Formats seconds since 1970 as UTC, such as `2026-01-01T00:00:00Z`.
fn format(seconds: u64) -> String {
    let (year, month, day) = civil(seconds / SECONDS_PER_DAY);
    let time = seconds % SECONDS_PER_DAY;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        time % 3600 / 60,
        time % 60
    )
}

/// Turns days since 1970-01-01 into a Gregorian year, month, and day, with
/// Howard Hinnant's days-to-civil algorithm, which counts in 400-year eras
/// that start on March 1.
fn civil(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

/// The markdown stamp line.
pub(crate) fn markdown(stamp: &str) -> String {
    format!("*Updated {stamp}*")
}

/// The HTML stamp comment, which goes before the doctype.
pub(crate) fn html_comment(stamp: &str) -> String {
    format!("<!-- Updated {stamp} -->")
}

/// The HTML stamp paragraph, the first element of `<main>`.
pub(crate) fn html_paragraph(stamp: &str) -> String {
    format!("<p class=\"stamp\">Updated <time datetime=\"{stamp}\">{stamp}</time></p>")
}

/// Tells whether a line is a stamp line in either format.
pub(crate) fn is_line(line: &str) -> bool {
    (line.starts_with("*Updated ") && line.ends_with('*'))
        || (line.starts_with("<!-- Updated ") && line.ends_with(" -->"))
        || line.starts_with("<p class=\"stamp\">Updated ")
}

/// Replaces every time in every stamp line with the placeholder, which has
/// the same length, so line numbers and lengths stay true.
pub(crate) fn mask(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if is_line(line.trim_end_matches(['\n', '\r'])) {
            out.push_str(&mask_line(line));
        } else {
            out.push_str(line);
        }
    }
    out
}

fn mask_line(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut at = 0;
    while at < line.len() {
        if bytes.get(at..at + PLACEHOLDER.len()).is_some_and(is_time) {
            out.push_str(PLACEHOLDER);
            at += PLACEHOLDER.len();
        } else {
            let next = line[at..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&line[at..at + next]);
            at += next;
        }
    }
    out
}

/// Tells whether bytes have the shape `DDDD-DD-DDTDD:DD:DDZ`.
fn is_time(bytes: &[u8]) -> bool {
    const SHAPE: &[u8] = b"0000-00-00T00:00:00Z";
    bytes.len() == SHAPE.len()
        && bytes.iter().zip(SHAPE).all(|(&byte, &shape)| {
            if shape == b'0' {
                byte.is_ascii_digit()
            } else {
                byte == shape
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epochs_format_as_utc() {
        assert_eq!(format(0), "1970-01-01T00:00:00Z");
        assert_eq!(format(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format(1_767_225_600), "2026-01-01T00:00:00Z");
        assert_eq!(format(1_767_225_600 + 3661), "2026-01-01T01:01:01Z");
        assert_eq!(format(4_107_542_400), "2100-03-01T00:00:00Z");
        assert_eq!(format(MAX_EPOCH), "9999-12-31T23:59:59Z");
    }

    #[test]
    fn epoch_values_parse_or_name_the_problem() {
        assert_eq!(parse("1767225600").unwrap(), 1_767_225_600);
        assert_eq!(parse(" 0\n").unwrap(), 0);
        for bad in ["", "-1", "1.5", "soon", "253402300800"] {
            let error = parse(bad).unwrap_err().to_string();
            assert!(error.contains("whole number of seconds"), "{error}");
            assert!(error.contains(&format!("`{bad}`")), "{error}");
        }
    }

    #[test]
    fn the_mask_hides_only_the_times_in_stamp_lines() {
        let stamp = "2026-01-01T00:00:00Z";
        let md = format!("{}\n\n# a 2026-01-01T00:00:00Z\n", markdown(stamp));
        assert_eq!(
            mask(&md),
            format!("{}\n\n# a 2026-01-01T00:00:00Z\n", markdown(PLACEHOLDER))
        );
        let html = format!(
            "{}\r\n<main>\n{}\n",
            html_comment(stamp),
            html_paragraph(stamp)
        );
        assert_eq!(
            mask(&html),
            format!(
                "{}\r\n<main>\n{}\n",
                html_comment(PLACEHOLDER),
                html_paragraph(PLACEHOLDER)
            )
        );
        assert_eq!(mask(&mask(&html)), mask(&html));
        assert_eq!(mask("*Updated soon*\n"), "*Updated soon*\n");
    }
}
