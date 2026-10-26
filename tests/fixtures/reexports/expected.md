*Updated 2026-01-01T00:00:00Z*

<a id="crate.reexports"></a>
# reexports

```
//! Re-exports.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [inner::Other](#type.reexports.inner.Other)
  - [inner::Thing](#type.reexports.inner.Thing)
- [Modules](#modules)
  - [reexports](#mod.reexports)
  - [inner](#mod.reexports.inner)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [assist()](#mod.reexports.inner)
- [helper()](#mod.reexports.inner)
- [Other](#type.reexports.inner.Other)
- [Thing](#type.reexports.inner.Thing)

<a id="unconnected"></a>
## Unconnected types

<a id="type.reexports.inner.Other"></a>
### inner::Other

```
/// Another thing.
struct Other;
```

<a id="type.reexports.inner.Thing"></a>
### inner::Thing

```
/// A thing.
struct Thing;
```

<a id="modules"></a>
## Modules

<a id="mod.reexports"></a>
### reexports

```
using inner::*;

using assist = inner::helper;

using inner::Other;

using inner::Thing;

internal:
using CrateThing = inner::Thing;
```

<a id="mod.reexports.inner"></a>
### inner

```
//! Holds the items.
```

- [inner::Other](#type.reexports.inner.Other)
- [inner::Thing](#type.reexports.inner.Thing)

```
/// A helper.
helper();
```
