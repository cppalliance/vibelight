*Updated 2026-01-01T00:00:00Z*

<a id="crate.names"></a>
# names

```
//! Name resolution.
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
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [Catalog](#type.names.Catalog)
  - [Pairs](#type.names.Pairs)
- [Level 2](#level.2)
  - [model::Record](#type.names.model.Record)
  - [api::inner::Handle](#type.names.api.inner.Handle)
  - [api::Token](#type.names.api.Token)
- [Unconnected types](#unconnected)
  - [api::inner::Shared](#type.names.api.inner.Shared)
  - [model::Shared](#type.names.model.Shared)
  - [Twin](#type.names.Twin)
  - [Twin](#type.names.Twin-2)
  - [Wrapper](#type.names.Wrapper)
- [Modules](#modules)
  - [names](#mod.names)
  - [api](#mod.names.api)
  - [api::inner](#mod.names.api.inner)
  - [model](#mod.names.model)

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

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.names.Catalog"></a>
### Catalog

```
/// Uses names resolved every way rule 39 allows.
struct Catalog
{
    /// Through a prefix path.
    first: crate::model::Record,

    /// Through a pub use rename at the root.
    second: Entry,

    /// Through a glob import.
    third: api::Handle,

    /// A unique name across the target.
    fourth: Token,

    /// An ambiguous name stays plain.
    fifth: Shared,
}
```

<a id="type.names.Pairs"></a>
### Pairs

```
/// Escapes in HTML: <, >, and & in types and comments.
using Pairs<'a> = [(Token, Option<Vec<Token>>)]& 'a;
```

<a id="level.2"></a>
## Level 2

<a id="type.names.model.Record"></a>
### model::Record

```
/// A record.
struct Record
{
    id: u32,
}
```

<a id="type.names.api.inner.Handle"></a>
### api::inner::Handle

```
/// A handle.
struct Handle;
```

<a id="type.names.api.Token"></a>
### api::Token

```
/// A unique token type.
struct Token;
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.names.api.inner.Shared"></a>
### api::inner::Shared

```
/// The other type named Shared.
struct Shared;
```

<a id="type.names.model.Shared"></a>
### model::Shared

```
/// One of two types named Shared.
struct Shared;
```

<a id="type.names.Twin"></a>
### Twin

```
/// The first of two types named Twin, under different cfgs.
// cfg: unix
struct Twin;
```

<a id="type.names.Twin-2"></a>
### Twin

```
/// The second Twin.
// cfg: windows
struct Twin;
```

<a id="type.names.Wrapper"></a>
### Wrapper

```
/// A generic parameter that shares a type's name stays plain.
struct Wrapper<Record>
{
    inner: Record,
}
```

<a id="modules"></a>
## Modules

<a id="mod.names"></a>
### names

- [Catalog](#type.names.Catalog)
- [Pairs](#type.names.Pairs)
- [Twin](#type.names.Twin)
- [Twin](#type.names.Twin-2)
- [Wrapper](#type.names.Wrapper)

```
using Entry = model::Record;
```

<a id="mod.names.api"></a>
### api

```
//! Re-exports through a glob.
```

- [api::Token](#type.names.api.Token)

```
using self::inner::*;
```

<a id="mod.names.api.inner"></a>
### api::inner

- [api::inner::Handle](#type.names.api.inner.Handle)
- [api::inner::Shared](#type.names.api.inner.Shared)

<a id="mod.names.model"></a>
### model

```
//! The record types.
```

- [model::Record](#type.names.model.Record)
- [model::Shared](#type.names.model.Shared)
