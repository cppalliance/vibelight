*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_levels"></a>
# order_levels

```
//! Levels, reach, and seen order.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.
- References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.
- `constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.
- `Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.
- A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top gathers them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [Run](#type.order_levels.Run)
  - [Able](#type.order_levels.Able)
  - [Report](#type.order_levels.Report)
- [Level 2](#level.2)
  - [Step](#type.order_levels.Step)
  - [Leaf](#type.order_levels.Leaf)
  - [Summary](#type.order_levels.Summary)
- [Level 3](#level.3)
  - [Config](#type.order_levels.Config)
  - [Trace](#type.order_levels.Trace)
  - [Log](#type.order_levels.Log)
- [Unconnected types](#unconnected)
  - [Orphan](#type.order_levels.Orphan)
- [Modules](#modules)
  - [order_levels](#mod.order_levels)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Able](#type.order_levels.Able)
- [Config](#type.order_levels.Config)
- [Leaf](#type.order_levels.Leaf)
- [Log](#type.order_levels.Log)
- [Orphan](#type.order_levels.Orphan)
- [Report](#type.order_levels.Report)
- [Run](#type.order_levels.Run)
- [Step](#type.order_levels.Step)
- [Summary](#type.order_levels.Summary)
- [Trace](#type.order_levels.Trace)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.order_levels.Run"></a>
### Run

```
/// The main entry: it reaches the most declarations.
struct Run
{
    config: Config,

    steps: Vec<Step>,
}
```

<a id="type.order_levels.Able"></a>
### Able

```
/// A root that ties Report on reach and comes first by path.
struct Able
{
    leaf: Leaf,
}
```

<a id="type.order_levels.Report"></a>
### Report

```
/// A root that ties Able on reach.
struct Report
{
    summary: Summary,
}
```

<a id="level.2"></a>
## Level 2

<a id="type.order_levels.Step"></a>
### Step

```
/// Named by Run; first names Trace, then Log.
struct Step
{
    a_trace: Trace,

    b_log: Log,

    config: Config,
}
```

<a id="type.order_levels.Leaf"></a>
### Leaf

```
/// Named only by Able.
struct Leaf;
```

<a id="type.order_levels.Summary"></a>
### Summary

```
/// Named only by Report.
struct Summary;
```

<a id="level.3"></a>
## Level 3

<a id="type.order_levels.Config"></a>
### Config

```
/// Named by Run at level 1 and by Step at level 2, so it lands at level 3.
struct Config
{
    limit: u32,
}
```

<a id="type.order_levels.Trace"></a>
### Trace

```
/// Named only by Step, before Log.
struct Trace;
```

<a id="type.order_levels.Log"></a>
### Log

```
/// Named only by Step.
struct Log;
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.order_levels.Orphan"></a>
### Orphan

```
/// Names nothing and nothing names it.
struct Orphan;
```

<a id="modules"></a>
## Modules

<a id="mod.order_levels"></a>
### order_levels

- [Able](#type.order_levels.Able)
- [Config](#type.order_levels.Config)
- [Leaf](#type.order_levels.Leaf)
- [Log](#type.order_levels.Log)
- [Orphan](#type.order_levels.Orphan)
- [Report](#type.order_levels.Report)
- [Run](#type.order_levels.Run)
- [Step](#type.order_levels.Step)
- [Summary](#type.order_levels.Summary)
- [Trace](#type.order_levels.Trace)
