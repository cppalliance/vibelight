//! Runs the `vibelight` binary so that `cargo vibelight` works with no
//! `.cargo/config.toml` alias: cargo finds this file on `PATH` by its
//! `cargo-vibelight` name, and it runs its sibling, which `cargo install`
//! puts in the same folder.

use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// The subcommand name cargo passes as the first argument.
const SUBCOMMAND: &str = "vibelight";

/// The tool this binary runs, which cargo installed beside it.
const TOOL: &str = if cfg!(windows) {
    "vibelight.exe"
} else {
    "vibelight"
};

fn main() -> ExitCode {
    let mut args: Vec<_> = env::args_os().skip(1).collect();
    // Cargo passes the subcommand's own name as the first argument, so
    // `cargo vibelight src` reaches this binary as `cargo-vibelight vibelight
    // src`. A person running this binary directly passes no such name.
    if args.first().is_some_and(|arg| arg == SUBCOMMAND) {
        args.remove(0);
    }
    let Some(exe) = sibling() else {
        return ExitCode::from(1);
    };
    match Command::new(&exe).args(&args).status() {
        Ok(status) => match status.code() {
            Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
            None => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("error: cannot run {}: {error}", exe.display());
            ExitCode::from(1)
        }
    }
}

/// The `vibelight` binary in this binary's folder, or `None` after saying
/// where it looked.
fn sibling() -> Option<PathBuf> {
    let shim = env::current_exe().ok();
    let found = shim
        .as_ref()
        .map(|shim| shim.with_file_name(TOOL))
        .filter(|tool| tool.is_file());
    if found.is_none() {
        let name = shim.map_or_else(
            || TOOL.to_owned(),
            |shim| shim.with_file_name(TOOL).display().to_string(),
        );
        eprintln!("error: cannot find {name}, which `cargo vibelight` runs");
        eprintln!("help: install both with `cargo install vibelight`");
    }
    found
}
