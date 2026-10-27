*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_modules"></a>
# order_modules

```
//! Declarations shown one module at a time.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations by level, and Levels indexes them: level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Levels](#levels)
- [order_modules](#mod.order_modules)
  - [Orphan](#type.order_modules.Orphan)
- [engine](#mod.order_modules.engine)
  - [engine::Runner](#type.order_modules.engine.Runner)
- [plan](#mod.order_modules.plan)
  - [plan::Plan](#type.order_modules.plan.Plan)
  - [plan::Step](#type.order_modules.plan.Step)
  - [plan::FakeStep](#type.order_modules.plan.FakeStep)
- [Other impls](#impls)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [engine](#mod.order_modules.engine)
- [engine::Runner](#type.order_modules.engine.Runner)
- [engine::start()](#mod.order_modules.engine)
- [Orphan](#type.order_modules.Orphan)
- [plan](#mod.order_modules.plan)
- [plan::FakeStep](#type.order_modules.plan.FakeStep) (test support)
- [plan::Plan](#type.order_modules.plan.Plan)
- [plan::Step](#type.order_modules.plan.Step)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [engine::Runner](#type.order_modules.engine.Runner)
- Level 2
  - [plan::Plan](#type.order_modules.plan.Plan)
- Level 3
  - [plan::Step](#type.order_modules.plan.Step)
- Test support
  - [plan::FakeStep](#type.order_modules.plan.FakeStep)
- Unconnected types
  - [Orphan](#type.order_modules.Orphan)

<a id="mod.order_modules"></a>
## order_modules

<a id="type.order_modules.Orphan"></a>
### Orphan

Counts: none

Used by: nothing

```
/// A value nothing uses.
pub struct Orphan;
```

<a id="mod.order_modules.engine"></a>
## engine

```
//! Runs things.
```

```
/// Starts the engine.
pub fn start();
```

<a id="type.order_modules.engine.Runner"></a>
### engine::Runner

Counts: 1 field

Used by: nothing

```
/// A runner.
pub struct Runner {
    /// Its plan.
    pub plan: crate::plan::Plan,
}
```

<a id="mod.order_modules.plan"></a>
## plan

```
//! Plans runs.
```

<a id="type.order_modules.plan.Plan"></a>
### plan::Plan

Counts: 1 field

Used by: [engine::Runner](#type.order_modules.engine.Runner)

```
/// A plan.
pub struct Plan {
    /// Its steps.
    pub steps: Vec<Step>,
}
```

<a id="type.order_modules.plan.Step"></a>
### plan::Step

Counts: none

Used by: [plan::Plan](#type.order_modules.plan.Plan)

```
/// One step.
pub struct Step;
```

<a id="type.order_modules.plan.FakeStep"></a>
### plan::FakeStep

Counts: none

Used by: nothing

```
/// A step for tests and benches.
#[cfg(any(test, feature = "test-support"))]
pub struct FakeStep;
```

<a id="impls"></a>
## Other impls

- `impl Clone for Box<Orphan> {}`
