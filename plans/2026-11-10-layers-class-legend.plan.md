---
name: Class keyword legend demo
overview: State vibelight's goal in its AGENTS.md, show the uses between crates on the home page as a table of layers, say on a facade's page how many of its users leave each API item unused, make struct blocks read like C++ on every page (`class` or `struct` by the first member shown, labels only where visibility changes), show private data dim on facade pages, and put a live syntax demo at the top of the home page legend.
todos:
  - id: agents-goal
    content: Replace the second bullet of vibelight/AGENTS.md with the three goal bullets of step 1, commit
    status: in_progress
  - id: sample-page
    content: Render the sample page in scratch (layers table, legend with demo, keyword cases, Prompt both ways), show a screenshot, wait for approval
    status: pending
  - id: uses-table
    content: "Home page: Uses between outlines becomes a table of layers (layer, crate, uses, used by, instability), highest layer first, with chips colored by role; a hint that explains layers and instability, with a Wikipedia link; the counting rule moves to the legend; index.md stays; tests, README, commit"
    status: pending
  - id: api-users
    content: "Facade pages: each item's Defined in line ends with 'not used by N of M' when N of the facade's M users never use it; no list of crates and no new line; a dim sidebar name when none uses it; two numbers in the opening sentence and one on the home page; a free function beside module_graph::crates; tests, values golden, legend, README, commit"
    status: pending
  - id: keyword-labels
    content: "Keyword and label rule on every page: decl.rs, groups.rs, render_html.rs, goldens with diff classifier, legend/README/AGENTS text, tests, commit"
    status: pending
  - id: facade-private-data
    content: Facade pages show private fields dim, no comments, packed; dim marker spans; names() and links skip them; replace 47b151a tuple code; tests, goldens, docs, commit
    status: pending
  - id: legend-demo
    content: Legend demo rendered at run time from an embedded Rust source; modtree entry from text; style and browse test; unit test pins the demo; commit
    status: pending
isProject: false
---

# Goal in AGENTS.md, crate layers, unused counts on facade pages, class keyword, private data on facade pages, legend demo

## What changes for the reader

**Layers, on the home page (today's "Uses between outlines")**

- The section answers one review question: is each crate in the right place, and do the uses go one way? So it becomes a table of layers, in the style of the tables above it: Layer, Crate, Uses, Used by, Instability.
- A crate's layer is 0 when it uses no other crate of the run, else one more than the highest layer among the crates it uses. Rows go from the highest layer down, so every use points down the table and the stack of the run reads top to bottom. For runvil that is 30 crates in 12 layers.
- Each crate in `Uses` and `Used by` is a chip with its item count, the largest count first, as in the `Re-exports from` column of the facades table. Every name takes the color of its role, as in the tables above.
- The heading reads `Layers` and gets the number of layers as its count. Under it, a hint says what a layer is and what instability measures, in the table's own words, and ends with a link to Wikipedia's page on software package metrics. That is the only link of the HTML that leaves the file.
- The rule for which crate a use counts for moves to the legend as one line.
- A crate that uses none and that none use stays out, as today. `index.md` keeps its section word for word, since it is written for agents.

**How many of a facade's users leave an item unused, on the facade's page**

- This answers "is every name needed?" with one number: no list of crates, and no new line. The crates that use a facade are the ones in its `Used by` cell of the layers table; runvil has 4. When some of them never use an item, the item's `Defined in` line ends with the count, such as `not used by 3 of 4`. An item that all of them use shows nothing.
- A crate uses an item when its production code writes the item's name, or the name of one of its members: in a signature, a body, an impl header, or a re-export. Doc links do not count.
- In both sidebar lists, the name of an item that none of them uses prints dim. The page's opening sentence says how many crates use the facade and how many items none of them uses, and the facade's row on the home page gives that second number.
- The limit, which the legend states: a value can be used without its type's name being written, as a field of an event is, so a count is a place to look and not proof. Members get no count, because a method call with a dot is not a path.
- A facade that no crate of the run uses shows none of this. A macro, or an item of a crate outside the run, has no count.
- As a check of the idea, today's markdown records of runvil suggest that `runvil::ids::Provenance` would read `not used by 4 of 4`, and that `runvil::event::ReplyOrigin`, which only `workshop_server` names, would read `not used by 3 of 4`.

**Keyword and labels, on every page**

- A struct prints `class` when the first member the page shows is not `pub`, and `struct` when it is. "First shown" follows print order (data first, widest visibility first), not source order.
- Inside a type's body a label prints only where the visibility differs from the section before it. `struct` starts at `pub` and `class` at `pub(self)`, so neither prints that label first.
- First member shown is `pub(crate)`, `pub(super)` or `pub(in path)`: no C++ keyword starts there, so it reads `class` and the label prints.
- Tuple struct: the first field in the header decides. No members at all: `struct`.
- `enum`, `union` and `trait` keep their keywords and start with no visibility in force, so their first label still prints. Module blocks keep every label, as today.

**Facade pages**

- A type shows all its fields. A field that is not `pub`, or is `#[doc(hidden)]`, prints at half strength (the existing `dim` class), with no comment and no blank lines between fields. Private methods stay off.
- Those fields are "private" there whatever their Rust spelling, so the keyword rule is always a clean fit: no `pub` field first means `class` and no label. One `private:` label prints only when `pub` fields come before them.
- A private tuple field prints dim in the header. This replaces the `_` and the dropped fields of commit `47b151a`.
- Inside dim text a type links only when the facade's API holds it; any other type is plain text. Dim fields count for neither the bold rule in the sidebar nor `Named, not re-exported`.

**Legend**

- At the top of the legend box, before the text: one block in crate-page form, one type with every access specifier and every kind of member, no comments.
- The block is rendered at run time from a Rust source embedded in the tool, by the same layout and render code as every page, so it cannot drift from what the pages print.
- Three legend lines are reworded: the facade page, the keyword, the labels.

## Steps

### 1. The goal in AGENTS.md (one commit)

The second bullet of [vibelight/AGENTS.md](vibelight/AGENTS.md), which starts "The HTML exists to make the structure", becomes these three, word for word:

- vibelight exists so that a person can understand the architecture of vibe-coded code without reading its bodies. A page answers structural questions: what each crate is for, which crates are facades and which implement them, what a user of each facade can name, how the crates depend on each other, and where each piece is written. It prints what the source says, the same on every run, and never a generated explanation, so a reader can take a page as fact.
- The reader of the HTML is reviewing a design and has minutes for a page. The review asks three things. Can the API be simpler: is every name needed, is there one way to do each thing, do two types have the same shape? Is each thing in the right place: the right crate and module, no more visible than its uses need, with dependencies that go one way? Are the comments enough to use an item without opening its body?
- Every rendering choice serves that review: show declarations and their comments, never bodies; put the public API first; say what a number or a mark means where it appears; make every item traceable to its file and line; and leave out anything that makes a page longer without making its structure clearer. The HTML is for a person and the markdown is for a model, and neither is shaped for the other's reader.

### 2. Sample page, then stop for your OK

- Write one static HTML page into the scratch folder with the real style sheet: the layers table with runvil's numbers, a facade sidebar and three items with and without a `not used by` count, the legend box with the demo, the four keyword cases, and `Prompt` in crate-page and facade-page form.
- Serve it on localhost, screenshot it, show it, stop the server. No code changes until you approve.

### 3. Layers table on the home page (one commit)

- [vibelight/src/html/root_page.rs](vibelight/src/html/root_page.rs): `Overview` takes the run's `index::Entry` list in place of the ready-made `uses` text. A new function works out each crate's layer from the entries, with a guard so that a cycle, which production uses should never form, cannot loop. It writes the section with `heading` and a `table.crates`, rows by layer from the highest down and by name inside a layer: the crate cell through `link` with its role's class, and chips as `facades` writes them. Entries and rows are pushed in the same loop of `pipeline.rs`, so they share an index; a chip finds its row by stem.
- The hint under the heading, word for word, in two paragraphs: "How the crates stack. Each crate sits above every crate it uses, so every use points down, and layer 0 uses no other crate here. A chip is a crate with the number of its distinct items in use." and "Instability is the number of crates a crate uses, divided by that plus the number of crates that use it. Near 0.00 a crate is stable: others lean on it, so a change to it is costly. Near 1.00 it is unstable: it leans on others, so it is cheap to change and their changes reach it. See Software package metrics on Wikipedia."
- The link goes to `https://en.wikipedia.org/wiki/Software_package_metrics` (checked: the page exists and defines Instability as `I = Ce / (Ce + Ca)`). It has `target="_blank"` and `rel="noopener"`; the app in `browse.js` already leaves a link with a target alone.
- [vibelight/src/index.rs](vibelight/src/index.rs): `uses_html` and its half of the unit test go. `render` and `uses_line` stay for `index.md`.
- [vibelight/src/pipeline.rs](vibelight/src/pipeline.rs): `push_folder` hands `folder.entries` to the overview.
- [vibelight/src/html/render_style.rs](vibelight/src/html/render_style.rs): one rule so the chip of a crate that is neither a facade nor an implementation crate is not gold.
- [vibelight/src/html/legend.rs](vibelight/src/html/legend.rs): one new line with the rule for which crate a use counts for. The line on a module's `Instability` is reworded to match the hint, counting modules.
- Tests: a unit test in `root_page.rs` pins the layers, the row order, the hint and the link. The link count at line 304 of [vibelight/tests/it/workspaces.rs](vibelight/tests/it/workspaces.rs) stays 21 if every name is still one link; its message is reworded. `every_link_lands` in that file takes any href that is no page for a file beside the output, so it learns to tell an `https:` link apart, and each run pins how many it has. The README's sentence on the overview's section follows. The home page has no golden file.
- Visual check: click the link in Cursor's browser. If that browser drops a link with a target, the target goes and the click leaves the outline, where Back returns.
- [vibelight/README.md](vibelight/README.md) says the app never leaves its file and makes no network requests. Say there that the one outside link is followed only on a click.

### 4. Unused counts on a facade's page (one commit)

- [vibelight/src/module_graph.rs](vibelight/src/module_graph.rs): a free function beside `crates`, free because `AnalysisRun` may not grow. For each declaration and module value of the run, it returns the other targets whose production code names it or one of its members, doc links left out. It reads `run.uses`, as `crates` does, and rolls a member up to its declaration with `owner`, which is there already. Its keys are a target and a `Reach`, so the HTML code names no analysis type.
- [vibelight/src/pipeline.rs](vibelight/src/pipeline.rs): `Analyzed` keeps that map. `facades` gives each `Facade` the map and the facade's users: the targets whose entry in `crate_graph` holds the facade, which are the ones the layers table shows.
- [vibelight/src/html/facade_page.rs](vibelight/src/html/facade_page.rs): `Named` gets the number of the facade's users that do not use it. `item_section` ends the `Defined in` line with `not used by N of M`, after a middle dot, when N is above 0, `symbol_link` adds the dim class when N equals M, and the opening sentence gets its two numbers. `fragment` returns, with its headings, how many items no user uses, so `pipeline.rs` hands that number to the overview without a second walk of the API.
- [vibelight/src/html/root_page.rs](vibelight/src/html/root_page.rs): `Overview` takes those numbers by target, and the facade's `Public API` cell gets one more line with its number.
- [vibelight/src/html/legend.rs](vibelight/src/html/legend.rs): one legend line for the count and the dim name, with the limit stated. No new style: the count sits in the `Defined in` line, and the `dim` class exists.
- Tests: unit tests in [vibelight/src/html/facade_page-tests.rs](vibelight/src/html/facade_page-tests.rs) for an item that every user uses, one that some do not, one that none does, and a facade with no user; a unit test for the new function in `module_graph`. Goldens: in the `values` workspace `front` is `relay`'s one user. It names three of `relay`'s four items, so those three do not change, `read` gains `not used by 1 of 1` and a dim name in the sidebar, and the opening sentence gains its numbers; read that diff. No crate of the run uses `garden`'s facade or `front`, so those pages must not change. README follows.
- Check before coding: the map's ids come from the analysis, before attach. The surfaces and the access stamps already cross from there to the HTML by declaration and value id; confirm this map can do the same.

### 5. Keyword and labels (one commit)

- [vibelight/src/html/layout/decl.rs](vibelight/src/html/layout/decl.rs): `head_spans` picks `class` or `struct` from the first member shown (first tuple field, else the first data section, else the first member section); `block_of` tells `body` which visibility is in force at the opening brace. The body's sections are sorted before the header is laid out, because the keyword comes from them.
- [vibelight/src/html/layout/groups.rs](vibelight/src/html/layout/groups.rs): `join_sections` takes the visibility in force and skips a label equal to the one before it. Module blocks pass none, so they print every label.
- [vibelight/src/html/render_html.rs](vibelight/src/html/render_html.rs): a comment box right under `{` gets the `first` class, as one right under a label does today.
- Goldens: about 54 `expected.html` files change, and their text only in two ways: a label goes, or `struct` becomes `class`. The spacing marks around a label that goes (`gap`, `mc first`) move with it. No `expected.md` changes.
- Docs: three lines in [vibelight/src/html/legend.rs](vibelight/src/html/legend.rs), the layout text in [vibelight/README.md](vibelight/README.md), and the bullet of [vibelight/AGENTS.md](vibelight/AGENTS.md) that says the HTML uses "Rust's item keywords".
- Tests: unit tests for the four keyword cases and the label rule.

### 6. Private data on facade pages (one commit)

- [vibelight/src/html/layout/decl.rs](vibelight/src/html/layout/decl.rs): `public_decl` keeps every field and marks the not-`pub` ones private; a private field prints as its name and type alone, with no comment, `cfg` line, or attribute. The `Shape::Unit` and `_` code of `47b151a` goes.
- [vibelight/src/html/layout.rs](vibelight/src/html/layout.rs) and [vibelight/src/html/render_spans.rs](vibelight/src/html/render_spans.rs): a pair of marker spans that open and close a dim run, written as `<span class="dim">`; inside one, `reference` links only through `api_link`.
- [vibelight/src/html/facade_page.rs](vibelight/src/html/facade_page.rs): `names` skips type names inside a dim run; the page's sentence becomes "what a user can name, and its data".
- Tests in [vibelight/src/html/facade_page-tests.rs](vibelight/src/html/facade_page-tests.rs) replace the two tuple tests of `47b151a`; facade goldens change where a type has private fields; README and legend follow, and so does the sentence of [vibelight/AGENTS.md](vibelight/AGENTS.md) that says a facade's page shows only what its users can name.

### 7. Legend demo (one commit)

- [vibelight/src/html/legend.rs](vibelight/src/html/legend.rs): the demo's Rust source as a constant; `html()` writes one box with the rendered demo, then the list.
- [vibelight/src/modtree.rs](vibelight/src/modtree.rs): an entry that walks a root from its text, with no file read, so the demo goes through collect, layout and render like any crate.
- [vibelight/src/html/render_style.rs](vibelight/src/html/render_style.rs): the box rules move from `ul.legend` to the wrapper.
- Tests: a unit test pins the demo's printed lines; the assertion at line 97 of [vibelight/tests/it/browse.rs](vibelight/tests/it/browse.rs) follows the new markup.

## Checks, every commit

- `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`.
- Goldens are the specification: update them with a scratch script, then run a second script that compares the text of every block before and after, tags left out, and stops on any change other than the ones the step names. Read what it flags before committing.
- Visual check on a scratch outline of runvil (never the runvil checkout): the home page legend, one crate page, the facade page.
- Commit message from the staged diff, checked with `check-message.ps1`. No push, no tag.

## Order and risks

- Step 1 is text only and goes first, because its three bullets are the test for every choice after it.
- The layers table (step 3) stands alone and is the first code change, as the smallest. The unused counts (step 4) come next, because they serve the first review question and need nothing from the later steps.
- Steps 6 and 7 both need the keyword rule of step 5, and every code step touches `legend.rs` or `render_style.rs`, so nothing runs in parallel.
- Steps 4 and 6 both change what a facade's item shows; step 6 must leave the count on the `Defined in` line as step 4 put it.
- `block_of` and `body` are near the 100-line function limit; expect to split one.
- If building a page document for the demo at run time proves heavy, fall back to a stored block that a test renders and compares, which still catches drift.
