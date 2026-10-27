*Updated 2026-01-01T00:00:00Z*

<a id="crate.names"></a>
# names

```
//! Name resolution.
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
- [names](#mod.names)
  - [Catalog](#type.names.Catalog)
  - [Pairs](#type.names.Pairs)
  - [Twin](#type.names.Twin)
  - [Twin](#type.names.Twin-2)
  - [Wrapper](#type.names.Wrapper)
- [api](#mod.names.api)
  - [api::Token](#type.names.api.Token)
- [api::inner](#mod.names.api.inner)
  - [api::inner::Handle](#type.names.api.inner.Handle)
  - [api::inner::Shared](#type.names.api.inner.Shared)
- [model](#mod.names.model)
  - [model::Record](#type.names.model.Record)
  - [model::Shared](#type.names.model.Shared)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [api](#mod.names.api)
- [api::Handle](#type.names.api.inner.Handle)
- [api::inner](#mod.names.api.inner)
- [api::inner::Handle](#type.names.api.inner.Handle)
- [api::inner::Shared](#type.names.api.inner.Shared)
- [api::Shared](#type.names.api.inner.Shared)
- [api::Token](#type.names.api.Token)
- [Catalog](#type.names.Catalog)
- [Entry](#type.names.model.Record)
- [model](#mod.names.model)
- [model::Record](#type.names.model.Record)
- [model::Shared](#type.names.model.Shared)
- [Pairs](#type.names.Pairs)
- [Twin](#type.names.Twin)
- [Wrapper](#type.names.Wrapper)

<a id="mod.names"></a>
## names

```
pub use model::Record as Entry;
```

<a id="type.names.Catalog"></a>
### Catalog

Counts: 5 fields

Used by: nothing

```
/// Uses names resolved every way rule 39 allows.
pub struct Catalog {
    /// Through a prefix path.
    pub first: crate::model::Record,

    /// Through a pub use rename at the root.
    pub second: Entry,

    /// Through a glob import.
    pub third: api::Handle,

    /// A unique name across the target.
    pub fourth: Token,

    /// An ambiguous name stays plain.
    pub fifth: Shared,
}
```

<a id="type.names.Pairs"></a>
### Pairs

Counts: none

Used by: nothing

```
/// Escapes in HTML: <, >, and & in types and comments.
pub type Pairs<'a> = &'a [(Token, Option<Vec<Token>>)];
```

<a id="type.names.Twin"></a>
### Twin

Counts: none

Used by: nothing

```
/// The first of two types named Twin, under different cfgs.
#[cfg(unix)]
pub struct Twin;
```

<a id="type.names.Twin-2"></a>
### Twin

Counts: none

Used by: nothing

```
/// The second Twin.
#[cfg(windows)]
pub struct Twin;
```

<a id="type.names.Wrapper"></a>
### Wrapper

Counts: 1 field

Used by: nothing

```
/// A generic parameter that shares a type's name stays plain.
pub struct Wrapper<Record> {
    pub inner: Record,
}
```

<a id="mod.names.api"></a>
## api

```
//! Re-exports through a glob.
```

```
pub use self::inner::*;
```

<a id="type.names.api.Token"></a>
### api::Token

Counts: none

Used by: [Catalog](#type.names.Catalog), [Pairs](#type.names.Pairs)

```
/// A unique token type.
pub struct Token;
```

<a id="mod.names.api.inner"></a>
## api::inner

<a id="type.names.api.inner.Handle"></a>
### api::inner::Handle

Counts: none

Used by: [Catalog](#type.names.Catalog)

```
/// A handle.
pub struct Handle;
```

<a id="type.names.api.inner.Shared"></a>
### api::inner::Shared

Counts: none

Used by: nothing

```
/// The other type named Shared.
pub struct Shared;
```

<a id="mod.names.model"></a>
## model

```
//! The record types.
```

<a id="type.names.model.Record"></a>
### model::Record

Counts: 1 field

Used by: [Catalog](#type.names.Catalog), [names](#mod.names)

```
/// A record.
pub struct Record {
    pub id: u32,
}
```

<a id="type.names.model.Shared"></a>
### model::Shared

Counts: none

Used by: nothing

```
/// One of two types named Shared.
pub struct Shared;
```
