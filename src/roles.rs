//! The part each target of a run plays in the run's structure. A facade is
//! a library whose public surface is mostly re-exports of other targets of
//! the run: the API that users of a crate family write against. Its family
//! is every target its production code reaches through uses, stopping at
//! other facades, and its family's libraries are its implementation crates.
//! A library that a surface count makes a facade but that sits inside
//! another facade's family, and that nothing outside that family uses, is
//! one of that family's implementation crates instead. A target outside a
//! family that uses one of its implementation crates directly bypasses the
//! facade. Every other target is a library or a binary. The config's
//! `facades` globs, when set, choose the facades in place of the counts.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::glob;
use crate::manifest::relative;
use crate::root::TargetKind;
use crate::surface::{Entry, Kind};

/// The part a target plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Role {
    Facade,
    Implementation,
    Library,
    Binary,
}

/// What a target's public surface holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    /// Every path that names no module, test support left out.
    pub(crate) items: usize,
    /// The public modules.
    pub(crate) modules: usize,
    /// For each other target of the run, how many of the items re-export
    /// one of its items.
    pub(crate) from: BTreeMap<usize, usize>,
}

impl Counts {
    /// How many of the items re-export items of other targets of the run.
    pub(crate) fn reexported(&self) -> usize {
        self.from.values().sum()
    }
}

/// The role of each target of a run, in target order, and what decides it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Roles {
    /// The role of each target.
    pub(crate) each: Vec<Role>,
    pub(crate) counts: Vec<Counts>,
    /// For each target, the facades that re-export its items, each with how
    /// many of their items do.
    pub(crate) fronted_by: Vec<BTreeMap<usize, usize>>,
    /// For each target, the facades whose families hold it, the one that
    /// re-exports the most of its items first, then the nearest.
    pub(crate) behind: Vec<Vec<usize>>,
    /// For each target, the implementation crates of a family it is outside
    /// of whose items it uses directly.
    pub(crate) bypasses: Vec<Vec<usize>>,
}

/// The config's choice of facades: globs over crate folders, measured from
/// the config's folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Chosen {
    pub(crate) globs: Vec<String>,
    pub(crate) from: PathBuf,
}

impl Chosen {
    /// Tells whether a glob names the crate folder `dir`.
    pub(crate) fn names(&self, dir: &Path) -> bool {
        relative(&self.from, dir)
            .is_some_and(|rel| self.globs.iter().any(|g| glob::matches(g, &rel)))
    }
}

/// Sorts the targets of a run: `kinds`, `surfaces`, and `uses` are each
/// target's kind, public surface, and the other targets whose items its
/// production code uses. `chosen`, when given, tells for each target
/// whether the config names it a facade.
pub(crate) fn of(
    kinds: &[TargetKind],
    surfaces: &[Vec<Entry>],
    uses: &[BTreeMap<usize, usize>],
    chosen: Option<&[bool]>,
) -> Roles {
    let n = kinds.len();
    let counts: Vec<Counts> = surfaces.iter().map(|surface| count(surface)).collect();
    let library = |t: usize| kinds[t] == TargetKind::Lib;
    let mut facade: Vec<bool> = (0..n)
        .map(|t| library(t) && chosen.map_or_else(|| mostly_reexports(&counts[t]), |c| c[t]))
        .collect();
    if chosen.is_none() {
        let internal: Vec<bool> = (0..n)
            .map(|t| facade[t] && internal(t, &facade, &counts, uses))
            .collect();
        for (t, inside) in internal.into_iter().enumerate() {
            facade[t] &= !inside;
        }
    }
    let families: Vec<BTreeMap<usize, usize>> = (0..n)
        .map(|f| {
            if facade[f] {
                reach(f, uses, |x| facade[x])
            } else {
                BTreeMap::new()
            }
        })
        .collect();
    let mut fronted_by = vec![BTreeMap::new(); n];
    for f in (0..n).filter(|&t| facade[t]) {
        for (&from, &k) in &counts[f].from {
            fronted_by[from].insert(f, k);
        }
    }
    let behind: Vec<Vec<usize>> = (0..n)
        .map(|t| {
            let mut found: Vec<(usize, usize, usize)> = (0..n)
                .filter(|&f| f != t)
                .filter_map(|f| {
                    families[f]
                        .get(&t)
                        .map(|&d| (f, d, fronted_by[t].get(&f).copied().unwrap_or(0)))
                })
                .collect();
            found.sort_by(|a, b| b.2.cmp(&a.2).then(a.1.cmp(&b.1)).then(a.0.cmp(&b.0)));
            found.into_iter().map(|(f, ..)| f).collect()
        })
        .collect();
    let each: Vec<Role> = (0..n)
        .map(|t| {
            if facade[t] {
                Role::Facade
            } else if !library(t) {
                Role::Binary
            } else if behind[t].is_empty() && fronted_by[t].is_empty() {
                Role::Library
            } else {
                Role::Implementation
            }
        })
        .collect();
    let bypasses = (0..n)
        .map(|t| {
            (uses[t].keys().copied())
                .filter(|&m| each[m] == Role::Implementation)
                .filter(|&m| {
                    let holders: Vec<usize> =
                        (0..n).filter(|&f| families[f].contains_key(&m)).collect();
                    !holders.is_empty()
                        && !holders
                            .iter()
                            .any(|&f| f == t || families[f].contains_key(&t))
                })
                .collect()
        })
        .collect();
    Roles {
        each,
        counts,
        fronted_by,
        behind,
        bypasses,
    }
}

/// Returns every target that `start`'s production code reaches through
/// uses, each with how many steps away it is, never entering a target that
/// `stop` names, nor reaching past one.
fn reach(
    start: usize,
    uses: &[BTreeMap<usize, usize>],
    stop: impl Fn(usize) -> bool,
) -> BTreeMap<usize, usize> {
    let mut found = BTreeMap::new();
    let mut queue = VecDeque::from([(start, 0usize)]);
    let mut seen = BTreeSet::from([start]);
    while let Some((at, steps)) = queue.pop_front() {
        for &next in uses[at].keys() {
            if stop(next) || !seen.insert(next) {
                continue;
            }
            found.insert(next, steps + 1);
            queue.push_back((next, steps + 1));
        }
    }
    found
}

/// Tells whether the would-be facade `t` belongs to another would-be
/// facade's family instead: that one reaches it, every target that uses it
/// lies in that family, and a smaller share of its surface re-exports.
fn internal(t: usize, facade: &[bool], counts: &[Counts], uses: &[BTreeMap<usize, usize>]) -> bool {
    let share = |c: &Counts| (c.reexported(), c.items.max(1));
    let (mine, my_items) = share(&counts[t]);
    (0..facade.len()).filter(|&f| f != t && facade[f]).any(|f| {
        let (theirs, their_items) = share(&counts[f]);
        let family = reach(f, uses, |x| facade[x] && x != t);
        mine * their_items < theirs * my_items
            && family.contains_key(&t)
            && (0..uses.len())
                .filter(|&u| uses[u].contains_key(&t))
                .all(|u| u == f || family.contains_key(&u))
    })
}

/// Tells whether at least half of a surface's items re-export items of
/// other targets of the run.
fn mostly_reexports(counts: &Counts) -> bool {
    counts.items > 0 && counts.reexported() * 2 >= counts.items
}

fn count(surface: &[Entry]) -> Counts {
    let mut counts = Counts::default();
    for entry in surface.iter().filter(|entry| !entry.test_support) {
        match &entry.kind {
            Kind::Module(_) => counts.modules += 1,
            Kind::Cross(_, to) => {
                counts.items += 1;
                *counts.from.entry(to.krate).or_default() += 1;
            }
            _ => counts.items += 1,
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CrossRef, Reach};

    fn entry(kind: Kind) -> Entry {
        Entry {
            path: vec!["x".to_owned()],
            kind,
            test_support: false,
        }
    }

    fn own(n: usize) -> Vec<Entry> {
        (0..n).map(|d| entry(Kind::Decl(d))).collect()
    }

    fn from(krate: usize, n: usize) -> Vec<Entry> {
        let to = CrossRef {
            krate,
            reach: Reach::Decl(0),
        };
        (0..n)
            .map(|_| entry(Kind::Cross("a::B".to_owned(), to)))
            .collect()
    }

    fn uses(pairs: &[(usize, usize)], n: usize) -> Vec<BTreeMap<usize, usize>> {
        let mut out = vec![BTreeMap::new(); n];
        for &(user, used) in pairs {
            out[user].insert(used, 1);
        }
        out
    }

    fn libs(n: usize) -> Vec<TargetKind> {
        vec![TargetKind::Lib; n]
    }

    #[test]
    fn a_facade_s_family_is_its_implementation_and_an_outsider_bypasses_it() {
        // 0 re-exports 1 and 2; 1 uses 3, which no facade re-exports; 4 uses
        // 3 from outside the family; 5 is a binary.
        let mut kinds = libs(5);
        kinds.push(TargetKind::Bin);
        let mut facade = from(1, 5);
        facade.extend(from(2, 3));
        facade.extend(own(2));
        facade.push(entry(Kind::Module(1)));
        let surfaces = vec![facade, own(9), own(4), own(2), own(3), own(1)];
        let uses = uses(&[(0, 1), (0, 2), (1, 2), (1, 3), (4, 3)], 6);
        let roles = of(&kinds, &surfaces, &uses, None);
        assert_eq!(
            roles.each,
            [
                Role::Facade,
                Role::Implementation,
                Role::Implementation,
                Role::Implementation,
                Role::Library,
                Role::Binary
            ]
        );
        assert_eq!(roles.counts[0].items, 10);
        assert_eq!(roles.counts[0].modules, 1);
        assert_eq!(roles.counts[0].reexported(), 8);
        assert_eq!(roles.fronted_by[1], BTreeMap::from([(0, 5)]));
        assert_eq!(roles.behind[3], [0], "a family member no facade re-exports");
        assert_eq!(
            roles.bypasses,
            [vec![], vec![], vec![], vec![], vec![3], vec![]],
            "members of one family use each other freely; the outsider bypasses"
        );
    }

    #[test]
    fn a_surface_of_mostly_its_own_items_is_no_facade() {
        let mut mixed = from(1, 2);
        mixed.extend(own(3));
        let roles = of(&libs(2), &[mixed, own(1)], &uses(&[], 2), None);
        assert_eq!(roles.each, [Role::Library, Role::Library]);
    }

    #[test]
    fn a_half_facade_that_only_a_facade_uses_belongs_to_its_family() {
        // 0 is all re-exports of 1 and 2; 1 is half re-exports of 2, and only
        // 0 uses it: the server behind its API crate.
        let mut api = from(1, 3);
        api.extend(from(2, 1));
        let mut server = from(2, 2);
        server.extend(own(2));
        let roles = of(
            &libs(3),
            &[api, server, own(4)],
            &uses(&[(0, 1), (0, 2), (1, 2)], 3),
            None,
        );
        assert_eq!(
            roles.each,
            [Role::Facade, Role::Implementation, Role::Implementation]
        );
    }

    #[test]
    fn a_half_facade_that_crates_outside_the_family_use_stays_a_facade() {
        // 2 is half re-exports and 0 re-exports some of it, but 3, outside
        // 0's family, uses it too: a contract crate other crates build on.
        let surfaces = vec![
            from(2, 4),
            own(1),
            {
                let mut contract = from(1, 3);
                contract.extend(own(2));
                contract
            },
            own(1),
        ];
        let roles = of(
            &libs(4),
            &surfaces,
            &uses(&[(0, 2), (2, 1), (3, 2)], 4),
            None,
        );
        assert_eq!(roles.each[2], Role::Facade);
        assert!(roles.bypasses[3].is_empty(), "using a facade is no bypass");
    }

    #[test]
    fn a_crate_of_only_re_exports_stays_a_facade_inside_another_family() {
        let surfaces = vec![from(1, 2), from(2, 2), own(3)];
        let roles = of(&libs(3), &surfaces, &uses(&[(0, 1), (1, 2)], 3), None);
        assert_eq!(
            roles.each,
            [Role::Facade, Role::Facade, Role::Implementation]
        );
    }

    #[test]
    fn the_config_chooses_the_facades_when_it_names_them() {
        let surfaces = vec![from(1, 3), own(2), own(2)];
        let chosen = [false, false, true];
        let roles = of(
            &libs(3),
            &surfaces,
            &uses(&[(0, 1), (2, 1)], 3),
            Some(&chosen),
        );
        assert_eq!(
            roles.each,
            [Role::Library, Role::Implementation, Role::Facade]
        );
    }

    #[test]
    fn a_binary_is_never_a_facade() {
        let roles = of(&[TargetKind::Bin], &[from(0, 3)], &uses(&[], 1), None);
        assert_eq!(roles.each, [Role::Binary]);
    }

    #[test]
    fn a_glob_names_a_crate_folder_below_the_config() {
        let chosen = Chosen {
            globs: vec!["crates/runvil".to_owned(), "crates/*-api".to_owned()],
            from: PathBuf::from("/repo"),
        };
        assert!(chosen.names(Path::new("/repo/crates/runvil")));
        assert!(chosen.names(Path::new("/repo/crates/server-api")));
        assert!(!chosen.names(Path::new("/repo/crates/runvil-engine")));
        assert!(!chosen.names(Path::new("/elsewhere/crates/runvil")));
    }
}
