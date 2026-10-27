*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_nesting"></a>
# order_nesting

```
//! Dependency order: each part follows its owner.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.
- Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [order_nesting](#mod.order_nesting)
  - [Run](#type.order_nesting.Run)
  - [Step](#type.order_nesting.Step)
  - [Trace](#type.order_nesting.Trace)
  - [Log](#type.order_nesting.Log)
  - [Config](#type.order_nesting.Config)
  - [Fork](#type.order_nesting.Fork)
  - [Left](#type.order_nesting.Left)
  - [Right](#type.order_nesting.Right)
  - [Base](#type.order_nesting.Base)
  - [Host](#type.order_nesting.Host)
  - [Store](#type.order_nesting.Store)
  - [Job](#type.order_nesting.Job)
  - [Able](#type.order_nesting.Able)
  - [Leaf](#type.order_nesting.Leaf)
  - [Report](#type.order_nesting.Report)
  - [Summary](#type.order_nesting.Summary)
  - [Orphan](#type.order_nesting.Orphan)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Able](#type.order_nesting.Able)
- [Base](#type.order_nesting.Base)
- [Config](#type.order_nesting.Config)
- [Fork](#type.order_nesting.Fork)
- [Host](#type.order_nesting.Host)
- [Job](#type.order_nesting.Job)
- [Leaf](#type.order_nesting.Leaf)
- [Left](#type.order_nesting.Left)
- [Log](#type.order_nesting.Log)
- [Orphan](#type.order_nesting.Orphan)
- [Report](#type.order_nesting.Report)
- [Right](#type.order_nesting.Right)
- [Run](#type.order_nesting.Run)
- [Step](#type.order_nesting.Step)
- [Store](#type.order_nesting.Store)
- [Summary](#type.order_nesting.Summary)
- [Trace](#type.order_nesting.Trace)

<a id="mod.order_nesting"></a>
## order_nesting

<a id="type.order_nesting.Run"></a>
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

<a id="type.order_nesting.Step"></a>
### Step

Counts: 3 fields

Used by: [Run](#type.order_nesting.Run)

```
/// Named by Run; first names Trace, then Log, then Config.
pub struct Step {
    pub a_trace: Trace,

    pub b_log: Log,

    pub config: Config,
}
```

<a id="type.order_nesting.Trace"></a>
### Trace

Counts: none

Used by: [Step](#type.order_nesting.Step)

```
/// Named only by Step, before Log.
pub struct Trace;
```

<a id="type.order_nesting.Log"></a>
### Log

Counts: none

Used by: [Step](#type.order_nesting.Step)

```
/// Named only by Step.
pub struct Log;
```

<a id="type.order_nesting.Config"></a>
### Config

Counts: 1 field

Used by: [Run](#type.order_nesting.Run), [Step](#type.order_nesting.Step)

```
/// Named by Run and by Step, so it follows Step's last part, not Run.
pub struct Config {
    pub limit: u32,
}
```

<a id="type.order_nesting.Fork"></a>
### Fork

Counts: 2 fields

Used by: nothing

```
/// A diamond root: it reaches three declarations through two branches.
pub struct Fork {
    pub a_left: Left,

    pub b_right: Right,
}
```

<a id="type.order_nesting.Left"></a>
### Left

Counts: 1 field

Used by: [Fork](#type.order_nesting.Fork)

```
/// Named only by Fork.
pub struct Left {
    pub base: Base,
}
```

<a id="type.order_nesting.Right"></a>
### Right

Counts: 1 field

Used by: [Fork](#type.order_nesting.Fork)

```
/// Named only by Fork, after Left.
pub struct Right {
    pub base: Base,
}
```

<a id="type.order_nesting.Base"></a>
### Base

Counts: none

Used by: [Left](#type.order_nesting.Left), [Right](#type.order_nesting.Right)

```
/// Named by Left and by Right, so it follows Right, the last of them.
pub struct Base;
```

<a id="type.order_nesting.Host"></a>
### Host

Counts: 1 field, 1 function

Used by: nothing

```
/// A root whose public method prints before its private field, so its
/// block names Job first, yet its field type, Store, still comes first.
pub struct Host {
    store: Store,
}

// impl in src/lib.rs
impl Host {
    /// Runs a job.
    pub fn run(&self, job: Job);
}
```

<a id="type.order_nesting.Store"></a>
### Store

Counts: none

Used by: [Host](#type.order_nesting.Host)

```
/// Named only by Host's field.
pub struct Store;
```

<a id="type.order_nesting.Job"></a>
### Job

Counts: none

Used by: [Host](#type.order_nesting.Host)

```
/// Named only by Host's method.
pub struct Job;
```

<a id="type.order_nesting.Able"></a>
### Able

Counts: 1 field

Used by: nothing

```
/// A root that ties Report on reach and comes first by path.
pub struct Able {
    pub leaf: Leaf,
}
```

<a id="type.order_nesting.Leaf"></a>
### Leaf

Counts: none

Used by: [Able](#type.order_nesting.Able)

```
/// Named only by Able.
pub struct Leaf;
```

<a id="type.order_nesting.Report"></a>
### Report

Counts: 1 field

Used by: nothing

```
/// A root that ties Able on reach.
pub struct Report {
    pub summary: Summary,
}
```

<a id="type.order_nesting.Summary"></a>
### Summary

Counts: none

Used by: [Report](#type.order_nesting.Report)

```
/// Named only by Report.
pub struct Summary;
```

<a id="type.order_nesting.Orphan"></a>
### Orphan

Counts: none

Used by: nothing

```
/// Names nothing and nothing names it.
pub struct Orphan;
```
