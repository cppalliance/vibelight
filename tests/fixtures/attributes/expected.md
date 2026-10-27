*Updated 2026-01-01T00:00:00Z*

<a id="crate.attributes"></a>
# attributes

```
//! Attributes that shape an API or its wire format.
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
- [attributes](#mod.attributes)
  - [Record](#type.attributes.Record)
  - [Level](#type.attributes.Level)

**MACROS:** 1 derive list

<a id="surface"></a>
## Public surface

- [helper()](#mod.attributes)
- [Level](#type.attributes.Level)
- [Record](#type.attributes.Record)
- [Row](#type.attributes.Record)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [Record](#type.attributes.Record)
- Unconnected types
  - [Level](#type.attributes.Level)

<a id="mod.attributes"></a>
## attributes

```
#[doc(hidden)]
pub use Record as Row;

/// A hidden helper.
#[deprecated]
#[doc(hidden)]
pub fn helper();
```

<a id="type.attributes.Record"></a>
### Record

Counts: 3 fields, 2 functions

Used by: [attributes](#mod.attributes)

**MACRO** derive: serde::Serialize

```
/// A record with a wire format.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// The record's name.
    #[serde(rename = "id")]
    pub name: String,

    /// Kept out of the docs.
    #[doc(hidden)]
    pub cache: u8,

    count: u8,
}

// impl in src/lib.rs
impl Record {
    /// Reads the name.
    #[must_use]
    #[track_caller]
    pub fn name(&self) -> &str;

    /// Makes an empty record.
    #[must_use = "a record does nothing until it is stored"]
    pub fn empty() -> Record;
}
```

<a id="type.attributes.Level"></a>
### Level

Counts: 2 variants

Used by: nothing

```
/// A level.
#[derive(Clone, Copy)]
#[non_exhaustive]
#[repr(u8)]
pub enum Level {
    /// The top.
    #[deprecated(since = "1.2", note = "use High")]
    Top = 2,

    /// The bottom.
    Low = 0,
}
```
