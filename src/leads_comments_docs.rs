//! The papercuts of a run's docs: each doc that holds a papercut word or
//! phrase outside its code spans and its boilerplate passages. The doc
//! group calls it with the docs it reads, so every doc is parsed once.

use crate::analysis::AnalysisRun;
use crate::leads::{Lead, Mode};
use crate::leads_comments::{PAPERCUT_PHRASES, hits, quoted, shown};
use crate::leads_comments_words::{prose_words, without_code};
use crate::leads_docs::Docs;

/// `papercut` in docs: a node's doc that holds a papercut word or phrase
/// outside its code spans, shown by the first line that holds one.
pub(crate) fn find(run: &AnalysisRun<'_>, docs: &Docs<'_>, out: &mut Vec<Lead>) {
    for doc in &docs.all {
        let text = doc.prose.kept();
        let mut found: Vec<&str> = Vec::new();
        let mut first = None;
        for line in text.lines() {
            for phrase in hits(&prose_words(&without_code(line)), &PAPERCUT_PHRASES) {
                if !found.contains(&phrase) {
                    found.push(phrase);
                    first.get_or_insert(line);
                }
            }
        }
        let Some(line) = first else {
            continue;
        };
        let evidence = format!("{} in the doc line \"{}\"", quoted(&found), shown(line));
        let lead = Lead::new("papercut", Mode::DirectSyntax, evidence);
        out.push(doc.add(run, lead));
    }
}
