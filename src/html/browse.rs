//! The HTML folder's one file, `index.html`, which works from `file:`.
//! Cursor's browser drops every click on a link to a `file:` URL, so the
//! whole HTML is one page that never leaves itself: the style sheet, the
//! app that `browse.js` holds, and every outline's page and the search
//! index, each compressed with zlib and written as base64 in one JSON block.
//! The app unpacks a page when it shows it, with the browser's
//! `DecompressionStream`, and records the page in the address's fragment.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use miniz_oxide::deflate::{CompressionLevel, compress_to_vec_zlib};

use crate::html::render_html::Anchors;
use crate::html::render_style::STYLE;
use crate::stamp;

/// The app, which the file holds in its last script element.
pub(crate) const SCRIPT: &str = include_str!("browse.js");

/// The folder's one file.
pub(crate) const FILE: &str = "index.html";

/// The stem of the overview's page, which the app shows first.
pub(crate) const INDEX: &str = "index";

/// The element that holds the data block, which the app reads.
const DATA_OPEN: &str = "<script type=\"application/json\" id=\"vibelight-data\">";

/// One page the file holds: its stem, the title of its tab, and its content.
pub(crate) struct Page<'a> {
    pub(crate) stem: &'a str,
    pub(crate) title: &'a str,
    pub(crate) content: &'a str,
}

/// Writes the folder's one file, with `stamp` on its first line: the style
/// sheet, the data block, and the app. The data block holds the search
/// index and each page packed, one page to a line, with the page's stamp
/// beside its packed content, which holds the stamp's placeholder. A page
/// whose content is the same as in `old`, the file a run left, keeps its
/// old stamp, so a run that changes nothing leaves the file alone.
pub(crate) fn file(stamp: &str, pages: &[Page<'_>], search: &str, old: Option<&str>) -> String {
    let kept = old.map(old_pages).unwrap_or_default();
    let mut out = String::new();
    let _ = writeln!(out, "{}", stamp::html_comment(stamp));
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>Outlines</title>\n<style>\n");
    out.push_str(STYLE);
    out.push_str("</style>\n</head>\n<body>\n");
    out.push_str("<noscript>The outlines need JavaScript.</noscript>\n");
    out.push_str(DATA_OPEN);
    let _ = writeln!(out, "\n{{\"search\":{},\n\"pages\":[", json(&pack(search)));
    for (i, page) in pages.iter().enumerate() {
        let data = pack(&stamp::mask(page.content));
        let page_stamp = match kept.get(page.stem) {
            Some((old_stamp, old_data)) if *old_data == data => old_stamp.as_str(),
            _ => stamp,
        };
        let _ = writeln!(
            out,
            "{{\"stem\":{},\"title\":{},\"stamp\":{},\"data\":{}}}{}",
            json(page.stem),
            json(page.title),
            json(page_stamp),
            json(&data),
            if i + 1 < pages.len() { "," } else { "" }
        );
    }
    out.push_str("]}\n</script>\n<script>\n");
    out.push_str(SCRIPT);
    out.push_str("</script>\n</body>\n</html>\n");
    out
}

/// Returns the search index as JSON: the stem of each page, and every id of
/// a declaration, function, constant, or static, with the index of its
/// page, then the headings of each facade's API, which `api` lists by page.
pub(crate) fn search(run: &[Anchors], api: &[Vec<String>]) -> String {
    let mut out = String::from("{\"pages\":[");
    for (i, page) in run.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json(page.file.trim_end_matches(".html")));
    }
    out.push_str("],\"ids\":[");
    let mut first = true;
    for (i, page) in run.iter().enumerate() {
        let ids = page.ids.iter().chain(page.values.values());
        let headings = api.get(i).into_iter().flatten();
        for id in ids.filter(|id| !id.is_empty()).chain(headings) {
            if !first {
                out.push(',');
            }
            first = false;
            let _ = write!(out, "\n[{i},{}]", json(id));
        }
    }
    out.push_str("\n]}");
    out
}

/// Each page of a file an earlier run wrote, by stem: its stamp and its
/// packed content. A file with no data block, or one that does not parse,
/// gives none, so every page takes the run's stamp.
fn old_pages(old: &str) -> BTreeMap<String, (String, String)> {
    let block = old
        .split_once(DATA_OPEN)
        .and_then(|(_, rest)| rest.split_once("</script>"))
        .map(|(block, _)| block);
    let Some(Ok(data)) = block.map(serde_json::from_str::<serde_json::Value>) else {
        return BTreeMap::new();
    };
    let pages = data["pages"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    pages
        .iter()
        .filter_map(|page| {
            let text = |key: &str| page[key].as_str().map(str::to_owned);
            Some((text("stem")?, (text("stamp")?, text("data")?)))
        })
        .collect()
}

/// Compresses text with zlib and writes it as base64, which the app's
/// `DecompressionStream("deflate")` reads back.
fn pack(text: &str) -> String {
    let level = CompressionLevel::BestCompression as u8;
    base64(&compress_to_vec_zlib(text.as_bytes(), level))
}

/// Writes bytes as standard base64, with padding.
fn base64(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = chunk.iter().enumerate().fold(0u32, |group, (i, &byte)| {
            group | u32::from(byte) << (16 - 8 * i)
        });
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(DIGITS[(group >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Quotes text as a JSON string that is safe inside a script element: a
/// `<` is written as its escape, so no text can close the element.
fn json(text: &str) -> String {
    serde_json::Value::String(text.to_owned())
        .to_string()
        .replace('<', "\\u003c")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::model::ValueRef;

    fn content(time: &str, body: &str) -> String {
        format!("<main>\n{}\n{body}</main>\n", stamp::html_paragraph(time))
    }

    /// The data block of a file, parsed.
    fn data(file: &str) -> serde_json::Value {
        let block = file
            .split_once(DATA_OPEN)
            .and_then(|(_, rest)| rest.split_once("</script>"))
            .expect("the file holds a data block")
            .0;
        serde_json::from_str(block).expect("the data block is JSON")
    }

    #[test]
    fn base64_follows_the_standard_alphabet_and_padding() {
        let cases = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (text, encoded) in cases {
            assert_eq!(base64(text.as_bytes()), encoded, "{text}");
        }
        assert_eq!(base64(&[0xfb, 0xff]), "+/8=");
    }

    #[test]
    fn the_file_holds_the_style_the_app_and_each_page_packed_with_its_stamp() {
        let alpha = content("2026-01-01T00:00:00Z", "<p>a</p>\n");
        let pages = [Page {
            stem: "alpha",
            title: "a <b>",
            content: &alpha,
        }];
        let file = file("2026-01-01T00:00:00Z", &pages, "{}", None);
        assert!(file.starts_with("<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n"));
        assert!(file.contains(STYLE));
        assert!(file.ends_with(&format!("<script>\n{SCRIPT}</script>\n</body>\n</html>\n")));
        assert!(
            file.contains("\"title\":\"a \\u003cb>\""),
            "a < cannot close the element"
        );
        let data = data(&file);
        assert_eq!(data["pages"][0]["stem"], "alpha");
        assert_eq!(data["pages"][0]["stamp"], "2026-01-01T00:00:00Z");
        let packed = data["pages"][0]["data"].as_str().unwrap();
        assert_eq!(packed, pack(&content(stamp::PLACEHOLDER, "<p>a</p>\n")));
        assert_eq!(data["search"].as_str().unwrap(), pack("{}"));
    }

    /// Pages named by stem, each titled with its stem.
    fn pages<'a>(list: &'a [(&'static str, String)]) -> Vec<Page<'a>> {
        list.iter()
            .map(|(stem, content)| Page {
                stem,
                title: stem,
                content,
            })
            .collect()
    }

    #[test]
    fn a_page_whose_content_is_the_same_keeps_its_old_stamp() {
        let (first, later) = ("2026-01-01T00:00:00Z", "2027-06-30T12:00:00Z");
        let before = [
            ("same", content(first, "<p>x</p>\n")),
            ("edited", content(first, "<p>x</p>\n")),
        ];
        let old = file(first, &pages(&before), "{}", None);
        let after = [
            ("same", content(later, "<p>x</p>\n")),
            ("edited", content(later, "<p>y</p>\n")),
        ];
        let data = data(&file(later, &pages(&after), "{}", Some(&old)));
        assert_eq!(data["pages"][0]["stamp"], first);
        assert_eq!(data["pages"][1]["stamp"], later);
        let rerun = [
            ("same", content(later, "<p>x</p>\n")),
            ("edited", content(later, "<p>x</p>\n")),
        ];
        let unchanged = file(later, &pages(&rerun), "{}", Some(&old));
        assert_ne!(unchanged, old, "the first line holds the run's stamp");
        assert_eq!(stamp::mask(&unchanged), stamp::mask(&old));
    }

    #[test]
    fn a_file_with_no_data_block_keeps_no_stamp() {
        assert!(old_pages("<!DOCTYPE html>\n<html>\n").is_empty());
        assert!(old_pages(&format!("{DATA_OPEN}\nnot json\n</script>")).is_empty());
    }

    #[test]
    fn the_search_index_lists_each_page_and_its_ids() {
        let mut values = BTreeMap::new();
        values.insert(ValueRef { module: 0, item: 0 }, "fn.b.run".to_owned());
        let run = [
            Anchors {
                file: "a.html".to_owned(),
                ids: vec!["type.a.A".to_owned()],
                values: BTreeMap::new(),
            },
            Anchors {
                file: "b.html".to_owned(),
                ids: Vec::new(),
                values,
            },
        ];
        assert_eq!(
            search(&run, &[vec!["api.Run".to_owned()]]),
            "{\"pages\":[\"a\",\"b\"],\"ids\":[\n[0,\"type.a.A\"],\n[0,\"api.Run\"],\
             \n[1,\"fn.b.run\"]\n]}"
        );
    }

    #[test]
    fn the_app_reads_the_data_block_and_cannot_close_its_own_element() {
        assert!(SCRIPT.contains("document.getElementById('vibelight-data')"));
        assert!(SCRIPT.contains("new DecompressionStream('deflate')"));
        assert!(!SCRIPT.to_ascii_lowercase().contains("</script"));
    }

    #[test]
    fn the_app_puts_the_time_back_only_on_the_stamp_line_that_mask_masks() {
        let start = "<p class=\"stamp\">Updated ";
        assert!(SCRIPT.contains(&format!("'{}'", stamp::PLACEHOLDER)));
        assert!(SCRIPT.contains(&format!("var STAMP_LINE = /^{start}.*$/gm;")));
        let line = stamp::html_paragraph(stamp::PLACEHOLDER);
        assert!(line.starts_with(start) && stamp::is_line(&line));
        let quoted = format!("<p>\"{}\"</p>\n", stamp::PLACEHOLDER);
        assert_eq!(stamp::mask(&quoted), quoted, "only a stamp line is masked");
    }
}
