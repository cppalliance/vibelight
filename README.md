# vibelight

[![CI](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/vibelight)](https://crates.io/crates/vibelight) [![Latest release](https://img.shields.io/github/v/release/cppalliance/vibelight)](https://github.com/cppalliance/vibelight/releases/latest) [![License](https://img.shields.io/github/license/cppalliance/vibelight)](LICENSE) [![Minimum Rust version](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

vibelight reads a Rust crate and writes an outline of it in the style of a C++ header: every type, trait, function, and module with its comments, and no function bodies. You see the shape of a whole crate on one page that you can skim, search, or hand to a reviewer.

## What it does

- **One outline per target.** The library, each binary, or a single `.rs` file gets two files: `<target>.md` and `<target>.html`. Both come from the same model, so their text always agrees.
- **A self-contained, linked HTML file.** It has inline CSS, no JavaScript, and makes no network requests. Every type name that refers to a declaration in the outline links to it, the contents list sits in a sticky sidebar, and the colors follow your system's light or dark setting.
- **Top-down order.** Declarations are ordered by how they name each other. The types nothing else names come first, and every type comes after the types that use it. A by-module index comes last.
- **Comments kept.** Doc comments and ordinary comments stay with the item they belong to.
- **Every macro marked.** Macro definitions, invocations, derives from other crates, and attribute macros each get a bold **MACRO** label, and a kill list at the top gathers them all in one place, with a link to where each one sits.
- **Links back to the source.** Each declaration, impl block, module, and macro links back to where it is written, such as `src/prompt.rs:136`. The links are relative to the output file and end in `#L136`, which GitHub, VS Code, and Cursor open at that line. They are on by default.

vibelight parses the source with [syn](https://crates.io/crates/syn) and never builds the crate. It needs no Cargo build, no nightly toolchain, and no network, so it works on a fresh clone and on code that does not compile right now.

## Install

Every versioned release on [GitHub Releases](https://github.com/cppalliance/vibelight/releases) has a prebuilt archive for Linux (x86_64 and ARM64), macOS (Intel and Apple Silicon), and Windows (x86_64), each with a `.sha256` checksum. It also has install scripts that download the right archive for your machine and put `vibelight` in Cargo's bin folder.

On macOS or Linux:

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cppalliance/vibelight/releases/latest/download/vibelight-installer.sh | sh
```

On Windows, in PowerShell:

```
powershell -ExecutionPolicy Bypass -c "irm https://github.com/cppalliance/vibelight/releases/latest/download/vibelight-installer.ps1 | iex"
```

To try the latest changes, use the [nightly](https://github.com/cppalliance/vibelight/releases/tag/nightly) prerelease. It is rebuilt once a day when `master` has new commits, for the same platforms, and each build replaces the one before. `vibelight --version` names the build's commit and date.

Once vibelight is published on crates.io, `cargo install vibelight` will build and install it. Until then, or to build your own copy, run `cargo install --path .` in a checkout of this repository.

## Run

```
vibelight <crate-dir-or-file.rs> [-o <out-dir>] [--no-source-links]
```

- `<crate-dir-or-file.rs>` is a crate directory, with or without a `Cargo.toml`, or a single `.rs` file.
- `-o` (or `--out`) is the directory to write into. It defaults to the current directory and is created if it is missing.
- `--no-source-links` leaves out the links back to files and lines. The output then depends only on the declarations themselves, so moving code between files, reordering items, or reformatting leaves the files byte for byte the same.

Existing files with the same names are overwritten. vibelight prints the path of each file it writes on stdout, sorted. For example, `vibelight ../my-crate -o outline` on a library crate named `my-crate` writes `outline/my_crate.md` and `outline/my_crate.html`.

## Targets

In a crate directory, vibelight reads `Cargo.toml` for the package name and any `[lib]` and `[[bin]]` entries, and finds `src/lib.rs`, `src/main.rs`, and `src/bin/*.rs` the way Cargo does. Each target gets its own pair of files:

- The library is named after the library target, with hyphens turned into underscores, so package `my-crate` gives `my_crate.md`.
- Each binary keeps its name as written. A binary with the same name as the library gets a `.bin` stem, so `foo.bin.md` sits beside `foo.md`.
- Without a manifest, the library and `src/main.rs` take the directory's name, and each `src/bin/*.rs` takes its file stem.
- A single `.rs` file is named after its file stem.

Tests, benches, examples, and `build.rs` are not outlined. A workspace root is not a crate, so point vibelight at a member crate's directory instead.

## What the output looks like

Each file opens with the target's name and its crate docs, then has these parts:

- **Contents:** links to every section and declaration.
- **Kill list:** a count of macros by kind, then one entry per macro, linking to the declaration or module that holds it.
- **Level 1, Level 2, and so on:** one heading per type. Level 1 holds the types nothing else names, and every other type sits one level below the deepest type that names it. Types that name each other in a cycle are placed together.
- **Unconnected types:** types that name nothing and that nothing names.
- **Other impls:** impls with no declared type to attach to, such as blanket impls or impls for `&T` or `Box<T>`, as `implements: Trait for Type;` lines.
- **Modules:** one heading per module, with its docs, links to its declarations, and its re-exports, macros, constants, statics, free functions, and extern blocks.
- **Spellings:** each item from outside the crate that the source writes in more than one way, with every spelling (see [Spellings](#spellings)).

A level, Unconnected types, Other impls, or Spellings shows up only when it has something in it. The markdown and HTML files hold the same headings, marks, and blocks. Markdown cannot put a link inside a code block, so only the HTML file links type names.

Every type prints as one C++-style block, with the members of all its inherent impls merged in, wherever they are written. Here is a type whose fields and methods are all private:

```
/// Holds the text the lexer is still reading.
class Cursor
{
    pos: usize,

    text: String,

    static into_text(self) -> String;

    static new(text: String) -> Cursor;

    bump() mut;

    peek() -> Option<char>;
}
```

Since nothing in it is public, it prints as a `class` with no label. A type with a public member prints as a `struct`, with a `private:` label before its private part.

The `fn` keyword and the bodies are gone. A method that takes `&self` gets no marker, a method that takes `&mut self` gets `mut` after its parameter list, and a function that takes no receiver or takes `self` by value is `static`. A bare `&` is read-only, as in Rust, and `mut` marks what can change, so `&str` prints as `str&` and `&mut T` as `T mut&`. Members are grouped (associated types, constants, fields, static functions, then member functions) and sorted by name inside each group.

Traits a type implements or derives print the C++ way. Only the C++ special member functions become members: `Default` is the default constructor, `From<T>` a converting constructor, and `Drop` the destructor, packed together after the fields, with `= default` when derived. Every other standard trait, `Clone` and the comparison traits included, goes in the base list by its short name, as in `struct Seed : Clone, Debug, Display`, and every other trait follows on its own line by its full path. A name counts as standard only when it really resolves to `std`, `core`, or `alloc`, so another crate's `Display`, the crate's own `Display`, or a `Display` that only comes through a glob import gets its own line. A derive from another crate keeps its bold **MACRO** line above the block.

## Spellings

The Spellings section lists each item from outside the crate that the declarations write in more than one way, such as `fmt::Debug` in one place and `std::fmt::Debug` in another, so you can pick one and change the rest. It covers `std`, `core`, `alloc`, and other crates. A bare prelude name such as `Option` counts as `std::option::Option`, a `core::` or `alloc::` path counts as the `std` path that re-exports it, and a `use ... as` rename is one more spelling. Each entry gives every spelling with how often it is written and, when source links are on, a link to each line:

```
- `std::fmt::Debug` is written 2 ways; pick one and search for the rest:
  - `Debug`, 2 times (source: [src/lib.rs:4](src/lib.rs#L4), [src/lib.rs:9](src/lib.rs#L9))
  - `fmt::Debug`, once (source: [src/lib.rs:12](src/lib.rs#L12))
```

Names of the crate's own items are left out, because `crate::model::X`, `X`, and `super::X` are all normal in different modules. So are names in comments, re-exports, `use` lines, and function bodies. Glob imports such as `use std::fmt::*` are not followed, so a name that only comes through one is not counted.

## Warnings and exit codes

When vibelight meets something it cannot fully handle, it keeps going and prints a warning on stderr, as `warning: <file>:<line>: <message>`, then a count at the end. Warnings cover files that do not parse, module files that cannot be found, constructs it does not support (also marked UNSUPPORTED in the outline), and type names it cannot resolve or that are ambiguous, which stay plain text instead of links. When the Spellings section has entries, one more line says how many items are written more than one way.

- `0`: the files were written, warnings included.
- `1`: the input cannot be used, for example the path is not a crate, the manifest is workspace-only, or a root file cannot be read. The reason is printed on stderr.
- `2`: a usage error, such as a missing argument or an unknown flag, reported by clap.

## Requirements and license

- Minimum Rust version: 1.85
- License: BSL-1.0 (Boost Software License 1.0), in `LICENSE`
