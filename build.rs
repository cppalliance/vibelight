//! Build script: fails the build when the package breaks a structure rule.

#[path = "build/structure.rs"]
mod structure;

use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let root = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR for build scripts"),
    );
    // A published package watches only files every tarball has, since a missing
    // watched path makes Cargo rerun this script on every build. `cargo package`
    // verifies in the checkout's target directory and shares this script's
    // fingerprint, so watching Cargo.toml.orig, which a checkout never has,
    // makes the checkout's next build run the full check again.
    if structure::is_packaged(&root) {
        println!("cargo::rerun-if-changed=Cargo.toml.orig");
        return;
    }
    for path in ["Cargo.toml", "build", "plans", "src", "tests"] {
        println!("cargo::rerun-if-changed={path}");
    }
    let violations = structure::check(&root).unwrap_or_else(|err| {
        eprintln!("structure check could not read the package: {err}");
        process::exit(1);
    });
    for violation in &violations {
        eprintln!("structure check: {violation}");
    }
    if !violations.is_empty() {
        process::exit(1);
    }
}
