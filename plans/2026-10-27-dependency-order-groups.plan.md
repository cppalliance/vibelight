---
name: Dependency order groups
overview: Replace the level order with a dependency order in which a type follows its only user directly and a shared type follows its last user, and group the HTML into module-based sections, each opened by a horizontal rule and a caption heading. The markdown keeps its module sections and drops the Levels index.
todos:
  - id: copy-plan
    content: Copy this plan to vibelight/plans/2026-10-27-dependency-order-groups.plan.md
    status: completed
  - id: goldens-first
    content: "Hand-write the order goldens first: rename order-levels to order-nesting, update order-cycles, add the order-groups fixture with md and html goldens"
    status: completed
  - id: order-dfs
    content: "order.rs: replace levels with Order.sequence using the owner-then-parts rule, with unit tests; split to order_graph.rs if over 500 lines"
    status: completed
  - id: groups
    content: "New groups.rs: module key, runs, lone-type folding, captions, with unit tests"
    status: completed
  - id: document
    content: settings.rs OrderBy::Groups, document_sections.rs grouped, document.rs Node::Rule, document_modules.rs drop levels_index
    status: completed
  - id: render
    content: render_html.rs and render_style.rs print the hr; render_md.rs handles Node::Rule
    status: completed
  - id: docs
    content: legend.rs line, README two bullets, ROOTED golden in workspaces.rs
    status: completed
  - id: regen-check
    content: Regenerate remaining goldens, check each keeps the same set of type sections, read every diff
    status: completed
  - id: verify
    content: fmt, clippy, test, 1.85 check, package, release build, runvil engine spot check
    status: completed
isProject: false
---

# Dependency order with module groups

All changes are in [vibelight](vibelight). Per [vibelight/AGENTS.md](vibelight/AGENTS.md), the first step copies this plan to `plans/2026-10-27-dependency-order-groups.plan.md`, and the order goldens are written by hand before the code, since goldens are the spec.

## The order (order.rs)

Today [vibelight/src/order.rs](vibelight/src/order.rs) assigns each group of types a level, one more than its deepest user, and prints level by level. That strands parts far from their owner: on the engine, `Scheduler` is at line 721 and its parts `Admission`, `EffectTable`, and `ChainArena` follow at lines 1840, 1979, and 2206.

The new rule replaces `levels` and `Order.levels` with one `Order.sequence: Vec<DeclId>`:

1. Cycles collapse into groups, as now. Unconnected and test-support types leave the sequence, as now.
2. Roots are the groups no other group names, ordered by reach, then path, as now.
3. Each group counts the distinct other groups that name it. Free functions, constants, and re-exports do not count here. They only keep a type from being unconnected, as today.
4. Emit a group's members using the existing seen-order loop. Then take the groups it names, in the order its blocks mention them and each only once, and decrement each one's count. A group whose count reaches zero is emitted immediately, with the same rule applied to its parts.
5. A type with one user therefore follows that user directly. A shared type follows its last user, so every user is still above it. Any group left unplaced (defensive, it cannot happen in a DAG) is appended in path order.

The block mention order is the printed order, so sibling order follows each format's own blocks. The two formats may order siblings differently, because the markdown groups functions by impl.

## The groups (new groups.rs)

A pure function in a new `src/groups.rs` splits the sequence into groups for the HTML:

- A decl's key is the first two segments of its module path. A decl in the root module has the empty key.
- A run is a maximal stretch of consecutive decls with equal keys.
- A run of one type joins the group before it. A first run of one type joins the group after it.
- A group's caption is its key joined with `::`, or the crate title for the empty key.

Dry run on the engine: 84 placed types form 25 runs, 11 of them single types, which folds to about 12 groups.

## Document and renderers

- [vibelight/src/settings.rs](vibelight/src/settings.rs): `OrderBy::Levels` becomes `Groups`, which the HTML uses. `Modules`, which the markdown uses, no longer has a Levels index.
- [vibelight/src/document_sections.rs](vibelight/src/document_sections.rs): `ordered` becomes `grouped`. Each group gets a `Node::Rule`, an H2 with id `group.N` and its caption, and an H3 per type. Test support and Unconnected types keep their H2s and also get a rule. The contents entry per group lists its types, which gives the sidebar the same grouping.
- [vibelight/src/document.rs](vibelight/src/document.rs): add `Node::Rule`. The `Groups` branch calls `grouped`, and the `Modules` branch drops `levels_index`.
- [vibelight/src/document_modules.rs](vibelight/src/document_modules.rs): delete `levels_index`. `module_sections` reads `order.sequence`, then test support, then unconnected.
- [vibelight/src/render_html.rs](vibelight/src/render_html.rs) and [vibelight/src/render_style.rs](vibelight/src/render_style.rs): `Node::Rule` prints `<hr>`, styled with the existing `--rule` color. [vibelight/src/render_md.rs](vibelight/src/render_md.rs) prints `---` for completeness, but never receives one.
- Keep every file under the 500 line rule that `build.rs` enforces. `order.rs` is at 479 lines, so if the rewrite passes 500, move `strongly_connected` and its tests to a new `src/order_graph.rs`.

## Docs

- [vibelight/src/legend.rs](vibelight/src/legend.rs): replace the Levels line with: each module section lists its types in dependency order, a type no other type names comes first, a type only one other type names follows it, and a type several name follows the last of them.
- [vibelight/README.md](vibelight/README.md): rewrite the "Top-down order" bullet (line 15) and the "types, by level" bullet (line 143), and add the group captions and rules to the HTML description.
- Inline `ROOTED` golden in [vibelight/tests/it/workspaces.rs](vibelight/tests/it/workspaces.rs): drop its Levels list and update its Legend line.

## Tests and goldens

Write by hand first, from the rules above:
- Rename the fixture `order-levels` to `order-nesting`. It holds a chain, a diamond, a type shared by a parent and its child, and two roots of different reach.
- `order-cycles`: update its golden for the new sequence.
- New fixture `order-groups`, with a lone first run, a three-segment module path, a run of several types, a lone run mid-sequence, and root-module types. Its HTML golden pins the `<hr>`, captions, `group.N` ids, and sidebar grouping.

Unit tests in `order.rs` cover:
- chain: A, B, C
- shared by a parent and its child: A, B, C
- diamond: A, B, C, D
- two roots by reach
- a cycle group
- sibling order from mentions

Unit tests in `groups.rs` cover the run, fold, and caption rules.

Then regenerate every other golden. All markdown goldens lose the Levels section and its contents entry, and all HTML goldens swap level headings for group headings. Check that nothing else changed:
- For each golden, compare the set of type sections (anchor, heading, and body) before and after. They must be identical, with only their order and the group headings moving.
- Read every diff by hand, per AGENTS.md.
- The arrangement tests must still pass.

## Verification

- In [vibelight](vibelight): `cargo fmt --check`, `cargo clippy --all-targets --locked`, `cargo test --locked`, the 1.85 floor check, `cargo package --locked --allow-dirty`, and `cargo build --release`.
- Plain `vibelight` at the runvil root. In the engine HTML, `Scheduler` must be followed directly by its parts. There should be about 12 groups, each opened by a rule and caption.
- Leave everything uncommitted for review.

Confidence is high that the order reads better. It is medium on the group boundary, since the module depth and the folding of single types are judgment calls that the engine run will test.