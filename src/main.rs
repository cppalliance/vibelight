//! Command-line entry point: writes a markdown and an HTML outline per target.

mod analysis;
mod analysis_check;
mod analysis_items;
mod analysis_paths;
#[cfg(test)]
mod analysis_tests;
mod analysis_uses;
mod attach;
mod attrs;
mod census;
mod census_check;
mod census_docs;
mod census_lex;
mod census_paths;
mod census_sources;
mod collect;
mod collect_attrs;
mod collect_bases;
mod collect_body;
mod collect_body_code;
mod collect_body_comments;
mod collect_body_facts;
mod collect_body_read;
mod collect_body_tokens;
mod collect_check;
mod collect_check_nodes;
mod collect_field;
mod collect_fn;
mod collect_generics;
mod collect_impl;
mod collect_trait;
mod collect_type;
mod collect_use;
mod collect_use_group;
mod comments;
mod config;
mod document;
mod document_external;
mod document_facts;
mod document_kill;
mod document_modules;
mod document_sections;
mod document_shapes;
mod document_spellings;
mod document_surface;
mod document_values;
mod exclude;
mod flatten;
mod folders;
mod glob;
mod groups;
mod index;
mod inputs;
mod layout;
mod layout_decl;
mod layout_head;
mod layout_items;
mod layout_marker;
mod layout_members;
mod layout_rust;
mod layout_rust_decl;
mod layout_rust_items;
mod layout_special;
mod leads;
mod leads_body;
mod leads_body_clone;
mod leads_body_literals;
#[cfg(test)]
mod leads_body_tests;
mod leads_census;
#[cfg(test)]
mod leads_census_crates_tests;
mod leads_census_deps;
mod leads_census_paths;
mod leads_census_place;
mod leads_census_reach;
mod leads_census_scope;
#[cfg(test)]
mod leads_census_tests;
mod leads_census_variants;
mod leads_check;
mod leads_comments;
mod leads_comments_docs;
#[cfg(test)]
mod leads_comments_tests;
mod leads_comments_words;
mod leads_containers;
mod leads_containers_ops;
mod leads_containers_owners;
#[cfg(test)]
mod leads_containers_tests;
mod leads_docs;
mod leads_docs_claims;
mod leads_docs_lists;
mod leads_docs_phrases;
mod leads_docs_refs;
mod leads_docs_repeat;
mod leads_docs_scan;
mod leads_docs_std;
#[cfg(test)]
mod leads_docs_tests;
mod leads_docs_text;
mod leads_docs_words;
#[cfg(test)]
mod leads_impls_tests;
mod leads_items;
mod leads_names;
mod leads_names_modules;
mod leads_names_roles;
#[cfg(test)]
mod leads_names_tests;
mod leads_shapes;
mod leads_shapes_impls;
mod leads_shapes_params;
#[cfg(test)]
mod leads_shapes_tests;
mod leads_shapes_traits;
mod leads_shapes_twins;
#[cfg(test)]
mod leads_tests;
mod leads_types;
mod leads_types_shape;
mod leads_words;
mod legend;
mod macros;
mod manifest;
mod manifest_facts;
mod manifest_roots;
mod model;
mod model_body;
mod model_impl;
mod modtree;
mod order;
mod order_graph;
mod pipeline;
mod prelude;
mod render_html;
mod render_md;
mod render_prose;
mod render_spans;
mod render_style;
mod render_values;
mod resolve;
mod resolve_run;
mod resolve_values;
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
mod wrap;

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
    } = pipeline::outline(&chosen, &plan, &stamp)?;
    let written = folders::write(&files, &plan)?;
    let mut paths: Vec<String> = files.iter().map(|(p, _)| p.display().to_string()).collect();
    paths.sort();
    for path in &paths {
        println!("{path}");
    }
    notes.extend(chosen.notes.iter().cloned());
    if let Some(path) = &written.gitignore {
        notes.push(format!("created {}", shown(path, &cwd)));
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
            "warning: {spelled} item{} written more than one way; see Spellings",
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
