---
name: fields first in html
overview: "Two changes to the HTML output only, for the architect's taste review: each type reads data first, then behavior, under C++ access labels that print only when the access changes, and each sidebar group is captioned by its lead type. No markdown, analysis, or facts code is touched."
todos:
  - id: split
    content: In layout_decl::body, route fields and enum variants into a data set of visibility sections, leaving the existing sections for everything else
    status: pending
  - id: access
    content: "Rewrite join_sections to walk the data sections then the member sections with one running access state: a label prints only when a section's visibility differs from the access in effect, starting from the keyword's default"
    status: pending
  - id: captions
    content: "Caption each HTML group as module plus lead type, such as 'execute::scheduler · EffectTable', with an ordinal only if a caption still repeats"
    status: pending
  - id: goldens
    content: Regenerate only the HTML goldens after each change, read each diff, and confirm no other file changed
    status: pending
  - id: docs
    content: Update the HTML legend line and the README passages that describe the HTML, so they match the output
    status: pending
isProject: false
---

# HTML for the architect's taste review

The HTML is for the one judgment the model cannot make: whether the taste implied by its choices is the taste the architect wants. The review reads each top-level object in this order:

- **The data members:** should these be here, and are these the right types?
- **The functions:** what does each take and return, such as a `String` that becomes JSON versus a typed value that serializes?
- **The access split:** how much is public, internal, and private, and is that the API you want?
- **The balance:** is responsibility divided between the functions the way you would divide it?

Both changes put the page in that reading order. The facts stay out of the HTML: they are the model's detection tools, and the markdown has them.

## Scope: the HTML output only

Every source file this plan edits is already HTML-only, which a search of the code confirms:

- The markdown is rendered by `render_slop_*` from the analysis (`analysis*`, `census*`, `leads*`) and never calls `layout_decl`, `layout`, `groups`, `document_sections`, or `order::order`. The Rust-notation layout files are gone.
- The `Notation` and `OrderBy` enums in [src/settings.rs](vibelight/src/settings.rs) each have one variant, so the C++ layout path serves the HTML and nothing else.

So no step needs an option flag, and the markdown cannot change. Two checks prove it at the end: every `expected.md` is untouched, and `git diff --stat` lists only the files named below.

Files this plan edits:

- [src/layout_decl.rs](vibelight/src/layout_decl.rs) and [src/layout.rs](vibelight/src/layout.rs): type bodies and access labels.
- [src/document_sections.rs](vibelight/src/document_sections.rs): group captions.
- [src/legend.rs](vibelight/src/legend.rs): the HTML legend.
- `tests/fixtures/*/expected.html`: the HTML goldens.
- [README.md](vibelight/README.md): only the passages that describe the HTML. This is the one file that is not output, so drop it if you want output alone.

Files it does not edit: `render_slop*`, `analysis*`, `census*`, `leads*`, `order*`, `groups.rs`, `settings.rs`, `tools/vibelight.md`, every `expected.md`.

## 1. Each type reads data, then behavior

### The problem

`execute::scheduler::Scheduler` has all fields private and all methods `pub(crate)`. The body splits by visibility first, so `internal:` runs lines 3 to 342 with 94 methods, and the eight fields only start at line 343 under `private:`. The group order inside a section is already right. The section split buries the data, so the data has to come out of the sections and print first.

### The rule: a label prints when the access changes

The keyword sets the starting access (`class` is private, `struct`, `union`, and `enum` are public), and a label stays in effect until the next one, as in a C++ header:

- The data sections print first, then the member sections, as one sequence of up to six sections.
- One running `current` access starts at the keyword's default. Before each non-empty section: a blank line if anything precedes it, then a label only if the section's visibility differs from `current`, then `current` becomes that visibility.
- A visibility that has both data and members can show its label twice.

```
class Scheduler          struct Run               class Probe
{                        {                        {
    queue: ...           internal:                    state: State
    ...8 fields              id: RunId
                                                  internal:
internal:                private:                     run() mut
    run() mut                cache: HashMap<..>
    ...94 methods            name: String         private:
}                                                     step()
                         public:                  }
                             static new() -> Run
                         }
```

`Probe`'s second `private:` is required: without it, `step()` would read as internal. So `join_sections` must not simply drop the default label wherever it falls; the running access does that correctly.

### Work

- [src/layout_decl.rs](vibelight/src/layout_decl.rs), `body`: a second `sections(6)` set holds the data. `MemberKind::Field` goes to `data[section(vis)][2]`, and an enum's variants go to `data[0][2]`, so data comes first for every type. The data set never passes through `order_groups`, so fields keep declared order. Tuple-struct fields are never `Field` members, and a variant's fields stay nested inside it. The inherent-impl caller of `body` in [src/layout_items.rs](vibelight/src/layout_items.rs) has no fields, so its output is unchanged.
- [src/layout.rs](vibelight/src/layout.rs), `join_sections`: take both sets in order and replace the `first` flag with `current: Vis` starting at `default`. With one set the output is identical to today's rule, because sections come in the fixed order public, internal, private, so only the first can equal the default. A struct can now print `public:` when public members follow non-public data. Update this doc comment and the `BodyParts::class` one.

### What may shift in the HTML

The dependency order reads each block's mentions in print order, and moving fields earlier changes that order. `declared_mentions` in [src/order.rs](vibelight/src/order.rs) already lists every field's types first, so the sidebar order should not move. The golden diffs settle it, and any reorder is read like any other diff.

## 2. The sidebar names the top-level objects

### The problem

You start a review from a top-level object, and the dependency order already puts each one first in its group. But the captions name only the module. The engine page has 17 groups, and `execute::scheduler` is a caption 4 times, with `execute::run`, `execute::requirements`, and `heading_address` twice each. A lone type from another module also joins its neighbor's group, so `heading_address` holds `execute::scheduler::chat::ArrivedRound`. The caption does not say which object a group is about.

### Work

- [src/document_sections.rs](vibelight/src/document_sections.rs), `grouped`: caption every group as its module plus its lead type, such as `execute::scheduler · Scheduler`, `execute::scheduler · EffectTable`, and `execute::scheduler · Discovered`. The lead is the first declaration in the group whose module falls under the group's key, falling back to the first declaration. Every group gets the lead, so the sidebar reads as a list of top-level objects.
- If a caption still repeats, the later ones get ` (2)`, ` (3)`.
- `Group::caption` in [src/groups.rs](vibelight/src/groups.rs) keeps returning the module path, so its tests stand unchanged. The caption reaches the sidebar and each group's heading, since both use the same text.

## Docs

- [src/legend.rs](vibelight/src/legend.rs) line 12 still says a type that is not public gets `// internal` above its header. That now applies only to a module's free items. Say that data comes first and a label appears where the access changes.
- [README.md](vibelight/README.md), HTML passages only:
  - Lines 15 and 147 say a group heading is named for its module; add the lead type.
  - "The HTML notation" (line 203 on): redraw the example data-first, and fix what the earlier HTML work left stale in the same passage: the semicolons, the bordered comment box, the `[[...]]` lines above members, the `// internal` line above a type header, and the three sections in a module block.
  - No markdown passage changes.

## Order of work

- The two changes touch different files and neither needs the other's output. Do them one at a time so each golden diff shows one change.
- Inside change 1, the split and the label rule land together. Neither is correct alone.
- Regenerating the goldens needs a throwaway harness. It is never committed, and it is deleted before the end, as `AGENTS.md` and your earlier instruction require. Each change's goldens go in the same commit as its code, when you ask for commits.
- The legend and README go last, written against the final output.

## Verification

From `c:\Users\Vinnie\cursor\vibelight`:

- `cargo test`, `cargo clippy --all-targets`, `cargo fmt --all --check`, all clean. The 500-line ceiling holds: `layout_decl.rs` is at 341 lines, and `layout.rs` is at 486, so the `join_sections` rewrite must not add more than 14 lines or move into its own file.
- `git diff --stat` shows only the files listed in Scope, and every `expected.md` is unchanged.
- The existing fixtures already cover the access cases (`visibility`, `structs`, `struct-class`, `unions`, `prompt`), so no new fixture is needed. If none shows a class that goes `internal:` then back to `private:`, stop and ask before adding one.
- `cargo run -- ../runvil/crates/runvil-internal/engine -o target/check`, then in `target/check/html/runvil_engine.html`:
  - `Scheduler` opens with its eight fields and no label, then `internal:` and its methods. No body opens with its keyword's default label.
  - The sidebar has 17 groups, each captioned with its lead type, and no caption repeats.
- In the browser, the comment toggle still hides member comments with no gaps left between members.

## Risks

- Captions get longer and will wrap in the 18rem sidebar, such as `execute::scheduler · ToolCallContinuation`.
- A visibility that has both data and members shows its label twice. That is how a C++ header reads, and the README will say so.
