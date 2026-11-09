//! The HTML side of a run's outline: each target's page content, a
//! facade's public API or everything its crate declares, and the files of
//! the HTML folder that no one target writes.

use std::collections::BTreeMap;
use std::path::Path;

use crate::document::Document;
use crate::inputs::Chosen;
use crate::model::{Crate, Loc};
use crate::pipeline::{Built, Outcome};
use crate::render_html::Anchors;
use crate::root::{Target, TargetKind};
use crate::settings::Plan;
use crate::{browse, facade_page, index, render_html, roles, root_page};

/// A link to the source line of a location, one per target, relative to
/// the HTML folder.
pub(crate) type MemberLink<'a> = Box<dyn Fn(Loc) -> String + 'a>;

/// Returns each target's member links, in target order.
pub(crate) fn member_links<'a>(krates: &'a [Crate], docs: &'a [Built]) -> Vec<MemberLink<'a>> {
    (krates.iter().zip(docs))
        .map(|(krate, page)| {
            Box::new(move |loc: Loc| {
                let loc = krate.real_loc(loc);
                page.link_base
                    .target(&krate.files[loc.file].rel, Some(loc.line))
            }) as MemberLink<'a>
        })
        .collect()
}

/// Returns every target as a facade's page reaches into it, in target
/// order.
pub(crate) fn owners<'a>(
    targets: &[(usize, &Target)],
    krates: &'a [Crate],
    docs: &'a [Built],
    links: &'a [MemberLink<'a>],
) -> Vec<facade_page::Owner<'a>> {
    (targets.iter().zip(krates).zip(docs).zip(links))
        .map(|(((&(_, target), krate), page), link)| facade_page::Owner {
            krate,
            doc: &page.doc,
            links: Some(&**link),
            file: format!("{}.html", target.stem),
        })
        .collect()
}

/// Returns the ids each target's page declares, in target order.
pub(crate) fn anchors(targets: &[(usize, &Target)], docs: &[Built]) -> Vec<Anchors> {
    (targets.iter().zip(docs))
        .map(|(&(_, target), page)| Anchors {
            file: format!("{}.html", target.stem),
            ids: page.doc.decl_ids.clone(),
            values: page.doc.value_ids.clone(),
        })
        .collect()
}

/// Leaves each facade's document with ids for only what its page shows, so
/// the run's anchors and search index name nothing the page leaves out.
pub(crate) fn keep_public(
    docs: &mut [Built],
    roles: &roles::Roles,
    surfaces: &[Vec<crate::surface::Entry>],
) {
    for ((page, role), surface) in docs.iter_mut().zip(&roles.each).zip(surfaces) {
        if *role == roles::Role::Facade {
            facade_page::keep_public(&mut page.doc, surface);
        }
    }
}

/// Renders target `t`'s page content, a facade's public API or a page of
/// everything its crate declares, with the ids of the headings a facade's
/// page adds for the search index.
pub(crate) fn page_content(
    t: usize,
    owners: &[facade_page::Owner<'_>],
    roles: &roles::Roles,
    surfaces: &[Vec<crate::surface::Entry>],
    run: &[Anchors],
    stamp: &str,
) -> (String, Vec<String>) {
    if roles.each[t] == roles::Role::Facade {
        let facade = facade_page::Facade {
            index: t,
            owners,
            surface: &surfaces[t],
            run,
        };
        return facade_page::fragment(&facade, stamp);
    }
    let own = &owners[t];
    let content = render_html::fragment(own.doc, own.links, run, stamp);
    (content, Vec::new())
}

/// What the files of the HTML folder that no one target writes are made
/// of.
pub(crate) struct Folder<'a> {
    pub(crate) base: &'a Path,
    pub(crate) rows: &'a [root_page::Row],
    pub(crate) roles: &'a roles::Roles,
    pub(crate) entries: &'a [index::Entry],
    pub(crate) run: &'a [Anchors],
    /// The headings each facade's page adds, by page.
    pub(crate) api: &'a [Vec<String>],
}

/// Returns the part each target plays, chosen by the config's `facades`
/// globs when it has them and by each target's surface otherwise.
pub(crate) fn roles_of(
    chosen: &Chosen,
    plan: &Plan,
    targets: &[(usize, &Target)],
    (surfaces, uses): (&[Vec<crate::surface::Entry>], &[BTreeMap<usize, usize>]),
) -> roles::Roles {
    let kinds: Vec<TargetKind> = targets.iter().map(|(_, target)| target.kind).collect();
    let named: Option<Vec<bool>> = plan.facades.as_ref().map(|facades| {
        (targets.iter())
            .map(|&(p, _)| facades.names(&chosen.packages[p].dir))
            .collect()
    });
    roles::of(&kinds, surfaces, uses, named.as_deref())
}

/// Returns a target's row of the overview.
pub(crate) fn row(target: &Target, dir: String, doc: &Document) -> root_page::Row {
    let dashed = target.title.replace('_', "-");
    let names = [
        target.title.as_str(),
        target.crate_name.as_str(),
        dashed.as_str(),
    ];
    root_page::Row {
        stem: target.stem.clone(),
        title: target.title.clone(),
        dir,
        summary: root_page::summary(doc.crate_docs.as_ref(), &doc.refs, &names),
    }
}

/// Adds the files of the HTML folder that no one target writes: the one
/// HTML page, the overview's data file, the search index, and the script.
/// The overview takes its name from the base folder of the run.
pub(crate) fn push_folder(outcome: &mut Outcome, dir: &Path, folder: &Folder<'_>, stamp: &str) {
    let uses = index::uses_html(folder.entries);
    let name = folder.base.file_name().map_or_else(
        || "Outlines".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let overview = root_page::Overview {
        name: &name,
        rows: folder.rows,
        roles: folder.roles,
        declarations: (folder.run.iter())
            .flat_map(|page| page.ids.iter().chain(page.values.values()))
            .filter(|id| !id.is_empty())
            .count(),
        uses: &uses,
    };
    let content = root_page::fragment(&overview, stamp);
    let files = [
        (dir.join("index.html"), browse::shell(stamp)),
        (
            dir.join(browse::data_file(browse::INDEX)),
            browse::data(browse::INDEX, &name, &content),
        ),
        (
            dir.join(browse::SEARCH_FILE),
            browse::search(folder.run, folder.api),
        ),
        (dir.join(browse::SCRIPT_FILE), browse::SCRIPT.to_owned()),
    ];
    outcome.files.extend(files);
}
