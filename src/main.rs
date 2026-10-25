//! Command-line entry point: writes a markdown and an HTML outline per target.

mod attach;
mod attrs;
mod collect;
mod collect_bases;
mod collect_field;
mod collect_fn;
mod collect_impl;
mod collect_type;
mod collect_use;
mod comments;
mod document;
mod document_kill;
mod document_spellings;
mod flatten;
mod glob;
mod inputs;
mod layout;
mod layout_decl;
mod layout_head;
mod layout_items;
mod layout_members;
mod layout_special;
mod macros;
mod manifest;
mod model;
mod modtree;
mod order;
mod pipeline;
mod render_html;
mod render_md;
mod resolve;
mod resolve_run;
mod root;
mod route;
mod source;
mod source_links;
mod spellings;
mod text;
mod typerefs;
mod typerefs_edits;
mod wrap;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;

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
    /// Crate directories or their `Cargo.toml` files, or a single `.rs`
    /// file. With the workspace switch, the one directory to start looking
    /// for the workspace from.
    #[arg(required_unless_present = "workspace")]
    crate_paths: Vec<PathBuf>,
    /// The directory to write the outline files into; created if missing.
    #[arg(short, long, default_value = ".")]
    out: PathBuf,
    /// Leave out the links back to source files and lines.
    #[arg(long)]
    no_source_links: bool,
    /// Outline every member of the workspace that holds the given
    /// directory, or the current one.
    #[arg(long)]
    workspace: bool,
    /// Also outline each path dependency whose directory, relative to the
    /// workspace root or else the first crate, matches this glob; repeat it
    /// for more globs.
    #[arg(long, value_name = "GLOB")]
    follow: Vec<String>,
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
    let chosen = inputs::choose(&args.crate_paths, args.workspace, &args.follow)?;
    fs::create_dir_all(&args.out)
        .with_context(|| format!("cannot create output directory {}", args.out.display()))?;
    let pipeline::Outcome {
        mut written,
        mut warnings,
        unresolved,
        ambiguous,
        spelled,
    } = pipeline::outline(&chosen, &args.out, !args.no_source_links)?;
    written.sort();
    for path in &written {
        println!("{path}");
    }
    for note in &chosen.notes {
        eprintln!("note: {note}");
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
    if spelled > 0 {
        eprintln!(
            "warning: {spelled} item{} written more than one way; see Spellings",
            if spelled == 1 { "" } else { "s" }
        );
    }
    let count =
        warnings.len() + usize::from(unresolved > 0 || ambiguous > 0) + usize::from(spelled > 0);
    if count > 0 {
        eprintln!("{count} warning{}", if count == 1 { "" } else { "s" });
    }
    Ok(())
}
