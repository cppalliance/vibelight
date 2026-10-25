//! Outlines the crates of a run in two phases: every crate is read and
//! resolved against one fixed crate map, then each crate's documents are
//! built, rendered, and written.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::document::Document;
use crate::inputs::{self, Chosen};
use crate::layout::Block;
use crate::model::{Loc, Warning};
use crate::render_html::Anchors;
use crate::root::Target;
use crate::source_links::LinkBase;
use crate::{
    attach, collect, document, layout_decl, modtree, order, render_html, render_md, resolve,
    resolve_run, spellings,
};

/// What a run wrote and reported.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    /// The paths of the written files.
    pub(crate) written: Vec<String>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) unresolved: usize,
    pub(crate) ambiguous: usize,
    /// How many items are written more than one way, over every target.
    pub(crate) spelled: usize,
}

/// Outlines every target of the chosen crates into `out`, with source
/// links when `links` is true. No output depends on the order the crates
/// are processed in: targets are handled in output stem order, and each
/// crate resolves against the same map.
///
/// # Errors
/// Returns an error when two targets would write the same file, a crate
/// cannot be read, or a file cannot be written.
pub(crate) fn outline(chosen: &Chosen, out: &Path, links: bool) -> Result<Outcome> {
    let targets = targets(chosen)?;
    let mut outcome = Outcome::default();
    let mut krates = Vec::new();
    for &(p, target) in &targets {
        let dir = &chosen.packages[p].input.dir;
        let tree = modtree::walk(dir, &target.root)?;
        let mut krate = collect::collect(tree, dir, &target.crate_name, &target.title);
        let tally = resolve::resolve(&mut krate);
        outcome.unresolved += tally.unresolved;
        outcome.ambiguous += tally.ambiguous;
        krates.push(krate);
    }
    resolve_run::link(&mut krates, &crate_map(chosen, &targets));
    let mut docs: Vec<(Document, Option<LinkBase>)> = Vec::new();
    for (krate, &(p, _)) in krates.iter_mut().zip(&targets) {
        let others = attach::attach(krate);
        let blocks: Vec<Block> = (0..krate.decls.len())
            .map(|d| layout_decl::decl_block(krate, d))
            .collect();
        let order = order::order(krate, &blocks);
        let spellings = spellings::find(krate);
        outcome.spelled += spellings.len();
        let base = if links {
            Some(
                LinkBase::new(&chosen.packages[p].input.dir, out)
                    .context("cannot resolve the crate and output directories")?,
            )
        } else {
            None
        };
        let doc = document::build(krate, &others, &order, &blocks, &spellings, base.as_ref());
        docs.push((doc, base));
    }
    let run: Vec<Anchors> = targets
        .iter()
        .zip(&docs)
        .map(|(&(_, target), (doc, _))| Anchors {
            file: format!("{}.html", target.stem),
            ids: doc.decl_ids.clone(),
        })
        .collect();
    for ((&(p, target), (doc, base)), krate) in targets.iter().zip(&docs).zip(&krates) {
        let markdown = render_md::render(doc);
        let member_link = base
            .as_ref()
            .map(|base| move |loc: Loc| base.target(&krate.files[loc.file].rel, Some(loc.line)));
        let html = render_html::render(
            doc,
            member_link.as_ref().map(|f| f as &dyn Fn(Loc) -> String),
            &run,
        );
        for (extension, text) in [("md", markdown), ("html", html)] {
            let path = out.join(format!("{}.{extension}", target.stem));
            fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;
            outcome.written.push(path.display().to_string());
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
