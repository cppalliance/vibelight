//! Builds the Spellings section: each item outside the crate that the
//! declarations write in more than one way, with every way it is written.

use crate::document::{Builder, Inline, Item, Link};
use crate::layout::Span;
use crate::spellings::Entry;

impl Builder<'_> {
    /// Builds one item per entry with a nested item per spelling. The text
    /// is the same with or without source links, so the search hint stays.
    /// The preferred spelling needs no change, so it lists no lines.
    pub(crate) fn spelling_items(&self, entries: &[Entry]) -> Vec<Item> {
        entries
            .iter()
            .map(|entry| Item {
                inlines: heading(entry),
                children: entry
                    .spellings
                    .iter()
                    .map(|spelling| {
                        let mut inlines = vec![
                            Inline::Code(vec![Span::Code(spelling.text.clone())]),
                            Inline::Text(format!(", {}", times(spelling.count))),
                        ];
                        let links: Vec<Link> = spelling
                            .locs
                            .iter()
                            .filter_map(|&loc| self.link(loc, true))
                            .collect();
                        let preferred = entry.preferred.as_ref() == Some(&spelling.text);
                        if !links.is_empty() && !preferred {
                            inlines.push(Inline::Source(links));
                        }
                        Item {
                            inlines,
                            children: Vec::new(),
                        }
                    })
                    .collect(),
            })
            .collect()
    }
}

/// Returns an entry's first line: the item, how many ways it is written,
/// and the spelling to prefer, or a hint to pick one when there is none.
fn heading(entry: &Entry) -> Vec<Inline> {
    let ways = entry.spellings.len();
    let mut inlines = vec![Inline::Code(vec![Span::Code(entry.item.clone())])];
    match &entry.preferred {
        None => inlines.push(Inline::Text(format!(
            " is written {ways} ways; pick one and search for the rest:"
        ))),
        Some(bare) => {
            inlines.push(Inline::Text(format!(" is written {ways} ways; prefer ")));
            inlines.push(Inline::Code(vec![Span::Code(bare.clone())]));
            let written = entry.spellings.iter().any(|s| s.text == *bare);
            inlines.push(Inline::Text(
                if written {
                    ":"
                } else {
                    ", which is not used yet:"
                }
                .to_owned(),
            ));
        }
    }
    inlines
}

fn times(count: usize) -> String {
    if count == 1 {
        "once".to_owned()
    } else {
        format!("{count} times")
    }
}
