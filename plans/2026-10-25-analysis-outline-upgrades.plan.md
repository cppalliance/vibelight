---
name: vibelight analysis upgrades
overview: "Self-contained plan for a fresh agent: improve vibelight's outlines for API analysis and add a UTC stamp as the first line of every output file. The work runs in phases 0 to 15, with phases 5 and 7 each split in two, starting with a headroom refactor that changes no output, with goldens written first and one commit per phase. Runvil-specific fixes are out of scope."
todos:
  - id: preflight
    content: "Preflight: read the files listed under Reading order; vibelight tree clean apart from the user's untracked tools/ folder (never stage it); HEAD af499b8 or re-verify line references; baseline cargo test green; copy this plan to vibelight/plans/YYYY-MM-DD-analysis-outline-upgrades.plan.md and commit it"
    status: pending
  - id: headroom
    content: "Phase 0, no output change: in collect.rs move is_public and sort_marks to attrs.rs and delimiter_range and tokens_span to macros.rs, dropping imports left unused; in model.rs move Impl, Special, and SpecialKind to model_impl.rs, re-exported; in document.rs move the Builder methods at 275-440 (section_item, link, mark, code, decl, module, other_impls) to document_sections.rs; at least 30 free lines in each of the three files; all tests pass unchanged"
    status: pending
  - id: stamp
    content: "Phase 1: UTC stamp first in every file (md line 1; html line-1 comment plus the first element of main); stamp.rs with SOURCE_DATE_EPOCH, clock expect, RFC 3339 Z, and a mask of every timestamp in a stamp line; the stamp passes through pipeline::outline to both renderers; placeholder in check mode; write only when masked content changed; check on masked text; is_outline accepts both forms; pin the epoch in the three test helpers; prepend stamps to every golden (subagent sweep); amend the Determinism hard rule and text and rules 58 and 59"
    status: pending
  - id: comment-attach
    content: "Phase 2: impl note groups with // impl headers, each group storing the index of its trait's base-list entry so the header is rendered at layout time from the routed base; commented use groups kept together; comments inside braces attach to the next name; comments-groups fixture; amend rule 20 and the use row of the construct table"
    status: pending
  - id: backticks
    content: "Phase 3: keep backticks in doc comments (intra-doc links become backticked names, link targets still dropped); update affected goldens, the flatten.rs unit tests, and its module doc; amend the links bullet of rule 20"
    status: pending
  - id: labels-receivers
    content: "Phase 4: Vis in model.rs with the conversion in attrs.rs; internal: sections in type and module blocks; vis on Decl and Module; is_static_fn matches only Receiver::None; visibility fixture; amend rules 5, 7, 8, 12, 14, 34"
    status: pending
  - id: surface-declared
    content: "Phase 5a: surface.rs walks pub mod and pub use; entries for directly declared public items, re-exported types and aliases, and re-exported modules; fn, const, and static entries link to their module; Public surface section with a contents entry; surface fixture; every golden with public items gains the section (subagent sweep); new rule; amend rules 31, 35, 36"
    status: pending
  - id: surface-values
    content: "Phase 5b: the surface's own lookup for re-exported fns, consts, and statics (following chains with a cycle guard), glob expansion with its own path walk, exported macro names recorded on the crate, and external re-exports by path; failed lookups listed by path; the surface fixture grows; amend the phase 5a rule"
    status: pending
  - id: attributes
    content: "Phase 6: allowlisted [[...]] lines (non_exhaustive, must_use, deprecated, repr, track_caller, doc(hidden), serde), sorted, read from the cfg_attr-expanded list; tuple and struct-variant fields stay out; Meta.attrs via ..Default::default(), and the declaration Meta copy at layout_decl.rs:22 carries it; attributes fixture (the arrangement tests cover it automatically); amend rule 21"
    status: pending
  - id: unconnected
    content: "Phase 7a: Unconnected counts names used by module items; Order stores each declaration's namers (declarations and modules) for phase 11; update the affected goldens; amend rules 24, 26, 27"
    status: pending
  - id: test-support
    content: "Phase 7b: Decl.test_support set by a pass right after collect, with the same check answering for module items and modules; Test support section; (test support) marks in Public surface; test-support fixture; amend rules 32, 33, 35, 36"
    status: pending
  - id: impl-members
    content: "Phase 8: record each inherent impl's member names and list them after each impl link in the Source line (Builder::decl, links on only); update the links golden; amend rules 44, 45"
    status: pending
  - id: legend
    content: "Phase 9: fixed Legend section (after the H1 and crate docs in both formats, so the parity test sees the same heading order) describing every convention; update every golden (subagent sweep); amend rules 31, 35, 36"
    status: pending
  - id: kill-list
    content: "Phase 10: kill-list switch and kill_list key (full, counts, off); one options struct from the settings through the pipeline into document::build, which phases 11, 13, and 14 extend; args.txt support in the goldens harness; kill-list fixture; amend rules 37, 56, 57"
    status: pending
  - id: facts-lines
    content: "Phase 11: facts flag and key; Node::Para; Counts line; Used by line read from the namers phase 7a stores in Order; Same shapes section; a legend line for the facts lines; facts fixture; new rules; amend rules 35, 36, 56, 57"
    status: pending
  - id: external-items
    content: "Phase 12: External items section under facts, with a map of each foreign declaration's first doc line built before the per-crate loop in pipeline.rs; facts fixture plus a garden workspaces test; new rule; amend rules 35, 36, 54"
    status: pending
  - id: module-order
    content: "Phase 13: order switch and key (levels, modules); module groups holding every declaration; a Levels index with Test support and Unconnected groups; Other impls kept; legend line; order-modules fixture; new rule; amend rules 32 to 36, 56, 57"
    status: pending
  - id: profile
    content: "Phase 14: profile switch and key (analysis) changing defaults only (links off, kill list counts, facts on, order modules); profile fixture, a CLI test without the no-source-links switch showing the profile turns links off, and a config test that explicit settings win; new rule; amend rules 56, 57"
    status: pending
  - id: docs-dogfood-gates
    content: "Phase 15: README (switch forms, profile, precedence, stamp behavior, stdout wording, the three-keys wording, every output change, a note to regenerate committed outlines with this release); dogfood like for like on the reference snapshot source, plus a robustness pass on the live runvil; fmt, clippy, test, package list"
    status: pending
isProject: false
---

# vibelight: analysis upgrades and UTC stamps

## Context for a fresh start

- **What vibelight is:** a single-crate Rust CLI that reads a Rust crate with syn, without building it, and writes a C++-header-style outline as `<target>.md` and `<target>.html`.
  - The outline shows every type, trait, function, and module with its comments, and no function bodies.
  - The repo is at `c:\Users\Vinnie\cursor\vibelight`, and the workspace root is `c:\Users\Vinnie\cursor`.
  - It's at version 0.2.0, on stable Rust (`rust-toolchain.toml` adds rustfmt and clippy), on Windows, in PowerShell.
- **Why this plan exists:** we used a vibelight outline of Runvil's engine crate (`runvil_engine.md`) to review that crate's API. The review found two kinds of problems:
  - places where the outline misleads a reader: comments attached to the wrong item, stripped backticks, and visibility shown only as public or private
  - places where it hides facts an analyst needs: the public surface, attributes, test-support code, impl-block clusters, the notation itself, and derived facts

  Each phase below fixes one group and says why. The user also wants a UTC timestamp at the top of every output file.
- **Paths:** paths are relative to the vibelight repo root unless written in full.
  - Bare file names like `collect.rs` are under `src/`.
  - Test files are under `tests/it/`.
  - Fixtures are under `tests/fixtures/<name>/`, each with `src/lib.rs` and `expected.md`, plus `expected.html` for some.
- **How a run flows**, by file:
  1. `main.rs` reads the switches; `settings.rs` and `config.rs` merge the settings; `inputs.rs` picks the crates.
  2. `pipeline::outline` takes over:
     - `modtree::walk`
     - `collect::collect` (with `collect_*.rs`, `comments.rs`, `attrs.rs`, and `macros.rs`)
     - `resolve::resolve`, then `resolve_run::link` across crates
  3. Then, for each crate:
     - `attach::attach`
     - `layout_decl::decl_block` (with the `layout*.rs` files)
     - `order::order`
     - `spellings::find`
     - `document::build` (with the `document*.rs` files)
     - `render_md` and `render_html`
  4. `main.rs` writes the files, or `check.rs` compares them with what's on disk.
- **Specs, in order of authority:**
  - the goldens (`expected.md` and `expected.html`), which `tests/it/goldens.rs` compares byte for byte
  - the numbered rules 1 to 60 in [vibelight/plans/2026-10-25-vibelight-tool.plan.md](vibelight/plans/2026-10-25-vibelight-tool.plan.md)
  - [vibelight/README.md](vibelight/README.md) for users
  - [vibelight/AGENTS.md](vibelight/AGENTS.md) for process
- **Tests** (all in one binary, run with `cargo test`):
  - `goldens.rs`: every fixture's output against its goldens
  - `arrange.rs`: rearranged sources must give byte-identical output
  - `parity.rs`: markdown and HTML hold the same headings, marks, and blocks
  - `links.rs`: the source-link layer
  - `cli.rs`, `config.rs`, `check.rs`, and `workspaces.rs`: the command line, config files, the check switch, and multi-crate runs
  - `structure.rs`: the build's structure rules
  - `support.rs`: the shared helpers
- **The determinism rule:** apart from source links, the same declarations always produce the same bytes, however the source is arranged (original plan, under Decisions and Determinism). The stamp becomes the second exemption in phase 1.

## Reading order

Read these before starting:
- this plan
- [vibelight/AGENTS.md](vibelight/AGENTS.md)
- [vibelight/README.md](vibelight/README.md)
- the original plan's sections "Decisions already settled" through "Config, per-format output, and checking" (rules 1 to 60), plus "Behavior that needs care"

These are optional, for seeing the problems this plan fixes:
- **The reference outline:** `c:\Users\Vinnie\cursor\cabinet\_scratch\vibelight-runvil\2026-10-25-1731\out\runvil_engine.md`, written by the current build from the runvil snapshot in the sibling folder `c:\Users\Vinnie\cursor\cabinet\_scratch\vibelight-runvil\2026-10-25-1731\src`, not from the live repo, which has moved on since. Both may be gone, because scratch folders are disposable. Useful places in it:
  - line 209: backticks stripped from `Run::new`'s doc
  - lines 4477-4489: `Error`'s doc running straight into an impl's doc
  - line 7417: a `use` group comment attached to `Argv`
  - line 5376: Unconnected types that free functions use

## Scope

- **In scope:** the vibelight changes below, plus a UTC stamp in every output file.
- **Out of scope:** Runvil API fixes, which come later.
- **Already done in the current build, so no work:**
  - standard derives sit in the base list
  - `Drop`, `Default`, and `From` print as special members (for example `~Scheduler();`)
  - the Spellings section exists
- **Decisions the user has made (do not revisit):**
  - The stamp records when a file's content last changed. Unchanged files are not rewritten, and the check ignores the stamp.
  - The stamp comes first in every file.
  - Impl-block grouping lives in the source-link layer, so the determinism rule stands.
  - Every suggestion is included, with the defaults below.

## Ground rules

- **Rule text:** where a phase says a rule is amended, edit that rule in place in the original plan, in the same commit as the change.
  - The original plan is the only numbered rulebook, and AGENTS.md says a written rule beats the C++ look, so it must stay current.
  - This plan's copy in `vibelight/plans/` is the record of what changed.
  - Every new section also amends rule 35 (its id), rule 36 (its contents entry), and the section order (rules 31, 33, and 34). Every new switch or config key amends rules 56 and 57. Each phase names its rules; this bullet is the cross-check.
- **Goldens first:**
  - Edit `expected.md` and `expected.html` by hand from the rule, read the diff, then implement.
  - Never regenerate goldens to make a test pass.
  - The big sweeps (phases 1, 5a, and 9) are split across subagents, a batch of fixtures each, with the garden goldens and `ROOTED` in one batch. Each subagent gets the rule text, edits its batch by hand, and never runs vibelight to produce a golden. The main agent reads the whole diff before the gate.
- **New fixtures:**
  - A new fixture's golden follows every rule in force at its phase, including the rules from earlier phases.
  - The fixture joins the `goldens!` list (`tests/it/goldens.rs:57-95`) in that same phase. Any folder under `tests/fixtures/` that isn't listed fails `every_fixture_has_a_golden_test`, so a fixture source written early waits in a scratch folder outside the repo (`c:\Users\Vinnie\cursor\cabinet\_scratch\vibelight-fixtures\<name>\`) until its phase.
  - Every fixture folder also joins the arrangement and link-stripping tests automatically, through `fixture_names()`.
- **Line numbers:** they come from commit `af499b8`. Phase 0 and every later phase move code, so locate each place by the function or type named next to its number.
- **File limits:** every src file stays at or under 500 lines and starts with `//!`. `build.rs` enforces both on every build and test, through `build/structure.rs`. Phase 0 makes room in the three tight files. `collect_impl.rs` (461) and `collect_use.rs` (455) are tight too, so new logic for them goes in new files.
- **New src files:** each needs its `mod` line in `main.rs` (alphabetical, lines 3-47). There is no other registry.
- **Per-phase gate:** `cargo fmt --all --check`, `cargo clippy --all-targets`, and `cargo test`, run in the repo. `.cargo/config.toml` makes every warning fatal.
- **Commits:**
  - Commit at the end of each phase, with goldens and code together, using the workspace commit rule for the message.
  - Stage files by name, because the untracked `tools/` folder is the user's and is not part of this work.
  - Never tag (AGENTS.md), and never push unless the user says so.
- **Switch names:** switches are typed with two leading dashes; this plan names them without the dashes.

## Defaults

- **On for everyone:**
  - the stamp
  - the comment fixes and kept backticks
  - the access labels and the receiver change
  - Public surface
  - attributes
  - the Unconnected fix and the Test support section
  - impl member lists, when source links are on
  - the legend
- **Opt-in:**
  - kill list `counts` or `off` (phase 10)
  - facts (phases 11 and 12)
  - module order (phase 13)
  - `profile = "analysis"` (phase 14) turns on all three and also turns source links off
- **Switch forms:**
  - `kill-list`, `order`, and `profile` take a value; `facts` is a plain flag.
  - The config keys are `kill_list`, `order`, and `profile` (strings) and `facts` (true or false).
  - An unknown value is an error naming the key and its allowed values.
- **Precedence:** switch, then config, then profile, then defaults. The profile only changes defaults.

## Preflight

- **Read** the files under Reading order.
- **Check the repo:** in `vibelight`, `git status` should show only the untracked `tools/` folder, with `master` at `af499b8`. If HEAD has moved, re-check each phase's line references before starting it.
- **Confirm a green baseline:** run `cargo test`.
- **Copy and commit this plan:** copy it to `vibelight/plans/YYYY-MM-DD-analysis-outline-upgrades.plan.md`, using the date of the run (the structure check enforces this name form), and commit it.

## Phase 0: make room (no output change)

- **Why:** three files are close to the 500-line cap, and later phases grow them.
- **`collect.rs` (493 lines):** move the four free helpers at the end of the file (lines 466-493). `loc` (444-450) and `generic_names` (452-463) stay. Nothing earlier in the file moves, so the plan's other `collect.rs` line numbers stay valid.
  - Move `is_public` (466-468) into `attrs.rs`, where phase 4 turns it into the `Vis` conversion. Its callers (`collect_field.rs`, `collect_fn.rs`, `collect_impl.rs`, `collect_use.rs`) already import `attrs`.
  - Move `sort_marks` (490-493) into `attrs.rs` too. Despite its name it sorts `cfg` lists, as `attrs::cfgs` does. Its callers are `collect_impl.rs:261` and `collect_use.rs:363`, which calls it by full path.
  - Move `delimiter_range` (471-478) and `tokens_span` (481-487) into `macros.rs`, which then needs imports for `Range`, `MacroDelimiter`, and `SourceFile`.
  - Point the call sites at the new homes, or re-export from `collect.rs` where that takes fewer lines. Drop the imports left unused in `collect.rs` (`Visibility`, `MacroDelimiter`, `name_order`), because warnings are fatal.
- **`model.rs` (464 lines):** move `Impl` (251-273), `Special` (201-211), and `SpecialKind` (190-197), with their docs, into a new `model_impl.rs`.
  - Re-export them from `model.rs` with a `pub(crate) use`, as `model.rs:23` does for the `text` types, so no caller changes. The two modules import each other, which Rust allows.
  - Phases 2 and 8 add the impl-note group and the impl location there.
- **`document.rs` (441 lines):** move the `Builder` methods at lines 275-440 (`section_item`, `link`, `mark`, `code`, `decl`, `module`, and `other_impls`) into a new `document_sections.rs`, as an `impl Builder<'_>` block like `document_kill.rs` and `document_spellings.rs` use.
  - Make `pub(crate)` the moved methods that `build()` calls, and what they use: the private fields `links`, `blocks`, `texts`, `nodes`, and `heading`, the `heading()` method, and the free function `anchor()`.
  - The method `other_impls` shares its name with the imported `layout_items::other_impls`, so call that one by path in the new file.
  - `build()` keeps its loops.
- **Target:** at least 30 free lines in each of the three files.
- **Check:** every golden and test passes unchanged.

## Phase 1: UTC stamp, first in every file

- **Why:** the user wants every output file to show, first, the universal time its content last changed.
- **New `src/stamp.rs`:**
  - `now()` returns `SOURCE_DATE_EPOCH` when it is set. A malformed value exits 1 with a message naming what was required and what was found.
  - Otherwise `now()` calls `SystemTime::now()` under an item-level `#[expect(...)]`.
    - `clippy.toml` bans the clock; the structure check bans only crate-level attributes, so an item-level `expect` is allowed.
    - Expect only the lints that actually fire. An unfulfilled expectation is a warning, and warnings are fatal in this repo.
  - UTC formatting is hand-written (days-to-civil), RFC 3339 with `Z`, at seconds precision. No new dependency. Use `try_from` rather than `as` casts, because pedantic cast lints are fatal here.
  - `mask(text)` replaces every timestamp inside a stamp line with a placeholder of the same length, so line numbers stay true. That's one timestamp in markdown, and three in HTML: the comment, the `datetime` attribute, and the element text.
- **Markdown:** line 1 is `*Updated 2026-01-01T00:00:00Z*`, then a blank line, then the crate anchor and the H1.
- **HTML:**
  - Line 1 is `<!-- Updated 2026-01-01T00:00:00Z -->`, placed before `<!DOCTYPE html>` (`render_html.rs:56`). The HTML standard allows comments before the doctype, and they do not switch browsers into quirks mode.
  - The first element inside `<main>` (opened at `render_html.rs:61`) is `<p class="stamp">Updated <time datetime="...">...</time></p>`, styled with the existing color variables.
  - It goes inside `<main>` because `body` is a flex row of `<nav>` and `<main>` (`render_html.rs:14`), and a `<p>` placed directly in `body` would become a third column.
- **Where the stamp comes from:**
  - `run()` in `main.rs` knows from `args` whether this is a check run before it calls `pipeline::outline` (`main.rs:163-169`; the check branch is at 176). So it computes the stamp first and passes it through `pipeline::outline` to both renderers, which are called inside the pipeline (`pipeline.rs:108` and `115-119`).
  - If the stamp goes on `Document`, also update the `Document` literal in the `render_html.rs` unit test (368-381).
  - A check run passes a fixed placeholder instead, so a malformed `SOURCE_DATE_EPOCH` never fails a check.
- **Writing:** `write_files` (`main.rs:233-247`) writes a file only when `mask(old) != mask(new)`. Stdout still lists every file of the run.
- **Checking:** `compare` (`check.rs:23-30`) and `first_difference` (33-43) work on masked text.
- **Stale detection:**
  - Today `is_outline` (`check.rs:65-75`) requires line 1 to be the crate anchor.
  - It must also accept a stamp line, then a blank line, then the anchor.
  - It keeps accepting the old form, so outlines written before this change still count as stale when no crate writes them. The old-form outline at `tests/it/check.rs:107` keeps proving that.
  - Amend rule 59, and the README's stale description changes to match.
- **Tests:**
  - Set `SOURCE_DATE_EPOCH=1767225600` with `Command::env` in the three helpers that launch the binary: `tests/it/support.rs:55`, `config.rs:31`, and `workspaces.rs:22`. The fourth launch, `cli.rs:119`, writes nothing and needs no epoch.
  - Prepend the stamps to every golden: 36 `expected.md`, the 4 multi-target files, 5 HTML files, the 6 garden files, and the inline `ROOTED` golden (`workspaces.rs:194`). Split this sweep across subagents (see Goldens first).
  - `tests/it/cli.rs:55` asserts that a file starts with the crate anchor. It now asserts the stamp line, a blank line, then the anchor.
  - The check test's expected "first difference at line 47" (`tests/it/check.rs:64` and `76`) shifts down by two.
  - Unit tests, on pure helpers (formatting seconds, parsing a value, the mask, and `is_outline` for both forms): epoch 0, 2000-02-29, 2026-01-01, 2100-03-01. Setting an environment variable inside a test is unsafe in edition 2024 and `unsafe_code` is forbidden, so reading the environment is covered only by the integration tests.
  - Integration tests:
    - the check passes when only the stamp differs
    - a second run with another epoch leaves files untouched
    - a changed declaration restamps the file
    - a malformed epoch exits 1 when writing, and a check still runs
- **Spec:** amend the Determinism hard rule (original plan line 83, which lists running twice), the Determinism text (line 461), rule 58 (a normal run now writes only files whose masked content changed, and still prints every path), and the `clippy.toml` comment. The stamp becomes the second exemption, beside source links.

## Phase 2: comment attachment

- **Why:** today an impl block's doc runs straight on from the type's own doc, so a reader can't tell where one ends. A comment above a `use` group lands on whichever name sorts first; in the reference outline, a comment about store operations sits on `Argv`.
- **Impl notes:**
  - Today `attach.rs:97-104` flattens per-impl notes into `decl.impl_notes`.
  - Keep each impl's notes as a group, using a group type in `model_impl.rs`. Each group stores the index of its trait's entry in `decl.bases` (pushed at `attach.rs:94`), or none for an inherent impl.
  - Store the index, not the `Base`. `route::finish` (`attach.rs:113`) rewrites the base-list names after the notes are flattened (104) and never touches `Impl.trait_`, so a stored `Base` would print the trait as written, such as `std::fmt::Display` where the base list says `Display`. At the start of the phase, confirm that nothing reorders `decl.bases` after attach; if something does, store the routed name instead.
  - Print a header line before each group in `layout_decl.rs:20-21`, read from the routed base at layout time:
    - for a trait impl, `// impl` plus the trait, rendered by the same code as the base list (`code(&base.text)`, `layout_head.rs:114`), so short and long names match it (for example `// impl runvil_lua::ErrorValue`)
    - for an inherent impl, `// impl TypeName`
  - The header is its own line kind, not a `Note::Plain`, which can't hold type references. In HTML it takes comment styling, with the trait as a comment reference, mapped the way `note_lines` maps them (`layout.rs:130-135`).
  - Headers apply only to notes left in `impl_notes`. A special member's notes already sit above the member, unchanged.
  - Two commented inherent impls on one type both get `// impl TypeName`. That separates them without telling them apart, which is accepted.
- **Use groups:**
  - Today `collect_use.rs:79-92` gives a group's notes to whichever expanded name sorts first.
  - A commented group prints its names together under the comment, sorted, with no blank line between them. The whole group sorts by its first name.
  - Uncommented groups expand as today.
  - A comment inside the braces belongs to the name after it. `node_notes` (`collect.rs:335-344`) takes every comment inside the `use` span, so handle groups in a new `collect_use_group.rs`, leaving `node_notes` as it is for every other item.
  - `Binding` and `expand()` carry no source spans today, so the new file adds them, recursing into nested groups. `collect_use.rs` has only 45 free lines.
- **Goldens:** comments-impls. Add a new fixture, `comments-groups`.
- **Rule 20** (its "Impls and modules" bullet) and the `use` row of the construct table (original plan line 233) are amended.

## Phase 3: keep backticks

- **Why:** with backticks stripped, parameter names read as prose. For example: "Builds a run of prompt with the arguments args and the context ctx."
- **Flattening:** `flatten.rs` `inline` (126-163) and `link` (173-216) keep backticks as literal text.
  - An intra-doc link `` [`X`] `` becomes `` `X` ``.
  - Markdown link targets are still dropped.
- **HTML:** still links the name inside the backticks.
- **Ragged lines:** lines left short by dropped link targets stay, because rule 20 forbids rewrapping.
- **Edge cases:**
  - A multi-line code span (`flatten.rs:95-102`) keeps a backtick at each end.
  - A link label keeps its backticks, so ``[`items`](Self::items)`` becomes `` `items` ``.
- **Goldens:** comments-kinds, prompt (md and html), spellings (md and html), and spellings-none, the fixtures with backticked docs.
- **Also update** the four flatten unit tests (`flatten.rs:336-384`), which assert that backticks are stripped, and the module doc on line 1, which says so.
- **Rule 20** (links bullet) is amended.

## Phase 4: access labels and receivers

- **Why:** in an internal crate almost everything is `pub(crate)` or `pub(super)`. Treating every non-`pub` item as private makes nearly every type look private. And `static` on a method that takes `self` by value misleads anyone who reads it as C++.
- **The `Vis` type:** add `Vis { Public, Internal, Private }` to `model.rs`. Its conversion from `syn::Visibility` goes in `attrs.rs`, replacing the `is_public` moved there in phase 0:
  - `pub` is Public
  - `pub(crate)`, `pub(super)`, and `pub(in path)` are Internal
  - `pub(self)` and no keyword are Private
- **Model changes:**
  - Replace `public: bool` on `Member` and `Item` (`model.rs:91`, `280`).
  - Update its setters: `collect_field.rs:51`, `collect_impl.rs:155/179/201`, `collect_fn.rs:42`, and `collect_use.rs:95/217/372`.
  - The hard-coded `public: true` sites become `Vis::Public`: `collect.rs:251`, `295`; `collect_use.rs:335`; and `collect_impl.rs:233`, `252`, `309`, `334`, `352`, `374`, `385`, `403`.
  - Renaming the field makes the compiler list every site, including ones not above: the `Parts.public` field in `collect_impl.rs` (22, 159, 183, 205, 450), and the readers `layout_decl.rs:86` (`is_class`) and `174` (`body`), and `layout_items.rs:27` (the extern split) and `36` (`module_block`).
- **Visibility on declarations and modules:**
  - Add `vis` to `Decl` (`collect_type.rs` `head()`, 300-358) and to `Module` (`item_mod`, `collect.rs:258-286`).
  - `head()` gains a parameter; it is already under a `too_many_arguments` expect, and its 6 callers pass the item's visibility. `item_mod` passes the visibility to the child through the `mod_lines` map (`collect.rs:51`, read at 130).
  - Phase 5a reads these fields. Type-level visibility is still not printed, so rule 3 stands.
  - `Use` needs none: re-exports are already `Item`s with a visibility (`collect_use.rs:95`), and a plain `use` never prints.
- **Layout:**
  - In `layout_decl.rs` `body()` (157-255) and `layout_items.rs` `module_block` (13-83), sections print in the order public, `internal:`, `private:`.
  - The first section's label is left out only when it is the default: public in a struct, private in a class.
  - `is_class` keeps its rule: a type with no `pub` member is a class.
- **Receivers:**
  - `is_static_fn` (`model.rs:118-120`) matches only `Receiver::None`. It drives both the member group (`layout_decl.rs:170`) and the `static` keyword (`layout_members.rs:70`), so a by-value `self` method joins the member functions and keeps `self` as its first parameter.
  - Update its doc, and the stale `const` doc on `Receiver::Shared` (`model.rs:53`).
- **Goldens:** receivers, struct-class, traits, references, reexports, prompt, modules, and the other fixtures with `pub(crate)`. Add a new fixture, `visibility`.
- **Rules 5, 7, 8, 12, 14, and 34** are amended.

## Phase 5a: Public surface, declared items

- **Why:** today the crate's real public API shows up only as re-export lines in the Modules section, near the end of the file.
- **New `src/surface.rs`:** walks from the crate root through public modules and public re-exports, using the `vis` that phase 4 adds to `Decl` and `Module`.
  - **Directly declared public items** in public modules are entries: declarations, modules, functions, constants, and statics.
  - **A re-exported type or alias** uses resolve's target: `refs[id].target` on the re-export's `Seg::Ref` (`collect_use.rs:116-129`).
  - **A re-exported module** (`pub use inner as name`) is walked like a `pub mod` under its new path. Resolve leaves its target empty, so look it up through `Scope.children`.
  - Re-exports of anything else wait for phase 5b and are left out until then.
- **New `src/document_surface.rs`:**
  - An H2 "Public surface" (id `surface`) after the kill list, with an entry in the contents.
  - Entries are links in path order. Declarations and modules link to their own headings. Functions, constants, and statics have no headings, so they link to their module's heading.
  - The section is left out when empty, as for binaries.
- **Goldens:** 36 of the 37 fixtures have public items at the root, so nearly every golden gains the section and its contents entry. So do the garden goldens and `ROOTED`, and the check test's "first difference" line number shifts. Split this sweep across subagents (see Goldens first); each section is written from the fixture's source and the new rule.
- **New fixture `surface`.** In this phase it covers:
  - a chain of `pub mod`s
  - a private module whose type is re-exported
  - a re-exported module
  - public functions, constants, and statics declared directly
- **A new rule** covers the section. Rules 31, 35, and 36 are amended.

## Phase 5b: Public surface, re-exported values, globs, and macros

- **Why:** re-exported functions, constants, and statics, globs, and exported macros are part of the API too, but resolve gives them no target.
- **A re-exported function, constant, or static** is looked up by name among the target module's items (`Module.items`), because resolve's `Scope.decls` knows only types and aliases.
  - When the target module re-exports the name in turn, follow that re-export, with a cycle guard.
  - A lookup that fails is listed by its path, with no link.
- **A glob re-export** carries no reference at all (its text is plain `path::*`). So the surface resolves the path itself (`crate::`, `self::`, `super::`, or relative, through `Scope.children`) and expands it to the target module's public items, with a cycle guard.
- **A re-export of another crate's item** is listed by its path.
- **`#[macro_export]` macros:**
  - `macros.rs` gains the check for the attribute; its `BUILTIN` list already names it (`macros.rs:45`).
  - `item_macro` (`collect.rs:211`) records each exported macro's name in a new `Crate.exported_macros` list. The only place that builds a `Crate` is `collect.rs:110-120`.
  - These macros are nameable at the root.
- **Links:** re-exported values and macros link to their module's heading, as in phase 5a.
- **The surface fixture grows:**
  - re-exported functions, constants, and statics, including one reached through a chain
  - a glob
  - a `macro_export` macro
  - an external re-export
- **Goldens:** reexports, macros, and any other fixture with these re-exports.
- **The phase 5a rule** is amended.

## Phase 6: API attributes

- **Why:** `non_exhaustive`, `must_use`, `deprecated`, `repr`, and serde attributes shape an API and its wire format. Today none of them is printed.
- **Allowlist in `attrs.rs`:**
  - The list is `non_exhaustive`, `must_use`, `deprecated`, `repr`, `track_caller`, `doc(hidden)`, and `serde(...)`.
  - Each prints as written inside `[[...]]`, sorted, one per line, after the `// cfg:` lines.
  - Read attributes from the `cfg_attr`-expanded list that `macros.rs` already builds (`effective`, 87-108, made `pub(crate)`). It returns `syn::Meta`, which shares a name with the model's `Meta`, so name it by path at the call sites. An attribute from a `cfg_attr` prints without its condition.
- **Model:**
  - Add `Meta.attrs` (`model.rs:37-45`).
  - `Meta` derives `Default`, so replace the nine literals that spell out every field with `..Default::default()`.
- **Capture and printing:**
  - Capture attributes on declarations, named fields, variants, members, and module items.
  - Tuple fields and struct-variant fields stay out, matching how their `cfg` lines are handled today.
  - Print them through `layout.rs` `above()` (179-187) and through the declaration `Meta` at `layout_decl.rs:22-26`, which today copies only `cfgs` and must copy `attrs` too. The other `..Meta::default()` sites (tuple fields, struct-variant fields, parameters, and later bindings of a re-export group) rightly drop them.
  - The `macros.rs` `BUILTIN` list still keeps these attributes out of the kill list.
- **Goldens:** prompt (md and html: `non_exhaustive`, `must_use`), unions (`repr(C)`), and macros (md and html: the `serde` helper). Attributes outside the allowlist, such as `allow`, `default`, `path`, and `doc`, stay hidden, so member-order, special-members, modules, and doc-forms don't change.
- **New fixture `attributes`:** it holds every allowlisted form, plus one attribute that stays hidden. The attribute-reversal arrangement test (`arrange.rs:152-180`) runs on every fixture, so it covers the new one with no edit. That test is why the lines print sorted.
- **Rule 21** is amended.

## Phase 7a: Unconnected fix

- **Why:** "Unconnected" lists types that free functions use, which falsely suggests dead code. The reexports golden already shows `inner::Thing` and `inner::Other` there.
- **Unconnected:**
  - In `order.rs`, before line 58, mark the types named by free functions, constants, statics, extern items, and re-exports.
  - Read them through the `&Crate` that `order()` already receives; no new plumbing is needed.
  - Read only the items' own texts in `Module.items`, not their notes. Re-export references are flagged `comment: true`, so ignore that flag here. Doc-comment references carry the same flag, and they stay out because they live in the notes.
  - Those types move to level 1 instead of Unconnected.
- **Keep the namers:**
  - `Order` (`order.rs:10-15`) holds only `levels` and `unconnected` today.
  - Add each declaration's namers to it: declarations from the naming graph, and modules whose items name it.
  - Phase 11 reads them, so nothing is computed twice.
- **Goldens:** functions, reexports, and any fixture whose types are named only by free functions, constants, statics, or re-exports.
- **Rules 24, 26, and 27** are amended.

## Phase 7b: Test support

- **Why:** test-support types, such as code behind `cfg(any(test, feature = "test-support"))`, sit at the top levels mixed in with production types. In runvil, `ScriptedChat` is test support only through its enclosing module (`engine/src/lib.rs:25`).
- **Test support classification:**
  - A new pass in `pipeline.rs`, right after collect (between lines 64 and 65), re-parses the stored cfg text of each declaration and of its enclosing modules (`Module.cfgs`, `model.rs:345`) with syn. The unit tests that call `collect::collect` directly don't run it, and they don't need it.
  - A cfg that lists `test` inside `any(...)` marks test support. `not(test)` does not, and test-only cfgs are already skipped.
  - The pass sets a new `Decl.test_support` flag. The same check answers for module items and modules, which the surface marks need.
- **Placement:**
  - Test-support declarations leave the level graph. The names they use still count for the Unconnected test from phase 7a.
  - They print in path order under an H2 "Test support" (id `test-support`) after the last level, with an entry in the contents.
  - Public surface entries for test-support items get ` (test support)` after the link.
- **Goldens:** cfg-test, and any fixture with an `any(test, ...)` cfg. The surface fixture gains a test-support item.
- **New fixture `test-support`.**
- **Rules 32 and 33** are amended, and rules 35 and 36 for the new section.

## Phase 8: impl member lists in the source layer

- **Why:** members of all impl blocks are merged and sorted by name, which hides how a large type's methods cluster by file. The user chose to show that cluster information only in the source-link layer, so the determinism rule still holds.
- **Recording:** at `attach.rs:45` (where `loc` is captured) and `55-59`, record each inherent impl's member names with its location. Take the names from `members` before `decl.members.extend(members)` at line 59. Use an impl-location type in `model_impl.rs` (a location plus sorted names) in place of the plain `impl_locs` list (`model.rs:246`, read only in `Builder::decl`).
- **Printing:**
  - In `Builder::decl` (in `document_sections.rs` after phase 0; `document.rs:328-340` today), each impl link in the `Source:` line gets ` (name, name, ...)` after it. Trait impls add nothing.
  - Output with links off is unchanged. The strip helpers (`support.rs:235-238`, `256-258`) already drop the whole line.
  - A type with many impls gets a very long `Source:` line. That's the cost of rule 45's one-line form, which is accepted.
- **Golden:** links (`expected.md:41`).
- **Rules 44 and 45** are amended.

## Phase 9: legend

- **Why:** the notation is explained only in the README. Someone reading the file on its own, often an AI, has to reverse-engineer it.
- **New `src/legend.rs`:** fixed text, about 14 lines, under an H2 "Legend" (id `legend`), with an entry in the contents.
- **Placement:**
  - In markdown, it follows the H1 and the crate docs and comes before the contents list.
  - In HTML, the contents are the `<nav>` sidebar, so the legend goes inside `<main>` after the H1 and the crate docs, before the kill list. It must not come before the H1, because the parity test compares heading order between the formats (`tests/it/support.rs:158` and `184`).
- **Text constraints:**
  - The text is the same with source links on or off, or the links-separation test (rule 45) fails.
  - No legend line in markdown starts with `#`, `**MACRO**`, or `**UNSUPPORTED:**`, and no HTML line starts with `<h1` to `<h3` or `<p class="mark"`, because the parity parser reads those as headings and marks.
- **It covers:**
  - `struct` and `class`, and the three access labels
  - receivers, including `self` as a first parameter
  - reference and pointer spelling
  - `constexpr` and `using`
  - special members and the base list
  - `// cfg:`, `[[...]]`, `// impl`, and MACRO
  - levels, Unconnected, and Test support
  - Public surface and its ` (test support)` marks
  - `Source:` lines with their member lists
  - the stamp on the first line
- **Goldens:** every golden changes (split the sweep across subagents, see Goldens first), and the check test's line numbers shift.
- **Rules 31, 35, and 36** are amended.

## Phase 10: kill list modes

- **Why:** the full macro kill list is noise for someone analyzing an API.
- **New setting:** `kill_list` (`full`, `counts`, or `off`), as a switch and a top-level config key. The code sites are:
  - `main.rs`: `Args` (70-115) and `flags()` (135-148, which lists every `Flags` field)
  - `settings.rs`: `Flags` and `plan()` (13-26, 59-115)
  - `config.rs`: the top-level key allow-list (79-83)
- **Plumbing for every later setting:** `document::build` (`document.rs:103-110`) takes only the links setting today.
  - Add one options struct to `settings::Plan`, passed through `pipeline::outline` and `Parts` (`pipeline.rs:140-174`) into `document::build`. Phases 11, 13, and 14 add their fields to it.
  - Every new setting is run-wide, so the HTML build that reuses the markdown document when both outputs match (`pipeline.rs:71`, `90`, `110`) stays correct.
- **Modes:**
  - `counts` empties the entries (`document.rs:185`).
  - `off` also drops the summary line (`render_md.rs:20-21`, `render_html.rs:71-78`).
- **Goldens harness:** an optional `args.txt` in a fixture adds switches (`goldens.rs:9-36`, `support.rs:54-61`).
- **New fixture `kill-list`.**
- **Rules 37, 56, and 57** are amended.

## Phase 11: facts lines and Same shapes

- **Why:** member counts and back-references point straight at oversized types and unused ones. Declarations with identical shapes are candidates for merging; the review found two enums that were exact duplicates.
- **Setting and node:** a `facts` flag and config key, a field on the phase 10 options struct. Add `Node::Para` (`document.rs:51-72`), rendered in both renderers, which match on `Node` exhaustively (`render_md.rs:24-55`, `render_html.rs:80-125`).
- **New `src/document_facts.rs`:** two lines under each H3, after `Source:`.
  - A `Counts:` line: variants, fields (with how many are `Option`), associated items, special members, and functions. Nothing on it depends on arrangement.
  - A `Used by:` line linking the declaration and module namers that phase 7a stores in `Order`.
- **New `src/document_shapes.rs`:** an H2 "Same shapes" (id `shapes`) at the end of the document, shown when facts are on.
  - It lists groups of two or more declarations of the same kind with the same variant or field list, comparing names and types and ignoring docs.
  - Each list must have at least two entries.
- **Legend:** gains a line for the facts lines when facts are on.
- **New fixture `facts`,** covering both the facts lines and Same shapes.
- **New rules** cover the facts lines and Same shapes. Rules 35, 36, 56, and 57 are amended.

## Phase 12: External items

- **Why:** names from other crates show up in signatures with no definition, so the reader can't tell what they are.
- **The section:** an H2 "External items" (id `external`), shown when facts are on.
  - It lists every non-std item named in signatures, grouped by crate, with use counts and users.
  - It groups by crate from `TypeRef.canonical` (`text.rs:99-108`), as `spellings::find` does. `spellings::find` never reads `foreign`; this section reads it to find the declaration behind a name from another crate of the run.
- **Doc lines from other crates:**
  - The per-crate loop in `pipeline.rs:73-92` borrows each crate mutably for `attach`. So before that loop, build a map from each foreign declaration to its first doc line, and pass it to `Parts::build`.
  - With facts on, the markdown therefore depends on which crates are in the run, and the README says so.
- **Tests:** the facts fixture covers single-crate behavior. A workspaces test on the garden covers doc lines from another crate of the run.
- **Spec:** a new rule covers the section. Rule 54, which says every markdown file is the same whatever crates are in the run, gains the facts exception. Rules 35 and 36 are amended.

## Phase 13: module order

- **Why:** level order scatters each module's types across the file.
- **Setting:** `order` (`levels` or `modules`), as a switch and a config key, a field on the phase 10 options struct.
- **New `src/document_modules.rs`:**
  - One H2 per module, in path order (id `mod.PATH`). Each holds the module's docs, its declarations as H3s (by level, then today's order within a level), and its trailing block.
  - Every declaration stays in its module, Test support and Unconnected types included.
  - An H2 "Levels" (id `levels`) indexes the declarations by level, with Test support and Unconnected as its last two groups.
  - Other impls keeps its own section after the modules.
  - The separate Modules section is dropped in this mode.
  - Type ids stay `type.PATH`, so links still work.
- **Legend:** gains the module-order variant of its levels line.
- **New fixture `order-modules`.**
- **Spec:** a new rule covers module order. Rules 32 to 36, 56, and 57 are amended.

## Phase 14: analysis profile

- **Why:** one switch should give analysis readers the right settings together.
- **Setting:** `profile` (`analysis`), as a switch and a config key.
- **What it does:** it changes defaults only, turning source links off in both formats, the kill list to `counts`, facts on, and order to `modules`. Explicit switches and config keys still win.
- **Tests:**
  - A new fixture `profile`, whose `args.txt` holds the profile switch.
  - The goldens harness always passes the `no-source-links` switch, so it can't show the profile turning links off. Add a CLI test that runs without that switch.
  - A config test showing that an explicit setting beats the profile.
- **Spec:** a new rule covers the profile. Rules 56 and 57 (its key and the precedence) are amended.

## Phase 15: docs, dogfood, gates

- **README:**
  - Document every new switch with its form, the config keys, the profile, and the precedence. Update the "three keys" wording (README lines 95 and 100).
  - Document every change in "What the output looks like".
  - Document the stamp in "Checking the files" and "Using it in a repository": it is the first line, it changes only when content changes, the check masks it, and stale detection accepts it before the crate anchor.
  - Stdout lists every file of the run, written or already current.
  - Add a release note: this release changes every outline (the stamp and the legend), so committed outlines must be regenerated with it. The version bump and the tag stay with the user, per AGENTS.md.
  - The legend and the README must agree.
- **Dogfood:**
  - Build with `cargo build --release`, which gives `c:\Users\Vinnie\cursor\vibelight\target\release\vibelight.exe`. Use that full path below.
  - **Like for like:** from the reference snapshot `c:\Users\Vinnie\cursor\cabinet\_scratch\vibelight-runvil\2026-10-25-1731\src`, run the binary on the crate path `crates/runvil`, with the `follow` switch set to `crates/runvil-internal/*`, and `-o` pointing at `c:\Users\Vinnie\cursor\cabinet\_scratch\vibelight-dogfood\out`. That's the same source and crate set as the reference outline. If the snapshot is gone, run from `c:\Users\Vinnie\cursor\runvil` instead and compare by item rather than by line.
  - A second identical run rewrites nothing, and the stamps hold.
  - The `check` switch passes.
  - Repeat with the analysis profile, into a sibling folder.
  - As a robustness pass, run once from `c:\Users\Vinnie\cursor\runvil` with the `workspace` switch over the whole live workspace, into another sibling folder: no panic and no UNSUPPORTED lines.
- **Confirm in the new `runvil_engine.md`,** comparing against the reference outline when it still exists:
  - the `// impl` header on `Error`
  - the lua module's group comment above its group
  - backticks kept
  - `internal:` labels
  - Public surface lists the crate-root re-exports
  - `[[non_exhaustive]]` on `RunResult`
  - `ScriptedChat` under Test support
  - `Laid`, `JumpTarget`, and `Visible` out of Unconnected
  - `Scheduler`'s `Source:` line with member lists
  - `ChatDispatch` and `ToolCallDispatch` under Same shapes (profile run)
  - External items present (profile run)
- **Gates:**
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets`
  - `cargo test`, which includes the structure check and its 500-line cap
  - `cargo package --list`

## Order, data flow, and parallelism

- **Order:** preflight and phase 0 run alone and first. Then 1, 2, 3, 4, 5a, 5b, 6, 7a, 7b, 8, 9, 10, 11, 12, 13, 14, and 15, one at a time, because goldens are the spec and almost every phase edits shared ones.
  - The stamp comes before every new fixture, so the fixtures include it from the start.
  - The legend comes after every change it describes, so its text is written once. Phases 11 and 13 add their mode-specific lines.
  - The settings phases come last, and the profile comes last of all, because it switches on the features before it.
- **Data flow:** every phase's inputs exist when it runs.
  - Phase 0 makes room for the code that phases 2, 4, and 8 add.
  - Phase 2 reads trait names from `decl.bases` at layout time, after `route::finish` has routed them.
  - Phase 4 adds `vis` to `Decl` and `Module`, which the phase 5a walk reads.
  - Phase 7a stores the declaration and module namers in `Order`, which phase 11 reads.
  - Phase 7b provides the test-support flag, which its own surface marks and phase 13 use.
  - Phase 10 adds the options struct that phases 11, 13, and 14 extend.
  - Phase 12 builds its own cross-crate doc map.
- **Parallel work:**
  - After phase 1, write the new fixtures' sources in parallel into the scratch folder (see New fixtures): `comments-groups`, `visibility`, `surface`, `attributes`, and `test-support`. Sources don't change with later rules. Each moves into `tests/fixtures/` in its own phase.
  - Write each golden in its own phase, not early, because phases 4, 6, 7a, 7b, and 9 would change early goldens again.
  - Split the golden sweeps in phases 1, 5a, and 9 across subagents (see Goldens first).
  - Code, commits, and gates stay serial.

## Execution notes

What the run did differently from the text above, and why:

- **Fatal warnings moved two fields.** An unread field fails the build, so `vis` on `Decl` and `Module` landed in phase 5a, where the surface walk reads it, and the namers in `Order` landed in phase 11, where the `Used by:` line reads them. Phase 7a added `order::module_mentions` alone.
- **Identical sweeps were scripted.** The stamp (phase 1) and the legend (phase 9) are the same text in every golden, so each was written once by hand from its rule and inserted with a script, then every golden was checked against the build. Phase 5a's per-fixture sections were written by five subagents from the rule, and all 38 matched the build on the first comparison.
- **Single `.rs` files have no Public surface.** Such a target is not a library (`Target.lib` is false), so rule 62 covers library targets only.
- **A module's trailing block comes before its declarations** in the module order (phase 13), because a reader takes what follows an H3 as part of that declaration.
- **The legend varies by option.** Its macro line follows `kill_list`, its levels line follows `order`, and the facts add one line.
- **`document::build` takes `document::Inputs`,** so the options did not push it past the clippy argument limit; several long functions were split for the clippy line limit.
- **The parity tests read `args.txt` too,** so the heading, mark, and id checks cover every mode.
- **Dogfood** ran on the reference snapshot: a second run rewrote nothing, both checks passed, and every item in the confirm list held. One Runvil finding is out of scope here: in `engine/src/lua.rs`, the comment about store operations sits above the large `pub(crate) use runvil_lua::{...}` group but describes the next line.
