*Updated 2026-01-01T00:00:00Z*

<a id="crate.surface"></a>
# surface

```
//! The public surface of a library.
```

- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [api::v1::Request](#type.surface.api.v1.Request)
  - [detail::Handle](#type.surface.detail.Handle)
- [Unconnected types](#unconnected)
  - [detail::tools::Tool](#type.surface.detail.tools.Tool)
  - [Internal](#type.surface.Internal)
  - [prelude::Marker](#type.surface.prelude.Marker)
- [Modules](#modules)
  - [surface](#mod.surface)
  - [api](#mod.surface.api)
  - [api::v1](#mod.surface.api.v1)
  - [detail](#mod.surface.detail)
  - [detail::tools](#mod.surface.detail.tools)
  - [prelude](#mod.surface.prelude)
  - [values](#mod.surface.values)
  - [values::inner](#mod.surface.values.inner)

**MACROS:** 1 definition
- **MACRO** `macro_rules! hello`, in [surface](#mod.surface)

<a id="surface"></a>
## Public surface

- [api](#mod.surface.api)
- [api::v1](#mod.surface.api.v1)
- [api::v1::Request](#type.surface.api.v1.Request)
- [api::v1::send()](#mod.surface.api.v1)
- [api::VERSION](#mod.surface.api)
- BTreeMap (std::collections::BTreeMap)
- [build()](#mod.surface.values)
- [CALLS](#mod.surface)
- [CURRENT](#mod.surface.values)
- [deep()](#mod.surface.values.inner)
- [Handle](#type.surface.detail.Handle)
- [hello!](#mod.surface)
- [kit](#mod.surface.detail.tools)
- [kit::Tool](#type.surface.detail.tools.Tool)
- [mark()](#mod.surface.prelude)
- [Marker](#type.surface.prelude.Marker)
- [MAX](#mod.surface.values)
- [start()](#mod.surface)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.surface.api.v1.Request"></a>
### api::v1::Request

```
/// A request.
struct Request;
```

<a id="type.surface.detail.Handle"></a>
### detail::Handle

```
/// A handle made public through a re-export.
struct Handle;
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.surface.detail.tools.Tool"></a>
### detail::tools::Tool

```
/// A tool.
struct Tool;
```

<a id="type.surface.Internal"></a>
### Internal

```
/// A crate-only type.
struct Internal;
```

<a id="type.surface.prelude.Marker"></a>
### prelude::Marker

```
/// A marker a glob re-exports.
struct Marker;
```

<a id="modules"></a>
## Modules

<a id="mod.surface"></a>
### surface

- [Internal](#type.surface.Internal)

```
using prelude::*;

using std::collections::BTreeMap;

using values::build;

using values::CURRENT;

using values::deep;

using detail::Handle;

using kit = detail::tools;

using values::MAX;

/// Says hello.
// MACRO macro_rules! hello

/// The number of calls so far.
static CALLS: u8 = 0;

/// Starts the library.
start();
```

**MACRO** macro_rules! hello

<a id="mod.surface.api"></a>
### api

```
//! The public API.
```

```
/// The version in use.
constexpr VERSION: u8 = 1;
```

<a id="mod.surface.api.v1"></a>
### api::v1

```
//! The first version.
```

- [api::v1::Request](#type.surface.api.v1.Request)

```
/// Sends a request.
send(request: Request) -> u8;
```

<a id="mod.surface.detail"></a>
### detail

```
//! Hidden helpers.
```

- [detail::Handle](#type.surface.detail.Handle)

```
/// A helper that stays hidden.
helper();
```

<a id="mod.surface.detail.tools"></a>
### detail::tools

```
//! Tools reached only through a re-export.
```

- [detail::tools::Tool](#type.surface.detail.tools.Tool)

<a id="mod.surface.prelude"></a>
### prelude

```
//! Names a glob brings in.
```

- [prelude::Marker](#type.surface.prelude.Marker)

```
/// A function a glob re-exports.
mark();
```

<a id="mod.surface.values"></a>
### values

```
//! Values made public through re-exports.
```

```
using self::inner::deep;

/// The largest value.
constexpr MAX: u8 = 9;

/// The current value.
static CURRENT: u8 = 0;

/// Builds a value.
build() -> u8;
```

<a id="mod.surface.values.inner"></a>
### values::inner

```
/// A function two re-exports away.
deep();
```
