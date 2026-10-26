*Updated 2026-01-01T00:00:00Z*

<a id="crate.surface"></a>
# surface

```
//! The public surface of a library.
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
  - [api::v1::Request](#type.surface.api.v1.Request)
  - [detail::Handle](#type.surface.detail.Handle)
- [Test support](#test-support)
  - [Fake](#type.surface.Fake)
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
- [Fake](#type.surface.Fake) (test support)
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

<a id="test-support"></a>
## Test support

<a id="type.surface.Fake"></a>
### Fake

```
/// A fake for tests and benches.
// cfg: any(test, feature = "test-support")
struct Fake;
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
// internal
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

- [Fake](#type.surface.Fake)
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
