---
name: vibelight output improvements
overview: Numbered list of every improvement I proposed for vibelight's output, checked against what vibelight already does. Five already exist behind `--facts` and `--order modules`. Eight are new and keep the rule that moving impls between files never changes the output; field use is a `Fields:` facts list rather than a regrouping. One opt-in switch for the one-time analysis breaks the rule on purpose. The work runs in sequence because the items share golden files. This plan runs before the one-time tool plan, which is written against its final flags.
todos:
  - id: copy-plan
    content: Copy this plan to vibelight/plans/YYYY-MM-DD-output-analysis-upgrades.plan.md per vibelight AGENTS.md
    status: completed
  - id: where-wrap
    content: "Item 11: one predicate per line for a where clause past MAX_WIDTH in wrap.rs, extend the long-lines fixture"
    status: completed
  - id: fields-order
    content: "Item 6: skip sort_entries for the fields group (group 2) in layout_decl.rs body(), update struct goldens and the README sort rule"
    status: completed
  - id: reexport-spacing
    content: "Item 12: merge a visibility section's re-export entries into one entry like use groups (layout_items.rs 53-63), keeping a blank line before a commented entry; update affected goldens"
    status: completed
  - id: legend-visibility
    content: "Items 9 and 10 in one commit: the Legend line about where methods appear, the // internal line above non-pub type headers with its Legend line, one golden sweep for both"
    status: completed
  - id: unconnected-bodies
    content: "Item 8 (every mode): let function-body mentions keep a type out of Unconnected in order.rs split, keep Used by signature-only, add a fixture"
    status: completed
  - id: method-clusters
    content: "Item 7: done as a Fields: list under --facts (each field with the functions that reach it through self, hubs as a count), after a Clusters: line proved to chain Scheduler into one group"
    status: completed
  - id: members-impl
    content: "Item 15: --members name|impl switch and members config key; impl groups member functions by inherent impl block in source order under // impl in <file> lines; Legend line when on; member-order-impl fixture"
    status: completed
  - id: optional-items
    content: "Items 13-14: deferred, with revisit conditions in the Outcome section"
    status: completed
  - id: confirm-runvil
    content: Run the spelled-out vibelight command (with --follow "**" and --members impl) on runvil's engine crate into scratch and confirm items 6-12 and 15 on the real crate
    status: completed
isProject: false
---

# vibelight output improvements

Each item below says whether it already exists, where the change would land, and how confident I am in it. The item numbers stay fixed, because the one-time tool plan refers to item 15.

The rule every new item except 15 keeps: moving impls between files never changes the output. Per [vibelight/AGENTS.md](vibelight/AGENTS.md), every output change ships with its updated golden files in the same commit, and the goldens are the specification.

## Already in vibelight (just use `--profile analysis`)

The `runvil_engine.md` I reviewed was generated with the default settings: levels, no facts. Five of my suggestions are already implemented:

1. **Member counts per type.** This is the `--facts` `Counts:` line ([vibelight/src/document_facts.rs](vibelight/src/document_facts.rs)). It would have flagged `Scheduler` right away.
2. **"Used by" for types.** This is the `--facts` `Used by:` line, which lists the declarations and modules that name the type in a signature.
3. **Summaries of other crates' types.** The `--facts` External items section gives each foreign item's first doc line for crates in the same run ([vibelight/src/document_external.rs](vibelight/src/document_external.rs)).
4. **Detecting identical types.** The `--facts` Same shapes section ([vibelight/src/document_shapes.rs](vibelight/src/document_shapes.rs)) would have caught `ChatDispatch` and `ToolCallDispatch`.
5. **Grouping by module instead of level.** That's `--order modules`.

Action: the one-time vibelight tool (see the one-time tool plan) runs with all of these, plus `--md-source-links on`, into scratch. If outlines are ever committed again, they keep the default profile.

## New improvements

6. **Struct fields in declaration order.** In `body()` in [vibelight/src/layout_decl.rs](vibelight/src/layout_decl.rs), fields are group 2, and lines 203-204 sort every group with `sort_entries` ([vibelight/src/layout.rs](vibelight/src/layout.rs) line 210). Enum variants already replace group 2 in source order at lines 206-208.
   - Skip the sort for group 2 only, so fields keep declaration order inside each visibility section.
   - Fields live in one declaration, so moving impls can't reorder them. Field order also carries meaning: drop order, derived `Ord`, and serde order.
   - Amend the README rule "sorted by name inside each group".
   - Confidence: high. It doesn't break the rule and it adds meaning.

7. **A `Clusters:` facts line: methods grouped by the fields they touch.** It shows under `Counts:` and `Used by:`, only with `--facts`. Members are not regrouped.
   - The `syn` `visit` feature already sees function bodies. For each method, record the `self.<field>` names it touches and the `self.<method>()` calls it makes.
   - Leave out hub fields that more than half the methods touch, such as `chains` and `ctx` on `Scheduler`, and list them once.
   - Join methods that share a remaining field or call each other.
   - Name each cluster after its most-touched field. Sort clusters by name and members by name inside each cluster.
   - The line looks like `Clusters: tasks (admit, can_admit, ...); pending (abort_effect, issue, ...); hubs: chains, ctx`.
   - Being a line rather than a regrouping, it never conflicts with item 15. In the one-time analysis, the body grouped by file and the `Clusters:` line side by side expose misfiled methods.
   - Add a sentence to the Legend's facts line.
   - Changes go in [vibelight/src/collect_impl.rs](vibelight/src/collect_impl.rs) (collect field and call sets) and [vibelight/src/document_facts.rs](vibelight/src/document_facts.rs) (the line).
   - Confidence: medium. The goal is clear, but the hub threshold needs tuning on `Scheduler`.

8. **Unconnected types count uses inside function bodies, in every mode.** `Listed` shows up as unconnected only because the one place that names it is a function body.
   - Let a mention in a body keep a type out of Unconnected, in `split` in [vibelight/src/order.rs](vibelight/src/order.rs) (around lines 197-223). Such a type moves to Level 1, as types named by free functions already do.
   - Leave `Used by:` based on signatures only, so the lists don't change with every edit to a body.
   - This changes the default output. A type changes sections only when a body edit adds or removes its last use, which is rare.
   - Confidence: medium. It fixes a misleading section with little churn.

9. **A Legend line about where methods appear.** Something like: "methods are listed on their type; a module whose file holds only impls shows only its docs." Add it to `SECTIONS` in [vibelight/src/legend.rs](vibelight/src/legend.rs).
   - It ships in one commit with item 10, so every golden's Legend changes once.
   - Confidence: high. It's one sentence and stops the misreading I made of `admission`, `waits`, and `timer`.

10. **Show a type's own visibility on its header.** `struct` versus `class` tells you whether the members are public, but an `enum`, trait, or alias shows nothing about the type itself. I learned `Error` is `pub(crate)` only from its prose.
    - Print a `// internal` line above the header for `pub(crate)`, `pub(super)`, and `pub(in path)` types, in the style of the existing `// cfg:` line, in [vibelight/src/layout_head.rs](vibelight/src/layout_head.rs). Add a Legend line.
    - It ships in one commit with item 9.
    - Confidence: medium. The notation needs a golden to pin it.

11. **Break long `where` clauses one predicate per line.** The clause already moves to its own line ([vibelight/src/wrap.rs](vibelight/src/wrap.rs)), but `setup_section_vm` still prints a line of about 300 characters.
    - When the `where` line is wider than `MAX_WIDTH`, put each predicate on its own line.
    - Extend the `long-lines` fixture.
    - Confidence: high. It's a plain readability bug.

12. **Print re-export lines without blank lines between them.** Braced and commented `pub use` groups already merge into one entry with no blank lines ([vibelight/src/layout_items.rs](vibelight/src/layout_items.rs) lines 53-63). Separate `pub use` statements stay separate entries, which the join double-spaces. In the engine outline that's about 250 lines.
    - Merge all re-export entries of a visibility section into one entry the same way, after sorting.
    - Keep a blank line before an entry that has its own comment, so the comment doesn't read as belonging to the line above.
    - Confidence: medium. Most goldens with re-exports change.

## Optional, lower value

13. **External items shows each in-run foreign type's fields or variants on one line**, not just its first doc line. The cost is that an edit in one crate changes the other crates' analysis copies. Only with `--facts`.
    - Confidence: low to medium. Useful, but it ties crates' outlines together.

14. **"Used by" for methods and functions, written as `Type::method`.**
    - A free function called by path can be resolved. A method call like `x.timeout()` can't be resolved without type information, which `syn` doesn't provide.
    - Limit it to free functions, or defer it.
    - Confidence: low. Without types it would be guesswork.

## Opt-in, breaks the layout rule on purpose

15. **`--members name|impl` switch, for the one-time analysis only.** The default `name` keeps today's output. `impl` groups a type's member functions by inherent impl block, in source order inside each block.
    - Each group opens with a `// impl in <file>` line, printed even when source links are off. That's because the switch already opts into output that depends on file layout.
    - Fields, associated items, and special members stay where they are. Only member functions regroup. The `Fields:` list from item 7 is unaffected.
    - Don't tie it to `--profile analysis`, which turns source links off by default, or to source links being on, which would change the default HTML. It's a switch, with a matching `members` config key.
    - Add a Legend line that shows only when the switch is on. Add a `member-order-impl` fixture.
    - It touches [vibelight/src/settings.rs](vibelight/src/settings.rs) (the option), [vibelight/src/config.rs](vibelight/src/config.rs) (the key), [vibelight/src/layout_decl.rs](vibelight/src/layout_decl.rs) (grouping in place of `sort_entries` for member functions), and [vibelight/src/legend.rs](vibelight/src/legend.rs).
    - The one-time vibelight tool, written after this plan, always passes `--members impl` and requires a binary that has it.
    - Confidence: medium. What it adds over the `Source:` lines is doc comments grouped by file and source order within each block.

## Order of work

The items share golden files, so separate branches would conflict. They run one after another, one commit each, with goldens written first:

- 11 first, the cheapest
- then 6
- then 12
- then 9 and 10 together
- then 8
- then 7
- then 15
- then decide on 13 and 14

The last step runs this command from runvil's root into scratch, to see each change on a real crate. The one-time tool doesn't exist yet at this point, so the command is spelled out here:

```text
vibelight crates/runvil-internal/engine --follow "**" --profile analysis --facts --order modules --kill-list counts --md-source-links on --members impl --md-out <scratch md folder> --html-out <scratch html folder>
```

## Notes

- Items 6, 8, 9, 10, 11, and 12 change the default output. Their goldens change in the same commits. No repository commits outlines today, so nothing else needs regenerating. A repository that commits outlines later regenerates them once with the new release, as the README says.
- Items 7 and 13 show only with `--facts`.
- Item 15 is off by default. Only the one-time tool turns it on.

## Outcome

Every item ran in the order above, one commit each, with its goldens written by hand first and every other golden diff read before it was accepted. Where the work departed from the text above:

- **Item 7 became a `Fields:` list.** The `Clusters:` line, joined through shared fields, chained 46 of runvil's `Scheduler`'s 103 functions into one group. Joining through calls was left out from the start for the same reason. The list shows each field, in declared order, with the functions that reach it through `self`, and a hub as a count. On `Scheduler` it shows the effect table (`pending`, `issued`, `orphaned`, `withdrawn`, `next_effect`) and the task arena (`tasks`, `spawned`, `resuming`) directly.
- **Item 10 covers private types too.** A type gets `// internal` or `// private`, because with only `// internal` a private type would look public.
- **Item 8 widens the owner rule.** A body inside an impl does not count for any type the impl's header names, so `impl Clone for Box<Engine>` does not keep `Engine` out of Unconnected.
- **Items 13 and 14 are deferred.** 13 ties each crate's analysis copy to the others' fields; revisit it when an analysis needs foreign shapes that the context crates' own outlines do not already give. 14 cannot resolve method calls without type information; revisit it if vibelight gains a resolver that sees types.

The confirmation run on runvil's engine crate shows every change: the `where` clause of `setup_section_vm` per predicate, packed re-exports, 44 `// internal` and 16 `// private` lines, `Listed` out of Unconnected, 83 `// impl in` groups, and 5 `Fields:` lists.
