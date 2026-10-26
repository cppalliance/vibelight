//! Outlines the crates of a run in two phases: every crate is read and
//! resolved against one fixed crate map, then each crate's documents are
//! built and rendered in memory, ready to write or to check.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::document::Document;
use crate::inputs::{self, Chosen};
use crate::layout::Block;
use crate::model::{Loc, Warning};
use crate::render_html::Anchors;
use crate::root::Target;
use crate::settings::{Output, Plan};
use crate::source_links::LinkBase;
use crate::{
    attach, collect, document, document_external, layout_decl, modtree, order, render_html,
    render_md, resolve, resolve_run, spellings, surface, test_support,
};

/// What a run would write and what it reports.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    /// Each file the run writes, with its text, in output-name order.
    pub(crate) files: Vec<(PathBuf, String)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) unresolved: usize,
    pub(crate) ambiguous: usize,
    /// How many items are written more than one way, over every target.
    pub(crate) spelled: usize,
}

/// One format's document for a target, with the base its source links
/// are measured from.
struct Built {
    doc: Document,
    base: Option<LinkBase>,
}

/// Outlines every target of the chosen crates in memory: the markdown
/// files for `markdown` and the HTML files for `html`, each in its own
/// folder with its own source-link switch. No output depends on the order
/// the crates are processed in: targets are handled in output stem order,
/// and each crate resolves against the same map. Every file carries
/// `stamp` on its first line, and `plan`'s options shape every document.
///
/// # Errors
/// Returns an error when two targets would write the same file or a crate
/// cannot be read.
pub(crate) fn outline(chosen: &Chosen, plan: &Plan, stamp: &str) -> Result<Outcome> {
    let markdown = plan.markdown.as_ref();
    let html = plan.html.as_ref();
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
    resolve_run::link(&mut krates, &crate_map(chosen, &targets));
    let first_docs = document_external::first_docs(&krates);
    let shared = markdown.is_some() && markdown == html;
    let mut docs: Vec<(Option<Built>, Option<Built>)> = Vec::new();
    for (krate, &(p, target)) in krates.iter_mut().zip(&targets) {
        let others = attach::attach(krate);
        let blocks: Vec<Block> = (0..krate.decls.len())
            .map(|d| layout_decl::decl_block(krate, d))
            .collect();
        let order = order::order(krate, &blocks);
        let spellings = spellings::find(krate);
        outcome.spelled += spellings.len();
        let surface = if target.lib {
            surface::walk(krate)
        } else {
            Vec::new()
        };
        let parts = Parts {
            inputs: document::Inputs {
                krate,
                others: &others,
                order: &order,
                blocks: &blocks,
                spellings: &spellings,
                surface: &surface,
                docs: &first_docs,
                options: plan.options,
            },
            crate_dir: &chosen.packages[p].input.dir,
        };
        let md = parts.build(markdown)?;
        let page = if shared { None } else { parts.build(html)? };
        docs.push((md, page));
    }
    let run: Vec<Anchors> = targets
        .iter()
        .zip(&docs)
        .map(|(&(_, target), (md, page))| {
            let built = page.as_ref().or(md.as_ref());
            Anchors {
                file: format!("{}.html", target.stem),
                ids: built
                    .map(|built| built.doc.decl_ids.clone())
                    .unwrap_or_default(),
                values: built
                    .map(|built| built.doc.value_ids.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();
    for ((&(p, target), (md, page)), krate) in targets.iter().zip(&docs).zip(&krates) {
        if let (Some(output), Some(built)) = (markdown, md) {
            let path = output.dir.join(format!("{}.md", target.stem));
            outcome
                .files
                .push((path, render_md::render(&built.doc, stamp)));
        }
        let page = if shared { md.as_ref() } else { page.as_ref() };
        if let (Some(output), Some(built)) = (html, page) {
            let member_link = built.base.as_ref().map(|base| {
                move |loc: Loc| base.target(&krate.files[loc.file].rel, Some(loc.line))
            });
            let text = render_html::render(
                &built.doc,
                member_link.as_ref().map(|f| f as &dyn Fn(Loc) -> String),
                &run,
                stamp,
            );
            let path = output.dir.join(format!("{}.html", target.stem));
            outcome.files.push((path, text));
        }
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
    Ok(outcome)
}

/// What one target's documents are built from.
struct Parts<'a> {
    inputs: document::Inputs<'a>,
    crate_dir: &'a Path,
}

impl Parts<'_> {
    /// Builds the document for one format, with source links measured from
    /// that format's folder when it has them on.
    fn build(&self, output: Option<&Output>) -> Result<Option<Built>> {
        let Some(output) = output else {
            return Ok(None);
        };
        let base = if output.links {
            Some(
                LinkBase::new(self.crate_dir, &output.dir)
                    .context("cannot resolve the crate and output directories")?,
            )
        } else {
            None
        };
        let doc = document::build(&self.inputs, base.as_ref());
        Ok(Some(Built { doc, base }))
    }
}

/// Lists every target of the run in output stem order, each with the index
/// of its crate, and fails when two targets would write the same file.
fn targets(chosen: &Chosen) -> Result<Vec<(usize, &Target)>> {
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
fn crate_map(chosen: &Chosen, targets: &[(usize, &Target)]) -> Vec<BTreeMap<String, usize>> {
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
