//! Runs the `cargo-vibelight` binary, which makes `cargo vibelight` work: it
//! must pass a run through to its sibling `vibelight` and keep its exit code.

use std::process::Command;

/// The `vibelight` binary cargo built beside the shim, which the shim runs.
fn tool() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_BIN_EXE_cargo-vibelight")).with_file_name(if cfg!(windows) {
        "vibelight.exe"
    } else {
        "vibelight"
    })
}

#[test]
fn the_shim_runs_the_sibling_it_sits_beside() {
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-vibelight"))
        .arg("--version")
        .output()
        .expect("the shim runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let version = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        version.trim(),
        format!("vibelight {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_shim_drops_the_subcommand_name_cargo_passes() {
    // `cargo vibelight --version` reaches the shim as `cargo-vibelight
    // vibelight --version`, so the name is dropped and the rest is passed on.
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-vibelight"))
        .args(["vibelight", "--version"])
        .output()
        .expect("the shim runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let version = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        version.trim(),
        format!("vibelight {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_shim_keeps_the_exit_code_of_the_tool() {
    // An unknown flag is a usage error, which exits with 2.
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-vibelight"))
        .args(["vibelight", "--no-such-flag"])
        .output()
        .expect("the shim runs");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn the_shim_passes_a_run_through_to_the_tool() {
    let tmp = tempfile::tempdir().expect("a temporary folder can be made");
    let crate_dir = tmp.path().join("demo");
    std::fs::create_dir_all(crate_dir.join("src")).expect("a fixture folder can be made");
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("a manifest is writable");
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        "//! Demo.\npub fn f() -> u8 { 1 }\n",
    )
    .expect("a source file is writable");
    let out_dir = tmp.path().join("out");
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-vibelight"))
        .args(["vibelight"])
        .arg(&crate_dir)
        .arg("-o")
        .arg(&out_dir)
        .env("SOURCE_DATE_EPOCH", "1767225600")
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the shim runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out_dir.join("demo.md").is_file(),
        "the run wrote its outline"
    );
    assert!(
        (crate::support::read_pages(&out_dir.join("html")).iter())
            .any(|(name, _)| name == "demo.html"),
        "the run wrote its page"
    );
}

#[test]
fn the_tool_the_shim_runs_was_built_beside_it() {
    assert!(
        tool().is_file(),
        "cargo builds the shim and the tool it runs into one folder, so \
         `cargo install vibelight` puts both in the bin folder"
    );
}
