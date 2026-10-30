---
name: Vibelight Cleanup Plan
overview: "Vibelight run on the vibelight crate: outline it, verify every finding against the code and docs, and leave a self-contained Cleanup Plan. The scope is resolved from vibelight.toml, and the plan fills in as findings are verified."
todos:
  - id: resolve-scope
    content: Resolve the target from vibelight.toml and fill in the scope in both plan copies
    status: completed
  - id: outlines
    content: Check outline freshness and schema; run plain vibelight when stale
    status: completed
  - id: analyze
    content: Dispatch analysis and drift subagents in parallel
    status: completed
  - id: verify
    content: Dispatch verifiers and write the Debt Inventory as findings settle
    status: completed
  - id: refine
    content: Refine the plan against the code and accept supported changes
    status: completed
isProject: false
---

# Vibelight: vibelight

<product-contract>

## Product Requirements

### Scope

- **Crate:** `vibelight` 0.2.0, `.`, binary target only, outline 1106 KB
- **Outlines:** rerun 2026-10-30, the first run here also wrote `vibelight.toml`
- **Context:** 0 other outlines, not analyzed
- **Excluded:** 0 crates, the config's `exclude` is empty
- **Doc roots:**
  - the operator named none
  - the repository's `README.md` and `AGENTS.md`
  - 10 dated plans in `plans/`, 2026-10-25 to 2026-10-30
  - 1 research note in `research/`
- **Limits:**
  - test, build, example, and doc test source adds qualified-path and lexical token hits
  - that source has no catalog records
  - macro-generated code is not counted
  - dynamic trait dispatch and external consumers are settled by search
  - the run warned that 6 items are written more than one way, see the spelling records
  - the dated plans from 2026-10-25 and 2026-10-26 were compared in a second pass, and findings from both passes are merged below
  - nothing was built or run: every statement about run-time behavior is read from the code

- Stats: 219 findings (199 raised, 20 added by verifiers), 141 accepted, 36 rejected, 42 merged, 101 cheap, 40 substantial, 0 operator decisions required (6 were answered by taking the recommended option, see the Decision Record)
- Goals: remove stored copies, duplicated rules, and unread fields, and make every doc comment and the README say what the code does
- Non-goals: no change to the wire format, the printed outlines, or any golden, except the two legend lines in the legend item
- Success criteria:
  - `cargo fmt --check`, `cargo clippy --all-targets --locked`, and `cargo test --locked` pass
  - `cargo +1.85 check --all-targets --locked` passes, which is the minimum Rust version that `Cargo.toml` declares and CI checks
  - every golden under `tests/fixtures` is unchanged except the legend lines named in the legend item
  - a fresh plain `vibelight` run at the repository root shows each declaration and comment change in `.vibelight`, and prints no new warning

## Functional Specification

### Debt Inventory

Each line names the debt, its main site, and the finding tag. The work items below hold the steps, so nothing here is an instruction.

State, stored copies of facts the code already derives:

- `AnalysisRun::limits` stores a list that `census.sources` already tells; two checks keep it in step. Site: `src/analysis.rs:219`, `src/census_check.rs:63` (S2-1)
- `NameCount::total` is always the sum of `by_place`. Site: `src/census.rs:112`, `src/census_check.rs:92` (S2-2)
- `PackageInfo::targets` repeats what each target's `package` already says. Site: `src/analysis.rs:46`, `src/analysis_check.rs:57` (V1-2)
- `Via::Unresolved` always equals `item.is_none()`, and only a debug check reads `via`. Site: `src/analysis_paths.rs:174` (V1-3)
- `count.uses() <= count.total` cannot fail. Site: `src/analysis_check.rs:416` (S2-11)
- `Start::at` repeats the position of the start token. Site: `src/census_paths.rs:22` (S2-10)
- `Body::file` repeats `Body::loc.file`, with a guard that keeps them equal. Site: `src/model_body.rs:77`, `src/collect_check.rs:96` (S6-5)
- `Target::lib` is `kind == TargetKind::Lib` stored twice, with a runtime check that they agree. Site: `src/root.rs:23`, `src/collect_check.rs:54` (S8-4)
- `Binding` stores its path twice. Site: `src/collect_use_group.rs:17` (S2-16)
- `Inputs::printed` is always the same slice as `Inputs::blocks`. Site: `src/pipeline.rs:249`, `src/document.rs:132` (V2-1)
- `ImplLoc` has four fields that nothing reads, hidden by a derived `PartialEq`. Site: `src/model_impl.rs:40`, `src/attach.rs:64` (S6-4)
- `DepEntry::workspace` has no reader outside a derived `PartialEq`. Site: `src/manifest_facts.rs:68` (V6-2)
- `ManifestFacts::validate` compares dependency entries twice, and the second comparison is implied. Site: `src/manifest_facts.rs:294` (S6-11)
- `Sig::self_text` is set for two of four `Receiver` values, and the type does not say so. Site: `src/model.rs:74`, `:98`, `src/collect_fn.rs:94` (S6-6)
- `Outline` and `Facts` each repeat the two fields of `AnalysisTarget`, and `krate` is a third copy. Site: `src/render_slop.rs:26`, `:139`, `src/render_slop_facts.rs:109` (S7-11)
- `Entry` allows rank states that `lift` has to repair after `push`. Site: `src/render_slop.rs:55`, `src/render_slop_items.rs:122` (S7-12)
- `MoreRecord` holds `items` and `text` as two options, and `LeadRecord::module` and `count` are two options that must be set together; the decision is pending. Site: `src/render_slop_schema.rs:224`, `:324` (S7-14, S7-15)
- A resolved type name holds three exclusive options as three fields. Site: `src/resolve.rs:32`, `src/text.rs:172`, `src/route.rs:115` (S8-3)
- Four HTML option enums and `Options` have one value each, and every `match` on them has one arm. Site: `src/settings.rs:13`, `src/document.rs:114` (S8-5, S2-12)
- The order-by-impl plumbing in `layout_decl` can never run, because `impls` is always empty. Site: `src/layout_decl.rs:57`, `:105`, `:251` (V11-2)

Duplication, rules written twice or more:

- The rule for which type an impl belongs to is written twice. Site: `src/attach.rs:26`, `src/analysis_items.rs:64` (S2-5)
- `test_only` is written twice with identical bodies. Site: `src/attrs.rs:19`, `src/macros.rs:111` (S2-6)
- `resolve` returns an unnamed triple that two loops rebuild into an `Edge`. Site: `src/analysis_paths.rs:163`, `:127`, `:145` (S2-7)
- Three readers parse the `macro_rules! ` prefix that the producer builds. Site: `src/collect.rs:256`, `src/census_sources.rs:264`, `src/leads_names_modules.rs:80`, `src/render_slop_items.rs:346` (S2-9)
- Three keyword tables list Rust's keywords with different contents. Site: `src/census_lex.rs:17`, `src/collect_body_tokens.rs:14`, `src/leads_comments_words.rs:65` (S2-1, S4-3)
- The lead and trail comment pair travels through 19 signatures with no type. Site: `src/comments.rs:63`, `src/collect_type.rs:42`, `src/collect_use.rs:29` (S2-2)
- The `attribute: ...` and `derive: ...` mark text is written in eight places and parsed in two. Site: `src/collect.rs:439`, `src/document_kill.rs:98`, `src/render_slop_items.rs:204` (S2-4, S8-12)
- `written_value` restates the decision `value` makes. Site: `src/collect_trait.rs:320`, `src/collect_use.rs:212` (S2-5)
- `check_ref` takes an `Option<bool>` that no caller leaves `None`. Site: `src/collect_check.rs:278` (S2-6)
- `member_refs` sits in a checker module but the analysis calls it too. Site: `src/collect_check_nodes.rs:49`, `src/analysis_uses.rs:349` (S2-9)
- The document builder passes unnamed tuples for marks and the kill list. Site: `src/document.rs:119`, `src/document_kill.rs:12` (S2-14)
- `MIN_TOKENS` and `SHINGLE` are defined in the producer and again in the checker. Site: `src/collect_body_tokens.rs:25`, `src/collect_check.rs:17` (S2-7)
- Two functions resolve the module that declares an item, and three more repeat their impl arm. Site: `src/leads.rs:177`, `src/leads_census_place.rs:48`, `src/leads_names.rs:154`, `src/leads_items.rs:263`, `src/leads_docs.rs:287` (S3-11)
- `sig_texts` is written four times, and the census copy reads only parameters and return type. Site: `src/render_slop_facts.rs:100`, `src/collect_check_nodes.rs:60`, `src/order.rs:320`, `src/leads_census_place.rs:80` (S3-12, S2-3, S6-3, S7-19)
- The dependency table names are written in four modules with different spellings. Site: `src/manifest_facts.rs:83`, `src/leads_census_deps.rs:33`, `src/scaffold.rs:209`, `src/manifest.rs:219` (S3-13, S6-10, S8-16)
- `relative` in `inputs` closes a cycle with `exclude`, and `shown` is written three times. Site: `src/exclude.rs:7`, `src/inputs.rs:211`, `src/pipeline.rs:270`, `:293` (S3-14, S2-10, S7-7)
- The `papercut` pass walks every documented node a second time. Site: `src/leads_comments_docs.rs:30`, `src/leads_docs.rs:133` (S4-8)
- The nine standard derive names are listed twice. Site: `src/leads_census_scope.rs:44`, `src/route.rs:16` (S4-10, S7-10, S8-7)
- The module-parent rule is written twice, and the shared copy sits in the detector layer. Site: `src/leads_items.rs:365`, `src/surface.rs:307` (S5-11, S8-15)
- `ItemId` has no constructor, and its `Decl` literal is written 34 times. Site: `src/analysis_items.rs:44`, `src/leads_shapes_twins.rs:118` (S5-12)
- The evidence format `name: type` is written nine times, and `typed` already exists. Site: `src/leads_docs.rs:327`, `src/leads_shapes_params.rs:172`, `src/leads_shapes.rs:103` (S5-13)
- `run_of` takes a byte that every caller sets to a backtick, and two loops walk `catalog.fields` in runs of one owner by hand. Site: `src/leads_docs_scan.rs:157`, `src/leads_shapes.rs:153`, `src/leads_shapes_twins.rs:161` (S5-16, V5-1)
- The wire-type rule is written in two layers and a third inline test. Site: `src/render_slop_wire.rs:144`, `src/leads_items.rs:473`, `src/leads_shapes_traits.rs:182` (S5-9, S5-10, S8-6)
- `strings` is written twice, in `manifest` and `manifest_facts`. Site: `src/manifest.rs:123`, `src/manifest_facts.rs:254` (S6-2)
- `Workspace::deps` holds a tuple that `Dep` already names, in the reverse field order. Site: `src/manifest.rs:37`, `:136`, `:256` (S6-22)
- `modtree` passes the same two directory strings through five functions and writes the children-directory rule twice. Site: `src/modtree.rs:105`, `:133`, `:206`, `:226` (S6-12)
- `name_order` writes `strip_raw` again as a closure. Site: `src/model.rs:442` (S6-23)
- `render_prose::escape` is `render_html::escape` written again, and `render_slop_lists::under` is `render_slop_items::under` with a module id in front. Site: `src/render_prose.rs:217`, `src/render_html.rs:446`, `src/render_slop_lists.rs:56`, `src/render_slop_items.rs:45` (S7-3, S7-4)
- The box padding is written in the style sheet and in `BOX_PADDING`, tied only by a doc sentence. Site: `src/render_html.rs:21`, `src/render_style.rs:46` (S7-21)
- Two lookup walkers resolve a path through the same steps and already differ in four ways; the decision is pending. Site: `src/resolve.rs:204`, `:255`, `src/resolve_run.rs:91`, `:101`, `:129`, `:142` (S8-1)
- `from_manifest` reads and parses a manifest the way `manifest::read` does. Site: `src/root.rs:127`, `src/manifest.rs:40` (S8-10)
- A type alias is used once while its long form is written three times. Site: `src/leads_containers_owners.rs:19` (S4-5)
- `own[u] ||` can never change the answer, because `unread` already holds for the target's package. Site: `src/leads_census_scope.rs:263` (S4-9)

Naming, visibility, and spelling:

- Two fields named `deps` hold different maps. Site: `src/analysis.rs:35`, `:184` (S2-14)
- `pub(crate)` items with one caller in their own file: `attrs::list_text` (`src/attrs.rs:122`), `generic_names` (`src/collect.rs:459`), `Builder::module_head` (`src/document_sections.rs:253`), `raw_doc` (`src/collect_attrs.rs:110`), `beautify` (`src/comments.rs:308`), `GITIGNORE` (`src/folders.rs:20`), `Scope::blind` and `Scope::kept` (`src/leads_census_scope.rs:282`, `:308`), `stem` (`src/leads_comments_words.rs:171`), `QUOTE_CHARS` (`src/leads_docs_text.rs:17`), `vis_of` (`src/leads_items.rs:357`), `Feature::enables`, `Feature::implicit`, `DepEntry::optional` (`src/manifest_facts.rs:43`, `:45`, `:65`), `order::module_mentions` (`src/order.rs:280`), `Finder::new` (`src/typerefs.rs:45`), and the field `Plan::out` (`src/settings.rs:77`) (S2-13, S2-17, S3-1, S4-6, V4-1, S5-8, S6-20, S8-8)
- Four types are written both qualified and bare. Site: `src/document.rs:262`, `src/collect_use.rs:170`, `src/collect_type.rs:326`, `src/collect_impl.rs:27` (S2-18)
- Two different types are named `Inline`. Site: `src/document.rs:31`, `src/render_prose.rs:12` (S2-19)
- `attach` means placing comments in one place and placing impls in another. Site: `src/comments.rs:63`, `src/attach.rs:1` (S2-20)
- `heading` holds an id in `Builder` and in `Inline::Kill`. Site: `src/document.rs:121`, `:42` (S2-13)
- `stmts` and `statements` name different quantities. Site: `src/model_body.rs:108` (S2-23)
- `Marker::VISIBILITY` documents a line that types never print, and copies `Marker::PLAIN`. Site: `src/layout_marker.rs:53` (S3-4)
- `SUMMARY_FROM` is an exclusive bound among inclusive `_FROM` names, and `neighbors` includes the candidate so `<= NEIGHBORS_FROM` reads as an exclusive bound. Site: `src/leads.rs:44`, `src/leads_census_place.rs:225` (S3-6, V3-2)
- `Doc` and `docs` in `leads_body_clone` name compared bodies, beside three other `Doc` types. Site: `src/leads_body_clone.rs:30` (S3-7)
- `attrs`, `groups`, and `section`/`sections` each name two things in `layout`. Site: `src/layout.rs:221`, `:260`, `:274`, `:279` (S3-8)
- `SHOWN` lists visibility words, in a crate where "shown" means shown as evidence. Site: `src/leads_docs_phrases.rs:83` (S4-7)
- `IMPL_SUFFIXES` is `WRAPPER_SUFFIXES` without `_helper`, and no doc says why. Site: `src/leads_names.rs:73` (S5-7)
- The stem of a `Loc` is spelled `target`, while `target` is an index everywhere else; `base` means a `LinkBase` and the run's base folder in `pipeline`. Site: `src/render_slop_schema.rs:151`, `src/pipeline.rs:45`, `:201`, `:213` (S7-8, S7-7)
- The `values` flag of six lookup functions shares its name with the `Scope::values` field. Site: `src/resolve.rs:204`, `:255`, `src/resolve_run.rs:71`, `:91`, `:101`, `:129`, `:142` (S8-2)
- The workspace-manifest error names a flag that no longer exists. Site: `src/root.rs:133` (S8-9, S9-1)

Doc comments and drift in the code:

- The `runnable` doc says "runs" but `no_run` blocks count, and one clause is dead. Site: `src/census_docs.rs:90`, `:99` (S2-12)
- The `attrs` module doc lists the wrong set. Site: `src/attrs.rs:1` (S2-16)
- The `member_names` doc leaves out the removal of repeated names. Site: `src/attach.rs:144`, `src/model_impl.rs:37` (S2-19)
- Four check modules promise a check that release builds never run. Site: `src/analysis_check.rs:1`, `src/census_check.rs:1`, `src/leads_check.rs:1`, `src/collect_check.rs:1` (S2-17, V1-1)
- A path starting with `::` after a colon or an arrow is dropped, and no doc says so; the decision is pending. Site: `src/census_paths.rs:72` (S2-18)
- The `document` docs describe a second renderer and a counts-only kill list that no longer exist. Site: `src/document.rs:1`, `:91`, `:98`, `:104`, `:132` (S2-11)
- The `collect_check` doc leaves out `validate_package_facts` and the debug-only gate. Site: `src/collect_check.rs:1` (S2-8)
- The `config` docs miss `workspace = false` and two of the seven keys. Site: `src/config.rs:1`, `:25` (S2-21)
- The `unsupported_item` doc leaves out that it warns. Site: `src/collect.rs:330` (S2-22)
- The `groups` module doc says one module per group, the code and README say two path segments. Site: `src/groups.rs:1` (S3-3)
- A comment in `layout_items` says a type's header shows its own visibility, which it does not. Site: `src/layout_items.rs:127` (V3-1)
- One doc describes two constants in each of two pairs. Site: `src/leads_body_literals.rs:18`, `:40` (S3-5)
- The `stem` doc leaves out the plural exception for words ending in `ss`. Site: `src/leads_comments_words.rs:167` (S4-1)
- Three detector docs name fewer triggers than the tables that drive them. Site: `src/leads_docs_claims.rs:106`, `:168`, `src/leads_docs_phrases.rs:54`, `tools/vibelight.md:389`, `:392` (S4-2)
- The `Phrase` doc describes one word, the `doc_echo` doc lists fewer sources than `vocabulary` gathers, `Func::is_own` says "a type's own" for free functions, and `Catalog::fields` has no doc though three readers depend on its order. Site: `src/leads_docs_text.rs:75`, `src/leads_docs_repeat.rs:147`, `src/leads_items.rs:100`, `:152` (S5-1, S5-2, S5-3, S5-4)
- The `derived-field` glossary entry misses the label word `msg`. Site: `tools/vibelight.md:233`, `src/leads_shapes.rs:25` (S5-6)
- The `Counts` doc leaves out `awaits`, the `Decl` and `Param` docs miss trait aliases and named variant fields, and the `Receiver::Mutable` doc says "dropped" though the HTML prints a `mut` marker. Site: `src/model_body.rs:183`, `src/model.rs:220`, `:106`, `:80` (S6-18, S6-19, V6-1)
- The `TOGGLE_ID` doc says the style sheet names the id, and it does not; the comment over the `rest` loop in `place` has lost words; the `Outcome::files` doc names one kind of file and one index; the `Place::support` doc says "whether" though the function returns `Some(true)` or `None`. Site: `src/render_html.rs:13`, `src/order_graph.rs:67`, `src/pipeline.rs:30`, `src/render_slop_items.rs:38` (S7-1, S7-2, S7-9, V7-1)
- Ten doc comments cite numbered rules of one dated plan. Site: `src/resolve.rs:1`, `src/wrap.rs:9`, `:23`, `:48`, `src/model_body.rs:9`, `src/collect_body.rs:2`, `src/model.rs:439`, `src/comments.rs:50`, `src/order.rs:36`, `:253` (S8-14)
- The `TypeRef` docs leave out sources of `comment` and of `may_be_value`, and the `surface` docs leave out extern types. Site: `src/text.rs:170`, `:183`, `:187`, `src/surface.rs:42`, `:246` (S8-13, V8-1, S8-18)
- The legend and the README name notation the page never prints: a `//` before `internal` and `private`, and a closing `;` on `implements:`. Site: `src/legend.rs:13`, `:15`, `README.md:148`, `:246`, `:250` (V11-3)

Documentation drift, `README.md` against the code (all cheap, doc side changes):

- Line 13 lists a per-declaration fact, how often its name is written, that no record holds (S9-3)
- Line 75 says a build member "starts with" `build`, the code needs the whole word (S9-14)
- Lines 83 and 87 list `src/bin/*.rs` only, the code also reads `src/bin/*/main.rs` (S9-9)
- Line 103 names two dependency tables, the code reads them under every `[target.*]` too (S9-10)
- Line 125 says every file starts with a stamp, `index.md` and `.gitignore` have none (S9-4)
- Line 145 names a "Kill list" part and a Contents link, the page has `MACROS:` and no link (S9-13)
- Line 156 never names the header key `dir` (S9-6)
- Line 163 never names the `mod` record keys `vis`, `types`, and `functions` (S9-7)
- Line 166 calls `macros` a count and names only some `body` keys (S9-5, V9-1)
- Line 178 gives one condition for a `value`, the code also caps it at 100 characters (S9-11, V9-3)
- Line 277 misses two warnings: unreadable doc files and module files that load again (S9-8)

Documentation drift elsewhere:

- `AGENTS.md:3` says a change in one crate never changes another crate's HTML, but it can (S9-2, V9-2); the same claim is at `tests/it/stamp.rs:3`
- `tools/vibelight.md:205` repeats the wrong `body` sentence (V9-1)

Documentation drift, the dated plans against the code (all cheap, prose only):

- Nine of ten plans mark every todo `pending` or `in_progress` though the work shipped; only `plans/2026-10-26-output-analysis-upgrades.plan.md` is right (S10-1)
- The markdown plan's wire examples and lead rules differ from the shipped schema and detectors (S10-2, S10-5)
- Five older plans name switches, config keys, rules, and files the crate no longer has (S10-7)
- The reading-revision plan differs from the shipped toggle, box, legend, and index page (S10-11)
- The zero-switch plan's root rule, created-file notes, fixed formats, and key list were overtaken (S10-13, S10-14)
- The data-first plan's example prints `struct Run`, and the code now prints `class Run` (V10-1)
- Rule 54 and the Determinism text of the original plan still promise byte-identical markdown (S11-9)
- 31 of the 71 rules of the original plan are stale in part, and the plan is still called the live rulebook; the decision is pending (V11-1)
- `AGENTS.md:1` tells an agent to copy a plan at the start and never to close it out; the decision is pending (V10-2)

Report defects in the vibelight output:

- None found; the one stderr warning (6 items written more than one way) is explained by the spelling finding about the six items written more than one way (S2-18)

Rejected candidates: 36 in all, 24 weak (no reader meets a consequence), 11 intentional (a doc, comment, commit, or plan gives the reason), 1 obsolete.

Merged: 42 candidates share a cause with a finding above and add no item of their own.

</product-contract>
<implementation-contract>

## Technical Design

- Two token layers stay two `Tok` types; one keyword table and one predicate, `census_lex::is_keyword`, serve both
  - why: the body scan keeps literal values and the census keeps spans and close indexes, so one type costs memory in every census source or lexes a body twice; the doc-attribute and macro-group rules take different shapes in the two layers (a tree slice against flat census tokens, flagged against dropped) and stay apart, with each module doc naming the differences that are meant
  - cost: two types stay, and the body scan's canonical tokens keep the 13 reserved words that the census table adds, so `SHINGLE_VERSION` goes from 2 to 3
- The attribute and derive mark text stays a string, with one writer and one reader
  - why: the text is a sort key and printed output that the goldens fix, so typed marks would change `MacroUse` and the kill-list join for the same gain
  - cost: the string protocol stays, but it is defined once, in `src/macros.rs`
- New methods go in `src/model_impl.rs` (92 lines), not `src/model.rs`
  - why: `model.rs` has 485 lines and `build/structure.rs` fails the build above 500
  - applies to `Sig::texts`, `MemberKind::refs`, `Decl::is_wire`, `Crate::parent_module`
- The census reads generics and where bounds as well as parameters and return type
  - why: a type named only in a bound ties an item to its module, and the narrower copy was never explained by any record
  - cost: fewer `defined-away-from-observed-users` leads on items that a bound ties
- `gen` leaves the census keyword table
  - why: in a 2021 crate `rng.gen()` is a method call, and a false name is harmless where a missed one is not
  - cost: an edition-2024 `gen` block shows as a name with no declaration
- Underscore dependency tables count everywhere, through one list of spellings
  - why: the tool reads other crates' manifests, and Cargo still reads `dev_dependencies` and `build_dependencies` before the 2024 edition
  - cost: a run can follow more path dependencies on a manifest that uses those spellings
- `Receiver` holds the receiver text in `Shared(String)` and `Mutable(String)`
  - cost: `Receiver` stops being `Copy`, and the `==` tests become `matches!`
- `Entry` ranks by one `Rank` enum
  - cost: equal re-export keys fall back to push order, so run the goldens
- The resolved reference becomes one enum of declaration, value, and outside path, and `foreign` stays a field
  - why: `route::spell` reads `foreign` beside `canonical`
  - cost: nine `TypeRef` literals and about ten reading modules change
- `Reached::at` takes an `ItemId`, not an `Option`
  - why: every site that builds a resolved edge has an item, so `Via::Unresolved` and `item.is_none()` agree by construction
- The four HTML option enums and `Options` go, and the legend is always on
  - cost: a second notation later adds an enum back

</implementation-contract>
<verification-contract>

## Testing Plan

- Gate commands, run from the repository root after each item (the build already fails on any warning through `.cargo/config.toml`, and CI runs the first three):
  - `cargo fmt --check`
  - `cargo clippy --all-targets --locked`
  - `cargo test --locked`
  - `cargo doc --no-deps --locked` after any item that edits a doc comment; read its warnings, because a broken intra-doc link only warns
  - `cargo +1.85 check --all-targets --locked` after an item that adds code, the MSRV job of CI (`rustup toolchain install 1.85 --profile minimal` first)
  - a fresh plain `vibelight` at the repository root (`cargo run --locked --quiet`), then read the changed declarations and comments in `.vibelight`
- New tests, each written first. Marked "fails today" when the assertion fails on the code at HEAD `5c141ef`, and "pins" when it passes before and after the change and only guards the refactor:
  - `a_census_scan_records_gen_as_a_method_name` in `src/census_lex.rs` (fails today): `names("fn f() { r.gen(); }")` holds an entry named `gen`, which `is_keyword("gen")` drops now
  - `a_body_scan_reads_gen_as_a_method` in `src/collect_body_tokens.rs` (pins): scanning `x.gen()` records the method `gen`, which fails if the body scan ever shares the table without `gen` leaving it
  - `a_comment_names_backticked_snake_path_and_camel_words` in `src/leads_comments_words.rs`, extended (fails today): ``named("Retry `try` and `box` first.")`` is empty
  - `a_crate_root_doc_papercut_is_a_lead` in `src/leads_comments_tests.rs` (fails today): a `//!` doc with a `TODO` raises `papercut`
  - `a_trait_impl_constant_with_a_comment_shows_no_value` in the collect tests (pins): no value is shown when a comment sits in its range
  - `a_type_named_only_in_a_bound_ties_a_candidate` in `src/leads_census_tests.rs` (fails today): a candidate named only in a where bound is not `defined-away-from-observed-users`
  - `an_impl_of_no_declaration_has_its_own_module` in `src/leads_census_tests.rs` (pins): the impl-module helper
  - `a_serde_type_is_not_a_thin_wrapper` in `src/leads_impls_tests.rs` (pins): a `serde` derive exempts `thin_wrapper`
  - `underscore_dependency_tables_are_followed` in `src/manifest.rs` (fails today): `[build_dependencies]` and `[dev_dependencies]` path entries behave as the hyphen spellings
  - `a_member_used_only_through_underscore_dev_dependencies_is_left_out` in `src/scaffold.rs` (fails today)
  - `a_heading_escapes_markup` for `render_prose::subheading` (pins): `&` and `<` in an `<h4>` text are escaped
  - `style_padding_matches_box_padding` in `src/render_html.rs` (pins): `STYLE` holds `padding:.2rem {BOX_PADDING} 0`
  - `a_path_after_a_colon_or_an_arrow_starts_a_qualified_path` in `src/census_paths.rs` (fails today), for `x: ::a::B`, `-> ::a::B`, and `=> ::a::B`
- Existing tests that guard the structural items, which must pass unchanged: `a_name_use_count_leaves_out_one_hit_per_declaration`, `a_binary_reaches_its_own_library_without_changing_the_crates`, `keywords_are_sorted_for_the_binary_search`, the `comments` tests, `walks_files_mod_rs_inline_and_path_modules`, and every golden under `tests/fixtures`
- Existing tests that change on purpose: `a_shingle_hashes_the_version_and_its_tokens` (the pinned hash follows the version bump), `the_glossary_in_the_lens_gives_every_registered_id_one_bullet_in_order` (must keep passing after the glossary edits), and `assert_eq!(plan.html().options, HTML)` at `src/settings.rs:202` (goes with `Options`)

</verification-contract>
<decision-record>

## Decision Record

- Keep `Via` and build it through `Reached` constructors, because the tests use it to show which rule resolved a path (V1-3)
- Make `Target::lib` go and keep the kind, because a runtime check exists only to catch the two disagreeing (S8-4)
- Keep `pipeline::shown` as it is, because no document says whether the base folder prints `.` or the full path (S3-14)
- Edit the lens glossary in `tools/vibelight.md` for three entries, because every run reads it and each edit is one sentence that the glossary test pins in form (S4-2, S5-6, V9-1)
- Fix the legend and README notation and update the 59 HTML goldens, because the legend is the one place a reader of the page learns the notation (V11-3)
- Add a unit test that ties `BOX_PADDING` to the style sheet, and leave the doc as the tie; do not add a CSS custom property, which rewrites every HTML golden for a cosmetic gain (S7-21)
- Put the `impls` removal inside the options item, because the options removal is what leaves it unreachable (V11-2)
- Give the older plans banners and a status line, not rewrites, and keep every rule number, because ten source docs cite them and `build/structure.rs` rejects a plan subfolder (S10-7)
- Merge the body-scan and detector keyword changes into one item, because both end in one table and the 14-word difference between the census table and the detector table is 13 once `gen` has left the census (S2-1, S4-3)
- Keep the paired options on `MoreRecord` and `LeadRecord` and document which fields go together, because serde documents `deny_unknown_fields` as unsupported together with `flatten`, and the schema check in `src/render_slop_schema.rs` and `src/render_slop_catalog.rs` derives `Deserialize` under `cfg(test)` with `deny_unknown_fields` on both records; the gain is one removed state and the other routes weaken the check (S7-14, S7-15)
- Treat a leading `::` after a colon or an arrow as a bug and fix it, because a path in a type position after a colon is a real use and no record says the drop is wanted; the cost is more qualified-path hits for `no-observed-path-use` and `test-only-observed-use` (S2-18)
- Keep the two lookup walkers `Scope::walk` and `World::walk` and do only the doc pointer and the `may_be_value` rename, because their four differences are unsettled and a merge changes a core module for no output change (S8-1)
- Add the close-out sentence to `AGENTS.md:1`, because eight of ten plans were added with their work and only one was closed out, so the stale todos would recur; the cost is that it steers every agent session (V10-2)
- Keep `plans/2026-10-25-vibelight-tool.plan.md` as the live rulebook with a status line that lists its 31 stale rules, because `AGENTS.md:2` still says a written rule beats either look and ten source docs cite its numbers (V11-1)
- Share only the keyword predicate between the body scan and the census, and name the differences in each module doc, because the doc-attribute and macro-group rules take different token shapes in the two layers and sharing them changes census token indexes for no output change (S2-1)
- Rejected: one `Tok` type, typed marks, an archive folder for plans, a CSS custom property for the padding
- Assumption: every line number is a line at HEAD `5c141ef`; apply the items in any order, except where Order of the items says otherwise, and find each site by symbol if a line has moved
- Risk: nothing was built, so each item's compile and golden claims rest on reading until the gate commands run

### Open Questions

None remain. The six questions the analysis raised were answered with the recommended option, and each answer is a decision in the Decision Record above: the leading `::` path (S2-18), the lookup walkers (S8-1), the `AGENTS.md` sentence (V10-2), the original plan's status (V11-1), the paired wire options (S7-14, S7-15), and the shared token rules (S2-1).

### Deferred and Out of Scope

- The base-folder message rule: whether `inputs::shown` or `pipeline::shown` is right for the base; revisit when a message about the base folder is reported as confusing (S3-14)
- The old anchor-layout outline cleanup in `src/folders.rs`: delete `is_anchor`, the two anchor arms of `is_outline`, the `old.md` and `oldest.md` cases, and the older-release sentences in `tests/it/folders.rs`; revisit after the first tag past `v0.2.0` that ships the JSON layout (S3-2)
- The `dogfood` todo of the markdown plan: revisit after a manual run at the runvil root (S10-1)
- The eleven per-target tables kept aligned by `check_targets`; revisit when a new per-target table is added without updating the check (S2-3)
- `Decl::kind` and `Decl::shape` overlap; revisit if a second place builds a `Decl` (S6-7)
- The `Option` flags on the wire records; revisit if a second producer of records appears (S7-13)
- Cargo's default target layout in `root` and `manifest_roots`; revisit if a package with two targets of one name is supported (S8-11)
- `constexpr` and `using` are in neither the legend nor the README notation; revisit when the legend is next revised (S11-15)
- One walker for `resolve::Scope::walk` and `resolve_run::World::walk`; revisit once a test that runs both on one input shows which of the four differences are meant (S8-1)
- One payload type for `MoreRecord` and one summary type for `LeadRecord`; revisit if the schema check stops using `deny_unknown_fields`, or if a hand-written test-only `Deserialize` is acceptable (S7-14, S7-15)
- Sharing the doc-attribute and macro-group rules between the body scan and the census; revisit if the census token indexes change for another reason (S2-1)

</decision-record>
<project-survey>

## Project Survey

None

</project-survey>
<execution-plan>

## Execution Instructions

- The letters and numbers in parentheses are tags from the analysis and are never needed to act
- Run the gate commands from the Testing Plan after each item: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`, plus `cargo doc --no-deps --locked` for an item that edits a doc comment and `cargo +1.85 check --all-targets --locked` for an item that adds code; every item below repeats the commands it needs
- The goldens under `tests/fixtures` (59 HTML pages and 59 markdown files, counting `expected.html`, `expected.<name>.html`, and the pages under `expected/`) must not change unless an item says so; never regenerate a golden to make a failing test pass, and read the diff of one that changes on purpose (`AGENTS.md`)
- `build/structure.rs` fails the build when a `.rs` file under `src`, `tests`, or `build` has more than 500 lines; the files closest to it are `src/leads_shapes_twins.rs` (499), `src/layout.rs` (498), `src/leads_containers_ops.rs` (496), `src/source_links.rs` (495), `src/render_slop_split.rs` (495), `src/model.rs` (485), `src/leads_items.rs` (483), and `src/collect_body_tokens.rs` (483), so an item that adds lines to one of them must delete as many
- Make one commit per item, so a golden that changes lands in the commit of its own intended output change; never push a tag

### Derive three stored fields on read

- Files: `src/analysis.rs`, `src/analysis_check.rs`, `src/census.rs`, `src/census_check.rs`
- Also: `src/leads_census_scope.rs`, `src/analysis_paths.rs`, `src/analysis_uses.rs`, `src/leads_census_deps.rs`, `src/analysis_tests.rs`
- Problem: three fields store what another field already tells, and debug checks keep the copies in step
- Remove `AnalysisRun::limits` (`analysis.rs:219-220`, built at `:279-282`, stored at `:302`):
  - add `Census::unread()` returning `impl Iterator<Item = SourceId> + '_`: the ids of sources with `scan != Scan::Lexed`, in source order
  - add `Census::fully_read()` (true when `unread()` is empty) for the two `is_empty()` uses at `census_check.rs:93` and `:101`
  - delete the `listed` conjunct (`census_check.rs:63`, and its place in the `if` at `:64`) and the sort check (`:68-70`)
  - point `leads_census_scope.rs:243` at `run.census.unread().chain(included)`
  - fix the docs at `census_check.rs:18` ("its sources and limits") and `:26-27` ("that the limits list exactly the unread sources")
- Remove `NameCount::total`:
  - delete the field and its doc (`census.rs:112-114`) and the `count.total += hit.count;` line in `Census::count` (`:189`)
  - add `NameCount::total()` that sums `by_place.values()`, and make `uses()` (`:125`) call it
  - delete the `summed` check (`census_check.rs:92` and its place in the `if`), and write `count.total() < count.declared` in `short` (`census_check.rs:93`)
  - in `analysis_check.rs:415-416` write `!count.by_place.is_empty()` for `count.total > 0`, and delete the always-true `count.uses() <= count.total`
  - in `analysis_tests.rs:369` read `count.total()`
- Remove `PackageInfo::targets` (`analysis.rs:46-47`, initialised at `:79`, filled at `:85`):
  - add `pub(crate) fn library_of(targets: &[TargetInfo], package: usize) -> Option<usize>` in `analysis.rs`, the first target of that package whose `kind` is `TargetKind::Lib`
  - drop `mut` from `let mut packages` (`analysis.rs:74`) once the `push` at `:85` is gone, because an unused `mut` fails the warning-free build
  - `analysis_paths.rs:75-82` (`augmented`): call `library_of(targets, info.package)` in place of the `packages[..].targets` search, and drop its `packages` parameter and the `PackageInfo` import at `:13`, which nothing else there reads; its one caller is `analysis.rs:236`
  - `analysis_uses.rs:103-108`: loop `for p in 0..run.packages.len()` and take `library_of(run.targets, p)` for the reach, so `package_reach` stays in package order
  - `leads_census_deps.rs:54-55`: `library_of(run.targets, p).or_else(|| run.targets.iter().position(|t| t.package == p))` keeps the first-target fallback
  - in `analysis_check.rs`: delete `listed`, `ordered`, and `owned` (`:57-60`, `:95-99`); keep a check that `info.package` indexes `self.packages`; recompute `libs` from `self.targets` (targets with `package == p` and `kind == TargetKind::Lib`) and keep `libs > 1`, with the message "package {p} has more than one library"
- Test first: the existing `a_name_use_count_leaves_out_one_hit_per_declaration` and `a_binary_reaches_its_own_library_without_changing_the_crates` (both in `src/analysis_tests.rs`) must pass unchanged
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; the goldens stay unchanged (S2-1, S2-2, S2-11, V1-2)

### Name the result of path resolution

- Files: `src/analysis_paths.rs`, `src/analysis_check.rs`
- Problem: `resolve` (`analysis_paths.rs:163-170`) returns `(Option<ItemId>, Option<ItemId>, Via)` from 13 tuples, so a swapped item and prefix compiles
- Add `struct Reached { item: Option<ItemId>, prefix: Option<ItemId>, via: Via }` with `Reached::at(item: ItemId, prefix: Option<ItemId>, via: Via)` and `Reached::open(prefix: Option<ItemId>)`, which sets `item` to `None` and `via` to `Via::Unresolved`
  - change the local closure `at` in `resolve` (`:172`) to return an `ItemId`, and pass `Some(own)` where `own` is a prefix
- Return `Reached` from `resolve`, and add `Edge::new(target, body, kind, fact, reached)` that both loops of `link_bodies` call (`:127-135`, `:145-153`)
- Keep `Edge` flat, so its readers of `edge.item`, `edge.prefix`, and `edge.via` do not change
- Build `Via::Unresolved` only through `Reached::open`, so `Unresolved` and `item.is_none()` agree by construction, then delete the `resolved` condition at `analysis_check.rs:313` and drop `Via` from the import at `analysis_check.rs:11`, its only use
- Keep `Via`: `analysis_tests.rs:107-119` and `:165-253` use it to show which rule resolved a path
- Test: the existing `edges()` cases in `src/analysis_tests.rs:139-290` must pass unchanged
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-7, V1-3)

### Keep one copy of each duplicated rule in the attach and macro code

- Files: `src/attach.rs`, `src/analysis_items.rs`, `src/analysis.rs`, `src/attrs.rs`, `src/macros.rs`, `src/collect.rs`, `src/census_sources.rs`, `src/leads_names_modules.rs`, `src/render_slop_items.rs`
- Impl owners: `attach::attach` (`attach.rs:26-34`) and `analysis_items::attach` (`analysis_items.rs:64-76`) compute the same owner of each impl
  - keep one `owners(krate: &Crate) -> Vec<Option<DeclId>>` in `attach.rs`
  - call it from `attach::attach` before its loop and from `AnalysisRun::build` (`analysis.rs:239`)
  - delete `analysis_items::attach` and fix the module doc at `analysis_items.rs:2`
  - nothing in the loop writes `refs`, `decls[..].kind`, or `impls[..].self_ref`, so computing owners first is the same result
- `test_only`: delete `attrs::test_only` (`attrs.rs:19-27`), make `macros::test_only` (`macros.rs:111-119`) `pub(crate)`, and call it from `attrs::is_test_only`
  - `attrs` already depends on `macros`, so no module edge is added
- Macro definitions: the producer builds the item text `macro_rules! {ident}` (`collect.rs:256-266`) and three readers test the prefix `macro_rules! `
  - add `macros::definition_text(ident: &str) -> String` and `macros::definition_name(text: &str) -> Option<&str>` (the `strip_prefix`) in `src/macros.rs` (275 lines)
  - call `definition_text` at the producer, which keeps the printed text byte for byte
  - readers: `census_sources.rs:264` (`ItemKind::Mark(text) => macros::definition_name(text).is_some()`), `leads_names_modules.rs:80` (the same arm), and `render_slop_items.rs:346` (`if let Some(name) = macros::definition_name(&mark.text)`)
  - leave the `(loc, text)` match at `render_slop_items.rs:321-329` as it is, because it finds the item whose text equals the mark's
- Tests: `test_only_predicates` (`attrs.rs:169`), `derives_merge_cfg_attr_and_skip_test_only` (`macros.rs:251`), and the goldens of the macro fixtures (`macros`, `links`, `surface`, `kill-list`, `comments-attach`) must pass unchanged
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-5, S2-6, S2-9)

### Drop `Start::at`

- File: `src/census_paths.rs`
- Problem: `Start` (`:22-29`) holds `at` beside its three flags, and the one reader uses the same token's position
- Delete the field and its assignment at `:41`, and in `scan` at `:105` pass `self.toks[i].span.start()` in place of `start.at`
- Test: `macro_bodies_are_macro_tokens_test_spans_mark_tokens_and_bad_text_is_none` (`src/census_lex.rs:313-340`) pins the line of each path
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-10)

### Fix doc comments, one name, and one visibility in the attribute, census, and check modules

- Files: `src/census_docs.rs`, `src/census.rs`, `src/attrs.rs`, `src/attach.rs`, `src/model_impl.rs`, `src/analysis_check.rs`, `src/census_check.rs`, `src/leads_check.rs`, `src/collect_check.rs`, and the files of the `AnalysisRun::deps` rename below
- `src/census_docs.rs:90-91`: change "a doc test rustdoc runs: Rust, and neither `ignore`d nor `compile_fail`" to "a doc test rustdoc compiles: Rust, and neither `ignore`d nor `compile_fail`; a `no_run` block counts"
  - delete the dead clause `|| word.starts_with("ignore-")` at `:99`
  - in `src/census.rs:28` (`Context::Doctest`) and `census_docs.rs:2-3`, change "runnable" to "compiled"
- `src/attrs.rs:1`: replace the module doc with "Reads the attributes and visibilities the outline cares about: `cfg`, `path`, `doc(hidden)`, API attributes, lint attributes, and the forms of `pub`."
- `src/attrs.rs:122`: make `list_text` private
- `src/attach.rs:144-155`: change the `member_names` doc to "Returns the names of an impl's members, sorted and without repeats, leaving out its macro marks and unsupported entries."
  - in `src/model_impl.rs:37-38`, change the `ImplLoc::members` doc to say "sorted and without repeats"
- Check modules, which promise a check that release builds never run (the gate is `validate`, run in debug builds at `pipeline.rs:187`):
  - `src/analysis_check.rs:1-5` becomes "Checks the invariants of a run's analysis view, which debug builds do before anything reads it, so that a broken view stops the run as an internal error rather than reaching a detector:"
  - `src/census_check.rs:1-3` becomes "Checks the invariants of a run's census as part of `validate`, which debug builds run before anything reads it:"
  - in `src/leads_check.rs:1-2`, add "which debug builds do" to the promise that the check runs "before anything reads" the leads
  - `src/collect_check.rs:1-3` becomes "Checks the analysis facts before anything reads them, so a broken fact stops a debug run as an internal error rather than reaching an analysis: the facts collection keeps for each crate, and the manifest facts of each package."
- Rename `AnalysisRun::deps` to `analysis_deps`, the term the `analysis_paths` docs use, and leave `TargetInfo::deps` and `Census`-side names alone; sites: `analysis.rs:186`, `:236-237`, `:287`; `analysis_check.rs:39`, `:67`, `:83`; `analysis_uses.rs:99`; `leads_census_paths.rs:79`, `:126`; `leads_census_scope.rs:260`; `analysis_tests.rs:156`
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-12, S2-13, S2-14, S2-16, S2-17, S2-19, V1-1)

### Use one keyword table in the body scan, the census, and the detectors

- Files: `src/census_lex.rs`, `src/collect_body_tokens.rs`, `src/leads_comments_words.rs`, `src/leads_names_roles.rs`
- Problem: three tables list Rust's keywords: the census (52 words, `census_lex.rs:17-22`), the body scan (36 words, `collect_body_tokens.rs:14-19`), and the detectors (38 words, `leads_comments_words.rs:65-70`). The census skips `macro_rules! name { ... }` bodies and treats `gen` as a keyword, so `rng.gen()` in a 2021 crate records no name; the detectors miss 14 reserved words
- Step 1, census (test first): add `a_census_scan_records_gen_as_a_method_name` to `src/census_lex.rs`, which asserts that `names("fn f() { r.gen(); }")` holds an entry named `gen`; it fails today because `is_keyword("gen")` drops the name
  - remove `"gen"` from `KEYWORDS` (`census_lex.rs:19`) and change `[&str; 52]` to `[&str; 51]`; the table stays sorted for `keywords_are_sorted_for_the_binary_search`
- Step 2, body scan: delete `KEYWORDS` (`collect_body_tokens.rs:11-19`) and import `crate::census_lex::is_keyword`
  - `macro_end` (`:262`) and `method_after` (`:275`) test `is_keyword(name)` in place of `KEYWORDS.contains`
  - `canonical_token` (`:311-317`) keeps an identifier's text when `is_keyword` says so, after the existing `true` and `false` arm that gives `LIT`, and gives `ID` otherwise
  - this keeps 13 reserved words as written in canonical tokens (`abstract become box do final macro override priv try typeof unsized virtual yield`), so change `SHINGLE_VERSION` to 3 (`:21-23`) and replace the pinned hash `0x20bd_8b4a_338a_100f` in `a_shingle_hashes_the_version_and_its_tokens` (`:383`) with the value the test prints
  - add `a_body_scan_reads_gen_as_a_method` to the tests there (scanning `x.gen()` records the method `gen`); it passes before and after and guards the table switch
  - add one sentence to the module doc: the census flags `macro_rules!` bodies as macro tokens, and this scan folds only invocations
- Step 4, name the differences that stay: the doc-attribute and macro-group rules are not shared, because they read different token shapes (`doc_attribute` takes a `&[TokenTree]`, `macro_end` takes flat `Tok`s and close indexes)
  - add to the `collect_body_tokens.rs` module doc: "This scan drops doc attributes while it flattens, and folds each macro invocation into one `MACRO` token; it shares only the keyword table with `census_lex`."
  - add to the `census_lex.rs` module doc: "This scan keeps doc-attribute tokens and flags them, and flags `macro_rules!` bodies as macro tokens; it shares only the keyword table with `collect_body_tokens`."
- Step 3, detectors: delete `KEYWORDS` (`leads_comments_words.rs:64-70`), write `!is_keyword(name)` at `:222` and `!is_keyword(w)` at `leads_names_roles.rs:278`, import `crate::census_lex::is_keyword` in both files, and drop `use crate::leads_comments_words::KEYWORDS` at `leads_names_roles.rs:14`
  - behavior changes only for the 13 reserved words above, which the detector tables now treat as keywords; with `gen` left in the census table the count would be 14
  - test first: extend `a_comment_names_backticked_snake_path_and_camel_words` (`leads_comments_words.rs:353`) so that ``named("Retry `try` and `box` first.")`` is empty; it fails today
- Tests that must pass: the `collect_body_tokens` tests (`:366-481`), the `census_lex` and `census_paths` tests, and `src/leads_body_tests.rs`
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`, `cargo +1.85 check --all-targets --locked`; the goldens stay unchanged (S2-1, S4-3)

### Pass leading and trailing comments as one value, and rename the comment placer

- Files: `src/comments.rs`, `src/collect.rs`, `src/collect_trait.rs`, `src/collect_impl.rs`, `src/collect_use.rs`, `src/collect_type.rs`, `src/collect_fn.rs`, `src/collect_field.rs`, `src/collect_use_group.rs`
- Problem: `lead: Vec<Comment>, trail: &[Comment]` travel together through 19 signatures, and `comments::attach` shares its name with the impl-placing `attach` module
- Change `comments::attach` to return one `Vec` of a struct with a child's leading and trailing comments, named `place_comments` and `Placed` in place of `attach` and `Attached` (`comments.rs:50-98`)
  - store both lists as `Vec<Comment>` and borrow the trailing one where `Collector::trailing` needs a slice (`collect.rs:390`)
  - pass the one value to the 19 functions and to `Parts` (`collect_impl.rs:29-30`), and drop the three-way zips at the ten call sites
  - ten call sites: `collect.rs:191`, `collect_trait.rs:43`, `collect_impl.rs:171`, `collect_use.rs:271`, `collect_type.rs:130`, `collect_fn.rs:90-98`, `collect_field.rs:34`, `:82`, `:120`, `collect_use_group.rs:55`
  - the unit test at `comments.rs:362` follows the rename
  - fix the doc on `Attached` that cites "rule 20" of a plan, so it states the contract in words
- The two `#[expect(clippy::too_many_arguments)]` at `collect_use.rs:163-166` and `collect_type.rs:317-320` stay, because nine arguments is still over the limit
- Tests: the `comments` unit tests (`comments.rs:344-386`) and the goldens `comments-attach`, `comments-groups`, `comments-impls`, `comments-kinds` must pass unchanged
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-2, S2-20)

### Give the attribute and derive mark text one writer and one reader

- Files: `src/macros.rs`, `src/collect.rs`, `src/collect_impl.rs`, `src/collect_type.rs`, `src/document_sections.rs`, `src/layout_items.rs`, `src/route.rs`, `src/document_kill.rs`, `src/render_slop_items.rs`
- Problem: the text `attribute: a, b` and `derive: A, B` is built in eight places and taken apart in two by splitting on `:` and `, `; changing the label breaks the kill-list join with no compile error
- Add `mark_text(kind: &str, names: &[String]) -> String` and `mark_parts(text: &str) -> Option<(&str, Vec<&str>)>` to `src/macros.rs` (275 lines), and use them at all ten sites
  - writers: `collect.rs:428-446` (`attribute_mark`), `collect_impl.rs:123-131`, `collect_type.rs:345-360`, `document_sections.rs:181-194`, `layout_items.rs:389-391`, `route.rs:127-137`
  - readers: `document_kill.rs:98-102` (`find_mark`) and `render_slop_items.rs:204-214`
  - let `head` and `item_impl` push through a variant of `attribute_mark` that takes the names, so the `MacroUse` push exists once
- Decision: keep the text and add the helpers, not typed marks, because typed marks change `MacroUse` and the join in `find_mark` for the same gain
- Tests: the HTML goldens `attributes`, `macros`, `kill-list`, `special-members`, `base-lists`, `spellings`, and the markdown golden `tests/fixtures/attributes/expected.md` must not change
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-4, S8-12)

### Collapse four small duplicates in the collect and check code

- `src/collect_trait.rs:320-327`: move `written_value` next to `value` (`collect_use.rs:212-242`) as `pub(crate)`, and end `value` with `(written_value(file, expr), Vec::new())` after its macro branch
  - delete `value`'s `has_comment` local and second `scan`; `has_comment` is read only by the final branch, and `written_value` already returns `None` for a macro call or a comment in range, so behavior is equal
  - pin test first: `a_trait_impl_constant_with_a_comment_shows_no_value` passes before and after, and no fixture covers it today
- `src/collect_check.rs:278`: change `check_ref(.., value: Option<bool>)` to `value: bool`, compare `r.may_be_value == value`, and word the doc "with `value` saying whether it may be a value"; the five callers pass `Some(true)` or `Some(false)` and drop the `Some`
- `src/collect_use_group.rs:17-26`, `:124-196`: keep `written` and add `Binding::path()` that strips `r#` from each segment; give `expand` one vector
  - readers: `collect_use.rs:49`, `:65`, `:77`, `collect_use_group.rs:119`
  - the `raw-idents` and `reexports` goldens pin both forms
- `src/collect_body_tokens.rs:25-27`: make `SHINGLE` and `MIN_TOKENS` `pub(crate)`, import them in `src/collect_check.rs`, and delete its copies at `:17-20` and the two-constant doc
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; `broken_facts_are_internal_errors` (`collect_check.rs:350`) must pass (S2-5, S2-6, S2-16, S2-7)

### Name the document builder's tuples and drop `Inputs::printed`

- Files: `src/document.rs`, `src/document_sections.rs`, `src/document_kill.rs`, `src/render_html.rs`, `src/pipeline.rs`
- Rename first: `Builder::heading`, `Inline::Kill::heading`, and the `find_mark` parameter become `heading_id`; the `marks` doc says "the id of the heading it sits under"
  - sites: `document.rs:42-46`, `:119-121`, `:164`, `:279-281`; `document_sections.rs:137-138`; `document_kill.rs:78-82`, `:96-117`; `render_html.rs:385-391`
- Then replace `Builder::marks: Vec<(String, String, String)>` with `MarkLine { heading_id, text, id }`, and the kill list's six-part tuple (`document_kill.rs:12-13`) with `KillEntry { kind, text, path, heading, id, loc }`
- Delete `Inputs::printed` and `Builder::printed`: `pipeline.rs:249-251`, `:260`; `document.rs:112-113`, `:132-133`, `:146`, `:157`; read `blocks[d]` at `document_sections.rs:195`; drop the `Notation` import at `pipeline.rs:18` if nothing else uses it
- Order: land the HTML options item first when both are open, because both edit `document.rs` and `pipeline.rs`
- Tests: every HTML golden must not change
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-13, S2-14, V2-1)

### Fix names, docs, visibility, and spellings in the collect, document, render, and config modules

- Visibility: drop `pub(crate)` from `generic_names` (`collect.rs:459`), `Builder::module_head` (`document_sections.rs:253`), `raw_doc` (`collect_attrs.rs:110`), `beautify` (`comments.rs:308`)
- Spelling: write the bare name for `std::cmp::Ordering` (`document.rs:262`), `syn::Attribute` (`collect_use.rs:170`), `syn::Generics` (`collect_type.rs:328`), `syn::Visibility` (`collect_type.rs:326`, `collect_impl.rs:27`), and add the `use` lines
- Rename `render_prose::Inline` to `InlineRenderer` (`render_prose.rs:12`, `:23`, `:111`, `:121`, `:177`, `:184`, `:199`; `render_html.rs:184`, `:202`)
- Rename `BodyFacts::stmts` to `stmt_lines` and `BodyCommentFact::statement` to `stmt`
  - sites: `model_body.rs:111-113`, `:278-279`; `collect_body_facts.rs:68`; `collect_body_comments.rs:116`; `collect_check.rs:236`, `:240`, `:257`, `:260`, `:371`; `leads_comments.rs:210`
  - keep `BodyFacts::statements`, because the README and the wire key use that word
- `src/config.rs:25`: change the `Crates::Unset` doc to "Neither `crates` nor a true or directory `workspace` is set."
  - `src/config.rs:1-3`: "Reads `vibelight.toml`: which crates a run covers, where it writes, and which doc passages and leads the detectors skip." then keep the two sentences after it
- `src/collect.rs:332` (`unsupported_item`): add to its doc "It also warns when `warn` is set; a module whose `problem` the tree walk already warned about passes `false`."
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-17, S2-18, S2-19, S2-21, S2-22, S2-23)

### Share one impl-module rule and one `sig_texts` across the detectors and renderers

- Files: `src/leads.rs`, `src/leads_census_place.rs`, `src/leads_names.rs`, `src/leads_items.rs`, `src/leads_docs.rs`
- Also: `src/model_impl.rs`, `src/collect_check_nodes.rs`, `src/analysis_uses.rs`, `src/order.rs`, `src/render_slop_facts.rs`
- Impl module: five places decide the module of an impl member, and the impl arm is the same in all of them (the attached declaration's module, else `impl_module`)
  - add one `pub(crate)` helper `impl_home(run, t, i)` beside `impl_module` in `leads_items.rs` that returns the module of impl `i` of target `t`
  - call it from `leads::module_of` (`leads.rs:177-188`, impl arm), `leads_census_place::home` (`:48-57`, impl arm), and the three inline copies at `leads_names.rs:154`, `leads_items.rs:263`, `leads_docs.rs:287`
  - keep `home`'s `ItemRef::Module(m) => m` arm, which differs from `module_of`'s `parent_module(..).unwrap_or(m)`, and fix its doc to say what that arm returns; `leads_names.rs:117` repeats `module_of`'s `Module` arm and should call `module_of`
  - pin test first: `an_impl_of_no_declaration_has_its_own_module` in `src/leads_census_tests.rs`, an impl written away from its type, of no declaration of the run; it passes before and after
  - size: `leads_items.rs` has 483 lines, so count lines after this item and the module-parent item
- `sig_texts`:
  - add one method `Sig::texts` on `Sig` that lists the texts that can name types: generics, parameters, return type, where predicates
  - put the `impl` in `src/model_impl.rs` (92 lines), because `model.rs` has 485 lines and the build fails above 500
  - call it from `render_slop_facts.rs:327`, `collect_check_nodes.rs:53`, `order.rs:308`, and `leads_census_place.rs:95`, `:102`; delete the four private and `pub(crate)` copies
  - decision: the census reads generics and bounds too, so a type named only in a bound ties a candidate to its module
  - test first (fails today): `a_type_named_only_in_a_bound_ties_a_candidate` in `src/leads_census_tests.rs`, a candidate named only in a where bound is not `defined-away-from-observed-users`
- `member_refs`: move it from `collect_check_nodes.rs:49-58` to `MemberKind::refs(&self) -> Option<Vec<RefId>>` in `model_impl.rs`, update `collect_check_nodes.rs:34` and `analysis_uses.rs:12`, `:349`, and delete the free function
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; the goldens stay unchanged (S3-11, S3-12, S2-3, S2-9)

### Give `DepKind` one table name

- Files: `src/manifest_facts.rs`, `src/leads_census_deps.rs`, `src/scaffold.rs`, `src/manifest.rs`, `README.md`
- Problem: the table names `dependencies`, `dev-dependencies`, `build-dependencies` are written in four modules, and only the facts reader accepts the underscore spellings
- Add `DepKind::table(self) -> &'static str` returning the hyphen name
  - call it from `leads_census_deps::table` (`:33-43`), from `path_deps` for Normal and Build (`manifest.rs:219-224`), and from `scaffold::dependents` (`:209-225`), which loops over the kinds and drops its string match
  - keep one list of the spellings Cargo reads, `dev-dependencies` and `dev_dependencies`, `build-dependencies` and `build_dependencies` (today `DEP_TABLES`, `manifest_facts.rs:83-89`), and have all four sites use it
- Decision: `path_deps` and `scaffold::dependents` honor the underscore spellings too, because the tool reads other crates' manifests and Cargo still reads those spellings before the 2024 edition
  - test first (fails today), in `src/manifest.rs`: `underscore_dependency_tables_are_followed`, a manifest with `[build_dependencies]` and `[dev_dependencies]` path entries is followed by `path_deps` the same as the hyphen spellings
  - test first (fails today), in `src/scaffold.rs`: `a_member_used_only_through_underscore_dev_dependencies_is_left_out`, a member used only through `[dev_dependencies]` is left out as a dev-only member
  - the README sentence at `README.md:103` is edited by the README item, which also says the underscore spellings count
- `inputs::package` (`inputs.rs:234-246`) builds `deps` with `path_deps` and `facts` with `manifest_facts::read`; after this item both views agree
- Tests that must pass: `path_dependencies_come_from_every_table_but_dev_dependencies` (`manifest.rs:285`), `scaffold.rs:328-330`, `leads_census_crates_tests.rs:100-115`, `facts_read_features_dependencies_and_target_roots` (`manifest_facts.rs:340`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; `manifest_facts.rs` has 475 lines, so the item must not add net lines there (S3-13, S6-10, S8-16)

### Break the `exclude` and `inputs` cycle and share `shown`

- Files: `src/exclude.rs`, `src/inputs.rs`, `src/manifest.rs`, `src/pipeline.rs`, `src/scaffold.rs`
- Move `relative` (`inputs.rs:304-312`) to `manifest.rs`, which already holds the path helpers `normalize` and `join` and imports only `glob`
  - update the importers: `exclude.rs:7`, `:35`; `scaffold.rs:15`, `:112`; `pipeline.rs:125`, `:273`, `:295`; `inputs.rs:196`, `:213`
- Make `inputs::shown` (`inputs.rs:211-216`) `pub(crate)` and call it from the closure in `pipeline::targets` (`:293-298`), deleting the closure
- Leave `pipeline::shown` (`:270-278`) as it is: it prints `.` for the base folder and `inputs::shown` prints the full path, and no document says which is right
- Tests that must pass: `tests/it/exclude.rs:171`, `tests/it/workspaces.rs:320`, the `exclude.rs` unit tests (`:60-117`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S3-14, S2-10)

### Fix names, docs, and visibility in the layout, groups, leads, and folders modules

- `src/folders.rs:20`: drop `pub(crate)` from `GITIGNORE`
- `src/groups.rs:1-2`: replace with "Splits the dependency order into the groups the HTML prints: each run of types whose modules share their first `DEPTH` path segments is a group, named for those segments."
- `src/layout_marker.rs:53-57`: delete `Marker::VISIBILITY`, and write `Marker::PLAIN.spaced` at `src/layout_decl.rs:94`
- `src/layout_items.rs:127-128`: replace the comment with "A module has no enclosing type whose `internal:` and `private:` sections could label an item, so each item's own visibility prints above it."
- `src/leads_body_literals.rs`, four docs:
  - `LITERAL_BODIES` (`:19`): "The fewest bodies that one string literal is in."
  - `LITERAL_TARGETS` (`:21`): "The fewest targets whose bodies hold one string literal, however few bodies that is."
  - `DOC_WORDS` (`:40-41`): "The fewest doc words, outside code blocks, of a `doc-outweighs-body` lead."
  - `DOC_TIMES` (`:43`): "How many times the body's canonical tokens the doc's words must exceed."
- Rename `SUMMARY_FROM` to `SUMMARY_ABOVE`: `src/leads.rs` lines 5, 45, 261, 299, 312; `src/leads_check.rs` lines 12, 47, 110; `src/leads_tests.rs` lines 12, 102, 103, 107, 110, 116, 118, 126
- `src/leads_census_place.rs:225-243` (`defined_away`): build `neighbors` as `held(run, t, c.module)` with the candidate filtered out (`.into_iter().filter(|&item| item != c.item).collect()`), test `neighbors.len() < NEIGHBORS_FROM` at `:226`, and drop the `item != c.item` filter at `:243`; `held` includes every listed candidate, so behavior is equal
- Rename in `src/leads_body_clone.rs`: `Doc` to `Compared`, `docs` and its locals and the `pairs` parameter to `compared`, `Group::docs` to `compared` (lines 30, 39, 48-50, 58, 61-62, 71, 85, 136-137, 149, 169, 176-177, 183-184, 213, 223)
- Rename in `src/layout.rs` (498 lines, so count lines after `cargo fmt`): `sections(groups)` to `empty_sections(per_section)`, with its doc, callers `layout_decl.rs:263-264` and `layout_items.rs:24`, and the two `use` lists; the `attrs: bool` parameter of `above` to `with_attrs` (`:217-221`, `:230`)
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S3-1, S3-3, S3-4, V3-1, S3-5, S3-6, V3-2, S3-7, S3-8)

### Read the crate-root doc in the `papercut` pass by sharing the doc walker

- Files: `src/leads_comments_docs.rs`, `src/leads_docs.rs`, `src/leads_comments.rs`
- Problem: `leads_comments_docs::docs` skips the crate-root doc (`if !module.path.is_empty()` at `:44`) and rewalks every documented node, so a `TODO` in a crate's `//!` doc raises no `papercut` lead
- Keep `leads_comments_docs` as a small module whose `find` takes `&Docs`, loops `docs.all`, reads `doc.prose.kept().lines()`, and ends with `doc.add(run, lead)`
  - before deleting, read `leads_docs.rs:133-340` and confirm `Docs::all` holds the crate-root doc; if it does not, add it there
  - call it from `leads_docs::find` (after `:340`), so every doc is parsed once
  - delete `Doc` and `docs` (about 100 lines); update the import at `leads_comments.rs:15`, the call at `:237`, and the module doc at `:7`
  - `papercut` stays in `leads_comments::IDS` and the glossary's Comments group; keep `leads_docs_text::without` `pub(crate)`
- Test first (fails today): `a_crate_root_doc_papercut_is_a_lead` in `src/leads_comments_tests.rs`, a crate-root `//!` doc that holds a papercut word raises a `papercut` lead
- Tests that must pass: `a_papercut_in_a_body_comment_or_a_doc_is_a_lead_but_not_in_code_spans` (`leads_comments_tests.rs:137`), `a_boilerplate_passage_is_left_unread_by_every_doc_detector` (`leads_docs_tests.rs:258`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; a golden that holds a crate-root `//!` doc with a papercut word would gain a lead, so read any golden diff before accepting it (S4-8)

### Fix doc comments, visibility, names, and one redundant term in the detectors

- `src/leads_comments_words.rs:167-170`: replace the `stem` doc with "Returns the stem a lowercase word compares by: an `ing`, `ed`, or `es` ending removed, `ies` turned into `y`, a plain plural `s` removed, then a final `e` removed. Each removal needs [`STEM_MIN`] letters to stay, except the plural `s`, which needs one fewer and is left on a word that ends in `ss`, so `parse`, `parses`, `parsed`, and `parsing` share one stem."
- `src/leads_docs_claims.rs`:
  - `doc_unknown_param` doc (`:106-107`): "a function's `# Arguments`, `# Args`, `# Parameters`, or `# Params` section"
  - module doc (`:1-2`): "the parameters a function's parameter section names"
  - `doc_signature_mismatch` doc (`:168-171`): 'an opening "Return", "Returns", or "Yields"'
- `src/leads_docs_phrases.rs:54-55`: "`legacy` only after `for`, or before `callers`, `code`, `shim`, or `support`"
- `tools/vibelight.md:389` and `:392`: give the glossary entries the same words, each still one sentence that ends in a period with no dash; `the_glossary_in_the_lens_gives_every_registered_id_one_bullet_in_order` (`src/leads_tests.rs:260`) pins the form
- Visibility: drop `pub(crate)` from `Scope::blind` (`leads_census_scope.rs:282`), `Scope::kept` (`:308`), and `stem` (`leads_comments_words.rs:171`)
- Rename `SHOWN` to `VISIBILITY_WORDS` (`leads_docs_phrases.rs:83`, and its four uses at `:94-97`)
- `leads_containers_owners.rs:19`: delete the `KeyedFields` alias and write the type in the signature of `find` (`:29`)
- `leads_census_scope.rs:263`: write `.any(|u| unread[run.targets[u].package])`, dropping `own[u] ||`; every place that sets `own[t]` also sets `unread` for the target's package (`:239-248`, `:253-256`), and `census_check::check_sources` (`src/census_check.rs:45-50`) guards that a source's package is its target's package
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S4-1, S4-2, S4-5, S4-6, V4-1, S4-7, S4-9)

### Give the module-parent rule, the `Decl` item id, and the `name: type` text one home each

- Files: `src/model_impl.rs`, `src/leads_items.rs`, `src/surface.rs`, `src/analysis_items.rs`, `src/leads_docs.rs`, `src/leads_types.rs`
- Module parent:
  - add `Crate::parent_module(&self, m: ModId) -> Option<ModId>` in `model_impl.rs`, because `model.rs` is near the 500-line cap
  - call it at the five sites: `leads.rs:180`, `leads_names.rs:117`, `leads_names_modules.rs:87`, `:294`, `surface.rs:237`
  - delete `leads_items::parent_module` (`:365-369`) and `surface::parent` (`:307-311`), and their imports
  - tests that reach it: `single-item-module` (`leads_names_tests.rs:212`), `name-collision` (`:66`), `analysis_tests.rs:241-242`
- `ItemId::decl(target: usize, d: DeclId) -> ItemId`:
  - add it next to `ItemId` (`analysis_items.rs:44-49`) and delete `leads_shapes_twins::decl_item` (`:118-123`, five callers at `:94`, `:268`, `:394`, `:432`, `:471`)
  - replace every literal `ItemId { target: _, item: ItemRef::Decl(_) }`: 34 sites in 17 files, found by a multi-line search for `ItemRef::Decl(` inside an `ItemId` literal; leave the `Module` and `Variant` literals as they are
  - `leads_shapes_twins.rs` has 499 lines, so this change must only shrink it
- `name: type` text:
  - move `leads_docs::typed(name, ty)` (`:327-331`) next to `Ty::shown` (`leads_types.rs:99`) as `Ty::typed(&self, name)`
  - delete `leads_shapes_params::arg_text` (`:172-175`, one caller at `:192`), and use the helper at `leads_shapes_params.rs:73`; `leads_shapes.rs:103`, `:110-114`, `:231`; `leads_names_roles.rs:230`, `:240`; `leads_containers.rs:73`; `leads_docs_phrases.rs:292`, `:320`
  - a missing name still prints `_`
  - the evidence text must not change, so the goldens stay unchanged
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S5-11, S5-12, S5-13)

### Replace the hand-made byte and run loops in the docs scan and shapes detectors

- `src/leads_docs_scan.rs:157-162`: drop the `b` parameter of `run_of`, rename it `backtick_run`, doc "Counts the backticks at and after `at`, before `end`."; callers `close_run` (`:169`), `matching` (`:194`), `Reader::code` (`:309`)
- `src/leads_shapes.rs:153-162` and `src/leads_shapes_twins.rs:161-167`: replace the `start` index and `take_while` with `catalog.fields.chunk_by(|a, b| (a.decl, a.variant) == (b.decl, b.variant))`; `Cargo.toml` sets `rust-version = "1.85"`, and `chunk_by` is stable since 1.77
- Tests that must pass unchanged: `leads_shapes_tests.rs:44-55` (derived field) and `:73-80` (contained shape), and the `leads_docs_text.rs` prose tests (`:245`, `:278`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`, `cargo +1.85 check --all-targets --locked` (S5-16, V5-1)

### Fix doc comments, one glossary entry, and two visibilities in the docs, items, and names detectors

- `src/leads_docs_text.rs:75-78`: the first sentence of the `Phrase` doc becomes "A phrase: its words in order, each a set of the lowercase words that may stand there."; keep the second sentence
- `src/leads_docs_repeat.rs`: in the `doc_echo` doc (`:147-151`) write "fields, variants, or types"; in the `vocabulary` doc (`:75-76`) add "members" after "variants"
  - leave the evidence text "is in its name, signature, or types" (`:164`) alone, because two goldens pin it
- `src/leads_items.rs:100-104`: the `Func::is_own` doc becomes "Tells whether no trait sets the function's signature: it is free, or a member of an inherent impl."
- `src/leads_items.rs:152`: add this doc to `Catalog::fields`: "In order of declaration, then variant: `fields_of` searches it by that order, and `derived_field` and `contained_shape` read it in runs of one owner."
- `tools/vibelight.md:233`: change the label words in the `derived-field` entry to "a message (or `msg`), name, label, title, description, or reason"; leave `LABEL_WORDS` as it is
- `src/leads_names.rs:76-78`: the `IMPL_SUFFIXES` doc gains "The suffixes of [`WRAPPER_SUFFIXES`] but `_helper`: a `_helper` function is no private half of a public wrapper, so it is never left out of the lead."
- Drop `pub(crate)` from `QUOTE_CHARS` (`leads_docs_text.rs:17`) and `vis_of` (`leads_items.rs:357`)
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; `leads_items.rs` has 483 lines, so count lines after the module-parent and impl-module items (S5-1, S5-2, S5-3, S5-4, S5-6, S5-7, S5-8)

### Hold the receiver text inside `Receiver`

- Files: `src/model.rs`, `src/collect_fn.rs`, `src/render_slop_text.rs`, `src/layout.rs`, `src/leads_names.rs`, `src/leads_items.rs`, `src/leads_census_scope.rs`
- Problem: `Sig::self_text` is set only when the receiver is `Shared` or `Mutable`, and nothing in the type says so
- Change `Receiver { None, Shared, Mutable, Value }` (`model.rs:74-83`) to hold the text in `Shared(String)` and `Mutable(String)`, and delete `Sig::self_text` (`:96-98`)
  - producer, `collect_fn.rs:94-113`: compute `let text = file.clean(range.clone());` before the `match`, pass it to `typed_receiver(ty, text)` (`:173-184`, which now builds `Receiver::Mutable(text)` or `Receiver::Shared(text)` and gives `Receiver::Value` for any other type), and build `Receiver::Shared(text)` and `Receiver::Mutable(text)` in the two reference arms; test `matches!(receiver, Receiver::Value)` at `:108`; the one `Sig` literal at `:133-142` drops `self_text`
  - readers: `render_slop_text.rs:69` (`let mut params: Vec<String> = ...` takes the text from the two payload variants); `layout.rs:368` and `leads_names.rs:429` (`==` tests become `matches!`); the match at `leads_items.rs:117-120` gains payload patterns
  - `Receiver` stops being `Copy`; fix the other readers at `leads_items.rs:97`, `:339`, `leads_census_scope.rs:168`, `model.rs:150`, which compare with `==` or `!=` and keep compiling through `PartialEq` or turn into `matches!`
  - the doc edit for `Receiver::Mutable` lands here, `model.rs:80`: "`&mut self`, dropped and marked by a trailing `mut` after the parameters in the HTML" (the layout reads it at `layout.rs:368-370`), and `Shared` keeps "dropped with nothing in its place"
- Tests: the goldens `tests/fixtures/receivers/expected.md` and `expected.html` must not change
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S6-6, V6-1)

### Delete unread and duplicated state in the model and manifest code

- `src/model_impl.rs:40-48`, `src/attach.rs:48`, `:64-67`: delete `ImplLoc::range`, `header`, `notes`, and `trait_impl`, with their docs, the `header` local, and the four fill expressions including `impl_notes.clone()`; drop `PartialEq, Eq` from `ImplLoc` if nothing then needs them; the HTML goldens (`tests/fixtures/links/expected.html`) pin the two kept fields
- `Body::file`: delete it (`src/model_body.rs:77`) and the writer at `src/collect_body_facts.rs:84`; read `body.loc.file` at `collect_check.rs:69`, `:93`; `analysis_check.rs:205`; `analysis_uses.rs:265`, `:266`; `leads_comments.rs:172`, `:181`; `leads_docs_phrases.rs:240`; `render_slop_facts.rs:235`; `collect_body_facts.rs:274`; delete the `body.loc.file != body.file` clause at `collect_check.rs:96`
- `DepEntry::workspace`: delete the field, its doc, its fill at `src/manifest_facts.rs:216`, and the lines at `:397`, `:417`, `:421`; keep the local `workspace` at `:197`
- `ManifestFacts::validate`: delete `&& w[0] < w[1]` at `manifest_facts.rs:297`, drop `PartialOrd, Ord` from `DepEntry` (`:58`) and the phrase "which lead its order" (`:56-57`); `DepKind` keeps `Ord`
- `strings`: make `manifest::strings` (`manifest.rs:123-133`) `pub(crate)`, delete the copy at `manifest_facts.rs:254-264`, and call `manifest::strings(...)` at `:202`, `:203`, `:229`; leave `config.rs:139`, which does different work
- `Workspace::deps`: store `Dep` values (`manifest.rs:12-23`, `:37`) in place of `(PathBuf, bool)`, build them in `shared_deps` (`:136-146`), and clone the entry in `deps_in` (`:256-268`); use a `Vec<Dep>` searched by key, so the key is not repeated in the map
- Tests that must pass: `path_dependencies_come_from_every_table_but_dev_dependencies` (`manifest.rs:285`), `members_expand_globs_and_prune_excluded_directories` (`:326`), `facts_read_features_dependencies_and_target_roots` (`manifest_facts.rs:340`), `validate_rejects_broken_facts` (`:462-473`), `broken_facts_are_internal_errors` (`collect_check.rs:350-376`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S6-4, S6-5, V6-2, S6-11, S6-2, S6-22)

### Give `modtree` one children-directory rule and one pair of directories

- File: `src/modtree.rs`
- Problem: `children` (`:133`), `test_module` (`:206`), `test_children` (`:237`), `candidates` (`:279`), `inline_dir` (`:296`) each take `child_dir: &str` and `path_base: &str`, two arguments of one type; and the rule "a file's children live in its folder when the file is `mod.rs`-like, else in a folder named for the module" is written at `:105-110` and `:226-231`
- Add `struct Dirs { child_dir, path_base }` and pass it in place of the two strings; inline modules pass one directory twice (`:165`, `:211`)
- Add one function for the children-directory rule, used at `:105-110` and `:226-231`
- Let `test_module` reuse the resolution half of `children` (strip the name, read `#[path]`, branch on inline content, build `candidates`, `load`, test `chain`); also name the results of `load` and `candidates` in the same edit
- Tests that must pass: `walks_files_mod_rs_inline_and_path_modules` (`:349`), `test_only_modules_record_their_files_without_joining_the_tree` (`:416`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; `modtree.rs` has 460 lines (S6-12)

### Fix doc comments, visibility, and one closure in the model, order, and manifest code

- `src/model_body.rs:183-184`: "How often a body writes each error, absence, panic, and `.await` marker outside macro arguments."
- `src/model.rs:220`: "A struct, enum, union, trait, trait alias, or type alias."
- `src/model.rs:106`: "One parameter, tuple field, or named variant field and the comments above it."
- Visibility: make `Feature::enables` (`manifest_facts.rs:43`), `Feature::implicit` (`:45`), `DepEntry::optional` (`:65`), and `order::module_mentions` (`order.rs:280`) private; `Feature::name` stays `pub(crate)`
- `src/model.rs:442-443`: replace the `strip` closure in `name_order` with `strip_raw` from `typerefs`, adding the import; `names_sort_without_raw_prefix_and_case` (`:467`) pins the order
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; the `Receiver::Mutable` doc is edited in the receiver item (S6-18, S6-19, S6-20, S6-23)

### Hold the analysis target once and rank entries by one type in the slop renderer

- Files: `src/render_slop.rs`, `src/render_slop_facts.rs`, `src/render_slop_items.rs`, `src/render_slop_decl.rs`, `src/render_slop_lists.rs`, `src/render_slop_wire.rs`, `src/render_slop_schema.rs`, `src/render_slop_split_tests.rs`
- Problem: `Outline` and `Facts` each hold `run` and `t` again beside `AnalysisTarget`, `krate` is `&run.krates[t]`, and `Entry` rank is four fields that `lift` repairs after `push`
- Target:
  - `Outline` holds one `target: AnalysisTarget<'a>` and derives `krate`; `Facts::new` takes the `AnalysisTarget`; delete the `run`, `t`, and duplicate `krate` fields
  - reads to update: `outline.krate` on 16 lines, `outline.run` or `outline.t` on 7, `self.t` and `self.run` in `Outline` (3) and `Facts` (about 12, `render_slop_facts.rs:178-265`)
- Rank:
  - add `enum Rank { Own, Reexport(String), Item { file: String, line: u32 } }` deriving `Ord` in that variant order, and let `Entry` hold one `rank` in place of `group`, `key`, `file`, and `line` (`render_slop.rs:55-64`)
  - keep `push` for `Rank::Item` (12 callers) and add `push_ranked(rank, ...)` for the two `lift` callers (`render_slop_items.rs:121-122`, `:146-147`); delete `lift` (`:112-121`)
  - the sort becomes module rank, then `rank` (`:160-166`); the order of equal re-export keys falls to push order, so run the goldens
- Rename `Loc::target` to `stem` (`render_slop_schema.rs:151-152`, `:159-160`, `:179-185`; `render_slop.rs:79`; `render_slop_split_tests.rs:19`); `Loc` serializes as one string, so the wire output does not change; leave the `t` parameters alone
- Tests: the 59 markdown goldens, `tests/fixtures/reexports/expected.md`, and `src/render_slop_tests.rs` must not change
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S7-11, S7-12, S7-8)

### Share `escape` and `under` in the renderers

- `src/render_prose.rs`: delete the private `escape` (`:217-229`) and write `crate::render_html::escape(words.trim())` at `:105`; `render_html` already calls `render_prose::block`, and `render_spans` already imports `render_html::escape`
  - pin test: add `a_heading_escapes_markup` for `render_prose::subheading`, that `&` and `<` in an `<h4>` text are escaped; no test covers `subheading`, and it passes before and after
- `src/render_slop_lists.rs`: delete the private `under` (`:56-61`), add `use crate::render_slop_items::under;`, and change the three calls in `surface` (`:84-86`) to `under(&krate.modules[*m].path, name)`
- Tests that must pass: the surface goldens (`tests/fixtures/surface/expected.md`, `tests/fixtures/value-links/expected.md`) unchanged
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S7-3, S7-4)

### Fix doc comments, one name, and one test in the renderer and order modules

- `src/render_html.rs:13-14`: the `TOGGLE_ID` doc becomes "The id of the checkbox that hides the member comments, which the checkbox's `id` and the sidebar label's `for` both name."
- `src/order_graph.rs:67-68`: the two comment lines become "Every group has a root above it, so nothing is left; a group that is left" and "anyway would still be placed, in path order, rather than lost."
- `src/pipeline.rs:30-31`: the `Outcome::files` doc becomes "Each file the run writes, with its text: for each target in output stem order its markdown outline and then its HTML page, then the markdown index and the HTML index."
- `src/render_slop_items.rs:38-39`: the `Place::support` doc becomes "Returns `Some(true)` when the node is test support: its owner is, or its own `cfg`s make it so, as for a declaration, and `None` otherwise."
- Rename the `LinkBase` names in `src/pipeline.rs` to `link_base`: `:45`, `:120`, `:201`, `:213`, `:253`, `:265`; `Chosen::base` and the `shown` parameter keep their name
- `src/render_slop_schema.rs`, wire records that stay as they are (S7-14, S7-15): add to the `MoreRecord` doc (`:310-311`) "Exactly one of `items` and `text` is set: `items` continues an array field and `text` continues a string field."; add to the `module` doc of `LeadRecord` (`:224-229`) "set together with `count`, and never alone" and to the `count` doc "set together with `module`"; the checker at `src/render_slop_check.rs:206-208` already enforces the pairing for the lead
- Add the pin test `style_padding_matches_box_padding` in `src/render_html.rs` (474 lines): `STYLE` holds `padding:.2rem {BOX_PADDING} 0`, next to the existing style tests in `render_style.rs:56-77`; leave `BOX_PADDING` and the style sheet as they are
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S7-1, S7-2, S7-9, V7-1, S7-7, S7-21, S7-14, S7-15)

### Remove the HTML options that can only take one value

- Files: `src/settings.rs`, `src/layout_decl.rs`, `src/layout_items.rs`, `src/layout_head.rs`, `src/layout_marker.rs`, `src/document.rs`, `src/pipeline.rs`
- Delete `MembersBy`, `OrderBy`, `KillList`, `Notation`, `Options`, the `HTML` constant, and `Output::options` (`settings.rs:13-59`, `:94-99`); `Output` keeps its `dir`
- Drop the `members` and `notation` parameters and the one-arm matches:
  - `layout_decl.rs:17`, `:20`, `:57-59`, and the test at `layout_head.rs:167`
  - `layout_items.rs:14`, `:19`, `:22`, `:74`, `:82`, `:313-317`, `:359`, `:386`
  - `document.rs:9`, `:114`, `:136`, `:158-159`, `:167`, `:175-181`, `:195-201`, `:208`; `Inputs::options`, `Builder::notation`
  - `pipeline.rs:18`, `:247`, `:249-251`; keep the legend always on, which removes the `legend` flag test
- Update the test `assert_eq!(plan.html().options, HTML)` at `settings.rs:202`
- New `settings.rs` module doc: "Settles what a run covers and where it writes, from the command line and `vibelight.toml`. The markdown has no options; only the config's boilerplate and quiet entries tune its leads."
- Doc edits in `document.rs` that land with this item:
  - module doc (`:1`): "Builds the outline document that the HTML renderer prints."
  - `legend` (`:91-93`): "The Legend section, printed after the crate docs."
  - `kill` (`:98-99`): "One entry per macro."
  - `value_ids` (`:104-106`): "The id of each module-level function, constant, and static."
- Delete the order-by-impl plumbing that this change leaves unreachable, in the same item (`impls` is empty on every run: `layout_decl.rs:57` sets it from the one-variant `MembersBy`, and `layout_items.rs:430` passes `&[]`):
  - `BodyParts::impls` and its doc (`layout_decl.rs:251-253`), the two `BodyParts` literals (`layout_decl.rs:71`, `layout_items.rs:430`), `by_impl` and `held` (`layout_decl.rs:265-266`, `:285-292`, `:296`)
  - the loop and the two parameters of `order_groups` (`:105-123`), leaving a plain sort; `impl_line` (`:126-135`); `Marker::IMPL_IN` (`layout_marker.rs:41-46`); the `Range` import at `layout_decl.rs:3`
  - no golden holds a `// impl in` line, so no golden changes
- Order: land this item before the document-builder item, because both edit `document.rs` and `pipeline.rs`
- The output must not change: every HTML and markdown golden stays as it is
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S8-5, S2-12, S2-11, V11-2)

### Store the resolved reference as one enum

- Files: `src/text.rs`, `src/resolve.rs`, `src/route.rs`, `src/resolve_run.rs`, `src/spellings.rs`, `src/render_spans.rs`, `src/analysis_check.rs`, `src/analysis_paths.rs`, `src/analysis_items.rs`, `src/leads_names_modules.rs`, `src/resolve_values.rs`, and the nine literal sites below
- Problem: `Found` (`resolve.rs:32-36`) and `TypeRef` (`text.rs:172-203`) hold `target: Option<DeclId>`, `value: Option<ValueRef>`, and `canonical: Option<Vec<String>>` as three options of which at most one is set; `route_derives` moves a reference between states by hand (`route.rs:115-116`)
- Add `enum Res { Decl(DeclId), Value(ValueRef), Outside(Vec<String>) }` beside `TypeRef` in `text.rs`, deriving what the three field types allow, and hold it as `res: Option<Res>` on `TypeRef` in place of the three fields; `foreign: Option<CrossRef>` stays a separate field, because `route::spell` reads it beside the outside path (`route.rs:181-190`)
  - add three accessors on `TypeRef`, `decl() -> Option<DeclId>`, `value() -> Option<ValueRef>`, and `outside() -> Option<&[String]>`, so most readers change one token
  - replace `Found` and its constructors with `Option<Res>` (`Found::default()` becomes `None`, `Found::decl(id)` becomes `Some(Res::Decl(id))`)
  - `route_derives` replaces one variant with another by assigning `res`
- Do it in two compiler-guided steps: first the type, the accessors, and the nine literals; then the readers
  - nine `TypeRef { .. }` literals: `text.rs:226`, `collect_body.rs:33`, `collect_body_facts.rs:115`, `comments.rs:211`, `collect_use.rs:85`, `typerefs.rs:240`, `render_values.rs:28`, `collect_bases.rs:85`, `route.rs:227`
  - readers: `resolve_run.rs:63`; `spellings.rs:142-143`; `render_spans.rs:146`; `analysis_check.rs:328`; `analysis_paths.rs:107`, `:257`; `route.rs:42-44`, `:162-195`; `analysis_items.rs:225`; `leads_names_modules.rs:272`; `resolve_values.rs:67`
- Size: `resolve.rs` has 478 lines, so count lines after `cargo fmt`
- Tests that must pass: the `resolve.rs` unit tests (`:313-475`), `resolve_values.rs` tests, `route.rs` tests (from `:211`), and the HTML and markdown goldens
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`, `cargo +1.85 check --all-targets --locked` (S8-3)

### Delete `Target::lib` and keep the kind

- Files: `src/root.rs`, `src/pipeline.rs`, `src/collect_check.rs`
- Delete `Target::lib` (`root.rs:23`) and its three constructors' assignments (`:120-121`, `:220-221`, `:235-236`)
- Read `target.kind == TargetKind::Lib` at `pipeline.rs:86` and `:319`, with `TargetKind` in scope
- In `collect_check::validate_package_facts` (`:54-62`) keep only the single-file half (`single && !alone`), and update its doc at `:37-39`
- Update the test `each_target_records_its_kind_beside_lib` (`root.rs:381-390`)
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S8-4)

### Share one wire-type test

- Files: `src/model_impl.rs`, `src/leads_items.rs`, `src/render_slop_wire.rs`, `src/leads_shapes_traits.rs`
- Problem: the rule "a type is a wire type when a written derive ends in `Deserialize` or `Serialize`, or an attribute starts with `serde`" is written in `leads_items::is_wire` (`:473-483`), `render_slop_wire::is_wire` (`:144-149`), and inline in `leads_shapes_traits::thin_wrapper` (`:182-184`, with `leaf(name)` in place of `rsplit("::")`)
- Add `impl Decl { fn is_wire(&self) -> bool }` in `src/model_impl.rs` with that rule
  - `leads_items::is_wire` keeps only the lookup of the `Decl` and calls it; `render_slop_wire::is_wire` calls it and its body goes
  - replace the inline copy in `thin_wrapper` with `is_wire(run, at)`, drop the `leaf` import at `leads_shapes_traits.rs:15` if nothing else uses it, and add `is_wire` to the `leads_items` import at `:14`
  - state the rule once on the method; the `render_slop_wire` module doc gains "A type is a wire type as `Decl::is_wire` says."
  - pin test first: `a_serde_type_is_not_a_thin_wrapper` in `src/leads_impls_tests.rs`, a `serde` type is exempt from `thin_wrapper` (today only the plain and method cases exist, `:276-285`); it passes before and after
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`; the goldens stay unchanged (S8-6, S5-9, S5-10)

### Remove three small duplicates in the root, route, and settings code

- `src/leads_census_scope.rs:42-54`: delete `STD_DERIVES` and its doc; write `derive_path(&leaf(name)).is_some()` in `plain` (`:365`) with `use crate::route::derive_path;`; `route::DERIVES` (`route.rs:16-26`) lists the same nine names, and `route::derive_path` stays `pub(crate)`
- `src/root.rs:127-131`: replace the read and parse with `let table = manifest::read(manifest)?;` and add `use crate::manifest;`
- `src/typerefs.rs:45`: make `Finder::new` private (its one caller is `typerefs.rs:187`); `src/settings.rs:77`: make the field `Plan::out` private, because only `settings.rs` reads it (`:91`, `:96`, and its tests), and change its doc to "The base folder; the HTML files go in its `html` folder."
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S4-10, S7-10, S8-7, S8-10, S8-8)

### Fix names, docs, and one error message in the resolver, text, wrap, and surface modules

- Rename the `values` parameter to `may_be_value` in `Scope::walk` and `Scope::lookup_in` (`resolve.rs:204`, `:255`, `:283-284`) and `World::extern_path`, `walk`, `follow`, `lookup_in` (`resolve_run.rs:91`, `:101`, `:129`, `:142`, `:179-181`), and the local at `resolve_run.rs:71`; the four docs read "With `may_be_value` set"; the field `Scope::values` keeps its name
- Replace the repeated order sentence in `World::lookup_in` (`resolve_run.rs:139-141`) with a pointer to `Scope::lookup_in`; the two walkers stay two, as decided
- `src/root.rs:133-137`: the workspace-manifest error becomes "{} is a workspace manifest; point vibelight at a member crate's directory, or set `workspace = true` in vibelight.toml"; the tests check only the phrase "workspace manifest" (`root.rs:397`, `tests/it/cli.rs:164`)
- `src/text.rs`:
  - struct doc (`:170`): "A type name found in a declaration, or a comment reference kept for analysis, which `comment` marks."
  - `comment` doc (`:183-186`): "True for a comment reference: a name from a comment, a re-export, a derive, a function body, a trait impl member, or a `serde` attribute path, which never counts for naming, toward the unresolved tally, or for spellings."
  - `may_be_value` doc (`:187-189`): "True for a name that may also be a function, constant, or static: one in a comment, a re-export, a function body's call or pattern path, or a `serde` attribute path. Never a derive, a type name, a struct literal's path, or a path prefix."
- Write the rule in words where a doc cites a plan's numbered rule:
  - `resolve.rs:1`: "Resolves each type name to a declaration, a function, constant, or static of the crate, or to its full path outside the crate."
  - `wrap.rs:9`: "A function declaration split where the long-line rule may break it."
  - `wrap.rs:23`: "A `where` clause laid out both ways the long-line rule prints it."
  - `wrap.rs:48`: "Lays out a function declaration at `indent` under the long-line rule. A parameter with a comment forces one parameter per line."
  - `model_body.rs:9`, `collect_body.rs:2` (rule 26), `model.rs:439` (rule 9), `comments.rs:50` (rule 20), `order.rs:36` (rule 30), `order.rs:253` (rule 24): replace "rule N" with the rule in words
- `src/surface.rs`: `Found::Value` doc (`:42`) becomes "A constant, a static, or an extern type, with whether it is test support."; the `lookup` doc list (`:246-249`) ends "a function, a constant, a static, or an extern type of the crate."
- Verify: `cargo doc --no-deps --locked`, then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S8-2, S8-9, S8-13, V8-1, S8-14, S8-18)

### Correct `README.md`, `AGENTS.md`, the lens, and two test docs against the code

- `README.md:13`: delete "how often its name is written, " and leave the rest of the sentence, whose body clause already matches the body record
- `README.md:75`: "a member whose name or folder starts with the word `build` (names split at `-` and `_`, so `build-ui` counts and `builder` does not), or has `xtask` or `xtasks` as a word of its name or path, as build tooling" (`scaffold::by_name`, `src/scaffold.rs:142-158`, is the rule)
- `README.md:83` and `:87`: "`src/bin/*.rs` and `src/bin/*/main.rs`", and add that each `src/bin/*/main.rs` takes its folder name
- `README.md:103`: add "and the same tables under `[target.*]`, and the `dev_dependencies` and `build_dependencies` spellings Cargo still reads"; dev-dependencies stay left out of the followed tables
- `README.md:125`: "Every outline and every HTML page starts with a stamp, ..." and add that `index.md` and the `.gitignore` have none
- `README.md:145`: call the part the `MACROS:` line at the top of the page, keep the description, and say the sidebar does not link it
- `README.md:156`: write "`dir`, the crate's folder below the base of the run (left out for the base itself)"
- `README.md:163`: add "The `mod` record names `vis` (left out for the crate root), `types` (how many structs, enums, unions, traits, and aliases it declares), and `functions` (how many free functions it has at each visibility, as `public`, `internal`, and `private`), each left out at 0."
- `README.md:166` and `tools/vibelight.md:205`: write the `body` sentence once and use it in both:
  - `statements` and `tokens`; `loops` and `loop_depth` (shown above 1); `tries` (`?` operators), `errs` and `nones` (paths ending in `Err` and `None`); `awaits`; `panics`; `macros`, the sorted names of the macros the body invokes; `single`, what a one-statement body does; counts are left out at 0
  - `tools/vibelight.md:205` is inside the `slop-lens` block, so the edit reaches every run; `src/leads_tests.rs:260-300` checks only the `lead-glossary` block, so it does not touch this line
- `README.md:178`: "A constant or static has a `value` only when its initializer is one literal of at most 100 characters"
  - `src/render_slop_catalog.rs:274`: the `ValueRecord::value` doc says "one short literal"
- `README.md:277`: add "doc files that cannot be read, and module files that load themselves again" to the list of warnings
- `AGENTS.md:3`: replace "but never another crate's HTML" with "but an HTML page changes only with its own crate and the crates it links to"
  - `tests/it/stamp.rs:3-5` and `tests/it/exclude.rs:4-6`: "depends on its own crate and the crates it links to"; the exclude cases leave out a downstream or followed crate, which is true for them but not in general
- Verify: read each edited line against the code it cites; then `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S9-2 to S9-11, S9-13, S9-14, V9-1, V9-2, V9-3)

### Bring the ten dated plans up to date

- Constraints: edit prose only; keep every rule number in `plans/2026-10-25-vibelight-tool.plan.md`, because source comments cite them; do not move or rename a plan, because `build/structure.rs:195-221` fails on a subfolder or a bad name
- Todo statuses: nine plans mark every todo `pending` or `in_progress`, 97 todos in all: `2026-10-25-vibelight-tool` 19, `2026-10-25-analysis-outline-upgrades` 19, `2026-10-26-one-time-analysis` 7, `2026-10-26-vibelight-agent-tool` 8, `2026-10-27-dependency-order-groups` 9, `2026-10-27-zero-switch-redesign` 11, `2026-10-29-html-reading-revision` 8, `2026-10-29-markdown-slop-renderer` 11, `2026-10-30-html-data-first` 5; `2026-10-26-output-analysis-upgrades` is already right
  - set every todo to `completed` whose work the history shows shipped (check with `git log --oneline -- plans/<name>` and the symbols the todo names)
  - leave the `dogfood` todo of `plans/2026-10-29-markdown-slop-renderer.plan.md` open, because a manual run at the runvil root leaves no trace here
  - `plans/2026-10-27-zero-switch-redesign.plan.md:6`: change the path to `vibelight/plans/2026-10-27-zero-switch-redesign.plan.md`
- Markdown plan (`plans/2026-10-29-markdown-slop-renderer.plan.md`):
  - replace the three example records (`:123-125`) with the shipped shape: a location is one string, no `url`, docs are separate `doc` records, no `getters` and no `method_name_hits`
  - `:133` and `:143`: "a location is one string, `<file>:<line>`, or `<stem>:<file>:<line>`, and no record has a `url`"
  - `:144`: add that a record wider than 900 characters continues in `more` records
  - `:290`: delete the key names `method_name_hits` and `token_hits`; edit `:289-290` and `:317` together
  - `:168`: the `direct-field-clusters` rule needs 4 or more methods; `:249`: the `repeated-number` rule needs a magnitude of 10 or more other than 16, 32, 64, 100, 128, 256, 512, 1000, and 1024
  - `:263-278`: replace with one sentence that the glossary inside `<lead-glossary>` in `tools/vibelight.md` is the reference for ids, rules, and form, and that `src/leads_tests.rs` keeps it equal to the registry
- Four older plans (2026-10-25 outline upgrades, 2026-10-26 one-time analysis, 2026-10-26 agent tool, 2026-10-26 output upgrades): add one banner line at the top of each body saying in plain words that the plan is a record of work done, and naming what replaced its switches and output: the zero-switch plan, the dependency-order plan, and the markdown renderer plan; leave the bodies as they are
- Original plan (`plans/2026-10-25-vibelight-tool.plan.md`), keeping every rule number and rule text:
  - limit the byte-identical promise to the HTML in rule 54 (`:205`), the Determinism hard rule (`:83`), `:473`, and the arrangement tests (`:497`), and add one sentence that the markdown is contextual to the run and follows file and line, as `tests/it/arrange.rs:1-6` and `AGENTS.md:3` say; edit rule 61 (`:215`) to say every outline and HTML page starts with a stamp, except `index.md` and `.gitignore`
  - the plan stays the live rulebook, as `plans/2026-10-25-analysis-outline-upgrades.plan.md:142-143` says, so leave that sentence alone; add this status line directly under the plan's title, keeping every rule number and rule text: "Status: the goldens outrank these rules. Rules 2, 3, 7, 8, 9, 20, 27, 28, 29, 31, 32, 35, 36, 37, 42, 48, 50, 54, 56, 57, 59, 60, 61, 64, 65, 66, 67, 68, 69, 70, and 71 are stale in part. The later plans hold the current rules: switches, config keys, and written folders in the zero-switch plan of 2026-10-27; levels, groups, and sections in the dependency-order plan of 2026-10-27; class and struct, the visibility line, and the data-first layout in the data-first plan of 2026-10-30; comment reflow and the legend in the reading-revision plan of 2026-10-29; and the markdown format, determinism, and stamps in the markdown renderer plan of 2026-10-29 and in `AGENTS.md`."
  - the split of rules to later plans is read from the verifiers' notes, so check each listed rule against the plan named for it before saving, and move a rule to the plan that actually replaced it
- `plans/2026-10-29-html-reading-revision.plan.md`: add one note at the top of the body that items 6 and 7 and the legend count were revised by later commits, and that `src/render_style.rs`, `src/render_html.rs`, and `src/index.rs` are the truth; the plan's wording at `:94`, `:162`, `:183`, `:218`, `:231` differs from the shipped toggle, box, legend, and index page
- `plans/2026-10-27-zero-switch-redesign.plan.md`: add to the Outcome's "Small changes not in the plan" bullet (`:171`) "the fallback root is the nearest folder at or above the current one that holds a `Cargo.toml`, and a first run notes only the config file and the `.gitignore`"; add one closing bullet after `:172` that Phase 2 and the fixed options were superseded by the dependency-order, reading-revision, and markdown renderer plans, and that `boilerplate` and `quiet` came later in commit `ca47e37`; leave `:80`
- `plans/2026-10-30-html-data-first.plan.md:66-80`: change `struct Run` to `class Run` in the example
- Verify: no tool reads a plan body; check the edited lines against the commits and symbols named above, then run `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (the plans folder naming test is in `build/structure.rs`) (S10-1, S10-2, S10-5, S10-7, S10-11, S10-13, S10-14, V10-1, V11-1)

### Make the legend and the README match the notation the page prints

- Files: `src/legend.rs`, `README.md`, and the 59 HTML goldens under `tests/fixtures`
- Problem: the page prints ` internal` or ` private` as a dim comment span with no `//`, prints `implements: Clone for Box<Engine>` with no closing `;`, and prints only the trait or type name for an impl comment; the legend and README say otherwise
- `src/legend.rs:13`: the line becomes "A module's item that is not public has a line above it that reads `internal` or `private`."
- `src/legend.rs:15`: drop the closing semicolon, so the line reads "`implements: Trait for Type` is an impl with no declared type to attach to, such as a blanket impl."
- `README.md:148`: drop the same semicolon; `:246`: "A module's trailing block marks each item that is not public with an `internal` or `private` line above it."; `:250`: say impl comments sit under a line that names the trait, or the type for an inherent impl
- All 59 HTML goldens hold both legend lines (each contains `implements: Trait for Type;`) and change in one commit: replace the two sentences in each HTML golden with the text `src/legend.rs` now holds, as the page prints it, and read the diff before accepting it, as `AGENTS.md:4` requires; the diff is two lines in each of 59 files
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`, with the 59 goldens updated for these two lines only (V11-3)

### Join a leading `::` after a colon or an arrow

- File: `src/census_paths.rs`
- Problem: `Toks::start` joins a leading `::` to the token before it when that token is `:` or the `>` of `->` or `=>`, so `x: ::std::vec::Vec<u8>` starts no path and no qualified-path hit is recorded; no doc or record gives a reason (`tests/fixtures/spellings/src/lib.rs:29` has the shape)
- Test first: add `a_path_after_a_colon_or_an_arrow_starts_a_qualified_path` next to `paths_split_use_groups_and_skip_turbofish_and_method_paths` (`:192-238`), with cases `x: ::a::B`, `-> ::a::B`, and `=> ::a::B`; it fails on today's code
- In `Toks::start` (`:57-76`), treat `>` as joined only when it closes generic arguments, meaning it is not preceded by `-` or `=` (the `angles_end` function at `:113-130` already knows that `->` is no closing angle), and drop `':'` from the joined set at `:72`
  - the continuation is joined by `colons(i - 2)` (`:61`), so `std` in `x: ::std::vec::Vec<u8>` starts the path once the colon no longer joins
- Effect: `no-observed-path-use` and `test-only-observed-use` see more qualified uses, so read the golden diffs of the `spellings` fixture and any lead that changes before accepting them
- Verify: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (S2-18)

### Add the close-out sentence to `AGENTS.md`

- File: `AGENTS.md:1`
- Append "When the work ends, set the copy's todos to `completed` and add a short Outcome section." to the first bullet
- Verify: read the line; no check reads it (V10-2)

### Order of the items

- Land the options removal before the document-builder item, because both edit `document.rs` and `pipeline.rs`
- The `AnalysisRun::deps` rename sits inside the doc-and-name item for the attribute, census, and check modules; it touches `analysis.rs`, `analysis_check.rs`, `leads_census_scope.rs`, and `analysis_tests.rs`, so land it before or after the derive-three-fields item, and find each site by symbol
- The keyword item covers all three tables, so after it one keyword table remains
- The legend item and the options item are independent, in either order
- Every other item is independent of the rest; the three doc-only items for plans, README, and legend do not touch code
- No item waits on an operator decision: the lookup walkers (S8-1) stay two, which is the doc pointer and rename already in the resolver names item; the wire records (S7-14, S7-15) stay as they are, with the doc sentences in the renderer docs item; and the token rules (S2-1) stay apart, with the module doc sentences in the keyword item

</execution-plan>

## Outcome

- All 37 work items ran on 2026-10-30, each by a fresh subagent, and the tree is green: `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked` (367 unit and 141 integration tests), `cargo doc --no-deps --locked`, and `cargo +1.85 check --all-targets --locked`
- A fresh plain `vibelight` run at the root shows 288 lead records, down from 340, an outline of 1090 KB, down from 1106 KB, and one spelling warning, down from six
- Goldens: 59 HTML goldens changed by the two legend sentences only, and no markdown golden changed
- Not in the plan: `tests/it/support.rs` now starts the binary in the system temp folder, because the generated `vibelight.toml` (`follow = ["**"]`) otherwise changed the output of `naming_the_crates_in_reverse_gives_the_same_files`
- Deviations worth knowing:
  - `Edge::via` has only test readers after the `via` check went, so it carries a `cfg_attr(not(test), expect(dead_code, reason = ...))`
  - `layout.rs` is at 500 lines and `modtree.rs` at 499, after small behavior-equal trims to stay under the cap
  - `Builder::printed` became `Builder::blocks`, because the builder still needs the blocks
  - the README `body` and `mod` sentences say fields after `tokens` and the last two `mod` fields are left out at 0 or empty, which is what the code does; `html/index.html` has a stamp, so only `index.md` and `.gitignore` have none
  - the markdown plan's glossary paragraph was replaced without deleting the Group F rules that follow it
  - doc comments in `src/layout_items.rs` still say `implements: Trait for Type;`, left for a later pass
- Not run: `cargo package --locked`, which needs a clean tree, and the `dogfood` todo of the markdown plan, a manual run at the runvil root
- Nothing was committed, pushed, or tagged

