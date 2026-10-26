*Updated 2026-01-01T00:00:00Z*

<a id="crate.surface"></a>
# surface

```
//! The public surface of a library.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [api::v1::Request](#type.surface.api.v1.Request)
  - [detail::Handle](#type.surface.detail.Handle)
  - [detail::tools::Tool](#type.surface.detail.tools.Tool)
  - [Internal](#type.surface.Internal)
- [Modules](#modules)
  - [surface](#mod.surface)
  - [api](#mod.surface.api)
  - [api::v1](#mod.surface.api.v1)
  - [detail](#mod.surface.detail)
  - [detail::tools](#mod.surface.detail.tools)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [api](#mod.surface.api)
- [api::v1](#mod.surface.api.v1)
- [api::v1::Request](#type.surface.api.v1.Request)
- [api::v1::send()](#mod.surface.api.v1)
- [api::VERSION](#mod.surface.api)
- [CALLS](#mod.surface)
- [Handle](#type.surface.detail.Handle)
- [kit](#mod.surface.detail.tools)
- [kit::Tool](#type.surface.detail.tools.Tool)
- [start()](#mod.surface)

<a id="unconnected"></a>
## Unconnected types

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

<a id="modules"></a>
## Modules

<a id="mod.surface"></a>
### surface

- [Internal](#type.surface.Internal)

```
using detail::Handle;

using kit = detail::tools;

/// The number of calls so far.
static CALLS: u8 = 0;

/// Starts the library.
start();
```

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
