*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_levels"></a>
# order_levels

```
//! Levels, reach, and seen order.
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
- [order_levels](#mod.order_levels)
  - [Run](#type.order_levels.Run)
  - [Able](#type.order_levels.Able)
  - [Report](#type.order_levels.Report)
  - [Step](#type.order_levels.Step)
  - [Leaf](#type.order_levels.Leaf)
  - [Summary](#type.order_levels.Summary)
  - [Config](#type.order_levels.Config)
  - [Trace](#type.order_levels.Trace)
  - [Log](#type.order_levels.Log)
  - [Orphan](#type.order_levels.Orphan)

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

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [Run](#type.order_levels.Run)
  - [Able](#type.order_levels.Able)
  - [Report](#type.order_levels.Report)
- Level 2
  - [Step](#type.order_levels.Step)
  - [Leaf](#type.order_levels.Leaf)
  - [Summary](#type.order_levels.Summary)
- Level 3
  - [Config](#type.order_levels.Config)
  - [Trace](#type.order_levels.Trace)
  - [Log](#type.order_levels.Log)
- Unconnected types
  - [Orphan](#type.order_levels.Orphan)

<a id="mod.order_levels"></a>
## order_levels

<a id="type.order_levels.Run"></a>
### Run

Counts: 2 fields

Used by: nothing

```
/// The main entry: it reaches the most declarations.
pub struct Run {
    pub config: Config,

    pub steps: Vec<Step>,
}
```

<a id="type.order_levels.Able"></a>
### Able

Counts: 1 field

Used by: nothing

```
/// A root that ties Report on reach and comes first by path.
pub struct Able {
    pub leaf: Leaf,
}
```

<a id="type.order_levels.Report"></a>
### Report

Counts: 1 field

Used by: nothing

```
/// A root that ties Able on reach.
pub struct Report {
    pub summary: Summary,
}
```

<a id="type.order_levels.Step"></a>
### Step

Counts: 3 fields

Used by: [Run](#type.order_levels.Run)

```
/// Named by Run; first names Trace, then Log.
pub struct Step {
    pub a_trace: Trace,

    pub b_log: Log,

    pub config: Config,
}
```

<a id="type.order_levels.Leaf"></a>
### Leaf

Counts: none

Used by: [Able](#type.order_levels.Able)

```
/// Named only by Able.
pub struct Leaf;
```

<a id="type.order_levels.Summary"></a>
### Summary

Counts: none

Used by: [Report](#type.order_levels.Report)

```
/// Named only by Report.
pub struct Summary;
```

<a id="type.order_levels.Config"></a>
### Config

Counts: 1 field

Used by: [Run](#type.order_levels.Run), [Step](#type.order_levels.Step)

```
/// Named by Run at level 1 and by Step at level 2, so it lands at level 3.
pub struct Config {
    pub limit: u32,
}
```

<a id="type.order_levels.Trace"></a>
### Trace

Counts: none

Used by: [Step](#type.order_levels.Step)

```
/// Named only by Step, before Log.
pub struct Trace;
```

<a id="type.order_levels.Log"></a>
### Log

Counts: none

Used by: [Step](#type.order_levels.Step)

```
/// Named only by Step.
pub struct Log;
```

<a id="type.order_levels.Orphan"></a>
### Orphan

Counts: none

Used by: nothing

```
/// Names nothing and nothing names it.
pub struct Orphan;
```
