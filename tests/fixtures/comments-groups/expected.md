*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_groups"></a>
# comments_groups

```
//! Comments on impl blocks and on use groups.
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
- [comments_groups](#mod.comments_groups)
  - [Handle](#type.comments_groups.Handle)
- [store](#mod.comments_groups.store)
  - [store::Key](#type.comments_groups.store.Key)
  - [store::Value](#type.comments_groups.store.Value)
- [traits](#mod.comments_groups.traits)
  - [traits::Persist](#type.comments_groups.traits.Persist)

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

<a id="mod.comments_groups"></a>
## comments_groups

```
// The key type.
pub use store::Key;

// Store operations, for scripts.
pub use store::read;
// Writes replace the old value.
pub use store::write;

pub use store::Value; // The value type.
```

<a id="type.comments_groups.Handle"></a>
### Handle

Counts: 2 functions

Used by: nothing

```
/// A handle.
pub struct Handle;

// impl in src/lib.rs
// Opening.
impl Handle {
    /// Opens a handle.
    pub fn open() -> Handle;
}

// impl in src/lib.rs
// Reading.
impl Handle {
    /// Reads a byte.
    pub fn get(&self) -> u8;
}

/// Shows the handle.
impl std::fmt::Display for Handle {}

// Persists the handle.
impl traits::Persist for Handle {}
```

<a id="mod.comments_groups.store"></a>
## store

```
//! Keys and values.
```

```
/// Reads a value.
pub fn read();

/// Writes a value.
pub fn write();
```

<a id="type.comments_groups.store.Key"></a>
### store::Key

Counts: none

Used by: [comments_groups](#mod.comments_groups)

```
/// A key.
pub struct Key;
```

<a id="type.comments_groups.store.Value"></a>
### store::Value

Counts: none

Used by: [comments_groups](#mod.comments_groups)

```
/// A value.
pub struct Value;
```

<a id="mod.comments_groups.traits"></a>
## traits

<a id="type.comments_groups.traits.Persist"></a>
### traits::Persist

Counts: 1 function

Used by: [Handle](#type.comments_groups.Handle)

```
/// Something that persists.
pub trait Persist {
    fn save(&self);
}
```
