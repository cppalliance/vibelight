//! Splits the dependency order into the groups the HTML prints: each run of
//! types whose modules share their first `DEPTH` path segments is a group,
//! named for those segments.

use crate::model::DeclId;

/// How many segments of a module path name a group: `engine::sched::deep`
/// names the group `engine::sched`.
const DEPTH: usize = 2;

/// One group of the sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Group {
    /// The first segments of the module path that name the group; empty for
    /// the crate root.
    pub(crate) key: Vec<String>,
    pub(crate) decls: Vec<DeclId>,
}

impl Group {
    /// Returns the group's caption: its module path, or `title` for the
    /// crate root.
    pub(crate) fn caption(&self, title: &str) -> String {
        if self.key.is_empty() {
            title.to_owned()
        } else {
            self.key.join("::")
        }
    }

    /// Returns the declaration that leads the group: the first member whose
    /// module names the group, falling back to the first member. A group is
    /// keyed for the run it prints first, so the search finds the member the
    /// caption names. `module_of` gives the path of the module that holds a
    /// declaration.
    pub(crate) fn lead(&self, module_of: impl Fn(DeclId) -> Vec<String>) -> DeclId {
        self.decls
            .iter()
            .copied()
            .find(|&d| {
                let mut key = module_of(d);
                key.truncate(DEPTH);
                key == self.key
            })
            .or_else(|| self.decls.first().copied())
            .unwrap_or_default()
    }
}

/// Splits `sequence` into groups. A run is a stretch of declarations whose
/// modules share their first two segments. A run of one declaration joins
/// the group before it, and one that opens the sequence joins the group
/// after it, so no group is a single type unless the sequence has no run of
/// two. A group that absorbs a run from the start of the sequence takes that
/// run's key, so its key always names the module of the declaration it
/// prints first. `module_of` gives the path of the module that holds a
/// declaration.
pub(crate) fn groups(sequence: &[DeclId], module_of: impl Fn(DeclId) -> Vec<String>) -> Vec<Group> {
    let mut runs: Vec<Group> = Vec::new();
    for &d in sequence {
        let mut key = module_of(d);
        key.truncate(DEPTH);
        match runs.last_mut() {
            Some(run) if run.key == key => run.decls.push(d),
            _ => runs.push(Group {
                key,
                decls: vec![d],
            }),
        }
    }
    let mut out: Vec<Group> = Vec::new();
    let mut leading: Option<Group> = None;
    for run in runs {
        let lone = run.decls.len() == 1;
        match (out.last_mut(), lone) {
            (Some(last), true) => last.decls.extend(run.decls),
            (None, true) => match &mut leading {
                Some(first) => first.decls.extend(run.decls),
                None => leading = Some(run),
            },
            (_, false) => {
                let mut group = run;
                if let Some(first) = leading.take() {
                    group.key = first.key;
                    let mut decls = first.decls;
                    decls.extend(group.decls);
                    group.decls = decls;
                }
                out.push(group);
            }
        }
    }
    if let Some(first) = leading {
        out.push(first);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the module paths named by `modules`, for declarations `0..`.
    fn paths(modules: &[&str]) -> Vec<Vec<String>> {
        modules
            .iter()
            .map(|m| {
                m.split("::")
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .collect()
    }

    /// Groups declarations `0..` whose module paths are given.
    fn split(modules: &[&str]) -> Vec<(String, Vec<usize>)> {
        let paths = paths(modules);
        let sequence: Vec<usize> = (0..modules.len()).collect();
        groups(&sequence, |d| paths[d].clone())
            .into_iter()
            .map(|g| (g.caption("crate"), g.decls))
            .collect()
    }

    /// Returns the lead of each group of declarations `0..` whose module
    /// paths are given.
    fn leads(modules: &[&str]) -> Vec<usize> {
        let paths = paths(modules);
        let sequence: Vec<usize> = (0..modules.len()).collect();
        groups(&sequence, |d| paths[d].clone())
            .into_iter()
            .map(|g| g.lead(|d| paths[d].clone()))
            .collect()
    }

    fn group(caption: &str, decls: &[usize]) -> (String, Vec<usize>) {
        (caption.to_owned(), decls.to_vec())
    }

    #[test]
    fn an_empty_sequence_has_no_groups() {
        assert_eq!(split(&[]), []);
    }

    #[test]
    fn a_run_in_one_module_is_one_group_named_for_it() {
        assert_eq!(split(&["a", "a", "a"]), [group("a", &[0, 1, 2])]);
    }

    #[test]
    fn the_crate_root_is_named_for_the_crate() {
        assert_eq!(split(&["", ""]), [group("crate", &[0, 1])]);
    }

    #[test]
    fn a_module_change_starts_a_group_and_the_same_module_again_starts_another() {
        assert_eq!(
            split(&["a", "a", "b", "b", "a", "a"]),
            [
                group("a", &[0, 1]),
                group("b", &[2, 3]),
                group("a", &[4, 5])
            ]
        );
    }

    #[test]
    fn only_the_first_two_segments_name_a_group() {
        assert_eq!(
            split(&["a::b", "a::b::c", "a::b::c::d", "a::x", "a::x"]),
            [group("a::b", &[0, 1, 2]), group("a::x", &[3, 4])]
        );
    }

    #[test]
    fn a_lone_type_joins_the_group_before_it() {
        assert_eq!(
            split(&["a", "a", "b", "c", "c"]),
            [group("a", &[0, 1, 2]), group("c", &[3, 4])]
        );
        assert_eq!(
            split(&["a", "a", "b", "c", "d", "e", "e"]),
            [group("a", &[0, 1, 2, 3, 4]), group("e", &[5, 6])]
        );
    }

    #[test]
    fn a_lone_type_that_opens_the_sequence_joins_the_group_after_it() {
        // The group the opening run joins takes that run's key, so the
        // caption names the module of the type the group prints first. An
        // empty key still names the crate.
        assert_eq!(split(&["x", "net", "net"]), [group("x", &[0, 1, 2])]);
        assert_eq!(split(&["", "net", "net"]), [group("crate", &[0, 1, 2])]);
        assert_eq!(split(&["x", "y", "z", "z"]), [group("x", &[0, 1, 2, 3])]);
    }

    #[test]
    fn a_group_that_absorbs_the_opening_run_leads_with_it() {
        assert_eq!(leads(&["x", "y", "z", "z"]), [0]);
        assert_eq!(leads(&["", "net", "net"]), [0]);
    }

    #[test]
    fn a_lone_type_that_joins_the_group_before_it_leads_only_its_own_module() {
        // The lone `b` joins a group keyed `a`, so the group still leads with
        // the first type of `a` and not the foreign one it prints last.
        assert_eq!(leads(&["a", "a", "b"]), [0]);
    }

    #[test]
    fn a_sequence_of_lone_types_is_one_group_named_for_the_first() {
        assert_eq!(split(&["a", "b", "c"]), [group("a", &[0, 1, 2])]);
        assert_eq!(split(&["a"]), [group("a", &[0])]);
    }
}
