---
name: Build vibelight tool
overview: "Create vibelight in C:\\Users\\Vinnie\\cursor\\vibelight: one executable that you point at a crate directory and it writes two files per target, a markdown outline and a fully linked HTML outline, of the crate's declarations and their comments in C++-header style, ordered from the top-level types down, with every macro marked and, by default, links back to the source files. It handles every ordinary Rust construct with nothing deferred. It reads source with syn and never builds the crate. Built to publish on crates.io as a single crate."
todos:
  - id: setup
    content: Copy this plan into vibelight/plans/ as YYYY-MM-DD-vibelight-tool.plan.md with the date of the run, then add Cargo.toml (one crate, BSL-1.0, edition 2024, lints, metadata, exclude list), rust-toolchain.toml (stable, with the rustfmt and clippy components), rustfmt.toml, clippy.toml (test allowances, disallowed types and methods), .gitattributes, .cargo/config.toml, .gitignore, LICENSE, a README stub, and the two approved AGENTS.md bullets below the existing rule
    status: pending
  - id: structure
    content: Implement build.rs and build/structure.rs (standard library only, skipped when Cargo.toml.orig exists, watching src, tests, build, plans, and the manifest) and tests/it/structure.rs, which tests each rule on temporary trees and runs the full check over the real tree; runs in parallel with everything after setup
    status: pending
  - id: source-links
    content: "Implement source_links.rs: link text relative to the crate, link targets relative to the output file with a line fragment, the file URL fallback across drives, no extended-length prefix; pure path logic with its own unit tests, in parallel with everything after setup"
    status: pending
  - id: root-modtree
    content: "Implement root.rs and modtree.rs: crate directory or .rs file, optional manifest, target names (lib, bins, the .bin suffix on a name clash, the file stem for a single file, the directory name without a manifest), CRLF to LF on read, the module walk with path attributes, mod.rs, inline and nested modules, cfg(test) skipping, missing-file and unparsable-file placeholders"
    status: pending
  - id: model-source
    content: "Implement model.rs and source.rs: the data model, per-node source slicing with whitespace and trailing-comma normalization, and byte offsets from line and character columns; in parallel with root-modtree"
    status: pending
  - id: goldens-constructs
    content: "Before collect and comments: write the construct fixtures, the comment fixtures, and the Prompt fixture, with their expected.md, by hand from the rules"
    status: pending
  - id: collect
    content: "Implement collect.rs, collect_type.rs, collect_fn.rs, and collect_use.rs: an exhaustive match over every syn Item, ImplItem, TraitItem, and ForeignItem variant with an UNSUPPORTED fallback, type-path spans through a syn visitor, doc comments, visibility, receivers, qualifiers, use maps, generic parameter scopes, the initializer rule, and the file and line of every declaration, impl block, member, module, and macro for source links"
    status: pending
  - id: comments
    content: "Implement comments.rs, in parallel with collect: read ordinary comments from the gaps between node spans, skipping string and character literals, and attach every comment by rule 20, including comments on hidden items, in macro bodies, on impl lines, on mod lines, and on impls that go to Other impls"
    status: pending
  - id: attach-macros
    content: "Implement attach.rs and macros.rs, in parallel with each other and with resolve: inherent impls merged into their exact self type, trait impls as base-list entries, every other impl sent to the Other impls section; macro detection with the derive-helper rule and cfg_attr, invocations that name a declaration, and the kill list data"
    status: pending
  - id: resolve
    content: "Implement resolve.rs: generic parameters in scope first, then path walking through crate-local use and pub use items, globs included, with a cycle guard, then same-module use, same-module declaration, and unique name; covers signatures and the backticked names and intra-doc links in comments; unresolved names stay plain and are counted on stderr"
    status: pending
  - id: layout-wrap
    content: "Implement layout.rs and wrap.rs: each declaration and module trailing block as styled segments (keyword, type reference, comment, plain) under the formatting rules, comments placed by rule 20, and the long-line rule (MAX_WIDTH 80) measured on visible text"
    status: pending
  - id: goldens-order
    content: "Before order: write the order fixtures and their expected.md by hand"
    status: pending
  - id: order
    content: "Implement order.rs: the naming graph from resolved segments, cycle groups, unconnected types, levels by the longest chain of namers, level 1 by reach, lower levels in seen order, all ties in path order"
    status: pending
  - id: goldens-render
    content: "Before render: write the link, anchor, macro mark, source link, and multi-target fixtures and their goldens by hand, including the three expected.html files"
    status: pending
  - id: render
    content: "Implement render_md.rs and render_html.rs, in parallel with each other: level, unconnected, other impls, and module sections, anchors, contents list, kill list, bold MACRO labels, marks outside the blocks in both files, repeat lines for marks inside fences, Source lines each followed by one blank line, fences longer than any backtick run inside them, and for HTML every resolved type linked, sidebar, syntax colors, target highlight, light and dark colors, all escaped"
    status: pending
  - id: main
    content: "Implement main.rs: clap arguments (crate path, output directory defaulting to the current directory and created if missing, and the no-source-links switch), write both files per target (overwriting), print their paths, warning summary on stderr, exit codes"
    status: pending
  - id: readme
    content: "Write the README, in parallel with main: what vibelight does, how to install and run it, the switch, and a short description of the output with one example"
    status: pending
  - id: dogfood
    content: Run on every crate under runvil/crates/runvil-internal, then on the rig, everruns, bashkit, and wg21-paperflow crates, the repos in parallel; require no panic, no UNSUPPORTED line, no dangling link, and identical output on a second run; check Prompt against its golden, one all-private class, the level order, and the macro kill list
    status: pending
  - id: gates
    content: Run cargo fmt check, cargo clippy, cargo test (the dev config makes warnings fatal), and the cargo package file list; report results
    status: pending
isProject: false
---

# vibelight: header-style outline of a Rust crate

## What it does

`vibelight <crate-dir>` writes two files for each target in the crate, `<target>.md` and `<target>.html`, into an output directory (the current directory unless one is given). It needs no Cargo build, no nightly toolchain, and no network. It parses the `.rs` files with `syn`, so it works on a freshly cloned repo and on code that does not currently compile. Parsing source is the design choice from this chat: rustdoc JSON is nightly-only and needs the whole dependency tree built, which is the step most likely to fail on someone else's repo.

Both files come from the same model, the same layout, and the same order, so their text never disagrees, and a test checks that. Each file then gets the most its format can do. Markdown cannot put a link inside a fenced code block, so the markdown file has no links on type names. The HTML file links every type.

Declarations are ordered from the top-level types down, not by module. A by-module index follows them. Comments are kept, because they are half of what makes the outline worth reading. Every ordinary Rust construct is handled in the first version, with nothing deferred. Apart from the source links, which point at files and lines and are on by default, the output depends only on the declarations, never on how the source is arranged.

## Decisions already settled

- **License:** BSL-1.0, matching [runvil/Cargo.toml](runvil/Cargo.toml) and wg21-paperflow. Copy the text from [runvil/LICENSE](runvil/LICENSE).
- **Package shape (hard rule):** exactly one crate, because it publishes to crates.io as a single crate. It is a binary-only crate named `vibelight`, edition 2024, with no workspace, no second manifest, no path dependency, and no xtask or helper crate. Reusable pieces stay modules inside it. Integration tests run the binary through `CARGO_BIN_EXE_vibelight`. The build enforces the one-crate rule.
- **Two outputs:** markdown and HTML for every target, each at the highest fidelity its format allows.
- **Coverage:** every language construct a normal Rust crate uses, with no construct left for later.
- **AGENTS.md:** gets exactly two bullets, word for word as approved (see Files to create), below your plan-copy rule. Everything else lives in the structure of the crate.
- **Goldens first:** fixture outputs are written by hand from these rules, each group just before the step that implements it, so the goldens are the spec and never a copy of the tool's output.
- **Determinism (hard rule):** apart from source links and the stamp on the first line (rule 61), the same aggregate declarations always produce byte-identical files, no matter how the source is arranged. Running twice leaves a file's bytes alone, stamp included, because a run does not rewrite a file whose content is the same. These all leave the output unchanged: reordering items in a file, moving an `impl` block to another module or file, splitting or merging `impl` blocks, reordering named-struct fields, derives, attributes, `use` lines, or `mod` lines, turning a file module into an inline one or back, reformatting, changing line endings, running twice, and running on another OS. Comments travel with the item they attach to (rule 20), so moving an item together with its comments changes nothing. One case is outside any rule's reach: a comment that stays put while the items under it move, such as a section divider, attaches to whatever item now follows it. What does change the output is a change to a declaration itself: a name, a path, a type, a signature, a comment attached to it, or the order of enum variants, tuple fields, or parameters, since that order is part of the declaration. See Determinism under Behavior that needs care.
- **Source links:** links from the output back to the file and line of each declaration, impl block, module, and macro. They are on by default, and a `no-source-links` switch turns them off. Files and lines can change without any declaration changing, so the source links are the one part of the output exempt from the determinism rule, and a test pins that exemption exactly (rule 45).

## Structural rules

Rules that matter live in the structure of the one crate, so a violation fails the build or the tests and nobody has to remember a sentence. [vibelight/AGENTS.md](vibelight/AGENTS.md) holds only your plan-copy rule and the two approved judgment rules that no check can enforce.

- **Manifest lints:** `unsafe_code` forbidden, clippy `all` deny, `pedantic` warn at priority -1, `unwrap_used` deny. Rust warnings are not denied in the manifest, because a new compiler release would then break `cargo install` for users.
- **`clippy.toml`:** allows `unwrap` and `expect` in tests, as the rulebook prescribes, so the `unwrap_used` deny does not fail every test. It also lists `HashMap`, `HashSet`, and `SystemTime` as disallowed types and `SystemTime::now` as a disallowed method, so a hash seed can never reach the output and the clock reaches only the stamp (rule 61), through one item-level `expect` in `stamp.rs`. Clippy checks resolved types, so the word in a string or a comment never trips it, and `all = deny` makes a violation fail.
- **`rust-toolchain.toml`:** stable, with the `rustfmt` and `clippy` components, which the gates and the arrangement tests use.
- **`.gitattributes`:** keeps every text file LF, so goldens compare byte for byte on Windows.
- **Dev-only flags:** `.cargo/config.toml` passes the deny-warnings flag to the compiler, so a plain `cargo build`, `cargo clippy`, and `cargo test` in a checkout fail on any warning, clippy's `pedantic` included. Cargo does not read that file when someone installs the published crate, and the exclude list keeps it out of the tarball.
- **`build.rs` structure check:** a small build script that uses only the standard library, with no build dependencies, so installs build nothing extra. It fails the build with a message that states the rule and the fix when:
  - the tree is more than one crate: a second `Cargo.toml`, a `[workspace]` table, a path dependency, or a `src/lib.rs`;
  - a `.rs` file under `src`, `tests`, or `build` has more than 500 lines (`tests/fixtures` is exempt);
  - a `mod.rs` file exists;
  - a file under `src` does not begin with a `//!` line;
  - a crate-level `#![deny]`, `#![warn]`, `#![forbid]`, or `#![allow]` appears in `src`, because lint levels belong in the manifest;
  - a file in `plans/` is not named `YYYY-MM-DD-` plus up to three kebab words plus `.plan.md`, which is your AGENTS.md rule made self-enforcing.
- **What the build script watches:** `src`, `tests`, `build`, `plans`, `Cargo.toml`, and `build.rs`. Watching the whole package would rerun the script on every build, because `target/` changes each time. A manifest added in some new folder is caught by the structure test instead, which runs the full check over the real tree on every `cargo test`.
- **Guard:** the check is skipped when `Cargo.toml.orig` is present. Cargo adds that file to every published tarball, so users who install from crates.io never run it, and `cargo package` verifies the tarball with the check skipped. A checkout has no such file, so the check always runs there.
- **Tests of the rules:** the rule code lives in `build/structure.rs`, which both `build.rs` and `tests/it/structure.rs` include as a module. Each rule is tested against small temporary trees, one violation and one clean case, without a second crate.
- **Exclude list:** `plans/`, `.cargo/`, and `AGENTS.md` stay out of the tarball.

## Formatting rules

**Execution tiebreaker.** When a case is not covered by a numbered rule, or the plan is unclear, make the layout look like a C++ header: use the C++ spelling, ordering, and layout, and add no Rust-only decoration. A numbered rule always wins over the C++ look. Pin each tiebreaker choice with a golden. The rules apply to both files.

Type blocks:
1. One block per type: merge the type's fields and the members of every inherent `impl`, even when the impls are in different files.
2. Reorder members so each visibility group is contiguous.
3. Remove `pub` everywhere. Type-level visibility is not shown.
4. Write no comments that mark visibility.
5. Show visibility with three labels, in the order `public:`, `internal:`, `private:`. `pub` is public; `pub(crate)`, `pub(super)`, and `pub(in path)` are internal, because an internal crate marks nearly everything that way; `pub(self)` and no keyword are private. There is no `protected:`. The first section's label is left out when it is the default, public in a struct or a module and private in a class, and every other section has its label.
6. Put the opening brace on its own line.
7. A type with at least one public member prints as `struct`. Its first section is public, so the `public:` label is left out. The `internal:` and `private:` sections that follow get their labels.
8. A type with at least one member and no public member prints as `class`. When its first section is private, the `private:` label is left out, because a class is private by default; when it has internal members, `internal:` comes first with its label, and `private:` follows with its own. Members include associated types and constants. Enums, traits, unions, and tuple or unit structs never become `class`.
9. Inside a section the order is associated types, associated constants, fields, special members (rule 10), static functions, then member functions. Within each group, members are sorted by name: an `r#` prefix is ignored, then case is ignored, then exact spelling breaks ties, then the member's full rendered text (which includes any `// cfg:` line). Enum variants and tuple-struct fields keep their declared order, because that order is part of the declaration and carries meaning. Named fields of structs and unions are sorted, even though field order can matter to derived ordering and drop order. A blank line separates the groups and comes before `private:`. Macro marks inside a block go at its end, after every section, sorted by macro name.
10. Traits on the type, from every impl wherever it is written and from its derives, print the C++ way. A standard trait (from `std`, `core`, `alloc`, or the prelude, found through `use` items) whose C++ equivalent is a special member function, a constructor or the destructor, becomes that member: `Default` is `Type();`, `From<T>` is `Type(T&);`, and `Drop` is `~Type();`, each with ` = default` when derived. The special members sit in that order, converting constructors sorted by argument, as one packed group after the fields or variants and before the static functions; a member with comment lines above it keeps a blank line from its neighbors. An impl with its own generic parameters or a `where` clause stays in the base list instead. Every other standard trait, `Clone`, `Copy`, `PartialEq`, `PartialOrd`, `Eq`, and `Ord` included, goes in the base list by its short name, sorted like members, with any generic arguments as written: `struct Name : Clone, Debug, PartialEq<str>`. An entry keeps its full path when its short name is also a struct, enum, union, or type alias of the crate, or the name of another entry for a different trait, as with `fmt::Write` beside `io::Write`; a trait of the crate never collides, because it prints in long form on its own line. A name goes on the short line only when it resolves to `std`, `core`, or `alloc`, however it is written (`Display` after a `use`, `fmt::Display`, `std::fmt::Display`, or a `use ... as` rename), so a lookalike never merges with it: another crate's `Display` prints its path, `other_crate::Display`, a `Display` of the crate prints its crate-relative path, and a `Display` that comes only through a glob import prints as written, each on its own line. A derive names a macro and never a type, so a bare standard derive stays standard even when the crate has a type of that name. Every other trait follows in long form, its crate-relative path or `crate_name::Trait`, one per line and sorted. When a long entry exists, or the one-line list passes `MAX_WIDTH`, each base line starts with `: ` or `, ` under the header, the short names wrapping to fit. A leading `unsafe` or a negative `!` stays on its entry. Supertraits use the same list, without special members.

Functions:
11. Remove the `fn` keyword. Qualifiers print in this order: `static` (rule 12), `constexpr` for `const fn`, `async`, `unsafe`, then `extern "ABI"` as written.
12. In a type or trait, a function with no receiver gets `static` and goes in the static group. A function with any other receiver is a member function, so a reader of C++ never takes one that consumes `self` for a function without an object. A free function never gets `static`, because at namespace scope in C++ that means file-local. A free function goes under `internal:` or `private:` by its visibility like any other item.
13. Drop `&self` from the parameter list and add nothing, because read-only is the default, as in Rust: `title() -> str&;`.
14. Drop `&mut self` from the parameter list and put `mut` right after the parameter list, where C++ puts `const`: `bump() mut;`. Every other receiver (`self`, `mut self`, `self: Box<Self>`, `self: Rc<Self>`, `self: Pin<&mut Self>`) stays as the first parameter, spelled by rule 18 (so `self: Pin<Self mut&>`), and the function stays a member function without `static`, so `into_name(self) -> String;` reads as a member that takes the object. `self: &Self` and `self: &mut Self` count as `&self` and `&mut self`.
15. A declaration ends in `;` and has no body. The return type keeps the Rust arrow (`-> T`).
16. Long lines. `MAX_WIDTH` is 80, counting the indent and visible text only. Your example line is 87 columns, so the limit has to be below that. A longer line is broken in this order, stopping as soon as every line fits:
    - after `->`, with the return type on the next line indented 4 more spaces than the declaration;
    - if the first line still does not fit, or there is no return type, one parameter per line indented 4 more, with the closing parenthesis, any `mut`, and the arrow and return type following the last parameter when they fit, or broken after `->` as in the first step;
    - a `where` clause that still does not fit moves to its own line indented 4 more, and this step also applies to type headers.

    A line that still does not fit stays as it is. Fields, comment lines, and variants (apart from the struct-like expansion in the construct table) are never broken.

Text:
17. A field is written `name: Type,` with a trailing comma.
18. Types and signatures print as written in the source, assembled from per-node slices (see Behavior that needs care), except that a reference or a raw pointer puts its sigil after the inner type: `&T` prints `T&`, `&mut T` prints `T mut&`, `&'a T` prints `T& 'a`, `&'a mut T` prints `T mut& 'a`, `*const T` prints `T*`, and `*mut T` prints `T mut*`, nested types inside out (`&Vec<&T>` prints `Vec<T&>&`). Behind a reference or a pointer, a trait object or `impl` type with a return type or more than one bound, and a function pointer with a return type, print inside parentheses, so `&'a dyn Fn(&str) -> R` prints `(dyn Fn(str&) -> R)& 'a` and never reads like a closure that returns a reference; parentheses already in the source stay and are not doubled. Read-only is the default, as in Rust, so no `const` qualifier is written, and `mut` marks what can change. Expressions, such as initializers, and a type the parser keeps as tokens stay as written. `Vec<Block>` stays short, and `std::result::Result` stays spelled out where the source spells it out.
19. Indent members 4 spaces. The `internal:` and `private:` labels sit at the same column as the type header.

Comments and attributes:
20. **Comments.** Every comment outside a function body or a macro body is kept, with one exception: a comment belongs to its item, so when the item is not shown, its comments are not shown either. That covers a plain `use`, `extern crate`, a `cfg(test)` item, a `const _` assertion, and a member of a trait impl. Comments inside function bodies, `macro_rules!` bodies, and macro arguments go with those bodies.
    - **Kinds:** doc comments (`///`, `/** */`, and `#[doc = "..."]`) print whole, as `///` lines, the Doxygen convention in C++ headers. Module and crate docs (`//!` and inner doc attributes) print whole as `//!` lines. Ordinary comments (`//` and `/* */`) print as written. Every comment keeps its source line breaks, is re-indented to its place in the output, and is never rewrapped. When an item has both kinds, they print in the order they were written.
    - **Attachment:** a comment belongs to the next item, field, variant, member, or parameter in its scope, with or without a blank line between them, and prints directly above it. A trailing comment on the same line stays at the end of that line and does not count toward `MAX_WIDTH`. A comment with nothing after it in its scope prints at the end of that block. A comment inside a type expression moves above the field, parameter, or declaration that holds the type. A parameter with a comment forces one parameter per line, with the comment above the parameter.
    - **Impls and modules:** a comment on an `impl` block is one written above its `impl` line. It prints in the type's block right after the type's own comments, ordered by trait name for trait impls and by first member name for inherent impls. Each impl's comments follow a header line, `// impl` and the trait as the base list prints it (such as `// impl Display` or `// impl runvil_lua::ErrorValue`), or `// impl` and the type's name for an inherent impl, so a reader sees where each impl's comments start. The trait in the header is a comment reference: it links in HTML and names nothing. An impl that becomes a special member has no header, because its comments go above that member. A comment just inside the block, before the first member, belongs to that member like any other, so a `///` there stays the method's doc comment. A comment above an impl that goes to Other impls prints above its `implements:` line. An ordinary comment on a `mod` line prints with the module's docs.
    - **Links:** both files flatten comment text the same way: markdown link syntax is removed and link targets are dropped, while backticks stay, so a backticked name or code span still reads as code. An intra-doc link such as ``[`X`]`` or ``[`items`](Self::items)`` shows its label, `` `X` `` or `` `items` ``. A line left short by a dropped link target stays short, because comments are never rewrapped. The HTML file then wraps the names that came from intra-doc links and backticked names in links, when they resolve to a declaration; the backticks stay outside the link. Bare words never become links, because words like `Run`, `Step`, and `Event` are too common in prose.
21. Attributes are not shown, with three exceptions: the macro marks, a `// cfg:` line for any `cfg` that is not test-only, and a `[[...]]` line for each attribute that shapes an API or its wire format: `non_exhaustive`, `must_use`, `deprecated`, `repr`, `track_caller`, `doc(hidden)`, and `serde(...)`. Several `cfg` attributes on one item print one `// cfg:` line each, sorted. Each API attribute prints as written inside `[[...]]`, one per line, sorted, after the `// cfg:` lines, on declarations, named fields, variants, members, and module items; an attribute inside a `cfg_attr` that is not test-only counts, without its condition. Tuple fields and struct-variant fields print neither line.
22. Mark every macro clearly, inline and in a kill list at the top, except a standard derive, which rule 10 prints as a special member or a base-list entry. The word MACRO is bold wherever the format allows bold (see Macro marking).

## Document order

Declarations are ordered by how they name each other, so the types nothing else names come first and each type comes after every type that names it.

23. **Declarations** are every struct, class, enum, union, trait, and type alias in the target.
24. **Naming:** declaration A names declaration B when a type reference in A's rendered block resolves to B. That covers field and variant types, associated types and constant types, every parameter, return type, bound, and `where` clause of A and its members, the base list, supertraits, and an alias's target. Self-references, comments, initializer values, and unresolved names do not count. Free functions, constants, statics, extern items, and re-exports are not declarations, so they give no type a level, but a type their own text names (not their comments) is in use, so it is never unconnected (rules 26 and 27).
25. **Cycles:** declarations that name each other in a cycle form one group (a strongly connected component) and are placed together.
26. **Unconnected:** a declaration that names nothing and that nothing names, neither a declaration nor a module item (rule 24), goes to the "Unconnected types" section at the end, in path order.
27. **Levels:** a group is at level 1 when no declaration names it, so a type that only module items name is at level 1. Otherwise its level is one more than the highest level of any group that names it. So every type sits below everything that names it, and level 2 holds exactly the types named only by level-1 types.
28. **Level 1 order:** the group that reaches the most declarations through naming comes first, with ties in path order. This puts the main entry types, such as a `Run`, ahead of small leftovers.
29. **Order within level 2 and below:** seen order. Scan the blocks already placed, top to bottom, and order the level's groups by where each is first named. A level never names a group in the same level, so the scan only reads levels above. Inside a cycle group, members follow seen order too, with path order for any member not yet named.
30. **Path order** compares full paths segment by segment, each segment compared as in rule 9. Apart from source links, nothing in the output depends on file names, line numbers, or the order items appear in the source. The module walk only discovers files.

## Document layout and anchors

31. The file opens with the stamp (rule 61), then an H1 naming the target, then a fence holding the crate docs as `//!` lines when there are any, then the Legend (rule 64), then the contents list and the kill list, then the Public surface section (rule 62).
32. Next come the level sections: an H2 for "Level 1: top-level types", then "Level 2", and so on. Each declaration has its own H3, in the order above. Test-support declarations (rule 63) are not in the levels. This is the level order; the module order (rule 68) replaces rules 32 to 34. The H3 text is the declaration's path below the root, such as `prompt::Prompt`. A type in the root module is just its name. A type alias prints as a one-line block.
33. Then come an H2 "Test support" (rule 63), an H2 "Unconnected types", and an H2 "Other impls", each only when it has something in it. Other impls holds every impl with no declared self type to attach to (see Trait impl in the construct table), as `implements: Trait for Type;` lines sorted by trait and then by self type. Where such an impl is written does not matter.
34. Then comes an H2 "Modules", with one H3 per module in path order, and after it the Spellings section when it has entries (rule 47). Each holds a fence with the module's docs as `//!` lines (the outer doc on its `mod` line, then its inner docs, joined the way rustdoc joins them, plus any ordinary comment on the `mod` line), a bullet list of its declarations in path order that links to their headings, and then its trailing block. The block's groups are re-exports, macro marks, constants, statics, free functions, and extern blocks (one per ABI, merged), each sorted like members. It is split into the public items, then `internal:`, then `private:` (rule 5), with no braces and items at column 0; an extern block prints once in each section that holds its items.
35. Every heading has an explicit anchor: an `<a id="..."></a>` on the line above it in markdown, and an `id` on the element in HTML. The ids are the same in both files: `crate.NAME`, `legend`, `surface`, `levels`, `level.N`, `test-support`, `unconnected`, `impls`, `modules`, `spellings`, `external`, `shapes`, `mod.PATH`, `type.PATH.Name` (parts joined with dots, case kept), plus `macro.N` for each macro mark in the HTML file, numbered in document order. Each module-level function, constant, and static has an id in the HTML file only: `fn.CRATE.MODULE.PATH.name`, `const.CRATE.MODULE.PATH.NAME`, and `static.CRATE.MODULE.PATH.NAME`, built like `type.PATH.Name` (parts joined with dots, case kept, `r#` removed). The id sits on the element around the item's own name in the module's block, the span or the source link, so the markdown file has no such anchors. Methods and extern block items get none. An id that would repeat, such as one type or one function declared twice under different `cfg`s, gets `-2`, `-3`, and so on, ordered by the declarations' rendered text. A type and a function of one name do not repeat each other's id, because the prefixes differ. Markdown has no native anchor syntax, so the anchors are raw tags. GitHub, VS Code, and Cursor previews render them.
36. Both files open with a contents list: the legend, the public surface, then the levels with their declarations, then the test-support and unconnected types, other impls, modules, spellings, and, with the facts on, External items and Same shapes. In the module order (rule 68) the levels entry is one `Levels` entry, each module's entry lists its declarations, and Other impls follows the modules. In markdown these links sit outside any fence, which Markdown allows.
37. The markdown file has no links on type names inside declarations, so each declaration stays a clean fenced block. Its kill list is a bullet list whose entries start with a bold **MACRO** label and link to the heading of the declaration or module that holds each macro, sorted by kind, then macro name, then that heading's path. A derive entry lists only the derives rule 10 does not print, and it goes away with its mark when none is left. The `kill_list` setting (rule 57) chooses how much of the kill list both files show: `full`, the default, shows the counts and every entry; `counts` shows only the counts line; `off` shows neither. The marks themselves always print.
38. The HTML file links every type name that resolves to a declaration in the document, wherever it appears: field types, parameter and return types, generic arguments, bounds, base lists (a long entry for a trait of this crate), special-member arguments, re-exports, and comments (rule 20). The link wraps the name as written. A name that resolves into another crate of the run (rule 54) links to that crate's page, as `runvil_model_client.html#type.runvil_model_client.model.ModelSet`, and carries the `cross` class. Std types, types of crates outside the run, generic parameters, primitives, and `Self` stay plain text. A name in a re-export line or a comment may also name a module-level function, constant, or static of this crate or of another crate of the run (rule 39), and then links to its `fn.`, `const.`, or `static.` id the same way, with the `cross` class across crates. A name that could be a type and a value links to the type. The markdown file links none of them.
39. Name resolution. Generic parameters in scope (of the item, its impl, and the member) are checked first and always stay plain, so in `struct Wrapper<Block>` the `Block` is never linked to a type of that name. A path with a prefix (`crate::`, `self::`, `super::`) or with more than one segment is walked module by module. The walk follows crate-local `use` and `pub use` items, globs included, with a guard against cycles. A one-segment name is looked up in this order: a `use` in the same module (renames and globs included), a declaration in the same module, then the standard prelude, as in Rust, so a bare `Result` that the module neither declares nor imports is the standard one even when other modules declare their own, and only then a name that is unique across the target, unless the module has a glob import from another crate, which may be where the name comes from. Otherwise the name stays plain, and stderr reports how many names were unresolved or ambiguous. A name that may be a value, which is one in a re-export line or a comment and never a field or parameter type, a bound, or a derive, is looked up in the same way but also finds the functions, constants, and statics of a module. In each module the order is a `use`, a glob, a type or trait declaration, then a function, constant, or static, then a child module, so a type wins over a function of the same name, and a function wins over a module of the same name. Only the last name of a path may be a value. A function, constant, or static is found by a `use`, a glob, a prefix path, or a same-module name, never by the unique-name fallback, which stays for types. Functions under different `cfg`s resolve to the one with the smallest `cfg` lines. A value never changes the unresolved and ambiguous counts, because names in re-exports and comments never count.
40. The HTML file is one self-contained file: inline CSS, no JavaScript, no network requests. Its features are syntax colors, a sticky contents sidebar, a highlight on the targeted declaration, macro marks styled to stand out with a bold MACRO label, kill list entries that link to each mark, links to another crate's page in yellow (a darker gold on the light theme and a lighter yellow on the dark theme, from one fixed style sheet), and light or dark colors that follow the system setting. All text is HTML-escaped. Marks that sit outside a block in markdown sit outside the block in HTML too, so the two files hold the same block text.
41. The width rule (16) is measured on visible text, so link tags and HTML escapes never count.

## Source links

42. **Switch:** source links are on by default. The `no-source-links` switch, typed with the usual two leading dashes, turns them off in both formats, and rule 57 gives each format a switch of its own. A run with links off gives files that depend only on the declarations. Everything that would be a link keeps its color: a member name becomes a `<span class="src">` with no underline and the default cursor, and the `Source:` lines and ` (source: ...)` suffixes, whose text is only a file and line, are left out.
43. **Form:** the link text is the file's path relative to the crate directory, with forward slashes, then a colon and the line, such as `src/prompt.rs:136`. The link target is the same file's path relative to the output file, with a `#L136` fragment, which GitHub, VS Code, and Cursor open at that line. When no relative path exists, as when the output and the crate are on different Windows drives, the target is an absolute `file:///` URL. A Windows extended-length path prefix never appears.
44. **Placement:**
    - under each declaration's H3, a `Source:` line with the definition first, then `impl` and every impl block that adds members or base-list entries, sorted by path and then line; each inherent impl's link is followed by the names of the members it adds, sorted, in parentheses, such as `impl [src/lib.rs:10](../src/lib.rs#L10) (start)`, so the line shows how a type's members cluster by file, and a trait impl adds nothing;
    - under each module's H3, a `Source:` line with the module's file, plus the line for an inline module;
    - at the end of each kill list entry and each Other impls line, as a ` (source: ...)` suffix, and at the end of each spelling (rule 47), as one suffix that lists every line it is written on, separated by commas;
    - for a mark inside a fence (an UNSUPPORTED line, or a macro inside an `impl` body), on the bold repeat line that markdown prints under the block for every such mark, because markdown cannot put a link inside a fence;
    - in the HTML file only, around each member name inside a block, linking to that member's own line.

    Under an H3 the order is the `Source:` line, then the marks, then the fence.
45. **Separation:** source links never change anything else in the output. The member lists of rule 44 are part of their `Source:` line, so they come and go with it. Each source link is a whole `Source:` line followed by exactly one blank line, a ` (source: ...)` suffix, or an HTML link around a member name, which is a `<span class="src">` around the same name without links. Removing them, each `Source:` line together with its blank line, gives byte for byte the output of a `no-source-links` run, and a test checks exactly that. So the weaker determinism is precise: the only bytes that may depend on arrangement are the source links.
46. **Stability with links on:** the same arrangement and the same output location give identical files on every run. Paths always use forward slashes, so only moved code, a different output location, or a different drive changes the links.

## Spellings

47. **Section:** an H2 "Spellings" after Modules lists each item outside the crate (in `std`, `core`, `alloc`, or another crate) that the declarations write in more than one way, so the source can settle on one. It counts every name that resolve gives a canonical path: field, parameter, and return types, generic arguments, bounds and `where` clauses, base lists and impl headers, supertraits, and derive names. Names in comments and re-exports, `use` items, and function bodies never count, and neither does the self type of an impl attached to its type, which the outline does not show and which a split impl writes again. Names of the crate's own items are left out too, because the same local item is legitimately written `crate::model::X`, `X`, or `super::X` in different modules. Names group by the item they name: a bare prelude or derive name stands for its `std` path, a `core::` or `alloc::` path for the `std` path that re-exports it, and a `use ... as` rename is one more spelling. A spelling is the path as written, without generic arguments or `r#`. An item with more than one spelling becomes an entry, `` `std::fmt::Debug` is written 2 ways; pick one and search for the rest: ``, with one nested item per spelling giving its count and, with source links on, a link to each line it is on. Entries sort by path and spellings by rule 9, and rule 49 names the spelling to prefer. The text is the same with links off, so the reader still knows to search, and the section and its contents entry are left out when no item has two spellings. Glob imports such as `use std::fmt::*` are not followed, so a name that comes only through one does not count.
48. **Count on stderr:** when the section has entries, stderr gets one more summary line, `warning: N items written more than one way; see Spellings`, which counts toward the warning total. The exit code stays 0.
49. **Preferred spelling:** an item of `std`, after the `core` and `alloc` mapping, prefers its bare last segment, as a base list prints it, such as `Debug`, `Display`, or `Cow`. Its entry says `` `std::fmt::Display` is written 3 ways; prefer `Display`: ``, or, when no written spelling is that bare name, `` `std::fmt::Display` is written 2 ways; prefer `Display`, which is not used yet: ``. The preferred spelling's line shows only its count, and every other spelling also lists its lines when source links are on. An item has no preferred spelling, and its entry keeps the pick-one wording, when it is not from `std`, when the crate declares a type or trait with that bare name, or when another `std` item that the crate uses has the same bare name, as with `fmt::Write` and `io::Write`, or `fmt::Result` and `Result`. Derive names count, so `derive(Debug)` is a use of the preferred `Debug`.

## Several crates in one run

50. **Choosing the crates:** a run takes several crate paths (each a directory, its `Cargo.toml`, or, alone, one `.rs` file), or the `workspace` switch, which reads the `[workspace] members` of the first manifest with a `[workspace]` table at the given directory or above it. A member glob such as `crates/*` expands to the folders it matches that hold a `Cargo.toml`, an `exclude` entry takes its folder and everything below it out of the globs, a member written out in full always counts, and a root manifest that is also a package makes the root a member. The `follow` switch, repeatable, takes a glob (`*`, `**`, `?`) matched against folders relative to the workspace root, or else the first crate: starting from the chosen crates, the run adds each path dependency whose folder matches, and then that crate's path dependencies, and nothing else.
51. **Every crate is read and written.** A crate pulled in by the workspace or follow switch with nothing to outline (only tests, examples, or benches, only a build script, or no library or binary source) is skipped with `note: skipped <name>: it has <reason>` on stderr, which is not a warning, and the exit code stays 0. A single named crate with nothing to outline exits 1, as before. Two targets of a run that would write the same file name stop the run with exit code 1 and an error that names both crates. File names are as before: the library name with underscores, a binary's name, and `.bin` for a binary named like its library.
52. **Crate map:** each crate may name its path dependencies from `[dependencies]` and `[build-dependencies]` and the same tables under every `[target.*]`, never dev-dependencies, plus `crate`, `self`, and `super`. A `workspace = true` entry takes its path from the root's `[workspace.dependencies]`. The name is the dependency's key when `package = "..."` renames it, and otherwise the dependency's library name, which honors `[lib] name`. Only a dependency that is a crate of the run, matched by folder, can be reached. A binary names its package's dependencies, not its own library, so a run of one crate links nothing across crates.
53. **Two phases:** every crate is read, collected, and resolved first; then each name is linked into the other crates against the one fixed map; then every document is built, so each page's anchor ids are known; then each is rendered in memory, and the run writes or checks the files (rule 58). Targets are handled in output-name order, and no output depends on the order the crates were named in, which a test checks by naming them in reverse.
54. **The hop across crates:** a path whose first name is a dependency of the crate, directly or through the crate's `use` items and globs, walks into that crate's modules from its root, through its `use` items, `pub use` chains, and globs, with the cycle guard keyed by crate, module, and name. A name it reaches is a foreign reference stored beside the target. It never changes the target or the canonical path of rule 47, so naming, order, the base lists, Spellings, and every markdown file are the same whatever crates are in the run. The one exception is the facts (rule 67): with them on, External items shows the first doc line of an item from another crate of the run, so the markdown then depends on which crates are in it. Field and parameter types, bounds, impl headers, long base-list entries such as `runvil_lua::ErrorValue`, re-export lines, and comments all link this way. The hop reaches functions, constants, and statics the same way for a name that may be a value, so `using runvil_engine::perform_vfs_op;` links to `runvil_engine.html#fn.runvil_engine.execute.perform_vfs_op`, however many crates re-export it on the way. The foreign reference then holds the function's module and item, beside the target and never in place of it, and External items (rule 67) never lists it.
55. **Warnings in a run of several crates:** a warning's file path starts from the workspace root, or else the first crate, such as `crates/alpha/src/lib.rs:9`. A run of one crate keeps crate-relative paths.

## Config, per-format output, and checking

56. **Config file:** `vibelight.toml`, found in the current directory or the nearest parent that holds one, the way Cargo finds its files. Its keys are `workspace` (true, or a directory where the workspace search starts), `crates` (a list of crate paths or globs; a glob expands to the matching folders that hold a `Cargo.toml`, and a matched crate with nothing to outline is skipped as in rule 51), `follow` (a list of globs, as the `follow` switch), `kill_list` (`full`, `counts`, or `off`, rule 37), `facts` (true or false, rule 65), `order` (`levels` or `modules`, rule 68), `profile` (`analysis`, rule 69), and the tables `[markdown]` and `[html]`, each with `out` (a folder), `source_links` (true or false), and `enabled` (true or false, default true). Every path in it is relative to its own folder. `workspace` and `crates` together, a value of the wrong type, and an unknown key, at the top or in a table, are errors that name the key; a key that takes one of a fixed set of words also lists the words it allows. A run with no crate path, no `workspace` switch, and no config that names crates is a usage error with clap's exit code.
57. **Per-format output:** the markdown and HTML files have separate folders and separate source-link settings, so each format measures its source links (rule 43) from its own folder. The switches are `out` (short `o`, the folder for both), `md-out` and `html-out`, `no-source-links` (links off in both), and `md-source-links` and `html-source-links`, which take `on` or `off`. Each format's setting comes from its own switch, then the switch for both formats, then the config, then the defaults: the config's folder, or the current directory without a config, and links on, unless a profile (rule 69) turns them off. Crate paths or the `workspace` switch replace the config's `workspace`, `crates`, and `follow`, and its output settings still apply; without them, `follow` switches replace the config's `follow`. A format with `enabled = false` is left out unless its own folder switch names a folder, and a config that leaves both formats out is an error. When both formats share a folder and a link setting, each target's document is built once. All HTML files of a run go in one folder, so the cross-crate links of rule 38 hold. The settings that shape every file of a run, whatever the format, come from their switch, then the config, then the default: `kill-list` and its key `kill_list` (rule 37), the plain `facts` switch and its key `facts` (rule 65), which are off by default, and `order` and its key `order` (rule 68). A profile (rule 69) changes only the defaults, so the order is always switch, then config, then profile, then default.
58. **In memory first:** a run renders every file in memory before it writes or checks anything. A normal run creates the folder of each format it writes, writes each file whose content changed, overwriting the file of the same name, and prints the path of every file of the run on stdout, sorted, written or already current. A file's content changed when its text, with every stamp masked (rule 61) and a CRLF line ending on disk counted as LF, differs from the file on disk.
59. **Check:** the `check` switch writes nothing and compares each markdown file of the run with the file on disk; the `check-html` switch compares the HTML files too. A CRLF line ending on disk counts as LF, and both texts have their stamps masked (rule 61), so a file that differs only in its stamp matches. Each problem is one stdout line: `differs: <file> (first difference at line N)`, `missing: <file>`, or `stale: <file>`. A file is stale when it is an `.md` file directly in the markdown folder, it starts like an outline, and no target of the run writes it. An outline starts with a stamp line, a blank line, and a crate anchor (`<a id="crate.NAME"></a>`), or, as releases before the stamp wrote it, with the crate anchor on its first line; no other file and nothing in a subfolder is ever stale. A last line follows, `check passed: N files match` or `check failed: N problems`, with exit code 0 or 1. Paths print relative to the current directory when they are inside it. The check takes crates and settings from the config, from crate paths and switches, or from both, exactly as a normal run does.
60. **Stale files in a normal run:** a normal run never deletes a file. Each stale markdown outline (rule 59) gets `warning: <file> is stale: no crate of this run writes it` on stderr, which counts toward the warning total, and the exit code stays 0.
61. **Stamp:** every file starts with the UTC time its content last changed, as RFC 3339 to the second with a `Z`, such as `2026-01-01T00:00:00Z`. In markdown, line 1 is `*Updated <time>*` and line 2 is blank. In HTML, line 1 is `<!-- Updated <time> -->`, before the doctype, and the first element of `<main>` is `<p class="stamp">Updated <time datetime="<time>"><time></time></p>`, in the muted color. The time comes from `SOURCE_DATE_EPOCH` when it is set, and a value that is not a whole number of seconds from 0 to 253402300799 stops a writing run with exit code 1 and a message that names the value. Otherwise it comes from the clock. A check run reads neither and renders a placeholder of the same length. To mask a text, every time in a stamp line becomes the placeholder, so line numbers stay true. The stamp is the second exemption from the determinism rule, beside source links, and it changes only when a run rewrites the file (rule 58).
62. **Public surface:** a library target's file has an H2 "Public surface" (id `surface`) right after the kill list, with an entry in the contents, listing every path a user of the crate can name, so the crate's real API is the first thing a reader sees. The walk starts at the crate root; a `pub mod` of a module it reaches is reached too, and nothing else is. In each module it reaches, at that module's public path, it lists every `pub` declaration, `pub mod`, function, constant, and static, extern-block items included, and every `pub use` under the name it binds; a re-exported module is walked again under its new name, with a guard against cycles. A re-exported type or alias uses resolve's target; a re-exported function, constant, or static is found by name among the target module's items, following that module's own re-export of the name when it has one, with a cycle guard. A glob re-export adds the target module's public items under the glob's module, and every `#[macro_export]` macro is named at the root. Only plain `pub` counts. Each entry is a link with the path as its text: a declaration or a module links to its own heading, and a function (its text ending in `()`), a constant, a static, or a macro (its text ending in `!`) links to the heading of the module that declares it. A re-export of another crate's item, and one the walk cannot place, shows the path as written after it, as `LuaProgram (runvil_lua::LuaProgram)`, and has no link. Entries sort by path (rule 30); a path that two declarations share, as under different `cfg`s, is listed once, linking to the heading whose id sorts first. An entry for test support (rule 63) has ` (test support)` after it: a test-support declaration, module, or module item, an item inside a test-support module, and a re-export that is test support or names test support. A binary, a single `.rs` file, and a library with nothing public have no section and no contents entry.
63. **Test support:** code behind a `cfg` that lists `test` inside `any(...)`, such as `cfg(any(test, feature = "test-support"))`, also compiles for tests, so it is test support and stays apart from production types. A declaration, module, or module item is test support when its own `cfg` lines or an enclosing module's are; `not(test)` never makes test support, and test-only code is already gone (rule 20). Test-support declarations leave the levels and print, in path order, under an H2 "Test support" (id `test-support`) after the last level, with an entry in the contents. The names they use still keep a type out of Unconnected (rule 26).
64. **Legend:** every file has an H2 "Legend" (id `legend`), with an entry first in the contents, that explains the notation in fixed text, so a reader of the file alone, often an AI, can follow it: `struct` and `class`, the three access labels, receivers, reference and pointer spelling, `constexpr` and `using`, special members and the base list, `// cfg:`, `[[...]]`, `// impl`, and MACRO, the Public surface and its marks, the levels, Test support, and Unconnected types, `Source:` lines and their member lists, and the stamp. In markdown it follows the H1 and the crate docs and comes before the contents list; in HTML, whose contents are the sidebar, it comes inside `<main>` after the H1 and the crate docs and before the kill list, so both files hold their headings in the same order. Its text is the same with source links on or off, and no line of it starts like a heading or a mark; its line about macros says what the kill list shows (rule 37), its line about the levels follows the order (rule 68), and with the facts on it gains a line about them (rule 65). The README and the legend agree.
65. **Facts lines:** with the facts on, each declaration's H3 has two lines after its `Source:` line and before its marks, which point at oversized and unused types. `Counts:` gives its variants, its fields with how many are an `Option`, its associated types and constants, its special members, and its functions, leaving out a kind it has none of, or says `none`; nothing on it depends on arrangement. `Used by:` links, in path order, the declarations whose blocks name it and then the modules whose items name it (rule 24), or says `nothing`.
66. **Same shapes:** with the facts on, an H2 "Same shapes" (id `shapes`) at the end of the file, with an entry at the end of the contents, lists each group of two or more declarations of one kind with the same variants or fields, as candidates for merging: enums compare their variants in declared order, and structs, tuple structs, and unions their fields, by name and type, never by docs. A declaration with fewer than two variants or fields is in no group. Each entry reads like `enums with the same 2 variants:` and links its declarations in path order; entries sort by their first declaration's path. The section is left out when no group has two declarations.
67. **External items:** with the facts on, an H2 "External items" (id `external`), with an entry in the contents, comes before Same shapes and tells the reader what each name from another crate is. It lists every item outside the crate and outside `std`, `core`, and `alloc` that the declarations' blocks and the module items name, by the canonical path of rule 47, grouped by its first segment, the crate as the source names it, in path order. Each item gives its use count and links its users, the declarations and then the modules, in path order, as `` `alpha::Meter`, 1 use by [Panel](#type.facade.Panel) ``. When the item is a declaration of another crate of the run (rule 54), a nested entry shows that declaration's first doc line. Names in comments, re-exports, and derives never count. The section is left out when nothing counts.
68. **Module order:** the `order` setting (rule 57) is `levels`, the default and rules 32 to 34, or `modules`, where the level order scatters a module's types across the file. In the module order, after the Public surface comes an H2 "Levels" (id `levels`), a list of the declarations of each level, then the test-support and the unconnected ones, as the last two groups. Then each module, in path order, gets an H2 with its id `mod.PATH`, its `Source:` line, its docs, and its trailing block, and then an H3 for each of its declarations, Test support and Unconnected types included, by level and then in the levels' order; the trailing block comes before the H3s, because a reader takes what follows an H3 as part of that declaration. Other impls follows the modules, and the separate Modules section is left out. Declaration ids stay `type.PATH`, so every link still lands.
69. **Profile:** the `profile` switch and its key `profile` take `analysis`, which gives a reader who analyzes an API the right settings with one word: source links off in both formats, the kill list at `counts`, the facts on, and the module order. A profile changes only defaults, so an explicit switch or config key, such as `source_links = true` or `order = "levels"`, still wins.

## Language constructs

The collector matches every variant of `syn::Item`, `ImplItem`, `TraitItem`, and `ForeignItem`. Nothing is skipped silently. If `syn` returns something unknown (a `Verbatim` node, or a variant added in a later release), the block gets a `// UNSUPPORTED: kind` line, the markdown file repeats it as a bold **UNSUPPORTED** line under the block (with a source link when links are on, rule 44), stderr gets a warning with the file and line, and the run continues. A file `syn` cannot parse at all gets a placeholder under its module and a stderr warning. Only the one fixture built to test the fallback may produce either.

Each row says how a construct prints. Comments go above every item, field, variant, member, and parameter they attach to (rule 20).

- **Named struct:** `struct Name<generics> : Bases where ...` block, rules 6 to 10. Generics print as written: lifetimes, bounds, defaults, const generics, where clauses.
- **Tuple struct:** one line, `struct Name(T, U);`, with `pub` removed from the fields. A tuple struct whose fields carry comments prints one field per line inside the parentheses, each comment above its field. **Unit struct:** `struct Name;`.
- **Enum:** `enum Name<generics> : Bases` block. Variants print as written, in their declared order: unit `A,`, tuple `B(T),`, struct-like `C { x: T },`, explicit discriminant `D = 4,`. A struct-like variant expands to one field per line, with field comments, when any field has a comment or the one-line form exceeds `MAX_WIDTH`. Members follow the usual groups.
- **Union:** `union Name` block with its named fields sorted like a struct's, never `class`.
- **Trait:** `trait Name<generics> : Supertraits where ...` block, with supertraits in the base list form of rule 10 and a lifetime bound sorted first. All items are public, so there is no `private:`. Groups: associated types (`using Item: Bounds;`), associated constants, static functions, member functions. Default method bodies are dropped. `unsafe trait` and `auto trait` keep their keyword. A trait alias prints as written.
- **Inherent impl:** merged into its type (rule 1), including impls with lifetimes or generics such as `impl<'a> TestVm<'a>`. The self type is matched with its generic arguments ignored.
- **Trait impl:** a special member or a base-list entry on the self type, such as `Type(String&);` for `From<String>` or `serde::Deserialize<'de>` (rule 10). An impl comment goes above the special member it becomes. The self type must be the type itself, with generic arguments ignored. An impl for `&T`, `&mut T`, `Box<T>`, or another wrapper, and an impl whose self type is not declared in this target (a blanket `impl<T> Tr for T`, tuples, slices, arrays, `dyn Trait`, primitives, foreign types), goes in the Other impls section (rule 33), as `implements: Trait for Type;`. For example, `impl IntoIterator for &'a ModelRoles` prints there and not in `ModelRoles`'s base list.
- **Associated constant:** `static constexpr NAME: Type = value;` in the constants group. **Associated type in a trait impl:** not shown, because trait impls print only as base-list entries.
- **Function:** rules 11 to 16. Generic parameters (type, lifetime, const), where clauses, parameter patterns, `mut` parameters, `impl Trait` and `dyn Trait` in any position, closure types, function pointers, and higher-ranked bounds print as written. The break in rule 16 targets the function's own return arrow, found through the `syn` node, never an arrow inside a type.
- **Module-level `const` and `static`:** `constexpr NAME: Type = value;` and `static NAME: Type = value;` (with `mut` kept) in the trailing block. The initializer is shown only when the whole line fits in `MAX_WIDTH` and the value contains no macro call. A value with a macro call is dropped and the item gets a MACRO mark naming the macro. An underscore `const _: ... = ...;` assertion is not shown.
- **Type alias:** its own H3, `using Name<generics> = Type;`.
- **Module:** file module, inline module, nested module, `#[path]` module, `pub(crate) mod`. Each gets an H3 in the Modules section (rule 34), including a module that holds only other modules.
- **`use` declaration:** any `use` with a visibility keyword (`pub`, `pub(crate)`, `pub(super)`, `pub(in path)`) is a re-export. It prints as `using path::Item;` in the trailing block. A group expands to one line per name, a rename prints as `using Alias = path::Item;`, and a glob prints as `using path::*;`. A comment inside a group's braces belongs to the name after it, and a comment at the end of a name's line stays at the end of that name's line. When the `use` itself has comments and binds more than one name, its names print together under those comments, sorted, with no blank line between them, and the group sorts by its first name. Otherwise each name is its own entry. A plain `use` is an import, is not shown, and feeds name resolution. The HTML file links the path of a re-export when it names a type, a trait, a function, a constant, or a static (rules 38 and 39), and the markdown file never links it.
- **Extern block:** `extern "C" { ... }` in the trailing block, holding its functions, statics, and types as declarations. Extern blocks with the same ABI in one module merge into one, with their items sorted like members. `extern crate` is not shown.
- **Attributes:** hidden, except macro marks, a `// cfg:` line for any `cfg` that is not test-only, and the `[[...]]` lines of rule 21. A `cfg` on an `impl` applies to each of its members, and each member gets the `// cfg:` line.
- **`cfg(test)`:** the item is skipped wherever it appears: module, impl, impl member, field, variant, trait item, or `use`. A predicate counts as test-only when it is `test` or an `all(...)` that contains `test`. `any(test, ...)` and `not(test)` are kept. `cfg_attr(test, ...)` is ignored, and any other `cfg_attr` is read for derive and attribute macros.
- **`#[doc(hidden)]` items:** shown like any other, with a `[[doc(hidden)]]` line (rule 21).
- **Identifiers:** raw identifiers such as `r#type` print as written.
- **Not shown, by design:** anything inside a function body (nested items, nested impls, closures, expression macros), because bodies are dropped. Integration tests, benches, examples, and `build.rs` are not targets of the outline.

## Output by example

The markdown file for the parser crate, abridged to one declaration. The kill list line shows the format only, and its name and counts are illustrative. So is the level heading, since where `Prompt` lands depends on what else in the crate names it. The four-backtick fence keeps the inner fence intact.

````
# runvil_parser

- [Level 1: top-level types](#level.1)
  - [prompt::Prompt](#type.runvil_parser.prompt.Prompt)
- [Modules](#modules)
  - [prompt](#mod.runvil_parser.prompt)

**MACROS:** 1 definition, 2 invocations, 4 derive lists
- **MACRO** `macro_rules! example`, in [example](#mod.runvil_parser.example)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.runvil_parser.prompt.Prompt"></a>
### prompt::Prompt

Source: [src/prompt.rs:136](src/prompt.rs#L136), impl [src/parse.rs:14](src/parse.rs#L14), [src/prompt.rs:147](src/prompt.rs#L147)

```
/// A fully parsed prompt file.
struct Prompt : Clone, Debug, Eq, PartialEq
{
    /// Parses the full source text of a prompt file into a Prompt and
    /// returns the events reported during parsing alongside the result.
    ///
    /// The caller supplies execution, an identifier that every event
    /// carries. Every event is reported under task 0 because parsing happens
    /// before any run starts.
    ///
    /// [36 more lines, through the # Errors section]
    static parse(input: str&, execution: str&) ->
        (std::result::Result<Prompt, ParseError>, Vec<Event>);

    /// Returns the parsed frontmatter.
    frontmatter() -> Frontmatter&;
    /// Returns the prompt's title, the text of its required H1 heading.
    title() -> str&;

private:
    /// The parsed YAML frontmatter.
    frontmatter: Frontmatter,
    /// Ordered live Lua and prose blocks from the H1.
    h1_blocks: Vec<Block>,
    /// Top-level sections (H2s) in file order.
    sections: Vec<Section>,
    /// The required H1 title.
    title: String,

    static parse_body(
        frontmatter: Frontmatter,
        shared: Arc<str>&,
        frontmatter_lines: u32,
        emitter: Emitter&) -> Result<Prompt>;
    static parse_inner(input: str&, emitter: Emitter&) -> Result<Prompt>;

    /// The entry-point section: the first top-level section in file order;
    /// the Engine reaches it through crate::detail::entry.
    entry() -> Option<Section&>;
    /// Returns the ordered live Lua and prose blocks from the H1; the Engine
    /// reaches them through crate::detail::h1_blocks.
    h1_blocks() -> [Block]&;
    /// Returns the top-level H2 sections in file order; the Engine reaches
    /// them through crate::detail::sections.
    sections() -> [Section]&;
}
```
````

The `Prompt` block is the real output, except that `parse`'s doc comment is cut short here: the bracketed line stands for its remaining 36 lines, through the Errors section. The hand-written golden for this fixture holds the whole comment, and the check compares against that golden. The `Source:` line is shown as it reads when the output sits in the crate directory. `Prompt` is defined at `prompt.rs` line 136, and its impl blocks start at `parse.rs` line 14 and `prompt.rs` line 147. The HTML file has the same headings, marks, and block text, with any derive mark under the heading outside the block. The difference is that a type name is a link, for example:

```
    sections: Vec&lt;<a href="#type.runvil_parser.prompt.Section">Section</a>&gt;,
```

An all-private type uses `class` and has no label (illustrative, not from the repo):

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

Other constructs (illustrative, not from the repo):

```
using Result<T> = std::result::Result<T, Error>;

struct SectionIndex(usize);

enum ReplyOrigin
{
    Chat,
    Infer,
}

trait Observer : Send, Sync
{
    using Item: Clone;

    static constexpr LIMIT: usize = 64;

    static finish(self) -> Summary;

    flush() mut;
    record(event: Event&);
}

struct Seed : Display
{
    static constexpr from_bytes(bytes: [u8; 32]) -> Seed;

    constexpr as_bytes() -> [u8; 32]&;

private:
    bytes: [u8; 32],
}
```

A module's trailing block, with a public re-export, then a private free function (illustrative):

```
using config::RunContext;

private:
request<const N: usize>(
    lua: Lua&,
    op: str&,
    fields: [(str&, Value); N]) -> mlua::Result<Value>;
```

## Macro marking

- The label is the word **MACRO**. It is bold in the markdown file wherever markdown allows bold, and bold with highlighting in the HTML file. Markdown cannot bold text inside a fenced block, so marks are placed outside fences whenever possible.
- Item-position invocations and `macro_rules!` definitions print a `MACRO` line naming the macro, in the macro group of their module's trailing block, and also appear in the kill list. In markdown the module section also repeats each one as a bold line under the block. With source links on, each kill list entry links to the macro's file and line. With `no-source-links`, the output never names a file or line, because that changes when code moves, so search for the macro's name instead.
- A module-level invocation whose tokens name a declaration in the same module also gets a MACRO line at the end of that declaration's block, since the items it generates usually belong to that declaration.
- `#[derive(...)]` prints `MACRO derive:` with the traits from every derive attribute on the item that rule 10 does not print, merged and sorted, and no mark when none is left. A standard derive with no special member goes in the base list, and a derived `From` or `Drop` stays on the mark. In both files it is a bold line directly under the declaration's H3 heading, outside the block.
- An invocation inside an `impl` or trait body prints its line inside that type's block. In markdown that line is a plain `// MACRO` comment, because it sits inside a fence, and the same macro is repeated as a bold line directly under the block.
- Attribute macros print `MACRO attribute:` with their names, sorted, placed like derive marks. Built-in and tool attributes (`cfg`, `doc`, `allow`, `repr`, `must_use`, `rustfmt`, `clippy`, and so on) never count. A single-segment unknown attribute on an item that also has a derive is treated as a derive helper (`serde`, `clap`, `error`) and ignored. A multi-segment path such as `tokio::main` always counts.
- A macro call inside a shown constant or static value is marked as described in the construct table.
- Macros inside function bodies are not shown, because bodies are dropped.
- `#[cfg(test)]` code is skipped, so test-only macros stay out of the list. Other `cfg` items stay and print a `// cfg:` line.

## Files to create

```
vibelight/
  AGENTS.md             already exists, two approved bullets added
  Cargo.toml  Cargo.lock  LICENSE  README.md
  rust-toolchain.toml   rustfmt.toml   clippy.toml
  .gitattributes        LF for every text file
  .gitignore
  .cargo/config.toml    dev-only compiler flags, not packaged
  build.rs              runs the structure check
  build/structure.rs    the structure rules, shared with the tests
  plans/                plan copies, not packaged
  src/main.rs           clap args, run, write both files per target
  src/root.rs           find the targets and their root files
  src/modtree.rs        walk mod declarations
  src/model.rs          data types
  src/source.rs         per-node slicing and normalization
  src/collect.rs        item dispatch and the UNSUPPORTED fallback
  src/collect_type.rs   struct, enum, union, trait, alias
  src/collect_fn.rs     functions, receivers, qualifiers
  src/collect_use.rs    use, extern, const, static
  src/comments.rs       ordinary comments and attachment
  src/attach.rs         attach impl blocks to types
  src/macros.rs         macro detection, kill list data
  src/resolve.rs        type name to declaration
  src/layout.rs         declarations to styled segments
  src/wrap.rs           the long-line rule
  src/order.rs          the document order
  src/render_md.rs      markdown file
  src/render_html.rs    HTML file
  src/source_links.rs   relative paths and source link text
  tests/it/main.rs      integration tests
  tests/it/structure.rs tests for the structure rules
  tests/fixtures/       source trees with expected.md, three with expected.html
```

Split further wherever a file nears 500 lines. Write each module's unit tests in the same step as the module. Follow [tools-public/rulebooks/rust-rulebook.md](tools-public/rulebooks/rust-rulebook.md): each module starts with a `//!` line, files stay under 500 lines, no `mod.rs`, `[lints]` tables instead of crate-root attributes, `rustfmt.toml` with `style_edition = "2024"`, and a stable `rust-toolchain.toml` like [runvil/rust-toolchain.toml](runvil/rust-toolchain.toml) plus the two components. Lint config starts from [tools-public/crates/png2jpg/Cargo.toml](tools-public/crates/png2jpg/Cargo.toml). [vibelight/AGENTS.md](vibelight/AGENTS.md) already exists with one rule: when executing a plan, copy it into `plans/` as `YYYY-MM-DD-{up-to-three-kebab-words}.plan.md`. Keep that rule and add these two bullets below it, word for word, and nothing else:

```
- When a rendering case is not covered by a written rule, make the output look like a C++ header. A written rule always beats the C++ look. Pin every such choice with a golden test.
- The golden files (`expected.md`, `expected.html`) are the specification. Change one only for an intended output change, in the same commit, after reading its diff. Never regenerate goldens to make a failing test pass.
```

The first execution step is the plan copy, for example `plans/2026-10-25-vibelight-tool.plan.md` with the date of the run, and the structure check then enforces the filename.

## Behavior that needs care

- **Targets.** The argument is a crate directory or one `.rs` file. In a directory, read `Cargo.toml` if present for the package name and the `[lib]` and `[[bin]]` paths and names. Without a manifest, fall back to `src/lib.rs`, `src/main.rs`, and `src/bin/*.rs`. A workspace-only manifest is an error that says to point at a crate directory. Each target gets its own file pair, named after the target:
  - the library: its name, with hyphens turned into underscores (package `runvil-parser` gives `runvil_parser.md`);
  - each binary: its name as written, and a binary whose name matches the library's gets `.bin` before the extension (`foo.bin.md` beside `foo.md`);
  - without a manifest, the library and `src/main.rs` take the directory's name, and each `src/bin/*.rs` its file stem;
  - a single `.rs` file argument: its file stem.

  So a crate with a lib and a bin, such as Runvil's cloud-providers crate, gets two pairs, and ids never collide.
- **Output files.** Each format's output directory is created if it is missing, and existing files of the same name are overwritten. Stdout lists the written paths, sorted. No file is ever deleted (rule 60).
- **Line endings.** Every file is read with CRLF turned into LF before parsing, so spans and slices never hold `\r`. Output is always LF. Apart from source links, it contains no file names or line numbers. Stderr warnings name the file and line, relative to the crate directory with forward slashes.
- **Module walk.** Follow `mod x;` to `x.rs` or `x/mod.rs`, nested files beside their directory, and `#[path = "..."]` relative to the declaring file's directory (inline modules nest the path). [runvil/crates/runvil-internal/engine/src/execute/config.rs](runvil/crates/runvil-internal/engine/src/execute/config.rs) uses `#[path = "config-limits.rs"]`, so this is tested first. A missing file prints a placeholder and a stderr warning and does not stop the run. `include!` is not followed and is reported as a macro.
- **Impl attachment.** Collect every inherent `impl Type` across files. `Prompt` is split between `parse.rs` and `prompt.rs`. Attach by name, preferring the same module, then a unique name across the target. Otherwise it goes to the Other impls section.
- **Per-node slicing.** Never slice a whole signature. Rustfmt writes `Prompt::parse` over four lines with a trailing comma, so collapsing a whole-signature slice gives `parse( input: &str, execution: &str, )`. Slice each parameter, the return type, each generic parameter, each `where` entry, each field and variant type, each base-list entry, and each alias target on its own, and join them with standard punctuation. Inside a slice, collapse whitespace runs to one space, remove whitespace just inside `(`, `[`, and `<` and just before `)`, `]`, `>`, and `,`, and drop a trailing comma before a closing bracket. A comment inside a slice is lifted out and placed by rule 20, so a slice itself never holds a comment. `->`, `=>`, and `::` are never split. The parameter boundaries are also what rule 16's one-parameter-per-line step needs. Spans use `proc-macro2` with span-locations (already used in [runvil/crates/build-xtask/src/facade_shape.rs](runvil/crates/build-xtask/src/facade_shape.rs)). Convert line and character column to a byte offset, and test with multibyte characters. If a span cannot be joined, fall back to printing tokens, with no links for that piece.
- **Type references.** The collector visits the `syn` type nodes of each declaration, records each type path's span, and splits each slice at those spans into plain and type-reference segments. Layout, order, and both renderers read the same segments.
- **Docs.** `///`, `/** */`, and `#[doc = "..."]` all reach the tool as doc attributes, so all three work. Crate docs come from the inner doc comments or from `#![doc = include_str!("x.md")]` when the argument is a literal path.
- **Ordinary comments.** `syn` and `proc-macro2` drop `//` and `/* */` comments, so `comments.rs` reads them from the source text in the gaps between the spans of neighboring nodes in the same scope, skipping string and character literals, and attaches each one by rule 20. A doc attribute and an ordinary comment in the same gap keep the order they were written in. A comment inside a node's span, such as one in a function or macro body, is never in a gap, which is how bodies keep their comments to themselves.
- **Fences.** A doc comment can hold its own code fence. Each markdown fence therefore uses one more backtick than the longest run of backticks that starts any line inside it, and at least three, so a comment can never close the block early.
- **Determinism.** Same aggregate declarations, same bytes, once source links (rule 45) and the stamp (rule 61) are set aside. Every order in the output comes from the declarations themselves (names, paths, and the naming graph), never from file names, line numbers, the module walk, or the order items appear in the source. Every multi-item collection the tool builds is sorted before it is printed: members, base lists, derives, attribute macros, `cfg` lines, re-exports, extern items, modules, other impls, and the kill list. Apart from source links and the stamp, the output holds no timestamps, paths, file names, or line numbers. Maps and sets are B-tree based, which clippy enforces. Stderr warnings are sorted too. You will review these files as diffs, so this matters as much as correctness. The stamp (rule 61) is the one timestamp, and a run leaves a file with the same content untouched, so its stamp moves only with the content.
- **Exit codes.** 0 when the files were written, warnings and skip notes included, or when every checked file matches (rule 59). 1 when the input cannot be used: not a crate, a workspace-only manifest, an unreadable root file, two crates of a run that would write the same file (rule 51), or a config error (rule 56); and 1 when a check finds a problem. Usage errors, including a run that names no crate anywhere, keep clap's own code. Warnings (UNSUPPORTED lines, unresolved or ambiguous names, missing files, and the count of items written more than one way, rule 48) print on stderr with a summary count.

## Dependencies

Confirm each is the intended crate on docs.rs and crates.io before adding it (rulebook section 9). Start from the Runvil workspace pins: `syn` 3 with `full` and `visit`, `proc-macro2` 1 with `span-locations`. Add `clap` 4 with derive, `anyhow`, and `toml` (read the manifest as a plain table). Write the HTML and its escaping by hand, with no template crate. The structure check uses only the standard library, so the crate has no build dependencies. Dev dependency: `tempfile`. The arrangement tests use `syn` (already a dependency) to find item spans and the toolchain's `rustfmt` to reformat, so they need no reprinting crate. Set `rust-version` to the oldest toolchain the dependencies allow, starting from 1.85.

## Execution order

Each step gets what it needs from the steps before it. Ordering runs after layout, because seen order reads the rendered blocks. These groups can run in parallel:

1. **setup**, alone.
2. **structure**, **source-links**, **root-modtree**, and **model-source**, all in parallel. None depends on another.
3. **goldens-constructs**, then **collect** and **comments** in parallel. Both need the parsed files and `source.rs`.
4. **attach-macros** and **resolve** in parallel. Resolve needs the declarations and use maps from collect, not the attached impls.
5. **layout-wrap**, then **goldens-order**, then **order**.
6. **goldens-render**, then **render**, with its two renderers in parallel.
7. **main** and **readme** in parallel.
8. **dogfood**, with the corpus repos in parallel, then **gates**.

## Verification

- **Goldens written first:** every fixture's `expected.md` is written by hand from the rules, each group just before the step that implements it (the goldens todos). Three fixtures also get a hand-written `expected.html`: the `Prompt` case, the macro-heavy crate, and the name-resolution crate. Goldens are rendered with `no-source-links`, except one fixture with links on, whose output is written to a fixed place beside the fixture so its relative links never change.
- **Parity test:** for every fixture, strip the tags from the HTML output, unescape it, and require the same headings, marks, and block text as the markdown file. This is the check that the two files never disagree.
- **Arrangement tests:** every fixture also runs in rearranged forms that must give byte-identical files with `no-source-links`, and byte-identical files once the source links are stripped when they are on. The forms are made by cutting the fixture text at item spans, so each item moves together with its attached comments, and never by reprinting a syntax tree, which would drop the ordinary comments:
  - every file's items reversed;
  - named-struct fields, derive lists, attributes, and `use` lines reversed or regrouped;
  - `impl` blocks split into several, merged into one, and moved to another module;
  - a file module turned inline, and an inline module moved to its own file;
  - the whole tree run through the toolchain's `rustfmt` with a narrow `max_width`, so formatting and line numbers change while comments stay.

  Enum variants, tuple fields, and parameters are never reordered, because their order is part of the declaration. No form moves items out from under a comment that stays put, the one case the determinism rule cannot cover. One more test confirms that changing a declaration does change the output.
- **Structure rule tests,** each against a small temporary tree with a violation and a clean case: a second manifest, a `[workspace]` table, a path dependency, a `src/lib.rs`, a file over 500 lines, a `mod.rs`, a module with no `//!` line, a crate-level lint attribute, a misnamed file in `plans/`, and the skip when `Cargo.toml.orig` exists. The same test file runs the full check over the real tree.
- **Construct fixtures,** one per row of the construct table:
  - structs: named, tuple, and unit; generics with lifetimes, defaults, and const generics;
  - enums with unit, tuple, struct-like, and discriminant variants;
  - a union;
  - a trait with associated types, constants, default methods, and supertraits;
  - inherent and trait impls: `unsafe impl`, a negative impl, a blanket impl, and impls for a reference, a `Box`, a tuple, and a slice;
  - functions: `const fn`, `async fn`, `unsafe fn`, and `extern "C" fn`; free functions with and without `pub`; `impl Trait` and `dyn Trait` parameters; higher-ranked bounds; `where` clauses on one line and on several;
  - a rustfmt-style multi-line signature with a trailing comma;
  - consts and statics with short values, long values, and macro values;
  - type aliases;
  - `pub use` groups, renames, and globs;
  - an extern block;
  - nested and inline modules;
  - `#[cfg(test)]` on a module, an impl, a method, a field, a variant, and a `use`;
  - `cfg_attr` and a `cfg` on an impl;
  - raw identifiers;
  - `/** */` and `#[doc]` comments;
  - CRLF line endings: the test writes a CRLF copy of an LF fixture into a temp directory, because `.gitattributes` keeps committed files LF, and the copy must give the same output;
  - a file `syn` cannot parse.
- **Order tests:**
  - a chain and a diamond, where a type named by a level-1 type and by a level-2 type lands at level 3;
  - two roots with different reach;
  - a two-type cycle and a cycle at level 1;
  - unconnected types;
  - seen order with two first mentions in one block;
  - ties settled by path order;
  - the Modules section listing each module's declarations.
- **Struct versus class tests:** a type with public and private members is a `struct` with a `private:` label, and a type with only private members is a `class` with no label. A type with no members stays a `struct`, an enum with only private methods stays an `enum`, and a tuple struct stays a one-line `struct`.
- **Receiver tests:**
  - `&self` is dropped with nothing added;
  - `&mut self` is dropped and `mut` follows the parameter list;
  - `self`, `mut self`, and `self: Box<Self>` stay as the first parameter, and the function is `static`;
  - `self: &Self` behaves as `&self`;
  - free functions never get `static`.
- **Ordering-within-block tests:** the five groups come in their order, and each group is alphabetical, ignoring case. Ties break by exact spelling, then rendered text. Enum variants and tuple-struct fields keep their declared order, union fields are sorted, and the base list is sorted.
- **Long-line tests:**
  - exactly `MAX_WIDTH` columns is not broken;
  - one column more breaks after `->`;
  - a first line still too long goes one parameter per line, with the tail on the last parameter;
  - no return type plus a long parameter list goes one parameter per line;
  - a long `where` clause moves to its own line;
  - a line that still does not fit stays as it is;
  - fields and comment lines never break, and a trailing comment does not count toward the width;
  - widths are measured with the indent counted and with link tags present in the HTML.
- **Link and anchor tests:**
  - ids are unique, and a duplicate gets a suffix;
  - every `href` and every markdown contents link points at an existing id;
  - every link to a `fn.`, `const.`, or `static.` id lands on an id, and the markdown file holds none of those ids or links;
  - a re-export of a function, constant, or static links in the same crate and across crates, through a rename and a chain of `pub use` items, and a type wins over a function of the same name;
  - names resolve through a prefix path, through a `pub use` chain, through a glob, and through an `as` rename;
  - a same-module name and a unique name resolve;
  - a generic parameter that shares a type's name stays plain;
  - an ambiguous name stays plain and is counted;
  - `<`, `>`, and `&` are escaped in the HTML.
- **Macro mark tests:** in both files, derive and attribute marks sit outside the block under the H3. In the markdown file, derive, attribute, module-level, and kill list marks use a bold `**MACRO**` label outside fences. An invocation inside an `impl` body is a plain `// MACRO` comment in the fence plus a bold line under the block, and an UNSUPPORTED line gets the same treatment. A module-level invocation that names a declaration also marks that declaration's block. In the HTML file every label is bold.
- **Comment tests:**
  - a comment directly above an item and one separated from it by a blank line both attach to the item;
  - a trailing comment stays on its line and does not change where lines break;
  - a comment with nothing after it prints at the end of its block;
  - a comment inside a parameter list puts one parameter per line, with the comment above its parameter;
  - a comment inside a type moves above the field that holds the type;
  - a comment above an inherent `impl` line and one above a trait `impl` line print after the type's own comments, in the stated order;
  - a `///` just inside an impl block, before the first method, stays that method's doc comment;
  - a comment above an impl that goes to Other impls prints above its `implements:` line;
  - an ordinary comment on a `mod` line prints with the module's docs;
  - comments on a plain `use`, a `cfg(test)` item, and a trait-impl member are not shown;
  - comments inside a function body and a `macro_rules!` body are not shown;
  - `/* */` comments print as written;
  - a doc comment and an ordinary comment on one item print in both written orders;
  - doc comments print as `///`, ordinary ones as `//`, and module and crate docs as `//!`;
  - the comment text in the HTML, with tags stripped, matches the markdown's;
  - a whole doc comment that contains its own code fence does not close the markdown fence;
  - a `//` inside a string literal is never taken for a comment.
- **Integration tests:** the `Prompt` case, an all-private class, split impls, a macro-heavy crate, a crate with a lib and a bin, a lib and a bin with the same name, a single `.rs` file argument, a custom `[lib] path` in a temp directory, no manifest at all, and an output directory that does not exist yet. A determinism test runs the binary twice on the same fixture and compares bytes.
- **Source link tests:**
  - stripping the source links, each `Source:` line with its one blank line, from a links-on run gives exactly the `no-source-links` output;
  - the link text is relative to the crate and the target is relative to the output file;
  - an output directory inside, beside, and above the crate all give working links;
  - a different Windows drive falls back to a `file:///` URL, and the extended-length path prefix never appears;
  - in markdown, no link ever lands inside a fence.
- **Corpus run:** every crate under [runvil/crates/runvil-internal](runvil/crates/runvil-internal), then the `rig`, `everruns`, `bashkit`, and `wg21-paperflow` crates, which add async code, unsafe code, and trait-heavy APIs that `runvil-internal` does not use. The repos can run in parallel. Require no panic, no UNSUPPORTED line, no dangling link, and identical output on a second run, and read a sample of each output by eye.
- **Real-code checks:**
  - on the parser crate under [runvil/crates/runvil-internal/parser](runvil/crates/runvil-internal/parser), the `Prompt` block must match its hand-written golden exactly;
  - on the engine crate, the main run type sits in level 1, and one all-private type prints as a `class`;
  - the engine kill list includes `events!`, `lifecycle_constants!`, and `nz!`;
  - opening the HTML in the Cursor browser, a few type links click through, and a few source links open the right file and line.
- **Before any publish:** read the package file list from `cargo package`, which also builds the tarball with the structure check skipped. Check that `plans/`, `.cargo/`, and `AGENTS.md` are not in it. Publishing itself stays with you.

The shell refusals earlier in this chat came from Ask mode's read-only sandbox, so commands should run normally when the plan executes. If they do not, the code can still be written, and the standard fmt, clippy, and test commands would be yours to run.

## Out of scope

None of these is a language construct. Workspace root input, expanded macro output, a public-only filter, a `protected:` section, a `cargo vibelight` subcommand, CI, links across targets or crates or to std docs, HTML features that need JavaScript (a search box, collapsible sections, a theme toggle), and the `repository` field (add it once the git repo exists).
