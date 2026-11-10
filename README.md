# vibelight

[![CI](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/cppalliance/vibelight/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/vibelight)](https://crates.io/crates/vibelight) [![Latest release](https://img.shields.io/github/v/release/cppalliance/vibelight)](https://github.com/cppalliance/vibelight/releases/latest) [![License](https://img.shields.io/github/license/cppalliance/vibelight)](LICENSE) [![Minimum Rust version](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

vibelight reads the Rust crates of a repository and writes an outline of each: every type, trait, function, and module with its comments, and no function bodies. Each target gets two files for two readers. The markdown file is for a model analyzing the code: one JSON record per line, starting with the leads vibelight computes for places that look like slop, then every declaration with its shape or signature as one line of Rust and everything vibelight observed about it, such as what names and uses it, which bodies call it, counts from each function body, the items of other crates, and the file and line of every declaration. The HTML is for a person auditing the code: it makes a program's structure legible, so the reader does not have to wade through its implementation to see how it is put together. It prints the declarations in the style of a C++ header, with the bodies removed and every type name linked, and it opens on an overview that tells the facades, the crates whose API users write against, from the implementation crates behind them.

## What it does

- **No switches.** Run `vibelight` at the root of a repository. The first run writes a `vibelight.toml` that chooses the crates worth analyzing and leaves out the ones that only support development, each with a comment that says why. Every later run reads it.
- **A folder git never sees.** The markdown goes in `.vibelight/`, the HTML in `.vibelight/html/`, and a `.gitignore` there keeps all of it out of git. The folder holds what the last run wrote: the outlines of a crate the run no longer covers are removed.
- **An index.** `.vibelight/index.md` lists each outline with its crate's folder, whether the run selected the crate or followed a path dependency to it, and its size. `html/index.html` opens on an overview of the same crates: the facades first, then the implementation crates behind each one, then the rest, each with a line from its crate docs.
- **One outline per target.** The library, each binary, or a single `.rs` file gets `<target>.md` and a page of the HTML, which `html/index.html` holds.
- **Facts for analysis.** The markdown gives each declaration the facts vibelight observes across the run: what names and re-exports it, which other crates and which kinds of test code use it, which bodies call a function, and for each body, counts of its statements, loops, and error paths, the fields and parameters it touches, and its comments. Before the declarations come the leads: places that look like slop, each with an id, its evidence, the kind of evidence it rests on, and its known blind spots.
- **One HTML file that works from disk, in Cursor's browser too.** `html/index.html` is the whole HTML, and it needs no server and no other file. Cursor's browser drops every click on a link to a `file:` URL, so the file is a small app that never leaves it. Every crate's page and the search index sit inside it compressed, so the file stays small, and the app unpacks a page only when you open it and shows it in a tab of its own. The address names what is shown, as `index.html#<target>/<id>`, so Back, Forward, and a reload work, and the tabs come back after a reload. `Ctrl K` or `/` searches every declaration of the run; a click with `Ctrl`, or a middle click, opens a link in a new tab; and a click on a source link copies its file and line, such as `crates/engine/src/run.rs:42`, since a browser inside an editor opens no source file. The file has inline CSS and makes no network requests. Every type name that refers to a declaration of the run links to it, and so does a re-exported function, constant, or static. The colors follow your system's light or dark setting.
- **Module by module, with visibility.** A crate's page has one section per module in path order: its path, a line of what it declares and at which visibility, then its types and items twice, first `At a glance`, without comments, then `Documented`, each with its docs. Inside a module, types are ordered by how they name each other: a type follows the type that owns it, so every type comes after the types that use it. Each type prints as one C++-style block with its data members first and the members of all its impls merged in, every member under a label for its visibility as written, such as `pub:` and `pub(crate):`, and a dim `reaches pub(crate)` line marks a node that reaches less far than its visibility says. The markdown also goes module by module in path order, with each module's records in source order.
- **Facades and their families.** A facade is a crate whose public surface is mostly re-exports of other crates of the run: the API a family of crates offers its users. vibelight finds the facades from their re-exports, or takes them from the config's `facades` key, and puts each other library behind the facade whose code reaches it. A facade's page shows only what its users can name: each item under the path they write, pulled from the crate that defines it, with its `pub` members alone, so the whole API reads in one place. The overview warns when a crate outside a family uses one of the family's implementation crates directly, past its facade.
- **Comments kept.** Doc comments and ordinary comments stay with the item they belong to. In the HTML a doc comment prints as prose, and a member's comment reflows above it as one paragraph; the `At a glance` view leaves them all out. The markdown also keeps the comments inside each function body, with their lines.
- **Every macro marked.** In the HTML, macro definitions, invocations, derives from other crates, and attribute macros each get a bold **MACRO** label, and the page lists them all at the top, each with a link to where it sits. The markdown has a `macro` record for each definition and invocation, and lists derives and attribute macros on the record they apply to.
- **Macros looked through.** Each item-position invocation of one of the crate's own `macro_rules!` macros is expanded, and the items it writes are outlined like any other, in the invoking module and at the invocation's line. The markdown marks each such top-level record with `via`, such as `"via":"events!"`, and the HTML's `Source:` line says "written by `events!`". An invocation that cannot be expanded stays a mark and warns; other macros are not expanded.
- **The public surface first.** A library's files list every path a user of the crate can name, through public modules and re-exports, before any type: a Public surface section in the HTML, and one `surface` record per path in the markdown. A facade's page is its public surface, each path with the declaration it names.
- **Rust for the model.** The markdown writes each signature and each type's shape as one line of Rust, with visibilities, generics, and `where` clauses as in the source. The leads come first, every record names its kind in `k`, and the header gives the schema version, so a model with only the file can follow it.
- **Links back to the source.** In the HTML, each declaration, impl block, module, and macro links back to where it is written, such as `src/prompt.rs:136`. The links are relative to the output file and end in `#L136`, which GitHub, VS Code, and Cursor open at that line. An item that a macro writes links to the invocation. In the markdown, each declaration's `loc`, or each lead's and impl's `locs`, is a string such as `src/prompt.rs:136`, and the header's `root` leads from the markdown folder to the crate, so `root` and the file together open the source.

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

Every install puts `cargo-vibelight` beside the `vibelight` binary, so `cargo vibelight` runs the same command with the same arguments, and no `.cargo/config.toml` alias is needed in any repository:

```
cargo vibelight ../my-crate -o outline
```

Both binary names come from one `cargo install vibelight`, because they are two targets of one package. `cargo-vibelight` runs the `vibelight` binary in its own folder and keeps its exit code, so the two always agree on every flag, and a `vibelight` moved away from it stops with a message that names the missing file.

- With no arguments, vibelight reads the `vibelight.toml` in the current directory or the first parent that holds one, the way Cargo finds its files, and outlines the crates it chooses. When there is none, it writes one first (see [The first run](#the-first-run)).
- `<crate-dir-or-file.rs>` is a crate directory, with or without a `Cargo.toml`, the `Cargo.toml` itself, or a single `.rs` file, in place of the crates the config chooses. Several crates can be named at once. A crate named here is always outlined, even when the config's `exclude` matches it, and the config's `follow` and `exclude` still apply to the crates the run follows into. A single `.rs` file cannot be mixed with crates, and the config's globs pass it by.
- `-o` (or `--out`) is the folder to write into, in place of the config's. The markdown files and the index go in it, and the HTML files in its `html` folder. It is created if missing.

A file whose content is the same as the one on disk is left alone; any other file of the same name is overwritten. vibelight prints the path of every file of the run on stdout, sorted, whether it wrote the file or the file was already current. For example, `vibelight ../my-crate -o outline` on a library crate named `my-crate` writes `outline/my_crate.md` and `outline/index.md`, and in `outline/html` the one file `index.html`, which holds the overview, the crate's page, the search index, and the app.

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
- a member whose name or folder starts with the word `build` (names split at `-` and `_`, so `build-ui` counts and `builder` does not), or has `xtask` or `xtasks` as a word of its name or path, as build tooling;
- a member whose name, or a folder on its path, holds the word `fuzz`, `bench`, `example`, `test`, or `fixtures`, or one of their plurals and kin, reading names as words split at `-` and `_`;
- a member that other members depend on only as a build dependency or only as a dev dependency.

A repository that is one crate gets `crates = ["."]` in place of `workspace = true`. The file is yours to edit, and vibelight never rewrites it. With no `Cargo.toml` at or above the current folder, the run stops with a usage error.

## Targets

In a crate directory, vibelight reads `Cargo.toml` for the package name and any `[lib]` and `[[bin]]` entries, and finds `src/lib.rs`, `src/main.rs`, `src/bin/*.rs`, and `src/bin/*/main.rs` the way Cargo does. Each target gets its own pair of files:

- The library is named after the library target, with hyphens turned into underscores, so package `my-crate` gives `my_crate.md`.
- Each binary keeps its name as written. A binary with the same name as the library gets a `.bin` stem, so `foo.bin.md` sits beside `foo.md`.
- Without a manifest, the library and `src/main.rs` take the directory's name, each `src/bin/*.rs` takes its file stem, and each `src/bin/*/main.rs` takes its folder name.
- A single `.rs` file is named after its file stem.

Tests, benches, examples, `build.rs`, and doc tests are not outlined and get no records, but the markdown's census reads them: their qualified paths and the names they write count toward the facts and leads of the code they use. Code behind a plain `cfg(test)` is never in the catalog either. A root `Cargo.toml` with only a `[workspace]` table is not a crate, so point vibelight at a member crate's directory, or let the config choose the members.

## Several crates in one run

A run covers several crates, and every crate in it gets its files. The config chooses them in one of two ways:

- **`workspace = true`** reads the `[workspace] members` of the root `Cargo.toml`, found at the config's folder or the first one above it with a `[workspace]` table. Globs such as `crates/*` expand to the folders that hold a `Cargo.toml`, and the workspace's `exclude` takes folders out of the globs, while a member written out in full always counts. A root `Cargo.toml` that is also a package counts as one of the members.
- **`crates = [...]`** lists the crates, by path or by glob.

`follow` then adds the path dependencies of the chosen crates, again and again, but only those in folders that match one of its globs, measured from the workspace root (or else the first crate). The first run writes `follow = ["**"]`, so every path dependency below the root is outlined too, as context. `exclude` leaves crates out of whatever the run selected, whether by `workspace`, by `crates`, or by `follow`. A glob matches a crate's folder, written with forward slashes on every platform, and is measured from the config's folder. An excluded crate is not in the run: it is not read, it writes no files, its path dependencies are not followed on its account, and a type name that points at it stays plain text, as for any crate outside the run. If exclusion leaves no crate, the run stops with the error of an empty run.

Three cases get a `note:` line on stderr, which is not a warning, and the exit code stays 0. A crate that `crates` names and `exclude` also matches is left out, with a line that says so, because that is probably a mistake. A glob that matches no crate gets `note: exclude pattern <glob> matches no crate`, so a typo cannot quietly keep the build tools in; a run of named crates leaves this note out, since it never looks at the rest. And a crate in the run with nothing to outline, such as one with only tests, is skipped with a line that says why. A single named crate with nothing to outline is still an error. Two crates that would write the same file name stop the run with an error that names both.

In the HTML file, a type, trait, or re-export that resolves into another crate of the run links to that crate's page, as `runvil_model_client.html#type.runvil_model_client.model.ModelSet`, so all HTML files of a run sit in one folder. A re-exported function, constant, or static links the same way, however many crates re-export it on the way, as `runvil_engine.html#fn.runvil_engine.execute.perform_vfs_op`. vibelight finds the other crate through the `[dependencies]` and `[build-dependencies]` of `Cargo.toml`, the `[build_dependencies]` spelling Cargo still reads, and the same tables under every `[target.*]`, including `workspace = true` entries, `package = "..."` renames, and `[lib] name`, but only path dependencies. The dev tables, `[dev-dependencies]` and `[dev_dependencies]`, are never followed. A crate can name only its own dependencies, and a type whose crate is not in the run stays plain text. In the markdown, an `external` record names each item of another crate that the declarations name, with how often and by what, and its doc summary when that crate is in the run. No file depends on the order the crates are named in.

The markdown is contextual to the run: its leads and facts count what every target and package of the run writes, so adding or excluding a crate can change another crate's markdown, and with it that file's stamp. A cross-crate lead prints in the markdown of every target it involves. An HTML page depends only on its own crate and the crates of the run it links to, and a facade's page also on the crates whose items it shows.

## vibelight.toml

The config has eight keys, and every path in it is relative to the folder that holds it, so the run is the same from any folder below it:

- `workspace = true` outlines every member of the workspace that holds the config's folder. `workspace = "<dir>"` starts the search from that directory instead.
- `crates = ["<path or glob>", ...]` lists the crates instead. A path names one crate, the same as on the command line. A glob such as `crates/*` adds each matching folder that holds a `Cargo.toml`, and skips one with nothing to outline with a `note:` line. Set `workspace` or `crates`, not both.
- `follow = ["<glob>", ...]` adds the path dependencies whose folders match.
- `exclude = ["<glob>", ...]` leaves crates out of the workspace, the `crates` list, and the followed dependencies alike.
- `out = "<dir>"` is the folder to write into. It defaults to `.vibelight` beside the config.
- `boilerplate = ["<text>", ...]` marks text that is meant to repeat, such as a sentence every crate doc ends with. Every doc detector leaves unread a doc passage, the lines up to a blank line, that holds one of the texts, with runs of white space compared as one space.
- `quiet = ["<lead-id>", "<lead-id>:<path-glob>", ...]` drops leads from the markdown. An id alone drops every lead of that id, and a glob after `:`, written with `::` between segments such as `forwarder:detail::*`, drops a lead of that id when it matches the path of one of its subjects below its crate root. The header of each markdown file counts the leads it dropped in `quieted`, and an entry that drops no lead is a `note:` line.
- `facades = ["<glob>", ...]` names the facades instead of letting vibelight find them: a library whose folder matches one of the globs is a facade, and no other crate is one. The other libraries still go behind the facades whose code reaches them. Without the key, a library is a facade when at least half of its public surface re-exports other crates of the run, unless another facade reaches it, all its users are in that facade's family, and it re-exports a smaller share than that facade does.

A key vibelight does not know, or a value of the wrong type, is an error that names the key, such as ``unknown key `markdown` ``. Without `workspace` or `crates`, a run that names no crate is a usage error.

## The folder

The markdown files and `index.md` go in the folder, and the HTML in its `html` folder, as the one file `html/index.html`. When the folder is vibelight's own, the config's or the default `.vibelight`, rather than one `-o` names, the run also keeps a `.gitignore` in it that holds `*.md`, `*.html`, and `*.js`, so no outline is ever committed. A `.gitignore` that holds only the first two lines, as earlier releases wrote it, is replaced with a note; any other `.gitignore` is left alone. The outlines carry source links and follow how the code is split across files, so they change whenever code moves; they are for reading, not for diffs.

Every outline and every HTML page starts with a stamp, the UTC time its content last changed, such as `*Updated 2026-01-01T00:00:00Z*` in markdown, an `<!-- Updated ... -->` comment that opens `html/index.html`, and a line at the top of each page's content. `index.md`, the `.gitignore`, the search index, and the app have none. A run rewrites a file only when its content changed, so the stamp moves only with the content. Inside `html/index.html` each page keeps its own stamp, so a page that did not change keeps its stamp when another page did, and the comment that opens the file moves with any change to it. The time comes from `SOURCE_DATE_EPOCH` when it is set, for reproducible builds, and otherwise from the clock; a value that is not a whole number of seconds stops the run before it writes anything.

The folder holds what the last run wrote. After writing, a run removes each outline that no crate of the run writes any more, such as the files of a crate that was renamed, removed, or excluded, with a `note:` line for each. Only files that start the way every outline does count: in the folder, a `.md` file with the stamp line and then the `vibelight-outline` header record, or an outline of the older markdown format, whose crate anchor opens the file or follows the stamp and a blank line; in `html`, an `.html` file with the stamp comment and then the doctype, and the `.js` files that earlier releases wrote beside it: a page's data file, whose first line is a `vibelightPage(` call, the search index, whose first line is a `vibelightSearch(` call, and the app, whose first line starts with `// vibelight: `. So a README beside them, other file types, and the index are left alone. A run that names one crate leaves only that crate's files, so use `-o` for a one-off outline of a crate the config leaves out.

`index.md` lists each outline of the run, with its crate's folder below the root, whether the run selected the crate or followed a path dependency to it, and its size in KB:

```
- [runvil_engine.md](runvil_engine.md): `crates/runvil-internal/engine`, selected, 436 KB
- [gamma.md](gamma.md): `vendor/gamma`, followed, 4 KB
```

When the production code of one outline uses items of another, a `## Uses between outlines` section follows the list. It has one line for each outline that uses another or that another uses: the outlines it uses, the outlines that use it, each with the number of distinct items, and its instability, `fan_out / (fan_in + fan_out)`, where `fan_out` counts the outlines it uses and `fan_in` the outlines that use it. Doc links do not count. A use counts for the crate that declares the item when the using crate depends on it, else for the first of its dependencies in the list that re-exports the item by name, directly or through other re-exports, else for the crate that declares it. A glob, or a re-export of a whole crate or module, even under another name, does not re-export its items by name. The overview in the HTML has the same section, with each name linked to its page:

```
- `runvil_engine` uses `runvil_lua` (50 items), `runvil_types` (34 items); used by `runvil` (12 items); instability 0.67
```

## Colors

The HTML file colors code the way an editor does. A link inside the page uses the link color, whether it goes to a type or to a function, constant, or static, and a link to another crate's page is yellow: a darker gold on the light theme and a lighter yellow on the dark theme, so it stays readable on both. On a facade's page, a link to an item that the page shows stays on the page, in the link color. A member name links to the line that declares it.

## What the output looks like

The HTML opens on the overview. It has the run's name, a short description, and counts of its crates, then three tables:
- **Facades**, each with its folder, the first sentence of its crate docs, how many items and modules its API has, and the crates it re-exports from, with how many items from each.
- **Implementation crates**, grouped by the facade they sit behind, each with how many of its items that facade re-exports, and a warning when a crate outside the family uses it directly.
- **Other crates**, the libraries that no facade reaches and the binaries, grouped by folder.

The uses between outlines follow, and the **Legend** of the notation closes the page (see [The HTML notation](#the-html-notation)), printed once for every page.

A facade's page shows its API and nothing else. Its sidebar lists the modules of the API, each with how many items it holds, then, under a rule, every symbol of the API by name, a function's ending in `()` and a macro's in `!`, each linking to its heading; a name that more than one module holds shows its module, dim. After the crate docs, each module of the API has a section with its items `At a glance` and then `Documented`. A documented item has a heading with the path its users write, such as `runvil::vfs::Vfs`, and a dim `Defined in` line that names the item where its crate defines it, with that file and line. Its docs and its block follow, pulled from the crate that defines it. A whole module re-exported from another crate shows as its `using` line.

Every other crate's page opens with the stamp, the target's name, and its crate docs. It then has these parts:

- **Sidebar:** the Public surface, then the module tree: every module under its parent, each with its types, functions (shown with `()`), constants, and statics in path order, the names on the public surface in bold. Extern items and macros are not listed.
- **The `MACROS:` line:** at the top of the page, a count of macros by kind, followed by one entry per macro, linking to the declaration or module that holds it. The sidebar does not link it.
- **Public surface:** for a library, every path a user of the crate can name, walking public modules and `pub use` re-exports, globs and `#[macro_export]` macros included. Each path links to what it names, a function, constant, or static to its own name in its module's items, a re-export of another crate's item shows the path it comes from after the name, and links to the item on that crate's page when the crate is in the run and stays plain text when it is not, and test-support code is marked `(test support)`.
- **One section per module,** in path order. The heading shows the module's parent path dim before its name, such as `engine::` then `parse`. Then come its `Source:` line, its docs, and its anatomy line: the module's visibility, how far it reaches when that is less, what it declares, and the same items by visibility, such as `pub mod, reaches pub(crate): 1 type, 2 functions; pub(crate) 2, pub(super) 1`. A module that uses other modules of the target, or that they use, then has a coupling line, such as `Used by 1 module: vm (3). Uses 2 modules: error (4), messages (2). Instability 0.67.`, with each name linked to its module's section. Then come the module's types and its block of items.
- **The types of a module, in dependency order.** A type follows the type that owns it: a type that only one other type names follows it directly, and a type that several name follows the last of them, so everything that names a type is above it. The types nothing else names are the roots, and they go by how many types they reach. A type's parts go in the order of its fields (or variants, alias target, or supertraits) as declared, and then the types its methods name. Types that name each other in a cycle are placed together. A type that only a free function, a constant, a static, a re-export, or a function body outside its own impls names is in use, so it is a root. After a module's connected types come its types that name nothing and that nothing names, and then its test-support types, those behind a `cfg` that lists `test` inside `any(...)`, such as `cfg(any(test, feature = "test-support"))`. A type's heading shows its module's path dim before its name.
- **Other impls:** impls with no declared type to attach to, such as blanket impls or impls for `&T` or `Box<T>`, as `implements: Trait for Type` lines.
- **Spellings:** each item from outside the crate that the source writes in more than one way, with every spelling (see [Spellings](#spellings)).

A section shows up only when it has something in it. Only the HTML file links type names.

The markdown has no sections. After the stamp, every line is one record, in this order:

- **Header:** a `vibelight-outline` record with the schema version, the crate's name, the kind of target (`lib`, `bin`, or `single-file`), `dir`, the crate's folder below the base of the run (left out for the base itself), `root`, the crate's folder from the markdown folder, and `quieted`, how many leads the config's `quiet` entries dropped from the file, when there are any.
- **Modules:** a `modules` record that is the module tree: each module's `path`, its written `vis` (left out for the root and for no keyword), its `eff` when the modules around it let it reach less than written, and its range of lines in the file, so a reader can open one module at a time.
- **Leads:** one `lead` record per place that looks like slop, with its `id`, its `subjects`, its `evidence`, its `locs`, and a `mode` that says whether the evidence is resolved, direct syntax, or lexical. A lead that rests on something vibelight did not see also lists its `limits`, the blind spots that could make it wrong. More than 5 leads of one id in one module become one summary record with a `module` and a `count` that names every site.
- **Surface:** for a library, one `surface` record per public path, with its kind and, when the path is a re-export, the item it names.
- **Dependencies:** one `deps` record per module whose production code uses items of other modules of the target, with `uses`, each such module's `path` and `n`, how many of its items are used, sorted by path. A module and a module inside it never depend on each other.
- **External:** one `external` record per item of another crate that the declarations name, with how often and by what, and its doc summary when that crate is in the run.
- **Spellings:** one `spelling` record per outside item written more than one way (see [Spellings](#spellings)).
- **Wire keys:** one `wire-keys` record per `serde` type, with the keys it reads or writes after renames and aliases.
- **The catalog:** every declaration, module by module in path order. A module's own `mod` record comes first, then its `use` records for re-exports, then every other record in source order. The other kinds are `struct`, `enum`, `union`, `trait`, `alias`, `fn`, `const`, `static`, `extern`, `impl`, `macro`, and `unsupported`, plus a `field` or `variant` record for each field or variant with a doc or comment of its own. The `mod` record counts what the module declares, each count left out at 0: `types` (structs, enums, unions, traits, and aliases), `functions`, `values` (constants and statics), and `reexports`, with extern items counted by their kind. `mix` counts the same items by written visibility, with `pub(self)` for no keyword, such as `{"pub":2,"pub(crate)":3,"pub(self)":4}`. `fan_in` counts the other modules that use its items, and `fan_out` the other modules whose items it uses, as the `deps` records show them, each left out at 0.
- **Docs:** a `doc` record right after the record it documents, with the same `path`.

Catalog records carry the facts vibelight observes across the run, each left out when there is nothing to say: `eff`, how far a declaration or module item reaches when a re-export makes that further than its module's reach narrowed by its own visibility; `named_by`, the declarations and modules whose signatures name a type; `reexported_by`, the modules that re-export it; `crates`, the other targets of the run whose code names it by path; `tests`, the kinds of test, bench, example, build, and doc test code that name it by path; `direct_callers`, the bodies whose direct calls by path resolve to a function; `body`, what a body holds, with no code; `ops`, the fields and parameters a body uses and the methods it calls on them; and `comments`, the comments inside a body. A `body` record holds `statements` and `tokens`; `loops` and `loop_depth` (shown above 1); `tries` (`?` operators), `errs` and `nones` (paths ending in `Err` and `None`); `awaits`; `panics`; `macros`, the sorted names of the macros the body invokes; `single`, what a one-statement body does; every field after `tokens` is left out at 0 or empty. A top-level record whose item a `macro_rules!` invocation of the crate wrote has `via`, the macro, such as `events!`, and its location is the invocation's. A missing fact is never proof: vibelight sees no other macro expansion, no generated code, dynamic dispatch, or code outside the run. Method-call syntax such as `section.level()` never fills `crates`, `tests`, or `direct_callers`, and the absence leads see it only as a bare name hit shared by every method of that name. The lead ids, such as `same-shape`, `direct-field-clusters`, `direct-impl-field-split`, and `no-observed-name-use`, are defined in the lead glossary of [`tools/vibelight.md`](tools/vibelight.md).

### Links to functions, constants, and statics

In the HTML file, each module-level function, constant, and static has an id on its name, `fn.<crate>.<module path>.<name>`, `const.<crate>.<module path>.<NAME>`, or `static.<crate>.<module path>.<NAME>`, written like the ids of types: parts joined with dots, case kept, and `r#` removed. A repeated id, such as a function declared twice under different `cfg`s, gets `-2`, `-3`, and so on. A method's name in the documented view has an id under its type's, `fn.<crate>.<module path>.<Type>.<name>`, with `-2`, `-3`, and so on for a name its type repeats; on a facade's page it is the item's heading id and the name, such as `api.Error.new`. The items of `extern` blocks have no ids, and the markdown file has none of these ids.

A facade's page gives each item of its API a heading with the id `api.<module path>.<name>`, such as `api.vfs.Vfs`, and each module of its API the id `api.mod.<module path>`, or `api.mod` for the crate root. An item the facade itself defines keeps its own id on its heading the first time it shows, so links from other pages land on it. The search index lists the `api` headings beside every other id, and the search shows a facade's item before the crate that defines it. A facade's own declarations that its API does not show have no id, and a link to one prints as plain text.

A re-export line (`using ...`) links to a function, constant, or static of this crate, or of another crate of the run, through renames and chains of `pub use` items. A backticked name or an intra-doc link in a comment links the same way, when the name is in scope through a `use`, a glob, a path, or an item of the same module. When a type and a function have the same name, the link goes to the type. A function name is never found just because it is the only one of that name in the crate, which is how types are found, so a comment that names a function of another module needs a path or a `use`.

### The markdown notation

The markdown is JSON Lines: after the stamp, every line is one compact JSON object, with no blank lines, indentation, or wrapping, so a model can read it a line at a time and a script can parse it. `k` names the kind of record, and `path` names its owner, an item path below the crate root, such as `contract::args::ArgDecl`, with `crate` for the root itself. The header's `schema` is 2. A reach is spelled from the node's own module: `pub` for anywhere, `pub(crate)` for the crate, `pub(self)` for the module itself, `pub(super)` for its parent, and `pub(in crate::a::b)` for another module around it.

A type's whole shape is one `rust` string, such as `pub struct Point { pub x: i32, pub y: i32, label: String }`, and a trait's `rust` is its header, with each trait item a record of its own. Each function's signature is one `sig` string with its `where` clause inline, and a trait impl's function also has `impl`, the impl header. An `impl` record is the impl header, with the `locs` of every block that writes it. A constant or static has a `value` only when its initializer is one literal of at most 100 characters. `attrs` lists the attributes that shape an API or its wire format, `derives` the derived traits, and `attributes` the attribute macros, with `cfgs` and `lints` beside them and `test_support` on test-support code. Plain comments above a declaration are in its `comment` field, comments with nothing after them in a module, type, or impl body are in `dangling`, and parameter comments are in `param_docs`.

A doc is a `doc` record of its own, never a field, because some tools cut long lines short. Its text is the doc as written, with its line breaks as `\n` and its fenced examples kept exactly. Every location is one string, `<file>:<line>`, with the file relative to the crate and written with forward slashes, such as `src/lib.rs:6`; `lead` and `impl` records have a `locs` array of them. A subject in another target of the run is written `::<stem>::<path>`, such as `::facade::Panel`, and a location there is `<stem>:<file>:<line>`, such as `facade:src/lib.rs:9`. A file never holds a colon, so a reader splits a location from the right and opens the file below the `root` of the outline of the target that holds it.

Since some tools cut long lines short, no line is longer than 900 characters. A record that would be keeps the fields that name it and as much of its widest field as fits, and `more` records right after it carry the rest. Each names its owner by `path`, or a lead by its first subject and its `id`, names the `field` it continues, such as `locs` or `spellings.3.locs` for one element's field, counts its `part` from 2, and holds either `items`, whole elements to add to that array, or `text`, a piece to add to that string. The `modules` ranges count these lines too.

Here are the first four lines of the outline of a small test crate, then the root module's record, a struct, its impl, a method, and a struct that a macro writes, from further down:

```text
*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":2,"crate":"links","target":"lib","root":".."}
{"k":"modules","items":[{"path":"crate","from":16,"to":33},{"path":"parts","from":34,"to":39}]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["parts::Gauge"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/parts.rs:12"],"limits":["macros"]}
```

```text
{"k":"mod","path":"crate","loc":"src/lib.rs:1","types":3,"mix":{"pub":3}}
{"k":"struct","path":"Engine","rust":"pub struct Engine { pub speed: u32 }","loc":"src/lib.rs:6"}
{"k":"doc","path":"Engine","text":"A type with an impl in another file."}
{"k":"impl","path":"Engine","rust":"impl Engine","locs":["src/lib.rs:10"]}
{"k":"fn","path":"Engine::start","sig":"pub fn start(&self)","loc":"src/lib.rs:12","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::start","text":"Starts the engine."}
{"k":"struct","path":"Needle","rust":"pub struct Needle","loc":"src/lib.rs:30","via":"needle!"}
{"k":"doc","path":"Needle","text":"A needle the macro writes."}
```

The `modules` record says lines 16 to 33 hold the root module and lines 34 to 39 hold `parts`, counting the stamp as line 1, and `parts` has no `vis` because its `mod` line has no keyword. Line 16, the root's `mod` record, says the root declares three types, all `pub`. `Needle` is written by the `needle!` invocation on line 30. The outline sits in `out` inside the crate, so its `root` is `..`, and `src/lib.rs:6` opens as `../src/lib.rs` at line 6.

### The HTML notation

Every type prints as one C++-style block that uses Rust's item keyword and its visibility as written, with the members of all its inherent impls merged in, wherever they are written. Here is a public type whose fields and methods are all private:

```
/// Holds the text the lexer is still reading.
pub struct Cursor
{
pub(self):
    text: String,

    pos: usize,

pub(self):
    static new(text: String) -> Cursor

    bump() mut

    into_text(self) -> String

    peek() -> Option<char>
}
```

The header names the type with `struct`, `enum`, `union`, or `trait` after its visibility, such as `pub(crate) enum Mode`, and a type with no visibility keyword shows none. The body lists first what has no visibility of its own: an enum's variants, a trait's items, and the type's macro marks. The data comes next, a struct's fields, and the other members follow, because the data is what a reviewer weighs first: should these fields be here, and are these the right types? Each of the two parts sorts its members into sections by visibility, each opened by its label as written, such as `pub:`, `pub(crate):`, `pub(super):`, or `pub(in crate::engine):`, and `pub(self):` for no keyword. The widest comes first: `pub`, then what reaches the whole crate, then what reaches a deeper module, so a `pub(super)` in a module just under the root sits beside `pub(crate)`, which it means. A visibility that holds both fields and methods prints its label twice, once for each part:

```
pub struct Run
{
pub(crate):
    id: RunId,

pub(self):
    cache: HashMap<Key, Value>,

    name: String,

pub:
    static new() -> Run

pub(crate):
    advance() mut
}
```

A module's block of items uses the same labels. Its macro marks and its extern blocks come first, one extern block per ABI with its items labeled inside its braces, and then its re-exports, constants, statics, and functions, each under its label.

A node that reaches less far than its visibility says, because a module around it is narrower, gets a dim line above it, such as `reaches pub(crate)` above a `pub struct` in a private module: that is the widest place its path can be named from. A member or an item that reaches exactly as far as its type or module leaves the line out, since the type or the module already says it.

A module's coupling line reads the same uses as the markdown's `deps` records: it lists the modules that use the module's items and the modules whose items it uses, each with the number of distinct items used, and gives the module's instability, `fan_out / (fan_in + fan_out)`, with two decimals. It counts each use under the module where the code is written, so moving an impl block or other code to another module changes the line, as it changes source links.

The `fn` keyword and the bodies are gone. A method that takes `&self` gets no marker, a method that takes `&mut self` gets `mut` after its parameter list, a method that takes `self` in any other way keeps it as its first parameter, and only a function with no receiver is `static`. A bare `&` is read-only, as in Rust, and `mut` marks what can change, so `&str` prints as `str&` and `&mut T` as `T mut&`. The data keeps its declared order. The other members are grouped (associated types, constants, special members, static functions, then member functions) and sorted by name inside each group.

Comments stay with what they belong to. Each impl block's comments sit under a line in the type's block that names the trait, or the type for an inherent impl, a comment above a `pub use` group prints above the whole group, and a comment inside the braces stays with the name after it. A declaration's doc comment prints as prose, in the page's own typeface rather than the code font: consecutive lines join one paragraph, a blank line starts a new one, a `- ` or `* ` line starts a bullet list, a `# Errors` line becomes a small heading, and a fenced example stays exactly as written in the code font. Text in `backticks` prints in the code font wherever it appears, so a name reads as a name inside a sentence. A comment above a member reflows as one paragraph, at most 80 characters wide: its line breaks, and the blank lines between its paragraphs, become spaces, and a `cfg` condition or a `# Errors` heading keeps its own line. It sits in a box with no fill and no outline, so only its text shows, lined up with the member's own text, and the member's declaration follows on the very next line.

Each module shows its types and items twice. `At a glance` comes first: the same blocks without comments, attribute lines, or blank lines, so each type's labels and members stack one to a line and the shape of the module shows at once. `Documented` follows, with every comment, for reading what each part means. In `At a glance`, each function's name links to the function in `Documented`; it keeps the text color and shows an underline when the pointer is on it, and the page scrolls to the function's comment. Attributes that shape the API or its wire format print as `[[...]]` at half contrast. A function or a field closes its line with them, such as `[[must_use]]`, `[[track_caller]]`, and `[[serde(rename = "id")]]`, and anything else gets them on lines above it, such as `[[non_exhaustive]]`, `[[repr(C)]]`, and `[[deprecated]]` on a variant. Other attributes stay hidden, and a signature has no semicolon. Each inherent impl's link in a `Source:` line is followed by the names of the members it adds.

A facade's page prints each item of its API the same way, but only as the facade's users see it: its `pub` members alone, none marked `#[doc(hidden)]`, and no `reaches` lines. A hidden tuple field reads `/* private field */` in its place, so the tuple keeps its shape. An item re-exported under another name prints under that name, and its `Defined in` line names it as its crate does.

The overview ends with the Legend of this notation, so a reader of the HTML alone can follow it: facades and their families, the two views of each module, the module sections and the sidebar, the coupling line, the item keywords and visibility, `static` and the `() mut` marker, the base list, the labels and the order of a type's body and a module's items, the `reaches` line, `[[...]]`, `implements:`, `MACRO`, how comments print, and the links, search, and tabs.

Traits a type implements or derives print the C++ way. Only the C++ special member functions become members: `Default` is the default constructor, `From<T>` a converting constructor, and `Drop` the destructor, packed together under the type's own label, with `= default` when derived. Every other standard trait, `Clone` and the comparison traits included, goes in the base list by its short name, as in `struct Seed : Clone, Debug, Display`, and every other trait follows on its own line by its full path. A name counts as standard only when it really resolves to `std`, `core`, or `alloc`, so another crate's `Display`, the crate's own `Display`, or a `Display` that only comes through a glob import gets its own line. A derive from another crate keeps its bold **MACRO** line above the block.

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

When vibelight meets something it cannot fully handle, it keeps going and prints a warning on stderr, as `warning: <file>:<line>: <message>`, then a count at the end. Warnings cover files that do not parse, module files that cannot be found, module files that load themselves again, doc files that cannot be read, constructs it does not support (also marked UNSUPPORTED in the outline), invocations of the crate's own `macro_rules!` macros that cannot be expanded, as ``macro `name!` not expanded: <reason>``, and type names it cannot resolve or that are ambiguous, which stay plain text instead of links. When the markdown has spelling records, one more line, `warning: <count> items written more than one way; see spelling records`, counts the items those records name. In a run of several crates, a warning's path starts from the workspace root (or else the first crate), such as `crates/alpha/src/lib.rs:9`.

A `note:` line is not a warning. Notes say that the run wrote `vibelight.toml` or the folder's `.gitignore`, that it skipped a crate with nothing to outline, that a crate the config lists is also excluded, that an `exclude` glob matches no crate, that a `quiet` entry matches no lead, or that it removed an outline no crate writes any more.

- `0`: the files were written, warnings included.
- `1`: the input cannot be used, for example the path is not a crate, the manifest is workspace-only, a root file cannot be read, `vibelight.toml` has an unknown key, or `SOURCE_DATE_EPOCH` is not a whole number of seconds. The reason is printed on stderr.
- `2`: a usage error, such as an unknown flag, no crate named and no Rust repository to write a config for, or a config that names no crates, reported by clap.

## Requirements and license

- Minimum Rust version: 1.85
- License: BSL-1.0 (Boost Software License 1.0), in `LICENSE`
