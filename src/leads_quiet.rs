//! Drops the leads that the config's `quiet` entries name. An entry is a
//! lead id, which drops every lead of that id, or a lead id, `:`, and a
//! glob, which drops a lead of that id when the glob matches one of its
//! subjects' paths below their crate's root, with `::` between segments,
//! such as `forwarder:detail::*`. An entry that drops no lead is a note,
//! as an `exclude` glob that matches no crate is.

use crate::glob;
use crate::leads::Lead;

/// One `quiet` entry, read.
struct Entry<'q> {
    text: &'q str,
    id: &'q str,
    /// The glob with `/` between its segments, as [`glob::matches`] reads
    /// it.
    glob: Option<String>,
}

impl Entry<'_> {
    fn matches(&self, lead: &Lead) -> bool {
        lead.id == self.id
            && self.glob.as_ref().is_none_or(|glob| {
                (lead.subjects.iter()).any(|s| glob::matches(glob, &s.path.join("/")))
            })
    }
}

/// Drops from `leads` each lead that an entry of `quiet` names. Returns,
/// for each of the run's `targets`, how many dropped leads its outline
/// would print, and a note for each entry that drops no lead, in the order
/// the entries are given.
pub(crate) fn apply(
    leads: &mut Vec<Lead>,
    quiet: &[String],
    targets: usize,
) -> (Vec<usize>, Vec<String>) {
    let entries: Vec<Entry<'_>> = (quiet.iter())
        .map(|text| match text.split_once(':') {
            Some((id, glob)) => Entry {
                text,
                id: id.trim(),
                glob: Some(glob.trim().replace("::", "/")),
            },
            None => Entry {
                text,
                id: text.trim(),
                glob: None,
            },
        })
        .collect();
    let mut used = vec![false; entries.len()];
    let mut counts = vec![0; targets];
    leads.retain(|lead| {
        let mut dropped = false;
        for (entry, used) in entries.iter().zip(&mut used) {
            if entry.matches(lead) {
                *used = true;
                dropped = true;
            }
        }
        if dropped {
            for &t in &lead.target_files {
                if let Some(count) = counts.get_mut(t) {
                    *count += 1;
                }
            }
        }
        !dropped
    });
    let notes = (entries.iter().zip(&used))
        .filter(|(_, used)| !**used)
        .map(|(entry, _)| format!("quiet entry {} matches no lead", entry.text))
        .collect();
    (counts, notes)
}

#[cfg(test)]
mod tests {
    use crate::leads::{Lead, Mode, Subject};

    fn lead(id: &'static str, paths: &[(usize, &str)]) -> Lead {
        let mut lead = Lead::new(id, Mode::DirectSyntax, "seen");
        for &(target, path) in paths {
            lead.subjects.push(Subject {
                target,
                module: 0,
                path: path.split("::").map(str::to_owned).collect(),
            });
            if !lead.target_files.contains(&target) {
                lead.target_files.push(target);
            }
        }
        lead
    }

    fn quiet(entries: &[&str]) -> (Vec<&'static str>, Vec<usize>, Vec<String>) {
        let mut leads = vec![
            lead("forwarder", &[(0, "detail::parse_error_from_inner")]),
            lead("forwarder", &[(0, "parse::run")]),
            lead("repeated-number", &[(0, "fence::is_lua"), (1, "lua::open")]),
            lead("papercut", &[(1, "crate")]),
        ];
        let entries: Vec<String> = entries.iter().map(|e| (*e).to_owned()).collect();
        let (counts, notes) = super::apply(&mut leads, &entries, 2);
        (leads.iter().map(|l| l.id).collect(), counts, notes)
    }

    #[test]
    fn an_id_drops_its_leads_and_a_glob_drops_those_whose_subject_it_matches() {
        let (kept, counts, notes) = quiet(&["repeated-number", "forwarder:detail::*"]);
        assert_eq!(kept, ["forwarder", "papercut"]);
        assert_eq!(counts, [2, 1], "a lead in two targets counts in both");
        assert_eq!(notes, Vec::<String>::new());
        let (kept, ..) = quiet(&["forwarder:**"]);
        assert_eq!(kept, ["repeated-number", "papercut"]);
        let (kept, ..) = quiet(&["papercut:crate"]);
        assert_eq!(kept, ["forwarder", "forwarder", "repeated-number"]);
    }

    #[test]
    fn an_entry_that_drops_no_lead_is_a_note_in_config_order() {
        let (kept, counts, notes) = quiet(&["forwarder:fence::*", "no-such-lead", "papercut"]);
        assert_eq!(kept, ["forwarder", "forwarder", "repeated-number"]);
        assert_eq!(counts, [0, 1]);
        assert_eq!(
            notes,
            [
                "quiet entry forwarder:fence::* matches no lead",
                "quiet entry no-such-lead matches no lead"
            ]
        );
    }
}
