# vibelight

[![CI](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/vibelight)](https://crates.io/crates/vibelight) [![Latest release](https://img.shields.io/github/v/release/cppalliance/vibelight)](https://github.com/cppalliance/vibelight/releases/latest) [![License](https://img.shields.io/github/license/cppalliance/vibelight)](LICENSE) [![Minimum Rust version](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

vibelight reads the Rust crates of a repository and writes an outline of each: every type, trait, function, and module with its comments, and no function bodies. Each target gets two files for two readers. The markdown file is for a model analyzing the code: the declarations written as Rust with the bodies left out, and everything vibelight knows about them, such as what uses each type, which functions reach each field, the items of other crates, and links back to every line. The HTML page is for a person browsing the code: the declarations in the style of a C++ header, with every type name linked.

## What it does

- **No switches.** Run `vibelight` at the root of a repository. The first run writes a `vibelight.toml` that chooses the crates worth analyzing and leaves out the ones that only support development, each with a comment that says why. Every later run reads it.
- **A folder git never sees.** The markdown goes in `.vibelight/`, the HTML in `.vibelight/html/`, and a `.gitignore` there keeps all of it out of git. The folder holds what the last run wrote: the outlines of a crate the run no longer covers are removed.
- **An index.** `.vibelight/index.md` lists each outline with its crate's folder, whether the run selected the crate or followed a path dependency to it, and its size.
- **One outline per target.** The library, each binary, or a single `.rs` file gets `<target>.md` and `html/<target>.html`.
- **Facts for analysis.** The markdown gives each type a count of what it holds and a list of what uses it, lists which functions reach each field of a large type, groups each type's functions by the impl that adds them, lists the items of other crates that the signatures name, and groups the types that share a shape.
- **A self-contained, linked HTML file.** It has inline CSS, no JavaScript, and makes no network requests. Every type name that refers to a declaration in the outline links to it, and so does a re-exported function, constant, or static. The contents list sits in a sticky sidebar, and the colors follow your system's light or dark setting.
- **Top-down order.** Declarations are ordered by how they name each other. The types nothing else names come first, and every type comes after the types that use it. The HTML lists the types by level and ends with a by-module index; the markdown gives each module its own section and indexes the levels.
- **Comments kept.** Doc comments and ordinary comments stay with the item they belong to.
- **Every macro marked.** Macro definitions, invocations, derives from other crates, and attribute macros each get a bold **MACRO** label. The markdown counts them at the top, and the HTML lists them all there, each with a link to where it sits.
- **The public surface first.** A library's file lists every path a user of the crate can name, through public modules and re-exports, before any type.
- **Rust for the model.** The markdown writes every declaration as Rust, bodies left out, with visibilities, signatures, and impl headers as in the source, and it explains its own markers in a Legend, so a model with only the file can follow it.
- **Links back to the source.** Each declaration, impl block, module, and macro links back to where it is written, such as `src/prompt.rs:136`. The links are relative to the output file and end in `#L136`, which GitHub, VS Code, and Cursor open at that line.

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
vibelight [<crate-dir-or-file.rs>...] [-o <dir>]
```

- With no arguments, vibelight reads the `vibelight.toml` in the current directory or the first parent that holds one, the way Cargo finds its files, and outlines the crates it chooses. When there is none, it writes one first (see [The first run](#the-first-run)).
- `<crate-dir-or-file.rs>` is a crate directory, with or without a `Cargo.toml`, the `Cargo.toml` itself, or a single `.rs` file, in place of the crates the config chooses. Several crates can be named at once. A crate named here is always outlined, even when the config's `exclude` matches it, and the config's `follow` and `exclude` still apply to the crates the run follows into. A single `.rs` file cannot be mixed with crates, and the config's globs pass it by.
- `-o` (or `--out`) is the folder to write into, in place of the config's. The markdown files and the index go in it, and the HTML files in its `html` folder. It is created if missing.

A file whose content is the same as the one on disk is left alone; any other file of the same name is overwritten. vibelight prints the path of every file of the run on stdout, sorted, whether it wrote the file or the file was already current. For example, `vibelight ../my-crate -o outline` on a library crate named `my-crate` writes `outline/my_crate.md`, `outline/html/my_crate.html`, and `outline/index.md`.

## The first run

When no crate is named and no `vibelight.toml` is found, vibelight finds the root of the repository: the nearest folder, at or above the current one, whose `Cargo.toml` has a `[workspace]` table, or else the nearest folder that holds a `Cargo.toml`. It writes `vibelight.toml` there, says so on stderr, and then runs. For a workspace like Runvil it looks like this:

```toml
# Written by vibelight. It chooses the crates a plain `vibelight` outlines:
# edit `exclude` to change which of them are analyzed.
workspace = true
follow = ["**"]
exclude = [
    "crates/build-ui",  # build tooling
    "crates/harness-internal/test-plugin",  # a test fixture or helper
]
out = ".vibelight"
```

It leaves out each workspace member that only supports development:

- a member that an alias in the root's `.cargo/config.toml` runs, through `-p`, `--package`, or `--manifest-path`;
- a member whose name or folder starts with `build`, or holds `xtask`, as build tooling;
- a member whose name, or a folder on its path, holds the word `fuzz`, `bench`, `example`, `test`, or `fixtures`, or one of their plurals and kin, reading names as words split at `-` and `_`;
- a member that other members depend on only as a build dependency or only as a dev dependency.

A repository that is one crate gets `crates = ["."]` in place of `workspace = true`. The file is yours to edit, and vibelight never rewrites it. With no `Cargo.toml` at or above the current folder, the run stops with a usage error.

## Targets

In a crate directory, vibelight reads `Cargo.toml` for the package name and any `[lib]` and `[[bin]]` entries, and finds `src/lib.rs`, `src/main.rs`, and `src/bin/*.rs` the way Cargo does. Each target gets its own pair of files:

- The library is named after the library target, with hyphens turned into underscores, so package `my-crate` gives `my_crate.md`.
- Each binary keeps its name as written. A binary with the same name as the library gets a `.bin` stem, so `foo.bin.md` sits beside `foo.md`.
- Without a manifest, the library and `src/main.rs` take the directory's name, and each `src/bin/*.rs` takes its file stem.
- A single `.rs` file is named after its file stem.

Tests, benches, examples, and `build.rs` are not outlined. A root `Cargo.toml` with only a `[workspace]` table is not a crate, so point vibelight at a member crate's directory, or let the config choose the members.

## Several crates in one run

A run covers several crates, and every crate in it gets its files. The config chooses them in one of two ways:

- **`workspace = true`** reads the `[workspace] members` of the root `Cargo.toml`, found at the config's folder or the first one above it with a `[workspace]` table. Globs such as `crates/*` expand to the folders that hold a `Cargo.toml`, and the workspace's `exclude` takes folders out of the globs, while a member written out in full always counts. A root `Cargo.toml` that is also a package counts as one of the members.
- **`crates = [...]`** lists the crates, by path or by glob.

`follow` then adds the path dependencies of the chosen crates, again and again, but only those in folders that match one of its globs, measured from the workspace root (or else the first crate). The first run writes `follow = ["**"]`, so every path dependency below the root is outlined too, as context. `exclude` leaves crates out of whatever the run selected, whether by `workspace`, by `crates`, or by `follow`. A glob matches a crate's folder, written with forward slashes on every platform, and is measured from the config's folder. An excluded crate is not in the run: it is not read, it writes no files, its path dependencies are not followed on its account, and a type name that points at it stays plain text, as for any crate outside the run. If exclusion leaves no crate, the run stops with the error of an empty run.

Three cases get a `note:` line on stderr, which is not a warning, and the exit code stays 0. A crate that `crates` names and `exclude` also matches is left out, with a line that says so, because that is probably a mistake. A glob that matches no crate gets `note: exclude pattern <glob> matches no crate`, so a typo cannot quietly keep the build tools in; a run of named crates leaves this note out, since it never looks at the rest. And a crate in the run with nothing to outline, such as one with only tests, is skipped with a line that says why. A single named crate with nothing to outline is still an error. Two crates that would write the same file name stop the run with an error that names both.

In the HTML file, a type, trait, or re-export that resolves into another crate of the run links to that crate's page, as `runvil_model_client.html#type.runvil_model_client.model.ModelSet`, so all HTML files of a run sit in one folder. A re-exported function, constant, or static links the same way, however many crates re-export it on the way, as `runvil_engine.html#fn.runvil_engine.execute.perform_vfs_op`. vibelight finds the other crate through the `[dependencies]` and `[build-dependencies]` of `Cargo.toml`, including `workspace = true` entries, `package = "..."` renames, and `[lib] name`, but only path dependencies. A crate can name only its own dependencies, and a type whose crate is not in the run stays plain text. In the markdown, External items shows the first doc line of an item from another crate of the run. No file depends on the order the crates are named in.

## vibelight.toml

The config has five keys, and every path in it is relative to the folder that holds it, so the run is the same from any folder below it:

- `workspace = true` outlines every member of the workspace that holds the config's folder. `workspace = "<dir>"` starts the search from that directory instead.
- `crates = ["<path or glob>", ...]` lists the crates instead. A path names one crate, the same as on the command line. A glob such as `crates/*` adds each matching folder that holds a `Cargo.toml`, and skips one with nothing to outline with a `note:` line. Set `workspace` or `crates`, not both.
- `follow = ["<glob>", ...]` adds the path dependencies whose folders match.
- `exclude = ["<glob>", ...]` leaves crates out of the workspace, the `crates` list, and the followed dependencies alike.
- `out = "<dir>"` is the folder to write into. It defaults to `.vibelight` beside the config.

A key vibelight does not know, or a value of the wrong type, is an error that names the key, such as ``unknown key `markdown` ``. Without `workspace` or `crates`, a run that names no crate is a usage error.

## The folder

The markdown files and `index.md` go in the folder, and the HTML files in its `html` folder. When the folder is vibelight's own, the config's or the default `.vibelight`, rather than one `-o` names, the run also keeps a `.gitignore` in it that holds `*.md` and `*.html`, so no outline is ever committed. The outlines carry source links and follow how the code is split across files, so they change whenever code moves; they are for reading, not for diffs.

Every file starts with a stamp, the UTC time its content last changed, such as `*Updated 2026-01-01T00:00:00Z*` in markdown and an `<!-- Updated ... -->` comment and a line at the top of the page in HTML. A run rewrites a file only when its content changed, so the stamp moves only with the content. The time comes from `SOURCE_DATE_EPOCH` when it is set, for reproducible builds, and otherwise from the clock; a value that is not a whole number of seconds stops the run before it writes anything.

The folder holds what the last run wrote. After writing, a run removes each outline that no crate of the run writes any more, such as the files of a crate that was renamed, removed, or excluded, with a `note:` line for each. Only files that start the way every outline does count: in the folder, a `.md` file with the stamp line, a blank line, and the crate anchor; in `html`, an `.html` file with the stamp comment and then the doctype. So a README beside them, other file types, and the index are left alone. A run that names one crate leaves only that crate's files, so use `-o` for a one-off outline of a crate the config leaves out.

`index.md` lists each outline of the run, with its crate's folder below the root, whether the run selected the crate or followed a path dependency to it, and its size in KB:

```
- [runvil_engine.md](runvil_engine.md): `crates/runvil-internal/engine`, selected, 436 KB
- [gamma.md](gamma.md): `vendor/gamma`, followed, 4 KB
```

## Colors

The HTML file colors code the way an editor does. A link inside the page uses the link color, whether it goes to a type or to a function, constant, or static, and a link to another crate's page is yellow: a darker gold on the light theme and a lighter yellow on the dark theme, so it stays readable on both. A member name links to the line that declares it.

## What the output looks like

Each file opens with the stamp, the target's name, and its crate docs. The markdown then has its **Legend**, what the notation means, in fixed text, so the file explains itself. Both formats then have these parts:

- **Contents:** links to every section and declaration.
- **Kill list:** a count of macros by kind. The HTML follows it with one entry per macro, linking to the declaration or module that holds it.
- **Public surface:** for a library, every path a user of the crate can name, walking public modules and `pub use` re-exports, globs and `#[macro_export]` macros included. Each path links to what it names, a re-export of another crate's item shows the path it comes from, and test-support code is marked `(test support)`.
- **The types, by level.** Level 1 holds the types no other type names, and every other type sits one level below the deepest type that names it. Types that name each other in a cycle are placed together. **Test support** holds types behind a `cfg` that lists `test` inside `any(...)`, such as `cfg(any(test, feature = "test-support"))`, kept apart from the production types, and **Unconnected types** holds types that name nothing and that nothing names. A type that a free function, a constant, a static, or a re-export names is in use, so it sits at level 1 instead, and so does a type that a function body names outside the type's own impls. The HTML gives each level its own heading. The markdown has a **Levels** section that indexes the types this way, and then one section per module that holds its items and its types.
- **Other impls:** impls with no declared type to attach to, such as blanket impls or impls for `&T` or `Box<T>`, as `implements: Trait for Type;` lines in the HTML and `impl Trait for Type {}` lines in the markdown.
- **Modules,** in the HTML: one heading per module, with its docs, links to its declarations, and its re-exports, macros, constants, statics, free functions, and extern blocks. In the markdown, each module's own section holds the same.
- **Spellings:** each item from outside the crate that the source writes in more than one way, with every spelling (see [Spellings](#spellings)).

The markdown adds the facts. Each type's heading gets a `Counts:` line (variants, fields with how many are an `Option`, associated items, special members, and functions) and a `Used by:` line that links what names it. A type with 8 or more functions also gets a `Fields:` list: each field, in declared order, with the functions that reach it through `self`, so the parts of a large type show. A hub, a field that more than half of its functions reach, gives only a count. Each type's functions are grouped by the inherent impl that adds them, in source order, each group under a `// impl in <file>` line, so you can see how a large type's methods are split across files. Two more sections follow: **External items**, each item of another crate that the signatures name, grouped by crate, with its uses and, for a crate of the run, its first doc line; and **Same shapes**, groups of types with the same variants or fields, which are candidates for merging.

A section shows up only when it has something in it. Markdown cannot put a link inside a code block, so only the HTML file links type names.

### Links to functions, constants, and statics

In the HTML file, each module-level function, constant, and static has an id on its name, `fn.<crate>.<module path>.<name>`, `const.<crate>.<module path>.<NAME>`, or `static.<crate>.<module path>.<NAME>`, written like the ids of types: parts joined with dots, case kept, and `r#` removed. A repeated id, such as a function declared twice under different `cfg`s, gets `-2`, `-3`, and so on. Methods and the items of `extern` blocks have no ids, and the markdown file has none of these ids.

A re-export line (`using ...;`) links to a function, constant, or static of this crate, or of another crate of the run, through renames and chains of `pub use` items. A backticked name or an intra-doc link in a comment links the same way, when the name is in scope through a `use`, a glob, a path, or an item of the same module. When a type and a function have the same name, the link goes to the type. A function name is never found just because it is the only one of that name in the crate, which is how types are found, so a comment that names a function of another module needs a path or a `use`.

### The markdown notation

The markdown writes each type as Rust with the bodies left out, so a model reads it the way it reads source: the item with its attributes and its fields, variants, or trait items, then each inherent impl that adds members, under a `// impl in <file>` line, then one line for each trait impl. Visibilities, signatures, and impl headers are written as in the source, without their comments; only `pub(in crate)`, `pub(in self)`, and `pub(in super)` print the way rustfmt writes them. `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out. A signature stays on one line, with one parameter per line only when a parameter has comments, and a `where` clause on lines of its own. A constant's value shows as `...` when it would make its line wider than 100 columns. Here is the type of the next section:

```
/// Holds the text the lexer is still reading.
struct Cursor {
    pos: usize,

    text: String,
}

// impl in src/lexer.rs
impl Cursor {
    fn new(text: String) -> Cursor;

    fn bump(&mut self);

    fn into_text(self) -> String;

    fn peek(&self) -> Option<char>;
}
```

### The HTML notation

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

Since nothing in it is public, it prints as a `class` with no label. A type with a public member prints as a `struct`. A type that is not public itself gets a `// internal` or `// private` line right above its header, so its own visibility shows too. Its members sit in three sections: public ones first without a label, then `internal:` for `pub(crate)`, `pub(super)`, and `pub(in path)`, which an internal crate uses for almost everything, then `private:`. A module's trailing block uses the same three sections.

The `fn` keyword and the bodies are gone. A method that takes `&self` gets no marker, a method that takes `&mut self` gets `mut` after its parameter list, a method that takes `self` in any other way keeps it as its first parameter, and only a function with no receiver is `static`. A bare `&` is read-only, as in Rust, and `mut` marks what can change, so `&str` prints as `str&` and `&mut T` as `T mut&`. Members are grouped (associated types, constants, fields, static functions, then member functions) and sorted by name inside each group, except fields, which keep their declared order, as enum variants do.

Comments stay with what they belong to. Each impl block's comments sit under an `// impl Trait` or `// impl Type` line in the type's block, a comment above a `pub use` group prints above the whole group, and a comment inside the braces stays with the name after it. Doc comments keep their backticks, so `` `args` `` still reads as code. Attributes that shape the API or its wire format print as `[[...]]` lines, such as `[[non_exhaustive]]`, `[[must_use]]`, `[[deprecated]]`, `[[repr(C)]]`, `[[track_caller]]`, `[[doc(hidden)]]`, and `[[serde(rename = "id")]]`; other attributes stay hidden. Each inherent impl's link in a `Source:` line is followed by the names of the members it adds.

Traits a type implements or derives print the C++ way. Only the C++ special member functions become members: `Default` is the default constructor, `From<T>` a converting constructor, and `Drop` the destructor, packed together after the fields, with `= default` when derived. Every other standard trait, `Clone` and the comparison traits included, goes in the base list by its short name, as in `struct Seed : Clone, Debug, Display`, and every other trait follows on its own line by its full path. A name counts as standard only when it really resolves to `std`, `core`, or `alloc`, so another crate's `Display`, the crate's own `Display`, or a `Display` that only comes through a glob import gets its own line. A derive from another crate keeps its bold **MACRO** line above the block.

## Spellings

The Spellings section lists each item from outside the crate that the declarations write in more than one way, such as `fmt::Debug` in one place and `std::fmt::Debug` in another, so you can pick one and change the rest. It covers `std`, `core`, `alloc`, and other crates, and counts the names in supertraits, bounds, `where` clauses, impl headers, and field, parameter, return, and generic argument types. A bare prelude name such as `Option` counts as `std::option::Option`, a `core::` or `alloc::` path counts as the `std` path that re-exports it, and a `use ... as` rename is one more spelling. Each entry gives every spelling with how often it is written. For a standard item, it also names the spelling to prefer: the bare name, as a base list prints it, such as `Debug` or `Cow`. That spelling shows only its count, and every other spelling also gets a link to each line:

```
- `std::fmt::Debug` is written 3 ways; prefer `Debug`:
  - `Debug`, 12 times
  - `fmt::Debug`, 2 times (source: [src/lib.rs:4](src/lib.rs#L4), [src/lib.rs:9](src/lib.rs#L9))
  - `std::fmt::Debug`, once (source: [src/lib.rs:12](src/lib.rs#L12))
```

When the bare name is not written anywhere yet, the entry says so. An item has no preferred spelling when the crate declares a type or trait with that bare name, when the crate uses another standard item with the same bare name (`fmt::Write` and `io::Write`), or when it comes from another crate. Its entry then says to pick one spelling and search for the rest.

Names of the crate's own items are left out, because `crate::model::X`, `X`, and `super::X` are all normal in different modules. So are names in comments, re-exports, `use` lines, and function bodies. Derive names are left out too, and so are those inside `cfg_attr`. A derive is written bare in practice, so it never matches a path in an impl header, a bound, or a type, and an entry that counted it would give you nothing to fix. A type that derives `Debug` next to an impl that writes `fmt::Debug` is not an entry. Glob imports such as `use std::fmt::*` are not followed, so a name that only comes through one is not counted.

## Warnings and exit codes

When vibelight meets something it cannot fully handle, it keeps going and prints a warning on stderr, as `warning: <file>:<line>: <message>`, then a count at the end. Warnings cover files that do not parse, module files that cannot be found, constructs it does not support (also marked UNSUPPORTED in the outline), and type names it cannot resolve or that are ambiguous, which stay plain text instead of links. When the Spellings section has entries, one more line says how many items are written more than one way. In a run of several crates, a warning's path starts from the workspace root (or else the first crate), such as `crates/alpha/src/lib.rs:9`.

A `note:` line is not a warning. Notes say that the run wrote `vibelight.toml` or the folder's `.gitignore`, that it skipped a crate with nothing to outline, that a crate the config lists is also excluded, that an `exclude` glob matches no crate, or that it removed an outline no crate writes any more.

- `0`: the files were written, warnings included.
- `1`: the input cannot be used, for example the path is not a crate, the manifest is workspace-only, a root file cannot be read, `vibelight.toml` has an unknown key, or `SOURCE_DATE_EPOCH` is not a whole number of seconds. The reason is printed on stderr.
- `2`: a usage error, such as an unknown flag, no crate named and no Rust repository to write a config for, or a config that names no crates, reported by clap.

## Requirements and license

- Minimum Rust version: 1.85
- License: BSL-1.0 (Boost Software License 1.0), in `LICENSE`
