//! Command-line entry point: writes a markdown and an HTML outline per target.

mod attach;
mod attrs;
mod collect;
mod collect_field;
mod collect_fn;
mod collect_impl;
mod collect_type;
mod collect_use;
mod comments;
mod document;
mod document_kill;
mod flatten;
mod layout;
mod layout_decl;
mod layout_items;
mod layout_members;
mod macros;
mod model;
mod modtree;
mod order;
mod render_html;
mod render_md;
mod resolve;
mod root;
mod source;
mod source_links;
mod text;
mod typerefs;
mod typerefs_edits;
mod wrap;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;

use crate::model::{Loc, Warning};
use crate::source_links::LinkBase;

/// The `--version` text; the nightly workflow sets `VIBELIGHT_NIGHTLY` at
/// build time to name its build.
const VERSION: &str = match option_env!("VIBELIGHT_NIGHTLY") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// Writes a C++-header-style outline of a Rust crate's declarations and
/// comments: one markdown file and one linked HTML file per target.
#[derive(Debug, Parser)]
#[command(version = VERSION, about)]
struct Args {
    /// A crate directory, or a single `.rs` file.
    crate_path: PathBuf,
    /// The directory to write the outline files into; created if missing.
    #[arg(short, long, default_value = ".")]
    out: PathBuf,
    /// Leave out the links back to source files and lines.
    #[arg(long)]
    no_source_links: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run(args: &Args) -> Result<()> {
    let input = root::find(&args.crate_path)?;
    fs::create_dir_all(&args.out)
        .with_context(|| format!("cannot create output directory {}", args.out.display()))?;
    let links = if args.no_source_links {
        None
    } else {
        Some(
            LinkBase::new(&input.dir, &args.out)
                .context("cannot resolve the crate and output directories")?,
        )
    };
    let mut written = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();
    let mut unresolved = 0;
    let mut ambiguous = 0;
    for target in &input.targets {
        let tree = modtree::walk(&input.dir, &target.root)?;
        let mut krate = collect::collect(tree, &input.dir, &target.crate_name, &target.title);
        let tally = resolve::resolve(&mut krate);
        unresolved += tally.unresolved;
        ambiguous += tally.ambiguous;
        let others = attach::attach(&mut krate);
        let blocks: Vec<layout::Block> = (0..krate.decls.len())
            .map(|d| layout_decl::decl_block(&krate, d))
            .collect();
        let order = order::order(&krate, &blocks);
        let doc = document::build(&krate, &others, &order, &blocks, links.as_ref());
        let markdown = render_md::render(&doc);
        let member_link = links
            .as_ref()
            .map(|base| move |loc: Loc| base.target(&krate.files[loc.file].rel, Some(loc.line)));
        let html = render_html::render(
            &doc,
            member_link.as_ref().map(|f| f as &dyn Fn(Loc) -> String),
        );
        for (extension, text) in [("md", markdown), ("html", html)] {
            let path = args.out.join(format!("{}.{extension}", target.stem));
            fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;
            written.push(path.display().to_string());
        }
        warnings.extend(krate.warnings);
    }
    written.sort();
    for path in &written {
        println!("{path}");
    }
    warnings.sort();
    warnings.dedup();
    for warning in &warnings {
        eprintln!(
            "warning: {}:{}: {}",
            warning.file, warning.line, warning.message
        );
    }
    if unresolved > 0 || ambiguous > 0 {
        eprintln!(
            "warning: {unresolved} type names unresolved, {ambiguous} ambiguous; they stay plain"
        );
    }
    let count = warnings.len() + usize::from(unresolved > 0 || ambiguous > 0);
    if count > 0 {
        eprintln!("{count} warning{}", if count == 1 { "" } else { "s" });
    }
    Ok(())
}
