*Updated 2026-01-01T00:00:00Z*

<a id="crate.field_index"></a>
# field_index

```
//! Which functions reach each field.
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
- [field_index](#mod.field_index)
  - [Scheduler](#type.field_index.Scheduler)
  - [Small](#type.field_index.Small)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Scheduler](#type.field_index.Scheduler)
- [Small](#type.field_index.Small)

<a id="mod.field_index"></a>
## field_index

<a id="type.field_index.Scheduler"></a>
### Scheduler

Counts: 6 fields, 8 functions

Used by: nothing

Fields:

- `chains`: a hub, reached by 5 of 8 functions
- `tasks`: count_tasks, spawn
- `queue`: admit, spawn
- `pending`: apply, issue
- `issued`: issue, withdraw
- `spare`: none

```
/// A scheduler whose fields its functions reach in groups.
pub struct Scheduler {
    chains: Vec<u32>,

    tasks: Vec<u32>,

    queue: Vec<u32>,

    pending: Vec<u32>,

    issued: Vec<u32>,

    spare: u8,
}

// impl in src/lib.rs
impl Scheduler {
    /// Makes one.
    pub fn new() -> Scheduler;

    /// Starts a task.
    pub fn spawn(&mut self);

    /// Admits queued tasks.
    pub fn admit(&mut self);

    /// Counts the tasks.
    pub fn count_tasks(&self) -> usize;

    /// Issues an effect.
    pub fn issue(&mut self);

    /// Applies an answer.
    pub fn apply(&mut self);

    /// Forgets the issued effects.
    pub fn withdraw(&mut self);

    /// Counts the chains.
    pub fn len(&self) -> usize;
}
```

<a id="type.field_index.Small"></a>
### Small

Counts: 2 fields, 2 functions

Used by: nothing

```
/// A type with too few functions for a fields list.
pub struct Small {
    a: u8,

    b: u8,
}

// impl in src/lib.rs
impl Small {
    /// Returns the first.
    pub fn first(&self) -> u8;

    /// Returns both.
    pub fn both(&self) -> u8;
}
```
