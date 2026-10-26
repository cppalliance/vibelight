*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_groups"></a>
# comments_groups

```
//! Comments on impl blocks and on use groups.
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
  - [Handle](#type.comments_groups.Handle)
  - [store::Key](#type.comments_groups.store.Key)
  - [store::Value](#type.comments_groups.store.Value)
- [Level 2](#level.2)
  - [traits::Persist](#type.comments_groups.traits.Persist)
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
