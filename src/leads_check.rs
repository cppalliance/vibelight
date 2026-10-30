//! Checks the invariants of a run's leads before anything reads them, so a
//! broken lead stops the run as an internal error rather than reaching an
//! outline: the registry, then each lead's id, mode, subjects, evidence,
//! locations, limits, targets, and summary, then the order of the leads and
//! the summary rule.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::leads::{
    LIMITS, Lead, Mode, SUMMARY_FROM, ids, lead_order, rests_on_absence, subject_order,
};
use crate::model::ModId;

/// Checks the leads of a run against the run.
///
/// # Errors
/// Returns a message that names the first invariant that fails.
pub(crate) fn validate(leads: &[Lead], run: &AnalysisRun<'_>) -> Result<(), String> {
    let registry: Vec<&str> = ids().collect();
    let unique: BTreeSet<&str> = registry.iter().copied().collect();
    let kebab = |id: &str| {
        !id.is_empty()
            && !id.starts_with('-')
            && !id.ends_with('-')
            && !id.contains("--")
            && id.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
    };
    if unique.len() != registry.len() || !registry.iter().all(|id| kebab(id)) {
        return Err("the registry repeats an id or holds one that is not kebab-case".to_owned());
    }
    let mut homes: BTreeMap<(&str, usize, ModId), (usize, bool)> = BTreeMap::new();
    for (n, lead) in leads.iter().enumerate() {
        check_lead(lead, run).map_err(|why| format!("a {} lead {why}", lead.id))?;
        let previous = n.checked_sub(1).and_then(|p| leads.get(p));
        if previous.is_some_and(|p| lead_order(run, p, lead) != Ordering::Less) {
            return Err(format!("the leads are not in order at a {} lead", lead.id));
        }
        if let Some((t, m)) = lead.home() {
            let home = homes.entry((lead.id, t, m)).or_default();
            home.0 += 1;
            home.1 |= lead.summary.is_some();
        }
    }
    for ((id, ..), (count, summarized)) in homes {
        if (summarized && count > 1) || (!summarized && count > SUMMARY_FROM) {
            return Err(format!("the {id} leads of one module are not summarized"));
        }
    }
    Ok(())
}

/// Checks one lead's fields.
fn check_lead(lead: &Lead, run: &AnalysisRun<'_>) -> Result<(), String> {
    if !ids().any(|id| id == lead.id) {
        return Err("has an id the registry does not hold".to_owned());
    }
    let absent = rests_on_absence(lead.id);
    let moded = match lead.mode {
        Mode::Resolved | Mode::Lexical => true,
        Mode::DirectSyntax => !absent,
    };
    if !moded || (absent && lead.limits.is_empty()) {
        return Err("rests on absence without observed evidence or limits".to_owned());
    }
    let one_line = !lead.evidence.trim().is_empty() && !lead.evidence.contains('\n');
    if lead.subjects.is_empty() || !one_line {
        return Err("has no subject or no evidence on one line".to_owned());
    }
    for subject in &lead.subjects {
        let placed = run
            .krates
            .get(subject.target)
            .is_some_and(|krate| subject.module < krate.modules.len());
        if !placed || subject.path.is_empty() || subject.path.iter().any(String::is_empty) {
            return Err(format!("names {} out of place", subject.path.join("::")));
        }
    }
    let ordered = (lead.subjects.windows(2)).all(|w| subject_order(run, &w[0], &w[1]).is_lt());
    if !ordered || !lead.locs.windows(2).all(|w| w[0] < w[1]) {
        return Err("has subjects or locations out of order".to_owned());
    }
    for loc in &lead.locs {
        let file = run
            .krates
            .get(loc.target)
            .and_then(|krate| krate.files.get(loc.file));
        if !file.is_some_and(|f| loc.line >= 1 && loc.line <= f.line_of(f.text.len())) {
            return Err(format!(
                "has a location out of its file at line {}",
                loc.line
            ));
        }
    }
    let known = lead.limits.iter().all(|limit| LIMITS.contains(limit));
    if !known || !lead.limits.windows(2).all(|w| w[0] < w[1]) {
        return Err("names a limit that is unknown or out of order".to_owned());
    }
    let targets: Vec<usize> = (lead.subjects.iter().map(|s| s.target))
        .chain(lead.locs.iter().map(|l| l.target))
        .collect::<BTreeSet<usize>>()
        .into_iter()
        .collect();
    if lead.target_files != targets {
        return Err("prints in other targets than its subjects and locations".to_owned());
    }
    if let Some(summary) = &lead.summary {
        let home = lead.home() == Some((summary.target, summary.module));
        if !home || summary.count <= SUMMARY_FROM {
            return Err(format!("summarizes {} leads out of place", summary.count));
        }
    }
    Ok(())
}
