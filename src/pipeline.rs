//! Outlines the crates of a run in two phases: every crate is read and
//! resolved against one fixed crate map, then each target's markdown is
//! rendered from the run's analysis view and each crate's HTML document is
//! built and rendered, all in memory, ready to write.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::document::Document;
use crate::inputs::{self, Chosen};
use crate::layout::Block;
use crate::model::{Crate, Loc, Warning};
use crate::render_html::Anchors;
use crate::render_slop::{AnalysisTarget, RenderInput};
use crate::root::Target;
use crate::settings::{Notation, Output, Plan};
use crate::source_links::LinkBase;
use crate::spellings::Entry;
use crate::{
    analysis, attach, collect, collect_check, document, index, layout_decl, leads, modtree, order,
    render_html, render_slop, render_slop_lists, resolve, resolve_run, spellings, surface,
    test_support,
};

/// What a run would write and what it reports.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    /// Each file the run writes, with its text: the outlines in output-name
    /// order, then the index.
    pub(crate) files: Vec<(PathBuf, String)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) unresolved: usize,
    pub(crate) ambiguous: usize,
    /// How many items the markdown's spelling records name, over every
    /// target: the outside items written more than one way.
    pub(crate) spelled: usize,
}

/// The HTML document for a target, with the base its source links are
/// measured from.
struct Built {
    doc: Document,
    base: LinkBase,
}

/// Outlines every target of the chosen crates in memory: a markdown file
/// for analysis and an HTML file for browsing, each in its own folder. No
/// output depends on the order the crates are processed in: targets are
/// handled in output stem order, and each crate resolves against the same
/// map. Every outline carries `stamp` on its first line.
///
/// # Errors
/// Returns an error when two targets would write the same file or a crate
/// cannot be read.
pub(crate) fn outline(chosen: &Chosen, plan: &Plan, stamp: &str) -> Result<Outcome> {
    let markdown = plan.markdown_dir();
    let html = plan.html();
    let targets = targets(chosen)?;
    let mut outcome = Outcome::default();
    let mut krates = Vec::new();
    for &(p, target) in &targets {
        let dir = &chosen.packages[p].input.dir;
        let tree = modtree::walk(dir, &target.root)?;
        let mut krate = collect::collect(tree, dir, &target.crate_name, &target.title);
        test_support::mark(&mut krate);
        let tally = resolve::resolve(&mut krate);
        outcome.unresolved += tally.unresolved;
        outcome.ambiguous += tally.ambiguous;
        krates.push(krate);
    }
    let deps = crate_map(chosen, &targets);
    resolve_run::link(&mut krates, &deps);
    let (texts, spelled) = analyze(chosen, &targets, &krates, deps, markdown, stamp)?;
    outcome.spelled = spelled;
    let mut docs: Vec<Built> = Vec::new();
    for (krate, &(p, target)) in krates.iter_mut().zip(&targets) {
        let others = attach::attach(krate);
        let krate: &Crate = krate;
        let spellings = spellings::find(krate);
        let surface = if target.lib {
            surface::walk(krate)
        } else {
            Vec::new()
        };
        let parts = Parts {
            krate,
            others: &others,
            spellings: &spellings,
            surface: &surface,
            crate_dir: &chosen.packages[p].input.dir,
        };
        docs.push(parts.build(&html)?);
    }
    let run: Vec<Anchors> = targets
        .iter()
        .zip(&docs)
        .map(|(&(_, target), page)| Anchors {
            file: format!("{}.html", target.stem),
            ids: page.doc.decl_ids.clone(),
            values: page.doc.value_ids.clone(),
        })
        .collect();
    let mut entries = Vec::new();
    for (((&(p, target), page), krate), text) in targets.iter().zip(&docs).zip(&krates).zip(texts) {
        entries.push(index::Entry {
            stem: target.stem.clone(),
            dir: shown(&chosen.base, &chosen.packages[p].dir),
            followed: chosen.packages[p].followed,
            bytes: text.len(),
        });
        let path = markdown.join(format!("{}.md", target.stem));
        outcome.files.push((path, text));
        let member_link: &dyn Fn(Loc) -> String =
            &|loc| page.base.target(&krate.files[loc.file].rel, Some(loc.line));
        let text = render_html::render(&page.doc, Some(member_link), &run, stamp);
        let path = html.dir.join(format!("{}.html", target.stem));
        outcome.files.push((path, text));
        let prefix = (chosen.packages.len() > 1)
            .then(|| inputs::relative(&chosen.base, &chosen.packages[p].dir))
            .flatten()
            .filter(|prefix| !prefix.is_empty());
        outcome
            .warnings
            .extend(krate.warnings.iter().cloned().map(|mut warning| {
                if let Some(prefix) = &prefix {
                    warning.file = format!("{prefix}/{}", warning.file);
                }
                warning
            }));
    }
    outcome
        .files
        .push((markdown.join(index::FILE), index::render(&entries)));
    outcome.files.push((
        html.dir.join("index.html"),
        index::render_html(&entries, stamp),
    ));
    Ok(outcome)
}

/// Builds the run's analysis view from its resolved, unattached crates and
/// its packages with `deps`, the crate map that resolution used, finds its
/// leads, and renders each target's markdown from them for the `markdown`
/// folder, in target order. Returns the markdown texts and how many items
/// their spelling records name. Debug builds check the facts, the view,
/// and the leads first.
///
/// # Errors
/// Returns an internal error when a fact, the view, or a lead breaks an
/// invariant, or a record cannot be written; and an error when a crate's
/// folder or the markdown folder cannot be resolved.
fn analyze(
    chosen: &Chosen,
    targets: &[(usize, &Target)],
    krates: &[Crate],
    deps: Vec<BTreeMap<String, usize>>,
    markdown: &Path,
    stamp: &str,
) -> Result<(Vec<String>, usize)> {
    if cfg!(debug_assertions) {
        for krate in krates {
            collect_check::validate_body_facts(krate)?;
        }
        for package in &chosen.packages {
            collect_check::validate_package_facts(package)?;
        }
    }
    let (infos, packages) = analysis::inputs(chosen, targets, krates, deps);
    let run = analysis::AnalysisRun::build(krates, &infos, &packages);
    if cfg!(debug_assertions) {
        run.validate().map_err(|why| {
            anyhow!("internal error: the analysis of the run breaks an invariant: {why}")
        })?;
    }
    let leads = leads::find(&run);
    if cfg!(debug_assertions) {
        leads::validate(&leads, &run).map_err(|why| {
            anyhow!("internal error: the leads of the run break an invariant: {why}")
        })?;
    }
    let mut texts = Vec::new();
    for (t, &(p, target)) in targets.iter().enumerate() {
        let base = LinkBase::new(&chosen.packages[p].input.dir, markdown)
            .context("cannot resolve the crate and output directories")?;
        let input = RenderInput {
            analysis_target: AnalysisTarget {
                run: &run,
                target: t,
            },
            leads: &leads,
            stamp,
            target_kind: target.kind,
            shown_dir: shown(&chosen.base, &chosen.packages[p].dir),
            root: base.root(),
        };
        texts.push(
            render_slop::render(&input)
                .context("internal error: a markdown record cannot be written")?,
        );
    }
    let spelled = (0..targets.len())
        .map(|t| render_slop_lists::spelled(&run, t).len())
        .sum();
    Ok((texts, spelled))
}

/// What one target's HTML document is built from.
struct Parts<'a> {
    krate: &'a Crate,
    /// The impls left for the Other impls section.
    others: &'a [usize],
    spellings: &'a [Entry],
    surface: &'a [crate::surface::Entry],
    crate_dir: &'a Path,
}

impl Parts<'_> {
    /// Builds the document for one format: its blocks and their order come
    /// from its options, and its source links are measured from its folder.
    fn build(&self, output: &Output) -> Result<Built> {
        let decls = 0..self.krate.decls.len();
        let blocks: Vec<Block> = decls
            .clone()
            .map(|d| layout_decl::decl_block(self.krate, d, output.options.members))
            .collect();
        let printed = match output.options.notation {
            Notation::Cpp => &blocks,
        };
        let order = order::order(self.krate, &blocks);
        let base = LinkBase::new(self.crate_dir, &output.dir)
            .context("cannot resolve the crate and output directories")?;
        let inputs = document::Inputs {
            krate: self.krate,
            others: self.others,
            order: &order,
            blocks: &blocks,
            printed,
            spellings: self.spellings,
            surface: self.surface,
            options: output.options,
        };
        let doc = document::build(&inputs, Some(&base));
        Ok(Built { doc, base })
    }
}

/// Shows a crate folder below the base of the run with forward slashes,
/// the base itself as `.`, and a folder outside it in full.
fn shown(base: &Path, dir: &Path) -> String {
    match inputs::relative(base, dir) {
        Some(rel) if rel.is_empty() => ".".to_owned(),
        Some(rel) => rel,
        None => dir.display().to_string(),
    }
}

/// Lists every target of the run in output stem order, each with the index
/// of its crate, and fails when two targets would write the same file.
pub(crate) fn targets(chosen: &Chosen) -> Result<Vec<(usize, &Target)>> {
    let mut targets: Vec<(usize, &Target)> = chosen
        .packages
        .iter()
        .enumerate()
        .flat_map(|(p, package)| package.input.targets.iter().map(move |t| (p, t)))
        .collect();
    targets.sort_by(|a, b| a.1.stem.cmp(&b.1.stem).then(a.0.cmp(&b.0)));
    for pair in targets.windows(2) {
        let ((first, target), (second, other)) = (pair[0], pair[1]);
        if target.stem == other.stem {
            let name = |p: usize| {
                let dir = &chosen.packages[p].dir;
                inputs::relative(&chosen.base, dir)
                    .filter(|rel| !rel.is_empty())
                    .unwrap_or_else(|| dir.display().to_string())
            };
            bail!(
                "{} and {} would both write {stem}.md and {stem}.html",
                name(first),
                name(second),
                stem = target.stem
            );
        }
    }
    Ok(targets)
}

/// Maps, for each target, every dependency name it may write to the index
/// of that dependency's library target in the run.
pub(crate) fn crate_map(
    chosen: &Chosen,
    targets: &[(usize, &Target)],
) -> Vec<BTreeMap<String, usize>> {
    let libs: BTreeMap<&Path, usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, (_, target))| target.lib)
        .map(|(i, &(p, _))| (chosen.packages[p].dir.as_path(), i))
        .collect();
    targets
        .iter()
        .map(|&(p, _)| {
            let package = &chosen.packages[p];
            package
                .deps
                .iter()
                .filter(|dep| dep.dir != package.dir)
                .filter_map(|dep| {
                    let &lib = libs.get(dep.dir.as_path())?;
                    let name = if dep.renamed {
                        dep.key.replace('-', "_")
                    } else {
                        targets[lib].1.crate_name.clone()
                    };
                    Some((name, lib))
                })
                .collect()
        })
        .collect()
}
