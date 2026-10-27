*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_groups"></a>
# order_groups

```
//! Groups of types by module.
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
- [order_groups](#mod.order_groups)
  - [App](#type.order_groups.App)
  - [Log](#type.order_groups.Log)
  - [Level](#type.order_groups.Level)
- [engine](#mod.order_groups.engine)
  - [engine::Core](#type.order_groups.engine.Core)
  - [engine::Plan](#type.order_groups.engine.Plan)
- [engine::sched](#mod.order_groups.engine.sched)
  - [engine::sched::Queue](#type.order_groups.engine.sched.Queue)
- [engine::sched::deep](#mod.order_groups.engine.sched.deep)
  - [engine::sched::deep::Slot](#type.order_groups.engine.sched.deep.Slot)
- [store](#mod.order_groups.store)
  - [store::Db](#type.order_groups.store.Db)
- [util](#mod.order_groups.util)
  - [util::Spare](#type.order_groups.util.Spare)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [App](#type.order_groups.App)
- [engine](#mod.order_groups.engine)
- [engine::Core](#type.order_groups.engine.Core)
- [engine::Plan](#type.order_groups.engine.Plan)
- [engine::sched](#mod.order_groups.engine.sched)
- [engine::sched::deep](#mod.order_groups.engine.sched.deep)
- [engine::sched::deep::Slot](#type.order_groups.engine.sched.deep.Slot)
- [engine::sched::Queue](#type.order_groups.engine.sched.Queue)
- [Level](#type.order_groups.Level)
- [Log](#type.order_groups.Log)
- [store](#mod.order_groups.store)
- [store::Db](#type.order_groups.store.Db)
- [util](#mod.order_groups.util)
- [util::Spare](#type.order_groups.util.Spare)

<a id="mod.order_groups"></a>
## order_groups

<a id="type.order_groups.App"></a>
### App

Counts: 3 fields

Used by: nothing

```
/// The only root: it names Core, Db, and Log, in that order.
pub struct App {
    pub a_core: engine::Core,

    pub b_db: store::Db,

    pub c_log: Log,
}
```

<a id="type.order_groups.Log"></a>
### Log

Counts: 1 field

Used by: [App](#type.order_groups.App)

```
/// Named only by App, in the crate root.
pub struct Log {
    pub level: Level,
}
```

<a id="type.order_groups.Level"></a>
### Level

Counts: none

Used by: [Log](#type.order_groups.Log)

```
/// Named only by Log, in the crate root.
pub struct Level;
```

<a id="mod.order_groups.engine"></a>
## engine

```
//! The engine.
```

<a id="type.order_groups.engine.Core"></a>
### engine::Core

Counts: 2 fields

Used by: [App](#type.order_groups.App)

```
/// Named only by App; names Plan, then Queue.
pub struct Core {
    pub a_plan: Plan,

    pub b_queue: sched::Queue,
}
```

<a id="type.order_groups.engine.Plan"></a>
### engine::Plan

Counts: none

Used by: [engine::Core](#type.order_groups.engine.Core)

```
/// Named only by Core.
pub struct Plan;
```

<a id="mod.order_groups.engine.sched"></a>
## engine::sched

```
//! The scheduler.
```

<a id="type.order_groups.engine.sched.Queue"></a>
### engine::sched::Queue

Counts: 1 field

Used by: [engine::Core](#type.order_groups.engine.Core)

```
/// Named only by Core; names Slot.
pub struct Queue {
    pub slot: deep::Slot,
}
```

<a id="mod.order_groups.engine.sched.deep"></a>
## engine::sched::deep

```
//! Three segments deep: its group is named for the first two.
```

<a id="type.order_groups.engine.sched.deep.Slot"></a>
### engine::sched::deep::Slot

Counts: none

Used by: [engine::sched::Queue](#type.order_groups.engine.sched.Queue)

```
/// Named only by Queue.
pub struct Slot;
```

<a id="mod.order_groups.store"></a>
## store

```
//! The store.
```

<a id="type.order_groups.store.Db"></a>
### store::Db

Counts: none

Used by: [App](#type.order_groups.App)

```
/// Named only by App: a lone type in its module.
pub struct Db;
```

<a id="mod.order_groups.util"></a>
## util

```
//! Helpers nothing uses.
```

<a id="type.order_groups.util.Spare"></a>
### util::Spare

Counts: none

Used by: nothing

```
/// Names nothing and nothing names it.
pub struct Spare;
```
