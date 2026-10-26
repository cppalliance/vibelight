*Updated 2026-01-01T00:00:00Z*

<a id="crate.clusters"></a>
# clusters

```
//! Functions grouped by the fields they reach.
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
- Level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, special members, and functions; a `Used by:` line links what names it; a `Clusters:` line groups a type's functions by the fields they reach through `self` when they form two or more groups, names each group after its most-reached field, and lists as hubs the fields more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Scheduler](#type.clusters.Scheduler)
  - [Small](#type.clusters.Small)
- [Modules](#modules)
  - [clusters](#mod.clusters)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Scheduler](#type.clusters.Scheduler)
- [Small](#type.clusters.Small)

<a id="unconnected"></a>
## Unconnected types

<a id="type.clusters.Scheduler"></a>
### Scheduler

Counts: 5 fields, 7 functions

Used by: nothing

Clusters: pending (apply, issue); queue (admit, count_tasks, spawn); hubs: chains

```
/// A scheduler with two groups of state.
struct Scheduler
{
    /// Makes one.
    static new() -> Scheduler;

    /// Admits queued tasks.
    admit() mut;

    /// Applies an answer.
    apply() mut;

    /// Counts the tasks.
    count_tasks() -> usize;

    /// Issues an effect.
    issue() mut;

    /// Counts the chains.
    len() -> usize;

    /// Starts a task.
    spawn() mut;

private:
    chains: Vec<u32>,

    tasks: Vec<u32>,

    queue: Vec<u32>,

    pending: Vec<u32>,

    issued: Vec<u32>,
}
```

<a id="type.clusters.Small"></a>
### Small

Counts: 2 fields, 2 functions

Used by: nothing

```
/// A type whose functions form one group, so it has no clusters line.
struct Small
{
    /// Returns both.
    both() -> u8;

    /// Returns the first.
    first() -> u8;

private:
    a: u8,

    b: u8,
}
```

<a id="modules"></a>
## Modules

<a id="mod.clusters"></a>
### clusters

- [Scheduler](#type.clusters.Scheduler)
- [Small](#type.clusters.Small)
