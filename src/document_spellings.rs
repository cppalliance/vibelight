//! Builds the Spellings section: each item outside the crate that the
//! declarations write in more than one way, with every way it is written.

use crate::document::{Builder, Inline, Item, Link};
use crate::layout::Span;
use crate::spellings::Entry;

impl Builder<'_> {
    /// Builds one item per entry with a nested item per spelling. The text
    /// is the same with or without source links, so the search hint stays.
    pub(crate) fn spelling_items(&self, entries: &[Entry]) -> Vec<Item> {
        entries
            .iter()
            .map(|entry| Item {
                inlines: vec![
                    Inline::Code(vec![Span::Code(entry.item.clone())]),
                    Inline::Text(format!(
                        " is written {} ways; pick one and search for the rest:",
                        entry.spellings.len()
                    )),
                ],
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
                        if !links.is_empty() {
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

fn times(count: usize) -> String {
    if count == 1 {
        "once".to_owned()
    } else {
        format!("{count} times")
    }
}
