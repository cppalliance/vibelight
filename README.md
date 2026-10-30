# vibelight

[![CI](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/vibelight)](https://crates.io/crates/vibelight) [![Latest release](https://img.shields.io/github/v/release/cppalliance/vibelight)](https://github.com/cppalliance/vibelight/releases/latest) [![License](https://img.shields.io/github/license/cppalliance/vibelight)](LICENSE) [![Minimum Rust version](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

vibelight reads the Rust crates of a repository and writes an outline of each: every type, trait, function, and module with its comments, and no function bodies. Each target gets two files for two readers. The markdown file is for a model analyzing the code: one JSON record per line, starting with the leads vibelight computes for places that look like slop, then every declaration with its shape or signature as one line of Rust and everything vibelight observed about it, such as what names and uses it, which bodies call it, counts from each function body, the items of other crates, and the file and line of every declaration. The HTML page is for a person browsing the code: the declarations in the style of a C++ header, with every type name linked.

## What it does

- **No switches.** Run `vibelight` at the root of a repository. The first run writes a `vibelight.toml` that chooses the crates worth analyzing and leaves out the ones that only support development, each with a comment that says why. Every later run reads it.
- **A folder git never sees.** The markdown goes in `.vibelight/`, the HTML in `.vibelight/html/`, and a `.gitignore` there keeps all of it out of git. The folder holds what the last run wrote: the outlines of a crate the run no longer covers are removed.
- **An index.** `.vibelight/index.md` lists each outline with its crate's folder, whether the run selected the crate or followed a path dependency to it, and its size. `html/index.html` is the same list as a page that links to each other page, so the HTML folder has a place to start.
- **One outline per target.** The library, each binary, or a single `.rs` file gets `<target>.md` and `html/<target>.html`.
- **Facts for analysis.** The markdown gives each declaration the facts vibelight observes across the run: what names and re-exports it, which other crates and which kinds of test code use it, how often its name is written, which bodies call a function, and for each body, counts of its statements, loops, and error paths, the fields and parameters it touches, and its comments. Before the declarations come the leads: places that look like slop, each with an id, its evidence, the kind of evidence it rests on, and its known blind spots.
- **A self-contained, linked HTML file.** It has inline CSS, no JavaScript, and makes no network requests. Every type name that refers to a declaration in the outline links to it, and so does a re-exported function, constant, or static. The contents list sits in a sticky sidebar, and the colors follow your system's light or dark setting.
- **Dependency order.** In the HTML, declarations are ordered by how they name each other. The types nothing else names come first, each followed by the types it owns: a type that only one other type names follows it directly, and a type that several name follows the last of them, so every type comes after the types that use it. The HTML splits the types into groups, one for each run of types in a module, each opened by a rule and a heading named for the module and the type that leads the group, such as `execute::scheduler · Scheduler`, and ends with a by-module index. The markdown goes module by module in path order instead, with each module's records in source order.
- **Comments kept.** Doc comments and ordinary comments stay with the item they belong to. In the HTML a doc comment prints as prose, and one button in the sidebar hides or shows every member's comment. The markdown also keeps the comments inside each function body, with their lines.
- **Every macro marked.** In the HTML, macro definitions, invocations, derives from other crates, and attribute macros each get a bold **MACRO** label, and the page lists them all at the top, each with a link to where it sits. The markdown has a `macro` record for each definition and invocation, and lists derives and attribute macros on the record they apply to.
- **The public surface first.** A library's files list every path a user of the crate can name, through public modules and re-exports, before any type: a Public surface section in the HTML, and one `surface` record per path in the markdown.
- **Rust for the model.** The markdown writes each signature and each type's shape as one line of Rust, with visibilities, generics, and `where` clauses as in the source. The leads come first, every record names its kind in `k`, and the header gives the schema version, so a model with only the file can follow it.
- **Links back to the source.** In the HTML, each declaration, impl block, module, and macro links back to where it is written, such as `src/prompt.rs:136`. The links are relative to the output file and end in `#L136`, which GitHub, VS Code, and Cursor open at that line. In the markdown, each declaration's `loc`, or each lead's and impl's `locs`, is a string such as `src/prompt.rs:136`, and the header's `root` leads from the markdown folder to the crate, so `root` and the file together open the source.

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

Tests, benches, examples, `build.rs`, and doc tests are not outlined and get no records, but the markdown's census reads them: their qualified paths and the names they write count toward the facts and leads of the code they use. Code behind a plain `cfg(test)` is never in the catalog either. A root `Cargo.toml` with only a `[workspace]` table is not a crate, so point vibelight at a member crate's directory, or let the config choose the members.

## Several crates in one run

A run covers several crates, and every crate in it gets its files. The config chooses them in one of two ways:

- **`workspace = true`** reads the `[workspace] members` of the root `Cargo.toml`, found at the config's folder or the first one above it with a `[workspace]` table. Globs such as `crates/*` expand to the folders that hold a `Cargo.toml`, and the workspace's `exclude` takes folders out of the globs, while a member written out in full always counts. A root `Cargo.toml` that is also a package counts as one of the members.
- **`crates = [...]`** lists the crates, by path or by glob.

`follow` then adds the path dependencies of the chosen crates, again and again, but only those in folders that match one of its globs, measured from the workspace root (or else the first crate). The first run writes `follow = ["**"]`, so every path dependency below the root is outlined too, as context. `exclude` leaves crates out of whatever the run selected, whether by `workspace`, by `crates`, or by `follow`. A glob matches a crate's folder, written with forward slashes on every platform, and is measured from the config's folder. An excluded crate is not in the run: it is not read, it writes no files, its path dependencies are not followed on its account, and a type name that points at it stays plain text, as for any crate outside the run. If exclusion leaves no crate, the run stops with the error of an empty run.

Three cases get a `note:` line on stderr, which is not a warning, and the exit code stays 0. A crate that `crates` names and `exclude` also matches is left out, with a line that says so, because that is probably a mistake. A glob that matches no crate gets `note: exclude pattern <glob> matches no crate`, so a typo cannot quietly keep the build tools in; a run of named crates leaves this note out, since it never looks at the rest. And a crate in the run with nothing to outline, such as one with only tests, is skipped with a line that says why. A single named crate with nothing to outline is still an error. Two crates that would write the same file name stop the run with an error that names both.

In the HTML file, a type, trait, or re-export that resolves into another crate of the run links to that crate's page, as `runvil_model_client.html#type.runvil_model_client.model.ModelSet`, so all HTML files of a run sit in one folder. A re-exported function, constant, or static links the same way, however many crates re-export it on the way, as `runvil_engine.html#fn.runvil_engine.execute.perform_vfs_op`. vibelight finds the other crate through the `[dependencies]` and `[build-dependencies]` of `Cargo.toml`, including `workspace = true` entries, `package = "..."` renames, and `[lib] name`, but only path dependencies. A crate can name only its own dependencies, and a type whose crate is not in the run stays plain text. In the markdown, an `external` record names each item of another crate that the declarations name, with how often and by what, and its doc summary when that crate is in the run. No file depends on the order the crates are named in.

The markdown is contextual to the run: its leads and facts count what every target and package of the run writes, so adding or excluding a crate can change another crate's markdown, and with it that file's stamp. A cross-crate lead prints in the markdown of every target it involves. An HTML page depends only on its own crate and the crates of the run it links to.

## vibelight.toml

The config has seven keys, and every path in it is relative to the folder that holds it, so the run is the same from any folder below it:

- `workspace = true` outlines every member of the workspace that holds the config's folder. `workspace = "<dir>"` starts the search from that directory instead.
- `crates = ["<path or glob>", ...]` lists the crates instead. A path names one crate, the same as on the command line. A glob such as `crates/*` adds each matching folder that holds a `Cargo.toml`, and skips one with nothing to outline with a `note:` line. Set `workspace` or `crates`, not both.
- `follow = ["<glob>", ...]` adds the path dependencies whose folders match.
- `exclude = ["<glob>", ...]` leaves crates out of the workspace, the `crates` list, and the followed dependencies alike.
- `out = "<dir>"` is the folder to write into. It defaults to `.vibelight` beside the config.
- `boilerplate = ["<text>", ...]` marks text that is meant to repeat, such as a sentence every crate doc ends with. Every doc detector leaves unread a doc passage, the lines up to a blank line, that holds one of the texts, with runs of white space compared as one space.
- `quiet = ["<lead-id>", "<lead-id>:<path-glob>", ...]` drops leads from the markdown. An id alone drops every lead of that id, and a glob after `:`, written with `::` between segments such as `forwarder:detail::*`, drops a lead of that id when it matches the path of one of its subjects below its crate root. The header of each markdown file counts the leads it dropped in `quieted`, and an entry that drops no lead is a `note:` line.

A key vibelight does not know, or a value of the wrong type, is an error that names the key, such as ``unknown key `markdown` ``. Without `workspace` or `crates`, a run that names no crate is a usage error.

## The folder

The markdown files and `index.md` go in the folder, and the HTML files in its `html` folder, with `html/index.html` as the page that links to every other one. When the folder is vibelight's own, the config's or the default `.vibelight`, rather than one `-o` names, the run also keeps a `.gitignore` in it that holds `*.md` and `*.html`, so no outline is ever committed. The outlines carry source links and follow how the code is split across files, so they change whenever code moves; they are for reading, not for diffs.

Every file starts with a stamp, the UTC time its content last changed, such as `*Updated 2026-01-01T00:00:00Z*` in markdown and an `<!-- Updated ... -->` comment and a line at the top of the page in HTML. A run rewrites a file only when its content changed, so the stamp moves only with the content. The time comes from `SOURCE_DATE_EPOCH` when it is set, for reproducible builds, and otherwise from the clock; a value that is not a whole number of seconds stops the run before it writes anything.

The folder holds what the last run wrote. After writing, a run removes each outline that no crate of the run writes any more, such as the files of a crate that was renamed, removed, or excluded, with a `note:` line for each. Only files that start the way every outline does count: in the folder, a `.md` file with the stamp line and then the `vibelight-outline` header record, or an outline of the older markdown format, whose crate anchor opens the file or follows the stamp and a blank line; in `html`, an `.html` file with the stamp comment and then the doctype. So a README beside them, other file types, and the index are left alone. A run that names one crate leaves only that crate's files, so use `-o` for a one-off outline of a crate the config leaves out.

`index.md` lists each outline of the run, with its crate's folder below the root, whether the run selected the crate or followed a path dependency to it, and its size in KB:

```
- [runvil_engine.md](runvil_engine.md): `crates/runvil-internal/engine`, selected, 436 KB
- [gamma.md](gamma.md): `vendor/gamma`, followed, 4 KB
```

## Colors

The HTML file colors code the way an editor does. A link inside the page uses the link color, whether it goes to a type or to a function, constant, or static, and a link to another crate's page is yellow: a darker gold on the light theme and a lighter yellow on the dark theme, so it stays readable on both. A member name links to the line that declares it.

## What the output looks like

Each HTML page opens with the stamp, the target's name, its crate docs, and a short **Legend** of its notation (see [The HTML notation](#the-html-notation)). It then has these parts:

- **Contents:** links to every section and declaration.
- **Kill list:** a count of macros by kind, followed by one entry per macro, linking to the declaration or module that holds it.
- **Public surface:** for a library, every path a user of the crate can name, walking public modules and `pub use` re-exports, globs and `#[macro_export]` macros included. Each path links to what it names, a re-export of another crate's item shows the path it comes from, and test-support code is marked `(test support)`.
- **The types, in dependency order.** The types nothing else names are the roots, and they go by how many types they reach. Each root is followed by the types it owns: a type that only one other type names follows it directly, and a type that several name follows the last of them, so everything that names a type is above it. A type's parts go in the order of its fields (or variants, alias target, or supertraits) as declared, and then the types its methods name. Types that name each other in a cycle are placed together. A type that only a free function, a constant, a static, a re-export, or a function body outside its own impls names is in use, so it is a root. The HTML groups the types: a run of types in one module, judged by the first two segments of the module path, is a group with a rule above it and a heading named for the module and for the group's lead type, the first type that belongs to that module, and a type alone in its module joins the group before it. A heading that would still repeat takes ` (2)`, ` (3)`, and so on. **Test support** holds types behind a `cfg` that lists `test` inside `any(...)`, such as `cfg(any(test, feature = "test-support"))`, kept apart from the production types, and **Unconnected types** holds types that name nothing and that nothing names. Each has its own heading and rule after the groups.
- **Other impls:** impls with no declared type to attach to, such as blanket impls or impls for `&T` or `Box<T>`, as `implements: Trait for Type;` lines.
- **Modules:** one heading per module, with its docs, links to its declarations, and its re-exports, macros, constants, statics, free functions, and extern blocks.
- **Spellings:** each item from outside the crate that the source writes in more than one way, with every spelling (see [Spellings](#spellings)).

A section shows up only when it has something in it. Only the HTML file links type names.

The markdown has no sections. After the stamp, every line is one record, in this order:

- **Header:** a `vibelight-outline` record with the schema version, the crate's name, the kind of target (`lib`, `bin`, or `single-file`), the crate's folder below the base of the run, `root`, the crate's folder from the markdown folder, and `quieted`, how many leads the config's `quiet` entries dropped from the file, when there are any.
- **Modules:** a `modules` record with each module's range of lines in the file, so a reader can open one module at a time.
- **Leads:** one `lead` record per place that looks like slop, with its `id`, its `subjects`, its `evidence`, its `locs`, and a `mode` that says whether the evidence is resolved, direct syntax, or lexical. A lead that rests on something vibelight did not see also lists its `limits`, the blind spots that could make it wrong. More than 5 leads of one id in one module become one summary record with a `module` and a `count` that names every site.
- **Surface:** for a library, one `surface` record per public path, with its kind and, when the path is a re-export, the item it names.
- **External:** one `external` record per item of another crate that the declarations name, with how often and by what, and its doc summary when that crate is in the run.
- **Spellings:** one `spelling` record per outside item written more than one way (see [Spellings](#spellings)).
- **Wire keys:** one `wire-keys` record per `serde` type, with the keys it reads or writes after renames and aliases.
- **The catalog:** every declaration, module by module in path order. A module's own `mod` record comes first, then its `use` records for re-exports, then every other record in source order. The other kinds are `struct`, `enum`, `union`, `trait`, `alias`, `fn`, `const`, `static`, `extern`, `impl`, `macro`, and `unsupported`, plus a `field` or `variant` record for each field or variant with a doc or comment of its own.
- **Docs:** a `doc` record right after the record it documents, with the same `path`.

Catalog records carry the facts vibelight observes across the run, each left out when there is nothing to say: `named_by`, the declarations and modules whose signatures name a type; `reexported_by`, the modules that re-export it; `crates`, the other targets of the run whose code names it by path; `tests`, the kinds of test, bench, example, build, and doc test code that name it by path; `direct_callers`, the bodies whose direct calls by path resolve to a function; `body`, counts of a body's statements, tokens, loops, error paths, panics, and macros, with no code; `ops`, the fields and parameters a body uses and the methods it calls on them; and `comments`, the comments inside a body. A missing fact is never proof: vibelight sees no macro expansion, generated code, dynamic dispatch, or code outside the run. Method-call syntax such as `section.level()` never fills `crates`, `tests`, or `direct_callers`, and the absence leads see it only as a bare name hit shared by every method of that name. The lead ids, such as `same-shape`, `direct-field-clusters`, `direct-impl-field-split`, and `no-observed-name-use`, are defined in the lead glossary of [`tools/vibelight.md`](tools/vibelight.md).

### Links to functions, constants, and statics

In the HTML file, each module-level function, constant, and static has an id on its name, `fn.<crate>.<module path>.<name>`, `const.<crate>.<module path>.<NAME>`, or `static.<crate>.<module path>.<NAME>`, written like the ids of types: parts joined with dots, case kept, and `r#` removed. A repeated id, such as a function declared twice under different `cfg`s, gets `-2`, `-3`, and so on. Methods and the items of `extern` blocks have no ids, and the markdown file has none of these ids.

A re-export line (`using ...;`) links to a function, constant, or static of this crate, or of another crate of the run, through renames and chains of `pub use` items. A backticked name or an intra-doc link in a comment links the same way, when the name is in scope through a `use`, a glob, a path, or an item of the same module. When a type and a function have the same name, the link goes to the type. A function name is never found just because it is the only one of that name in the crate, which is how types are found, so a comment that names a function of another module needs a path or a `use`.

### The markdown notation

The markdown is JSON Lines: after the stamp, every line is one compact JSON object, with no blank lines, indentation, or wrapping, so a model can read it a line at a time and a script can parse it. `k` names the kind of record, and `path` names its owner, an item path below the crate root, such as `contract::args::ArgDecl`, with `crate` for the root itself. The header's `schema` is 1.

A type's whole shape is one `rust` string, such as `pub struct Point { pub x: i32, pub y: i32, label: String }`, and a trait's `rust` is its header, with each trait item a record of its own. Each function's signature is one `sig` string with its `where` clause inline, and a trait impl's function also has `impl`, the impl header. An `impl` record is the impl header, with the `locs` of every block that writes it. A constant or static has a `value` only when its initializer is one literal. `attrs` lists the attributes that shape an API or its wire format, `derives` the derived traits, and `attributes` the attribute macros, with `cfgs` and `lints` beside them and `test_support` on test-support code. Plain comments above a declaration are in its `comment` field, comments with nothing after them in a module, type, or impl body are in `dangling`, and parameter comments are in `param_docs`.

A doc is a `doc` record of its own, never a field, because some tools cut long lines short. Its text is the doc as written, with its line breaks as `\n` and its fenced examples kept exactly. Every location is one string, `<file>:<line>`, with the file relative to the crate and written with forward slashes, such as `src/lib.rs:6`; `lead` and `impl` records have a `locs` array of them. A subject in another target of the run is written `::<stem>::<path>`, such as `::facade::Panel`, and a location there is `<stem>:<file>:<line>`, such as `facade:src/lib.rs:9`. A file never holds a colon, so a reader splits a location from the right and opens the file below the `root` of the outline of the target that holds it.

Since some tools cut long lines short, no line is longer than 900 characters. A record that would be keeps the fields that name it and as much of its widest field as fits, and `more` records right after it carry the rest. Each names its owner by `path`, or a lead by its first subject and its `id`, names the `field` it continues, such as `locs` or `spellings.3.locs` for one element's field, counts its `part` from 2, and holds either `items`, whole elements to add to that array, or `text`, a piece to add to that string. The `modules` ranges count these lines too.

Here are the first four lines of the outline of a small test crate, then a struct, its impl, and a method from further down:

```text
*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"links","target":"lib","root":".."}
{"k":"modules","items":[{"path":"crate","from":15,"to":29},{"path":"parts","from":30,"to":35}]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["parts::Gauge"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/parts.rs:12"],"limits":["macros"]}
```

```text
{"k":"struct","path":"Engine","rust":"pub struct Engine { pub speed: u32 }","loc":"src/lib.rs:6"}
{"k":"doc","path":"Engine","text":"A type with an impl in another file."}
{"k":"impl","path":"Engine","rust":"impl Engine","locs":["src/lib.rs:10"]}
{"k":"fn","path":"Engine::start","sig":"pub fn start(&self)","loc":"src/lib.rs:12","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::start","text":"Starts the engine."}
```

The `modules` record says lines 15 to 29 hold the root module and lines 30 to 35 hold `parts`, counting the stamp as line 1. The outline sits in `out` inside the crate, so its `root` is `..`, and `src/lib.rs:6` opens as `../src/lib.rs` at line 6.

### The HTML notation

Every type prints as one C++-style block, with the members of all its inherent impls merged in, wherever they are written. Here is a type whose fields and methods are all private:

```
/// Holds the text the lexer is still reading.
class Cursor
{
    pos: usize,

    text: String,

    static new(text: String) -> Cursor

    bump() mut

    into_text(self) -> String

    peek() -> Option<char>
}
```

Since nothing in it is public, it prints as a `class` with no label. A type with a public member prints as a `struct`. The data comes first, a type's fields or an enum's variants, and the other members follow, because the data is what a reviewer weighs first: should these fields be here, and are these the right types? Each of the two parts sorts its members into three sections, in this order: public, then `internal:` for `pub(crate)`, `pub(super)`, and `pub(in path)`, which an internal crate uses for almost everything, then `private:`. The keyword sets the access at the top of the body, private for a `class` and public for a `struct`, and a label prints only where the access changes from the one in effect, as in a C++ header. So a visibility that holds both fields and methods prints its label twice, once for each part:

```
struct Run
{
internal:
    id: RunId,

private:
    cache: HashMap<Key, Value>,

    name: String,

public:
    static new() -> Run

internal:
    advance() mut
}
```

A type's own visibility does not print above its header. A module's trailing block marks each item that is not public with a `// internal` or `// private` line above it.

The `fn` keyword and the bodies are gone. A method that takes `&self` gets no marker, a method that takes `&mut self` gets `mut` after its parameter list, a method that takes `self` in any other way keeps it as its first parameter, and only a function with no receiver is `static`. A bare `&` is read-only, as in Rust, and `mut` marks what can change, so `&str` prints as `str&` and `&mut T` as `T mut&`. The data keeps its declared order. The other members are grouped (associated types, constants, special members, static functions, then member functions) and sorted by name inside each group.

Comments stay with what they belong to. Each impl block's comments sit under an `// impl Trait` or `// impl Type` line in the type's block, a comment above a `pub use` group prints above the whole group, and a comment inside the braces stays with the name after it. A declaration's doc comment prints as prose, in the page's own typeface rather than the code font: consecutive lines join one paragraph, a blank line starts a new one, a `- ` or `* ` line starts a bullet list, a `# Errors` line becomes a small heading, and a fenced example stays exactly as written in the code font. Text in `backticks` prints in the code font wherever it appears, so a name reads as a name inside a sentence. An ordinary comment beside a member keeps the fixed layout of its block, so its alignment still shows, and sits in a box with no fill and no outline, so only its text shows, lined up with the member's own text.

One button in the sidebar hides every one of those member comments at once, and shows them again. It is a checkbox and a label, so it needs no script, it works with the keyboard, and the page still makes no network request. The comments are shown when the page opens. A comment on the same line as code, such as `field: u8, // note`, stays visible, because removing it would change that line. Attributes that shape the API or its wire format print as `[[...]]` at half contrast. A function or a field closes its line with them, such as `[[must_use]]`, `[[track_caller]]`, and `[[serde(rename = "id")]]`, and anything else gets them on lines above it, such as `[[non_exhaustive]]`, `[[repr(C)]]`, and `[[deprecated]]` on a variant. Other attributes stay hidden, and a signature has no semicolon. Each inherent impl's link in a `Source:` line is followed by the names of the members it adds.

The page opens with a short Legend of this notation, so a reader of the page alone can follow it: `class` and `struct`, `static` and the `() mut` marker, the base list, the access labels and the data-first order of a type's body, `[[...]]`, `implements:`, `MACRO`, and how comments print.

Traits a type implements or derives print the C++ way. Only the C++ special member functions become members: `Default` is the default constructor, `From<T>` a converting constructor, and `Drop` the destructor, packed together after the fields, with `= default` when derived. Every other standard trait, `Clone` and the comparison traits included, goes in the base list by its short name, as in `struct Seed : Clone, Debug, Display`, and every other trait follows on its own line by its full path. A name counts as standard only when it really resolves to `std`, `core`, or `alloc`, so another crate's `Display`, the crate's own `Display`, or a `Display` that only comes through a glob import gets its own line. A derive from another crate keeps its bold **MACRO** line above the block.

## Spellings

The HTML's Spellings section lists each item from outside the crate that the declarations write in more than one way, such as `fmt::Debug` in one place and `std::fmt::Debug` in another, so you can pick one and change the rest. It covers `std`, `core`, `alloc`, and other crates, and counts the names in supertraits, bounds, `where` clauses, impl headers, and field, parameter, return, and generic argument types. A bare prelude name such as `Option` counts as `std::option::Option`, a `core::` or `alloc::` path counts as the `std` path that re-exports it, and a `use ... as` rename is one more spelling. Each entry gives every spelling with how often it is written. For a standard item, it also names the spelling to prefer: the bare name, as a base list prints it, such as `Debug` or `Cow`. That spelling shows only its count, and every other spelling also gets a link to each line:

```
- `std::fmt::Debug` is written 3 ways; prefer `Debug`:
  - `Debug`, 12 times
  - `fmt::Debug`, 2 times (source: [src/lib.rs:4](src/lib.rs#L4), [src/lib.rs:9](src/lib.rs#L9))
  - `std::fmt::Debug`, once (source: [src/lib.rs:12](src/lib.rs#L12))
```

When the bare name is not written anywhere yet, the entry says so. An item has no preferred spelling when the crate declares a type or trait with that bare name, when the crate uses another standard item with the same bare name (`fmt::Write` and `io::Write`), or when it comes from another crate. Its entry then says to pick one spelling and search for the rest.

Names of the crate's own items are left out, because `crate::model::X`, `X`, and `super::X` are all normal in different modules. So are names in comments, re-exports, `use` lines, and function bodies. Derive names are left out too, and so are those inside `cfg_attr`. A derive is written bare in practice, so it never matches a path in an impl header, a bound, or a type, and an entry that counted it would give you nothing to fix. A type that derives `Debug` next to an impl that writes `fmt::Debug` is not an entry. Glob imports such as `use std::fmt::*` are not followed, so a name that only comes through one is not counted.

The markdown has a `spelling` record for each of these items, with its preferred spelling and each spelling's count and locations, but it leaves out a bare spelling of a name the crate itself declares, since the crate needs the path wherever its own item of that name is in scope.

## Warnings and exit codes

When vibelight meets something it cannot fully handle, it keeps going and prints a warning on stderr, as `warning: <file>:<line>: <message>`, then a count at the end. Warnings cover files that do not parse, module files that cannot be found, constructs it does not support (also marked UNSUPPORTED in the outline), and type names it cannot resolve or that are ambiguous, which stay plain text instead of links. When the markdown has spelling records, one more line, `warning: <count> items written more than one way; see spelling records`, counts the items those records name. In a run of several crates, a warning's path starts from the workspace root (or else the first crate), such as `crates/alpha/src/lib.rs:9`.

A `note:` line is not a warning. Notes say that the run wrote `vibelight.toml` or the folder's `.gitignore`, that it skipped a crate with nothing to outline, that a crate the config lists is also excluded, that an `exclude` glob matches no crate, that a `quiet` entry matches no lead, or that it removed an outline no crate writes any more.

- `0`: the files were written, warnings included.
- `1`: the input cannot be used, for example the path is not a crate, the manifest is workspace-only, a root file cannot be read, `vibelight.toml` has an unknown key, or `SOURCE_DATE_EPOCH` is not a whole number of seconds. The reason is printed on stderr.
- `2`: a usage error, such as an unknown flag, no crate named and no Rust repository to write a config for, or a config that names no crates, reported by clap.

## Requirements and license

- Minimum Rust version: 1.85
- License: BSL-1.0 (Boost Software License 1.0), in `LICENSE`
