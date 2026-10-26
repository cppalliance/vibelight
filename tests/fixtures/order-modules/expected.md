*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_modules"></a>
# order_modules

```
//! Declarations shown one module at a time.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- A type's block holds the members of all its inherent impls, wherever they are written, so a module that holds only impls shows nothing but its docs.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A `// internal` or `// private` line above a type's header gives the type's own visibility; a public type has none.
- A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.
- References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.
- `constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.
- `Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.
- A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top gathers them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations by level, and Levels indexes them: level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
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

```
/// A value nothing uses.
struct Orphan;
```

<a id="mod.order_modules.engine"></a>
## engine

```
//! Runs things.
```

```
/// Starts the engine.
start();
```

<a id="type.order_modules.engine.Runner"></a>
### engine::Runner

```
/// A runner.
struct Runner
{
    /// Its plan.
    plan: crate::plan::Plan,
}
```

<a id="mod.order_modules.plan"></a>
## plan

```
//! Plans runs.
```

<a id="type.order_modules.plan.Plan"></a>
### plan::Plan

```
/// A plan.
struct Plan
{
    /// Its steps.
    steps: Vec<Step>,
}
```

<a id="type.order_modules.plan.Step"></a>
### plan::Step

```
/// One step.
struct Step;
```

<a id="type.order_modules.plan.FakeStep"></a>
### plan::FakeStep

```
/// A step for tests and benches.
// cfg: any(test, feature = "test-support")
struct FakeStep;
```

<a id="impls"></a>
## Other impls

- `implements: Clone for Box<Orphan>;`
