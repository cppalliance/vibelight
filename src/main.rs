//! Command-line entry point: writes a markdown and an HTML outline per target.

mod access;
mod access_stamp;
#[cfg(test)]
mod access_tests;
mod analysis;
mod anatomy;
mod attach;
mod attrs;
mod census;
mod collect;
mod comments;
mod config;
mod exclude;
mod flatten;
mod folders;
mod glob;
mod html;
mod index;
mod inputs;
mod macro_rules;
mod macros;
mod manifest;
mod manifest_facts;
mod manifest_roots;
mod md;
mod model;
mod model_body;
mod modtree;
mod module_graph;
#[cfg(test)]
mod module_graph_tests;
mod order_graph;
mod pipeline;
mod prelude;
mod resolve;
mod resolve_run;
mod resolve_values;
mod roles;
mod root;
mod route;
mod scaffold;
mod settings;
mod source;
mod source_links;
mod spellings;
mod stamp;
mod surface;
mod test_support;
mod text;
mod typerefs;
mod typerefs_edits;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

/// The `--version` text; the nightly workflow sets `VIBELIGHT_NIGHTLY` at
/// build time to name its build.
const VERSION: &str = match option_env!("VIBELIGHT_NIGHTLY") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// Writes an outline of each Rust crate's declarations and comments: a
/// markdown file with everything a model analyzing the code can use, and a
/// linked HTML page for browsing it. Run it with no arguments in a
/// repository: the first run writes a `vibelight.toml` that chooses the
/// crates, and every run reads it.
#[derive(Debug, Parser)]
#[command(version = VERSION, about)]
struct Args {
    /// Crate directories or their `Cargo.toml` files, or a single `.rs`
    /// file, in place of the crates `vibelight.toml` chooses.
    crate_paths: Vec<PathBuf>,
    /// The folder to write into, in place of the config's: the markdown
    /// files and the index go in it, and the HTML files in its `html`
    /// folder. It is created if missing.
    #[arg(short, long, value_name = "DIR")]
    out: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode> {
    let cwd = env::current_dir().context("cannot read the current directory")?;
    let stamp = stamp::now()?;
    let mut notes = Vec::new();
    let mut config = config::find(&cwd)?;
    if config.is_none() && args.crate_paths.is_empty() {
        let Some(written) = scaffold::write(&cwd)? else {
            usage("name a crate, or run vibelight inside a Rust repository");
        };
        let path = shown(&written.path, &cwd);
        notes.push(match written.left_out {
            0 => format!("created {path}; edit it to choose the crates"),
            n => format!(
                "created {path}, which leaves out {n} crate{}; edit it to choose the crates",
                plural(n)
            ),
        });
        config = Some(config::read(&written.path)?);
    }
    let plan = settings::plan(&args.crate_paths, args.out.as_deref(), config.as_ref());
    let Some(selection) = &plan.selection else {
        usage("name a crate, or set `workspace` or `crates` in vibelight.toml");
    };
    let chosen = inputs::choose(selection, &plan.follow, &plan.exclude)?;
    let pipeline::Outcome {
        files,
        mut warnings,
        unresolved,
        ambiguous,
        spelled,
        notes: quiet_notes,
    } = pipeline::outline(&chosen, &plan, &stamp)?;
    let written = folders::write(&files, &plan)?;
    let mut paths: Vec<String> = files.iter().map(|(p, _)| p.display().to_string()).collect();
    paths.sort();
    for path in &paths {
        println!("{path}");
    }
    notes.extend(chosen.notes.iter().cloned());
    notes.extend(quiet_notes);
    if let Some(path) = &written.gitignore {
        notes.push(format!("created {}", shown(path, &cwd)));
    }
    if let Some(path) = &written.upgraded {
        notes.push(format!(
            "updated {} to ignore the pages' script files too",
            shown(path, &cwd)
        ));
    }
    for path in &written.removed {
        notes.push(format!(
            "removed {}: no crate of this run writes it",
            shown(path, &cwd)
        ));
    }
    for note in &notes {
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
            "warning: {spelled} item{} written more than one way; see spelling records",
            plural(spelled)
        );
    }
    let count =
        warnings.len() + usize::from(unresolved > 0 || ambiguous > 0) + usize::from(spelled > 0);
    if count > 0 {
        eprintln!("{count} warning{}", plural(count));
    }
    Ok(ExitCode::SUCCESS)
}

/// Stops with clap's usage error, which exits with 2.
fn usage(message: &str) -> ! {
    Args::command()
        .error(ErrorKind::MissingRequiredArgument, message)
        .exit()
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// Shows a path relative to the current directory when it lies inside it.
fn shown(path: &Path, cwd: &Path) -> String {
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    full.strip_prefix(cwd).map_or_else(
        |_| path.display().to_string(),
        |rel| rel.display().to_string(),
    )
}
