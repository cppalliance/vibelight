*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_groups"></a>
# comments_groups

```
//! Comments on impl blocks and on use groups.
```

- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [Handle](#type.comments_groups.Handle)
- [Level 2](#level.2)
  - [traits::Persist](#type.comments_groups.traits.Persist)
- [Unconnected types](#unconnected)
  - [store::Key](#type.comments_groups.store.Key)
  - [store::Value](#type.comments_groups.store.Value)
- [Modules](#modules)
  - [comments_groups](#mod.comments_groups)
  - [store](#mod.comments_groups.store)
  - [traits](#mod.comments_groups.traits)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Handle](#type.comments_groups.Handle)
- [Key](#type.comments_groups.store.Key)
- [read()](#mod.comments_groups.store)
- [store](#mod.comments_groups.store)
- [store::Key](#type.comments_groups.store.Key)
- [store::read()](#mod.comments_groups.store)
- [store::Value](#type.comments_groups.store.Value)
- [store::write()](#mod.comments_groups.store)
- [traits](#mod.comments_groups.traits)
- [traits::Persist](#type.comments_groups.traits.Persist)
- [Value](#type.comments_groups.store.Value)
- [write()](#mod.comments_groups.store)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.comments_groups.Handle"></a>
### Handle

```
/// A handle.
// impl Display
/// Shows the handle.
// impl traits::Persist
// Persists the handle.
// impl Handle
// Reading.
// impl Handle
// Opening.
struct Handle
    : Display
    , traits::Persist
{
    /// Opens a handle.
    static open() -> Handle;

    /// Reads a byte.
    get() -> u8;
}
```

<a id="level.2"></a>
## Level 2

<a id="type.comments_groups.traits.Persist"></a>
### traits::Persist

```
/// Something that persists.
trait Persist
{
    save();
}
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.comments_groups.store.Key"></a>
### store::Key

```
/// A key.
struct Key;
```

<a id="type.comments_groups.store.Value"></a>
### store::Value

```
/// A value.
struct Value;
```

<a id="modules"></a>
## Modules

<a id="mod.comments_groups"></a>
### comments_groups

- [Handle](#type.comments_groups.Handle)

```
// The key type.
using store::Key;

// Store operations, for scripts.
using store::read;
// Writes replace the old value.
using store::write;

using store::Value; // The value type.
```

<a id="mod.comments_groups.store"></a>
### store

```
//! Keys and values.
```

- [store::Key](#type.comments_groups.store.Key)
- [store::Value](#type.comments_groups.store.Value)

```
/// Reads a value.
read();

/// Writes a value.
write();
```

<a id="mod.comments_groups.traits"></a>
### traits

- [traits::Persist](#type.comments_groups.traits.Persist)
