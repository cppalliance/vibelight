//! The leads of a run: what the detector groups find in the analysis view,
//! for an analyst to confirm or extinguish in the source. A lead is never a
//! finding. It says how its evidence was found and, when it rests on
//! something the run does not observe, what the run cannot see. More than
//! [`SUMMARY_FROM`] leads of one id in one module become one summary lead.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::leads_items::{impl_module, parent_module};
use crate::model::{FileId, Loc, ModId, path_order};
use crate::{leads_body, leads_census, leads_docs, leads_names, leads_shapes};

pub(crate) use crate::leads_check::validate;

/// How a lead's evidence was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Mode {
    /// Through observed uses, call edges, or a library's public surface.
    Resolved,
    /// From the syntax of declarations, signatures, and bodies, with the
    /// names in them resolved.
    DirectSyntax,
    /// From name hits in the lexical census.
    Lexical,
}

/// The blind spots a lead may name, sorted.
pub(crate) const LIMITS: [&str; 6] = [
    "cfg-selection",
    "dynamic-dispatch",
    "external-consumers",
    "external-impls",
    "generated-code",
    "macros",
];

/// The most leads of one id in one module that print one by one.
pub(crate) const SUMMARY_FROM: usize = 5;

/// One thing a lead names: an item of the run, or a part of one such as a
/// parameter or a variant's field, by its path below its crate's root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Subject {
    pub(crate) target: usize,
    /// The module that declares it: a member's or a part's is its
    /// declaration's, and a module's is its parent's.
    pub(crate) module: ModId,
    pub(crate) path: Vec<String>,
}

/// A line of a target's source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceLoc {
    pub(crate) target: usize,
    pub(crate) file: FileId,
    pub(crate) line: u32,
}

/// What a summary lead stands for: the leads of its id in one module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Summary {
    pub(crate) target: usize,
    pub(crate) module: ModId,
    /// How many leads it stands for.
    pub(crate) count: usize,
}

/// One lead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Lead {
    pub(crate) id: &'static str,
    pub(crate) mode: Mode,
    /// Sorted by package, target, and path, each once.
    pub(crate) subjects: Vec<Subject>,
    /// What the detector saw, on one line.
    pub(crate) evidence: String,
    /// Where the subjects are written, sorted, each once.
    pub(crate) locs: Vec<SourceLoc>,
    /// The blind spots of a lead that rests on absence, sorted, each once.
    pub(crate) limits: Vec<&'static str>,
    /// The targets whose outlines print it, ascending: every target its
    /// subjects and locations are in.
    pub(crate) target_files: Vec<usize>,
    pub(crate) summary: Option<Summary>,
}

impl Lead {
    /// Starts a lead with no subject.
    pub(crate) fn new(id: &'static str, mode: Mode, evidence: impl Into<String>) -> Lead {
        Lead {
            id,
            mode,
            subjects: Vec::new(),
            evidence: evidence.into(),
            locs: Vec::new(),
            limits: Vec::new(),
            target_files: Vec::new(),
            summary: None,
        }
    }

    /// Adds an item as a subject, located where it is written.
    pub(crate) fn item(self, run: &AnalysisRun<'_>, item: ItemId) -> Lead {
        let loc = run.loc(item);
        let lead = self.part(run, item, &[]);
        match loc {
            Some(loc) => lead.at(item.target, loc),
            None => lead,
        }
    }

    /// Adds a part of an item as a subject, such as a parameter or a
    /// variant's field: its path is the item's, then `part`.
    pub(crate) fn part(mut self, run: &AnalysisRun<'_>, item: ItemId, part: &[String]) -> Lead {
        let mut path = run.path(item);
        path.extend(part.iter().cloned());
        self.subjects.push(Subject {
            target: item.target,
            module: module_of(run, item),
            path,
        });
        self
    }

    /// Adds a location in target `t`.
    pub(crate) fn at(mut self, t: usize, loc: Loc) -> Lead {
        self.locs.push(SourceLoc {
            target: t,
            file: loc.file,
            line: loc.line,
        });
        self
    }

    /// Names the blind spots of a lead that rests on absence.
    pub(crate) fn limits(mut self, limits: &[&'static str]) -> Lead {
        self.limits.extend_from_slice(limits);
        self
    }

    /// Returns the one module that holds every subject, with its target,
    /// when the lead prints in that target alone.
    pub(crate) fn home(&self) -> Option<(usize, ModId)> {
        let first = self.subjects.first()?;
        let alone = self.target_files == [first.target];
        let shared =
            (self.subjects.iter()).all(|s| (s.target, s.module) == (first.target, first.module));
        (alone && shared).then_some((first.target, first.module))
    }

    /// Sorts the lead's lists, drops repeats, and names the targets it
    /// prints in.
    fn settle(&mut self, run: &AnalysisRun<'_>) {
        self.subjects.sort_by(|a, b| subject_order(run, a, b));
        self.subjects
            .dedup_by(|a, b| (a.target, &a.path) == (b.target, &b.path));
        self.locs.sort();
        self.locs.dedup();
        self.limits.sort_unstable();
        self.limits.dedup();
        let targets: BTreeSet<usize> = (self.subjects.iter().map(|s| s.target))
            .chain(self.locs.iter().map(|l| l.target))
            .collect();
        self.target_files = targets.into_iter().collect();
    }
}

/// Returns the module that declares an item, as [`Subject::module`] holds
/// it.
fn module_of(run: &AnalysisRun<'_>, item: ItemId) -> ModId {
    let krate = &run.krates[item.target];
    match item.item {
        ItemRef::Module(m) => parent_module(krate, m).unwrap_or(m),
        ItemRef::Decl(d) | ItemRef::Own(d, _) | ItemRef::Variant(d, _) => krate.decls[d].module,
        ItemRef::Value(at) => at.module,
        ItemRef::Inherent(i, _) | ItemRef::TraitImpl(i, _) => match run.attached[item.target][i] {
            Some(d) => krate.decls[d].module,
            None => impl_module(krate, i),
        },
    }
}

/// A detector group: its lead ids in glossary order, those among them that
/// rest on absence, and what finds them.
struct Group {
    ids: &'static [&'static str],
    absent: &'static [&'static str],
    find: fn(&AnalysisRun<'_>, &mut Vec<Lead>),
}

const GROUPS: [Group; 5] = [
    Group {
        ids: leads_shapes::IDS,
        absent: leads_shapes::ABSENT,
        find: leads_shapes::find,
    },
    Group {
        ids: leads_names::IDS,
        absent: leads_names::ABSENT,
        find: leads_names::find,
    },
    Group {
        ids: leads_docs::IDS,
        absent: leads_docs::ABSENT,
        find: leads_docs::find,
    },
    Group {
        ids: leads_census::IDS,
        absent: leads_census::ABSENT,
        find: leads_census::find,
    },
    Group {
        ids: leads_body::IDS,
        absent: leads_body::ABSENT,
        find: leads_body::find,
    },
];

/// Returns every registered lead id, group by group, in glossary order.
pub(crate) fn ids() -> impl Iterator<Item = &'static str> {
    GROUPS.iter().flat_map(|group| group.ids.iter().copied())
}

/// Tells whether the leads of an id rest on absence, so that each names
/// its limits.
pub(crate) fn rests_on_absence(id: &str) -> bool {
    GROUPS.iter().any(|group| group.absent.contains(&id))
}

/// Runs every detector group and returns the run's leads, each with its
/// lists sorted, more than [`SUMMARY_FROM`] leads of one id in one module
/// summarized, in lead order.
pub(crate) fn find(run: &AnalysisRun<'_>) -> Vec<Lead> {
    let mut found = Vec::new();
    for group in &GROUPS {
        (group.find)(run, &mut found);
    }
    for lead in &mut found {
        lead.settle(run);
    }
    let mut leads = summarize(run, found);
    leads.sort_by(|a, b| lead_order(run, a, b));
    leads
}

/// Collapses each set of more than [`SUMMARY_FROM`] leads that share an id
/// and a home module into one summary lead: their subjects, locations, and
/// limits, their evidence in lead order, and their weakest mode.
fn summarize(run: &AnalysisRun<'_>, leads: Vec<Lead>) -> Vec<Lead> {
    let mut homes: BTreeMap<(&'static str, usize, ModId), Vec<Lead>> = BTreeMap::new();
    let mut out = Vec::new();
    for lead in leads {
        match lead.home() {
            Some((t, m)) => homes.entry((lead.id, t, m)).or_default().push(lead),
            None => out.push(lead),
        }
    }
    for ((id, target, module), mut group) in homes {
        if group.len() <= SUMMARY_FROM {
            out.extend(group);
            continue;
        }
        group.sort_by(|a, b| lead_order(run, a, b));
        let mut summary = Lead::new(id, Mode::Resolved, String::new());
        let mut seen = BTreeSet::new();
        let mut evidence = Vec::new();
        for lead in &group {
            summary.mode = summary.mode.max(lead.mode);
            if seen.insert(lead.evidence.as_str()) {
                evidence.push(lead.evidence.as_str());
            }
            summary.subjects.extend(lead.subjects.iter().cloned());
            summary.locs.extend(lead.locs.iter().copied());
            summary.limits.extend(lead.limits.iter().copied());
        }
        summary.evidence = evidence.join("; ");
        summary.settle(run);
        summary.summary = Some(Summary {
            target,
            module,
            count: group.len(),
        });
        out.push(summary);
    }
    out
}

/// Orders subjects by package, target, and path.
pub(crate) fn subject_order(run: &AnalysisRun<'_>, a: &Subject, b: &Subject) -> Ordering {
    let package = |s: &Subject| run.targets[s.target].package;
    (package(a).cmp(&package(b)))
        .then(a.target.cmp(&b.target))
        .then_with(|| path_order(&a.path, &b.path))
}

/// Orders leads by id in registry order, then by subjects, evidence, and
/// locations; a summary follows the lead with the same lists.
pub(crate) fn lead_order(run: &AnalysisRun<'_>, a: &Lead, b: &Lead) -> Ordering {
    let rank = |id: &str| ids().position(|known| known == id).unwrap_or(usize::MAX);
    let subjects = || {
        for (x, y) in a.subjects.iter().zip(&b.subjects) {
            let order = subject_order(run, x, y);
            if order != Ordering::Equal {
                return order;
            }
        }
        a.subjects.len().cmp(&b.subjects.len())
    };
    (rank(a.id).cmp(&rank(b.id)))
        .then_with(subjects)
        .then_with(|| a.evidence.cmp(&b.evidence))
        .then_with(|| a.locs.cmp(&b.locs))
        .then_with(|| a.summary.is_some().cmp(&b.summary.is_some()))
}
