*Updated 2026-01-01T00:00:00Z*

<a id="crate.names"></a>
# names

```
//! Name resolution.
```

- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [Catalog](#type.names.Catalog)
  - [Pairs](#type.names.Pairs)
- [Level 2](#level.2)
  - [model::Record](#type.names.model.Record)
  - [api::Token](#type.names.api.Token)
  - [api::inner::Handle](#type.names.api.inner.Handle)
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
- [api::inner](#mod.names.api.inner)
- [api::inner::Handle](#type.names.api.inner.Handle)
- [api::inner::Shared](#type.names.api.inner.Shared)
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
    /// An ambiguous name stays plain.
    fifth: Shared,

    /// Through a prefix path.
    first: crate::model::Record,

    /// A unique name across the target.
    fourth: Token,

    /// Through a pub use rename at the root.
    second: Entry,

    /// Through a glob import.
    third: api::Handle,
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

<a id="type.names.api.Token"></a>
### api::Token

```
/// A unique token type.
struct Token;
```

<a id="type.names.api.inner.Handle"></a>
### api::inner::Handle

```
/// A handle.
struct Handle;
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
