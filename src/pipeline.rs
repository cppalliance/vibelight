//! Outlines the crates of a run in two phases: every crate is read and
//! resolved against one fixed crate map, then each crate's documents are
//! built and rendered in memory, ready to write.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::document::Document;
use crate::inputs::{self, Chosen};
use crate::layout::Block;
use crate::model::{Crate, Loc, Warning};
use crate::render_html::Anchors;
use crate::root::Target;
use crate::settings::{Notation, Output, Plan};
use crate::source_links::LinkBase;
use crate::spellings::Entry;
use crate::{
    analysis, attach, collect, collect_check, document, document_external, index, layout_decl,
    layout_rust_decl, modtree, order, render_html, render_md, resolve, resolve_run, spellings,
    surface, test_support,
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
    /// How many items are written more than one way, over every target.
    pub(crate) spelled: usize,
}

/// One format's document for a target, with the base its source links
/// are measured from.
struct Built {
    doc: Document,
    base: LinkBase,
}

/// Outlines every target of the chosen crates in memory: a markdown file
/// for analysis and an HTML file for browsing, each in its own folder with
/// its own options. No output depends on the order the crates are
/// processed in: targets are handled in output stem order, and each crate
/// resolves against the same map. Every outline carries `stamp` on its
/// first line.
///
/// # Errors
/// Returns an error when two targets would write the same file or a crate
/// cannot be read.
pub(crate) fn outline(chosen: &Chosen, plan: &Plan, stamp: &str) -> Result<Outcome> {
    let markdown = plan.markdown();
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
    for krate in &krates {
        collect_check::validate_body_facts(krate)?;
    }
    for package in &chosen.packages {
        collect_check::validate_package_facts(package)?;
    }
    {
        let (infos, packages) = analysis::inputs(chosen, &targets, &krates, deps);
        let run = analysis::AnalysisRun::build(&krates, &infos, &packages);
        run.validate().map_err(|why| {
            anyhow!("internal error: the analysis of the run breaks an invariant: {why}")
        })?;
    }
    let first_docs = document_external::first_docs(&krates);
    let mut docs: Vec<(Built, Built)> = Vec::new();
    for (krate, &(p, target)) in krates.iter_mut().zip(&targets) {
        let others = attach::attach(krate);
        let krate: &Crate = krate;
        let spellings = spellings::find(krate);
        outcome.spelled += spellings.len();
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
            docs: &first_docs,
            crate_dir: &chosen.packages[p].input.dir,
        };
        docs.push((parts.build(&markdown)?, parts.build(&html)?));
    }
    let run: Vec<Anchors> = targets
        .iter()
        .zip(&docs)
        .map(|(&(_, target), (_, page))| Anchors {
            file: format!("{}.html", target.stem),
            ids: page.doc.decl_ids.clone(),
            values: page.doc.value_ids.clone(),
        })
        .collect();
    let mut entries = Vec::new();
    for ((&(p, target), (md, page)), krate) in targets.iter().zip(&docs).zip(&krates) {
        let text = render_md::render(&md.doc, stamp);
        entries.push(index::Entry {
            stem: target.stem.clone(),
            dir: shown(&chosen.base, &chosen.packages[p].dir),
            followed: chosen.packages[p].followed,
            bytes: text.len(),
        });
        let path = markdown.dir.join(format!("{}.md", target.stem));
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
        .push((markdown.dir.join(index::FILE), index::render(&entries)));
    outcome.files.push((
        html.dir.join("index.html"),
        index::render_html(&entries, stamp),
    ));
    Ok(outcome)
}

/// What one target's documents are built from, whatever the format.
struct Parts<'a> {
    krate: &'a Crate,
    /// The impls left for the Other impls section.
    others: &'a [usize],
    spellings: &'a [Entry],
    surface: &'a [crate::surface::Entry],
    docs: &'a [Vec<Option<String>>],
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
        let rust: Vec<Block>;
        let printed = match output.options.notation {
            Notation::Cpp => &blocks,
            Notation::Rust => {
                rust = decls
                    .map(|d| layout_rust_decl::decl_block(self.krate, d))
                    .collect();
                &rust
            }
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
            docs: self.docs,
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
