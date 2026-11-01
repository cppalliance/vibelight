//! Stamps every shown node of a crate with how far it reaches, for the
//! HTML: the rank its written visibility sorts by, its effective access,
//! and that access spelled when it is less than written. The stamps go on
//! each node before attach, which moves impl members into their types and
//! so loses the module their visibility is written in.

use crate::access::{Access, spell, written};
use crate::analysis::AnalysisRun;
use crate::analysis_items::{ItemId, ItemRef};
use crate::model::{Crate, ModId, Vis};

/// How far one node reaches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Stamp {
    /// 0 for `pub`, otherwise 1 plus the depth of the module that the
    /// written visibility reaches, so wider visibilities sort first.
    pub(crate) rank: usize,
    /// The access in effect.
    pub(crate) eff: Access,
    /// The access in effect spelled from the node's own module, when it is
    /// less than the written visibility gives.
    pub(crate) reaches: Option<String>,
}

impl Stamp {
    /// Returns the access a container's `reaches` note shows, if it shows
    /// one, which its nodes that reach as far leave unsaid.
    pub(crate) fn note(&self) -> Option<Access> {
        self.reaches.as_ref().map(|_| self.eff)
    }
}

/// Returns the stamp of a node whose visibility `vis` is written in module
/// `m` and whose access in effect is `eff`.
fn stamp(krate: &Crate, vis: Vis, m: ModId, eff: Access) -> Stamp {
    let given = written(krate, vis, m);
    let rank = match given {
        Access::World => 0,
        Access::Within(w) => 1 + krate.modules[w].path.len(),
    };
    Stamp {
        rank,
        eff,
        reaches: (eff != given).then(|| spell(krate, eff, m)),
    }
}

/// The stamps of one crate's shown nodes, by the indexes collection gave.
pub(crate) struct Stamps {
    modules: Vec<Stamp>,
    decls: Vec<Stamp>,
    /// Each declaration's own members: fields and trait items.
    members: Vec<Vec<Stamp>>,
    /// Each impl's inherent members.
    impls: Vec<Vec<Stamp>>,
    /// Each module's items.
    items: Vec<Vec<Stamp>>,
}

/// Returns the stamps of target `t`'s nodes.
pub(crate) fn stamps(run: &AnalysisRun<'_>, t: usize) -> Stamps {
    let krate = &run.krates[t];
    let accesses = &run.accesses[t];
    let eff = |item: ItemRef| {
        run.access_of(ItemId { target: t, item })
            .unwrap_or(Access::World)
    };
    let modules = (krate.modules.iter().enumerate())
        .map(|(m, module)| {
            let parent = module.parent.unwrap_or(0);
            stamp(krate, module.vis, parent, accesses.modules[m])
        })
        .collect();
    let decls = (krate.decls.iter().enumerate())
        .map(|(d, decl)| stamp(krate, decl.vis, decl.module, accesses.decls[d]))
        .collect();
    let members = (krate.decls.iter().enumerate())
        .map(|(d, decl)| {
            (decl.members.iter().enumerate())
                .map(|(k, member)| stamp(krate, member.vis, decl.module, eff(ItemRef::Own(d, k))))
                .collect()
        })
        .collect();
    let impls = (krate.impls.iter().enumerate())
        .map(|(i, imp)| {
            (imp.members.iter().enumerate())
                .map(|(k, member)| {
                    let reach = eff(ItemRef::Inherent(i, k));
                    stamp(krate, member.vis, imp.module, reach)
                })
                .collect()
        })
        .collect();
    let items = (krate.modules.iter().enumerate())
        .map(|(m, module)| {
            (module.items.iter().enumerate())
                .map(|(k, item)| {
                    let at = crate::model::ValueRef { module: m, item: k };
                    stamp(krate, item.vis, m, accesses.value(at))
                })
                .collect()
        })
        .collect();
    Stamps {
        modules,
        decls,
        members,
        impls,
        items,
    }
}

/// Writes each stamp onto its node, before attach.
pub(crate) fn apply(krate: &mut Crate, stamps: Stamps) {
    for (module, stamp) in krate.modules.iter_mut().zip(stamps.modules) {
        module.access = Some(stamp);
    }
    let decls = krate.decls.iter_mut().zip(stamps.decls);
    for ((decl, stamp), members) in decls.zip(stamps.members) {
        decl.meta.access = Some(stamp);
        for (member, stamp) in decl.members.iter_mut().zip(members) {
            member.meta.access = Some(stamp);
        }
    }
    for (imp, members) in krate.impls.iter_mut().zip(stamps.impls) {
        for (member, stamp) in imp.members.iter_mut().zip(members) {
            member.meta.access = Some(stamp);
        }
    }
    for (module, items) in krate.modules.iter_mut().zip(stamps.items) {
        for (item, stamp) in module.items.iter_mut().zip(items) {
            item.meta.access = Some(stamp);
        }
    }
}
