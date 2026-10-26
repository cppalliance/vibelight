# vibelight

[![CI](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/vibelight)](https://crates.io/crates/vibelight) [![Latest release](https://img.shields.io/github/v/release/cppalliance/vibelight)](https://github.com/cppalliance/vibelight/releases/latest) [![License](https://img.shields.io/github/license/cppalliance/vibelight)](LICENSE) [![Minimum Rust version](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

vibelight reads a Rust crate and writes an outline of it in the style of a C++ header: every type, trait, function, and module with its comments, and no function bodies. You see the shape of a whole crate on one page that you can skim, search, or hand to a reviewer.

## What it does

- **One outline per target.** The library, each binary, or a single `.rs` file gets two files: `<target>.md` and `<target>.html`. Both come from the same model, so their text always agrees.
- **A self-contained, linked HTML file.** It has inline CSS, no JavaScript, and makes no network requests. Every type name that refers to a declaration in the outline links to it, and so does a re-exported function, constant, or static. The contents list sits in a sticky sidebar, and the colors follow your system's light or dark setting.
- **Top-down order.** Declarations are ordered by how they name each other. The types nothing else names come first, and every type comes after the types that use it. A by-module index comes last.
- **Comments kept.** Doc comments and ordinary comments stay with the item they belong to.
- **Every macro marked.** Macro definitions, invocations, derives from other crates, and attribute macros each get a bold **MACRO** label, and a kill list at the top gathers them all in one place, with a link to where each one sits.
- **The public surface first.** A library's file lists every path a user of the crate can name, through public modules and re-exports, before any type.
- **A legend.** Each file explains its own notation, so a reader with only the file, a person or an AI, can follow it.
- **Links back to the source.** Each declaration, impl block, module, and macro links back to where it is written, such as `src/prompt.rs:136`. The links are relative to the output file and end in `#L136`, which GitHub, VS Code, and Cursor open at that line. They are on by default.
- **Facts for analysis, when you ask.** With `--facts`, each type gets counts and a list of what uses it, and two sections list the other crates' items the signatures name and the types that share a shape. `--profile analysis` turns on everything an API reviewer wants in one switch.

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

vibelight is also on crates.io, so `cargo install vibelight` builds and installs the latest published version. To build your own copy, run `cargo install --path .` in a checkout of this repository.

## Run

```
vibelight [<crate-dir-or-file.rs>...] [-o <dir>] [--md-out <dir>] [--html-out <dir>]
          [--no-source-links] [--md-source-links on|off] [--html-source-links on|off]
          [--kill-list full|counts|off] [--facts] [--order levels|modules]
          [--profile analysis]
          [--workspace] [--follow <glob>] [--check] [--check-html]
```

- `<crate-dir-or-file.rs>` is a crate directory, with or without a `Cargo.toml`, the `Cargo.toml` itself, or a single `.rs` file. Several crates can be named at once (see [Several crates in one run](#several-crates-in-one-run)); a single `.rs` file cannot be mixed with them. With no crate named and no `--workspace`, a [`vibelight.toml`](#vibelighttoml) chooses the crates.
- `-o` (or `--out`) is the directory to write both formats into. It defaults to the current directory and is created if it is missing.
- `--md-out` and `--html-out` set the directory of one format, in place of `-o`, so the markdown and HTML files can live apart.
- `--no-source-links` leaves out the links back to files and lines in both formats. The output then depends only on the declarations themselves, so moving code between files, reordering items, or reformatting leaves the files byte for byte the same, and a run does not touch them (see the stamp in [Checking the files](#checking-the-files)).
- `--md-source-links on|off` and `--html-source-links on|off` turn the source links of one format on or off, whatever `--no-source-links` says.
- `--kill-list full|counts|off` chooses how much of the macro kill list to show: every entry (the default), only the counts line, or neither. The **MACRO** marks themselves always print.
- `--facts` adds the facts: a `Counts:` and a `Used by:` line under each type, and the External items and Same shapes sections (see [What the output looks like](#what-the-output-looks-like)).
- `--order levels|modules` orders the types by level, then a Modules section (the default), or in one section per module, with a Levels section that indexes them.
- `--profile analysis` changes the defaults for analyzing an API: source links off, `--kill-list counts`, `--facts`, and `--order modules`. Any switch or config key you give still wins.
- `--check` and `--check-html` write nothing and compare the files on disk with what the run would write (see [Checking the files](#checking-the-files)).

A file whose content is the same as the one on disk is left alone; any other file of the same name is overwritten. vibelight prints the path of every file of the run on stdout, sorted, whether it wrote the file or the file was already current. For example, `vibelight ../my-crate -o outline` on a library crate named `my-crate` writes `outline/my_crate.md` and `outline/my_crate.html`.

## Targets

In a crate directory, vibelight reads `Cargo.toml` for the package name and any `[lib]` and `[[bin]]` entries, and finds `src/lib.rs`, `src/main.rs`, and `src/bin/*.rs` the way Cargo does. Each target gets its own pair of files:

- The library is named after the library target, with hyphens turned into underscores, so package `my-crate` gives `my_crate.md`.
- Each binary keeps its name as written. A binary with the same name as the library gets a `.bin` stem, so `foo.bin.md` sits beside `foo.md`.
- Without a manifest, the library and `src/main.rs` take the directory's name, and each `src/bin/*.rs` takes its file stem.
- A single `.rs` file is named after its file stem.

Tests, benches, examples, and `build.rs` are not outlined. A root `Cargo.toml` with only a `[workspace]` table is not a crate, so point vibelight at a member crate's directory, or use `--workspace`. A root `Cargo.toml` that is also a package counts as one of the members.

## Several crates in one run

A run can cover several crates, and every crate in it gets its files. There are three ways to choose them on the command line, and a [`vibelight.toml`](#vibelighttoml) can hold the same choice:

- **Paths:** `vibelight crates/alpha crates/beta` names the crates one by one.
- **`--workspace`:** `vibelight --workspace` reads the `[workspace] members` of the root `Cargo.toml`, found at the given directory or the first one above it with a `[workspace]` table. Globs such as `crates/*` expand to the folders that hold a `Cargo.toml`, and `exclude` takes folders out of the globs, while a member written out in full always counts.
- **`--follow <glob>`:** from the crates named, follow their path dependencies, again and again, but only into folders that match a glob, measured from the workspace root (or else the first crate). It can be given more than once. For example, `vibelight crates/harness --follow "crates/harness-internal/*"` adds the harness's private crates and nothing else.

A crate in the run with nothing to outline, such as one with only tests, is skipped with a `note:` line on stderr. A single named crate with nothing to outline is still an error. Two crates that would write the same file name stop the run with an error that names both.

In the HTML file, a type, trait, or re-export that resolves into another crate of the run links to that crate's page, as `runvil_model_client.html#type.runvil_model_client.model.ModelSet`, so all HTML files of a run belong in one folder. A re-exported function, constant, or static links the same way, however many crates re-export it on the way, as `runvil_engine.html#fn.runvil_engine.execute.perform_vfs_op`. vibelight finds the other crate through the `[dependencies]` and `[build-dependencies]` of `Cargo.toml`, including `workspace = true` entries, `package = "..."` renames, and `[lib] name`, but only path dependencies. A crate can name only its own dependencies, and a type whose crate is not in the run stays plain text. The markdown files have no type links, so they do not depend on which crates are in the run, except with `--facts`: then External items shows the first doc line of an item from another crate of the run. No file depends on the order the crates are named in.

## vibelight.toml

A `vibelight.toml` file keeps a repository's settings, so a plain `vibelight` with no arguments does the whole run. vibelight looks for it in the current directory and then in each parent directory, the way Cargo looks for its files, and uses the first one it finds. Every path in it is relative to the folder that holds it, so the run is the same from any folder below it.

```toml
workspace = true
follow = ["vendor/*"]

[markdown]
out = "vibelight"
source_links = false

[html]
out = "vibelight/html"
source_links = true
```

The file has three keys that choose the crates, four keys that shape every file, and two tables that place the files:

- `workspace = true` outlines every member of the workspace that holds the config's folder, the same as `--workspace`. `workspace = "<dir>"` starts the search from that directory instead.
- `crates = ["<path or glob>", ...]` lists the crates instead. A path names one crate, the same as on the command line. A glob such as `crates/*` adds each matching folder that holds a `Cargo.toml`, and skips one with nothing to outline with a `note:` line. Set `workspace` or `crates`, not both.
- `follow = ["<glob>", ...]` works like `--follow`.
- `kill_list = "full"`, `"counts"`, or `"off"` works like `--kill-list`.
- `facts = true` or `false` works like `--facts`.
- `order = "levels"` or `"modules"` works like `--order`.
- `profile = "analysis"` works like `--profile`.
- `[markdown]` and `[html]` take the same three keys, each for its own format. `out = "<dir>"` is the folder for the files, and defaults to the config's folder. `source_links = true` or `false` turns the links back to source files and lines on or off, and defaults to on, or to off with the analysis profile. `enabled = false` turns the format off, and defaults to true.

A key vibelight does not know, or a value of the wrong type, is an error that names the key, such as ``unknown key `markdown.dir` ``. A key that takes one of a few words also lists them, such as ``kill_list` is one of `full`, `counts`, `off` ``. A config that turns both formats off is an error too.

### Which setting wins

A flag wins over the config, the config wins over a profile, and a profile wins over the defaults; a profile changes nothing but defaults. For each format:

- The folder comes from that format's own flag (`--md-out` or `--html-out`), then `-o`, then the config's `out`, then the config's folder, or the current directory when there is no config.
- The source links come from that format's own flag (`--md-source-links` or `--html-source-links`), then `--no-source-links`, then the config's `source_links`, then on, or off with `--profile analysis`.
- A format with `enabled = false` is not written, unless its own folder flag names a folder for it.

The kill list, the facts, and the order come from their flag, then their config key, then the profile, then the defaults: the full kill list, no facts, and the level order. So `profile = "analysis"` with `order = "levels"` gives the analysis settings in the level order.

Crate paths or `--workspace` on the command line replace the config's `workspace`, `crates`, and `follow`, but the config still places the files. With neither, the config chooses the crates, and `--follow` on the command line replaces the config's `follow`. With no crates named anywhere, vibelight stops with a usage error.

## Checking the files

`vibelight --check` does the whole run in memory and writes nothing. It compares each markdown file the run would write with the file on disk, and prints one line for each problem on stdout:

- `differs: <file> (first difference at line N)`: the file is not what the run would write.
- `missing: <file>`: the run would write the file, but it is not there.
- `stale: <file>`: no crate of the run writes this markdown outline any more, for example after a crate was renamed or removed. Only `.md` files directly in the markdown folder count, and only those that start the way every outline does: the stamp line, a blank line, and the crate anchor, or, for outlines from releases before the stamp, the crate anchor on the first line. So a README beside them, other file types, and subfolders such as `html/` are left alone.

A last line says `check passed: N files match` or `check failed: N problems`, and the exit code is 0 or 1. `--check-html` compares the HTML files too. A CRLF line ending on disk counts as LF, so a checkout on Windows still passes. Paths are shown relative to the current directory when they are inside it.

Every file starts with a stamp, the UTC time its content last changed, such as `*Updated 2026-01-01T00:00:00Z*` in markdown and an `<!-- Updated ... -->` comment and a line at the top of the page in HTML. A run rewrites a file only when its content changed, so the stamp moves only with the content, and the check ignores the stamp. The time comes from `SOURCE_DATE_EPOCH` when it is set, for reproducible builds, and otherwise from the clock; a value that is not a whole number of seconds stops a run that writes, and never a check.

The check covers exactly the crates and settings of its run, so a file that only another set of crates writes shows up as stale. With a `vibelight.toml`, a plain `vibelight --check` from anywhere in the repository checks the same run that a plain `vibelight` writes. Crate arguments and flags work with `--check` the same way as without it.

A normal run never deletes a file. For each stale outline it prints `warning: <file> is stale: no crate of this run writes it` on stderr instead, so you can remove the file yourself.

## Using it in a repository

A layout that works well keeps one outline in git for reviewers to read in diffs, and a linked copy beside it for browsing:

- `vibelight/` holds the markdown files, committed, with source links off. They change only when a declaration or a comment changes, and the stamp on their first line changes with them, so every diff of them shows a change to the crate's surface and nothing else.
- `vibelight/html/` holds the HTML files, with source links on, for reading locally. It is gitignored, because its links change whenever code moves.
- Each crate's library gets one file in each folder, named after the library with hyphens turned into underscores, such as `vibelight/runvil_engine.md` and `vibelight/html/runvil_engine.html`. A binary gets its own pair, named after the binary.

For a workspace like Runvil, put this `vibelight.toml` at the root:

```toml
workspace = true

[markdown]
out = "vibelight"
source_links = false

[html]
out = "vibelight/html"
source_links = true
```

And this line in `.gitignore`:

```
/vibelight/html/
```

Run `vibelight` from anywhere in the repository to refresh both folders, and commit the markdown together with the change that caused it.

To keep the committed outline current, add a CI step that fails when it is out of date. In GitHub Actions, after the checkout step:

```yaml
      - name: Check the vibelight outline
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cppalliance/vibelight/releases/download/v0.2.0/vibelight-installer.sh | sh
          vibelight --check
```

Version 0.1.0 predates the config file and `--check`, so pin v0.2.0 or later. The step pins one release, because a newer vibelight can lay out the same code a little differently. The pinned version and the committed outline change together: when you move to a newer release, change the version in the step and commit the markdown that release writes in the same commit, so the check passes on every commit.

The release after 0.2.0 changes every outline: each file gains the stamp and the Legend, and most gain a Public surface section. Regenerate the committed outlines with it, in the same commit that moves the pinned version.

## Colors

The HTML file colors code the way an editor does. A link inside the page uses the link color, whether it goes to a type or to a function, constant, or static, and a link to another crate's page is yellow: a darker gold on the light theme and a lighter yellow on the dark theme, so it stays readable on both. With the HTML source links off, a member name that would link to its source line keeps its look as plain colored text, without an underline or a pointer, and no file or line appears anywhere.

## What the output looks like

Each file opens with the stamp (see [Checking the files](#checking-the-files)), the target's name, and its crate docs, then has these parts:

- **Legend:** what the notation means, in fixed text, so the file explains itself.
- **Contents:** links to every section and declaration.
- **Kill list:** a count of macros by kind, then one entry per macro, linking to the declaration or module that holds it. `--kill-list` can shorten it to the counts or leave it out.
- **Public surface:** for a library, every path a user of the crate can name, walking public modules and `pub use` re-exports, globs and `#[macro_export]` macros included. Each path links to what it names, a re-export of another crate's item shows the path it comes from, and test-support code is marked `(test support)`.
- **Level 1, Level 2, and so on:** one heading per type. Level 1 holds the types no other type names, and every other type sits one level below the deepest type that names it. Types that name each other in a cycle are placed together.
- **Test support:** types behind a `cfg` that lists `test` inside `any(...)`, such as `cfg(any(test, feature = "test-support"))`, kept apart from the production types.
- **Unconnected types:** types that name nothing and that nothing names. A type that a free function, a constant, a static, or a re-export names is in use, so it sits at level 1 instead.
- **Other impls:** impls with no declared type to attach to, such as blanket impls or impls for `&T` or `Box<T>`, as `implements: Trait for Type;` lines.
- **Modules:** one heading per module, with its docs, links to its declarations, and its re-exports, macros, constants, statics, free functions, and extern blocks.
- **Spellings:** each item from outside the crate that the source writes in more than one way, with every spelling (see [Spellings](#spellings)).

With `--facts`, each type's heading also gets a `Counts:` line (variants, fields with how many are an `Option`, associated items, special members, and functions) and a `Used by:` line that links what names it, and two more sections follow: **External items**, each item of another crate that the signatures name, grouped by crate, with its uses and, for a crate of the run, its first doc line; and **Same shapes**, groups of types with the same variants or fields, which are candidates for merging. With `--order modules`, a Levels section indexes the types by level, and then each module gets its own section that holds its items and its types, in place of the level sections and the Modules section.

A level, Test support, Unconnected types, Other impls, Spellings, External items, or Same shapes shows up only when it has something in it. The markdown and HTML files hold the same headings, marks, and blocks. Markdown cannot put a link inside a code block, so only the HTML file links type names.

### Links to functions, constants, and statics

In the HTML file, each module-level function, constant, and static has an id on its name, `fn.<crate>.<module path>.<name>`, `const.<crate>.<module path>.<NAME>`, or `static.<crate>.<module path>.<NAME>`, written like the ids of types: parts joined with dots, case kept, and `r#` removed. A repeated id, such as a function declared twice under different `cfg`s, gets `-2`, `-3`, and so on. Methods and the items of `extern` blocks have no ids, and the markdown file has none of these ids.

A re-export line (`using ...;`) links to a function, constant, or static of this crate, or of another crate of the run, through renames and chains of `pub use` items. A backticked name or an intra-doc link in a comment links the same way, when the name is in scope through a `use`, a glob, a path, or an item of the same module. When a type and a function have the same name, the link goes to the type. A function name is never found just because it is the only one of that name in the crate, which is how types are found, so a comment that names a function of another module needs a path or a `use`.

Every type prints as one C++-style block, with the members of all its inherent impls merged in, wherever they are written. Here is a type whose fields and methods are all private:

```
/// Holds the text the lexer is still reading.
class Cursor
{
    pos: usize,

    text: String,

    static new(text: String) -> Cursor;

    bump() mut;

    into_text(self) -> String;

    peek() -> Option<char>;
}
```

Since nothing in it is public, it prints as a `class` with no label. A type with a public member prints as a `struct`. Its members sit in three sections: public ones first without a label, then `internal:` for `pub(crate)`, `pub(super)`, and `pub(in path)`, which an internal crate uses for almost everything, then `private:`. A module's trailing block uses the same three sections.

The `fn` keyword and the bodies are gone. A method that takes `&self` gets no marker, a method that takes `&mut self` gets `mut` after its parameter list, a method that takes `self` in any other way keeps it as its first parameter, and only a function with no receiver is `static`. A bare `&` is read-only, as in Rust, and `mut` marks what can change, so `&str` prints as `str&` and `&mut T` as `T mut&`. Members are grouped (associated types, constants, fields, static functions, then member functions) and sorted by name inside each group.

Comments stay with what they belong to. Each impl block's comments sit under an `// impl Trait` or `// impl Type` line in the type's block, a comment above a `pub use` group prints above the whole group, and a comment inside the braces stays with the name after it. Doc comments keep their backticks, so `` `args` `` still reads as code. Attributes that shape the API or its wire format print as `[[...]]` lines, such as `[[non_exhaustive]]`, `[[must_use]]`, `[[deprecated]]`, `[[repr(C)]]`, `[[track_caller]]`, `[[doc(hidden)]]`, and `[[serde(rename = "id")]]`; other attributes stay hidden. With source links on, each inherent impl's link in a `Source:` line is followed by the names of the members it adds, which shows how a large type's methods cluster by file.

Traits a type implements or derives print the C++ way. Only the C++ special member functions become members: `Default` is the default constructor, `From<T>` a converting constructor, and `Drop` the destructor, packed together after the fields, with `= default` when derived. Every other standard trait, `Clone` and the comparison traits included, goes in the base list by its short name, as in `struct Seed : Clone, Debug, Display`, and every other trait follows on its own line by its full path. A name counts as standard only when it really resolves to `std`, `core`, or `alloc`, so another crate's `Display`, the crate's own `Display`, or a `Display` that only comes through a glob import gets its own line. A derive from another crate keeps its bold **MACRO** line above the block.

## Spellings

The Spellings section lists each item from outside the crate that the declarations write in more than one way, such as `fmt::Debug` in one place and `std::fmt::Debug` in another, so you can pick one and change the rest. It covers `std`, `core`, `alloc`, and other crates. A bare prelude name such as `Option` counts as `std::option::Option`, a `core::` or `alloc::` path counts as the `std` path that re-exports it, and a `use ... as` rename is one more spelling. Each entry gives every spelling with how often it is written. For a standard item, it also names the spelling to prefer: the bare name, as a base list prints it, such as `Debug` or `Cow`. That spelling shows only its count, and every other spelling also gets a link to each line, when source links are on:

```
- `std::fmt::Debug` is written 3 ways; prefer `Debug`:
  - `Debug`, 12 times
  - `fmt::Debug`, 2 times (source: [src/lib.rs:4](src/lib.rs#L4), [src/lib.rs:9](src/lib.rs#L9))
  - `std::fmt::Debug`, once (source: [src/lib.rs:12](src/lib.rs#L12))
```

When the bare name is not written anywhere yet, the entry says so. An item has no preferred spelling when the crate declares a type or trait with that bare name, when the crate uses another standard item with the same bare name (`fmt::Write` and `io::Write`), or when it comes from another crate. Its entry then says to pick one spelling and search for the rest.

Names of the crate's own items are left out, because `crate::model::X`, `X`, and `super::X` are all normal in different modules. So are names in comments, re-exports, `use` lines, and function bodies. Glob imports such as `use std::fmt::*` are not followed, so a name that only comes through one is not counted.

## Warnings and exit codes

When vibelight meets something it cannot fully handle, it keeps going and prints a warning on stderr, as `warning: <file>:<line>: <message>`, then a count at the end. Warnings cover files that do not parse, module files that cannot be found, constructs it does not support (also marked UNSUPPORTED in the outline), and type names it cannot resolve or that are ambiguous, which stay plain text instead of links. When the Spellings section has entries, one more line says how many items are written more than one way. In a run of several crates, a warning's path starts from the workspace root (or else the first crate), such as `crates/alpha/src/lib.rs:9`, and a skipped crate gets a `note:` line, which is not a warning.

A stale outline in the markdown folder also gets a warning line (see [Checking the files](#checking-the-files)).

- `0`: the files were written, warnings included. With `--check`, every file matches.
- `1`: the input cannot be used, for example the path is not a crate, the manifest is workspace-only, a root file cannot be read, `vibelight.toml` has an unknown key, or a run that writes finds a `SOURCE_DATE_EPOCH` that is not a whole number of seconds. The reason is printed on stderr. With `--check`, a file differs, is missing, or is stale.
- `2`: a usage error, such as an unknown flag, or no crate named on the command line or in a `vibelight.toml`, reported by clap.

## Requirements and license

- Minimum Rust version: 1.85
- License: BSL-1.0 (Boost Software License 1.0), in `LICENSE`
