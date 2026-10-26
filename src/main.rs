//! Command-line entry point: writes a markdown and an HTML outline per target.

mod attach;
mod attrs;
mod check;
mod collect;
mod collect_bases;
mod collect_field;
mod collect_fn;
mod collect_impl;
mod collect_type;
mod collect_use;
mod collect_use_group;
mod comments;
mod config;
mod document;
mod document_kill;
mod document_sections;
mod document_spellings;
mod document_surface;
mod flatten;
mod glob;
mod inputs;
mod layout;
mod layout_decl;
mod layout_head;
mod layout_items;
mod layout_members;
mod layout_special;
mod legend;
mod macros;
mod manifest;
mod model;
mod model_impl;
mod modtree;
mod order;
mod pipeline;
mod prelude;
mod render_html;
mod render_md;
mod resolve;
mod resolve_run;
mod root;
mod route;
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
mod wrap;

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, ValueEnum};

use crate::settings::{Flags, KillList};

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
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is one command-line switch"
)]
struct Args {
    /// Crate directories or their `Cargo.toml` files, or a single `.rs`
    /// file. With the workspace switch, the one directory to start looking
    /// for the workspace from. With none, `vibelight.toml` chooses them.
    crate_paths: Vec<PathBuf>,
    /// The directory to write both formats into; created if missing.
    #[arg(short, long, value_name = "DIR")]
    out: Option<PathBuf>,
    /// The directory for the markdown files, in place of the one for both.
    #[arg(long, value_name = "DIR")]
    md_out: Option<PathBuf>,
    /// The directory for the HTML files, in place of the one for both.
    #[arg(long, value_name = "DIR")]
    html_out: Option<PathBuf>,
    /// Leave out the links back to source files and lines in both formats.
    #[arg(long)]
    no_source_links: bool,
    /// Turn the source links of the markdown files on or off.
    #[arg(long, value_name = "on|off")]
    md_source_links: Option<Switch>,
    /// Turn the source links of the HTML files on or off.
    #[arg(long, value_name = "on|off")]
    html_source_links: Option<Switch>,
    /// How much of the macro kill list to show: every entry, only the
    /// counts, or nothing.
    #[arg(long, value_name = "full|counts|off")]
    kill_list: Option<KillList>,
    /// Outline every member of the workspace that holds the given
    /// directory, or the current one.
    #[arg(long)]
    workspace: bool,
    /// Also outline each path dependency whose directory, relative to the
    /// workspace root or else the first crate, matches this glob; repeat it
    /// for more globs.
    #[arg(long, value_name = "GLOB")]
    follow: Vec<String>,
    /// Write nothing. Compare the markdown files with what a run would
    /// write, and exit with 1 when one differs, is missing, or is stale.
    #[arg(long)]
    check: bool,
    /// Check the HTML files too.
    #[arg(long)]
    check_html: bool,
}

/// A switch value for one format's source links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Switch {
    On,
    Off,
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

fn flags(args: &Args) -> Flags {
    let switch = |s: Option<Switch>| s.map(|s| s == Switch::On);
    Flags {
        paths: args.crate_paths.clone(),
        workspace: args.workspace,
        follow: args.follow.clone(),
        out: args.out.clone(),
        md_out: args.md_out.clone(),
        html_out: args.html_out.clone(),
        no_source_links: args.no_source_links,
        md_source_links: switch(args.md_source_links),
        html_source_links: switch(args.html_source_links),
        kill_list: args.kill_list,
    }
}

fn run(args: &Args) -> Result<ExitCode> {
    let cwd = env::current_dir().context("cannot read the current directory")?;
    let config = config::find(&cwd)?;
    let plan = settings::plan(&flags(args), config.as_ref())?;
    let Some(selection) = &plan.selection else {
        Args::command()
            .error(
                ErrorKind::MissingRequiredArgument,
                "name a crate, pass --workspace, or add a vibelight.toml that names the crates",
            )
            .exit();
    };
    let chosen = inputs::choose(selection, &plan.follow)?;
    let checking = args.check || args.check_html;
    let stamp = if checking {
        stamp::PLACEHOLDER.to_owned()
    } else {
        stamp::now()?
    };
    let pipeline::Outcome {
        files,
        mut warnings,
        unresolved,
        ambiguous,
        spelled,
    } = pipeline::outline(&chosen, &plan, &stamp)?;
    let written: BTreeSet<PathBuf> = files.iter().map(|(path, _)| path.clone()).collect();
    let stale = plan
        .markdown
        .as_ref()
        .map(|output| check::stale(&output.dir, &written))
        .unwrap_or_default();
    let code = if checking {
        check_files(&files, &stale, args.check_html, &cwd)
    } else {
        write_files(&files, &plan)?;
        ExitCode::SUCCESS
    };
    for note in &chosen.notes {
        eprintln!("note: {note}");
    }
    if !checking {
        for path in &stale {
            eprintln!(
                "warning: {} is stale: no crate of this run writes it",
                shown(path, &cwd)
            );
        }
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
            plural(spelled)
        );
    }
    let stale_warnings = if checking { 0 } else { stale.len() };
    let count = warnings.len()
        + stale_warnings
        + usize::from(unresolved > 0 || ambiguous > 0)
        + usize::from(spelled > 0);
    if count > 0 {
        eprintln!("{count} warning{}", plural(count));
    }
    Ok(code)
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// Writes every file whose content changed, creating the folders of the
/// formats it writes, and prints the path of every file of the run, sorted.
/// A file whose text differs only in its stamp is left as it is, so its
/// stamp keeps the time its content last changed.
fn write_files(files: &[(PathBuf, String)], plan: &settings::Plan) -> Result<()> {
    for output in [&plan.markdown, &plan.html].into_iter().flatten() {
        fs::create_dir_all(&output.dir)
            .with_context(|| format!("cannot create output directory {}", output.dir.display()))?;
    }
    for (path, text) in files {
        if check::compare(path, text).is_some() {
            fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))?;
        }
    }
    let mut written: Vec<String> = files.iter().map(|(p, _)| p.display().to_string()).collect();
    written.sort();
    for path in &written {
        println!("{path}");
    }
    Ok(())
}

/// Compares the markdown files, and the HTML files when `html` is true,
/// with the files on disk, prints one line for each problem and a count,
/// and returns the exit code.
fn check_files(files: &[(PathBuf, String)], stale: &[PathBuf], html: bool, cwd: &Path) -> ExitCode {
    let checked: Vec<&(PathBuf, String)> = files
        .iter()
        .filter(|(path, _)| html || path.extension().is_some_and(|e| e == "md"))
        .collect();
    let mut problems: Vec<check::Problem> = checked
        .iter()
        .filter_map(|(path, text)| check::compare(path, text))
        .collect();
    problems.extend(stale.iter().cloned().map(check::Problem::Stale));
    for problem in &problems {
        match problem {
            check::Problem::Differs(path, line) => println!(
                "differs: {} (first difference at line {line})",
                shown(path, cwd)
            ),
            check::Problem::Missing(path) => println!("missing: {}", shown(path, cwd)),
            check::Problem::Stale(path) => println!("stale: {}", shown(path, cwd)),
        }
    }
    if problems.is_empty() {
        let n = checked.len();
        let files = if n == 1 {
            "file matches"
        } else {
            "files match"
        };
        println!("check passed: {n} {files}");
        ExitCode::SUCCESS
    } else {
        println!(
            "check failed: {} problem{}",
            problems.len(),
            plural(problems.len())
        );
        ExitCode::from(1)
    }
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
