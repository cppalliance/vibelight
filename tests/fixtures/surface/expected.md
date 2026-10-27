*Updated 2026-01-01T00:00:00Z*

<a id="crate.surface"></a>
# surface

```
//! The public surface of a library.
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
- [surface](#mod.surface)
  - [Fake](#type.surface.Fake)
  - [Internal](#type.surface.Internal)
- [api](#mod.surface.api)
- [api::v1](#mod.surface.api.v1)
  - [api::v1::Request](#type.surface.api.v1.Request)
- [detail](#mod.surface.detail)
  - [detail::Handle](#type.surface.detail.Handle)
- [detail::tools](#mod.surface.detail.tools)
  - [detail::tools::Tool](#type.surface.detail.tools.Tool)
- [prelude](#mod.surface.prelude)
  - [prelude::Marker](#type.surface.prelude.Marker)
- [values](#mod.surface.values)
- [values::inner](#mod.surface.values.inner)

**MACROS:** 1 definition

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

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [api::v1::Request](#type.surface.api.v1.Request)
  - [detail::Handle](#type.surface.detail.Handle)
- Test support
  - [Fake](#type.surface.Fake)
- Unconnected types
  - [detail::tools::Tool](#type.surface.detail.tools.Tool)
  - [Internal](#type.surface.Internal)
  - [prelude::Marker](#type.surface.prelude.Marker)

<a id="mod.surface"></a>
## surface

```
pub use prelude::*;
pub use std::collections::BTreeMap;
pub use values::build;
pub use values::CURRENT;
pub use values::deep;
pub use detail::Handle;
pub use detail::tools as kit;
pub use values::MAX;

/// Says hello.
// MACRO macro_rules! hello

/// The number of calls so far.
pub static CALLS: u8 = 0;

/// Starts the library.
pub fn start();
```

**MACRO** macro_rules! hello

<a id="type.surface.Fake"></a>
### Fake

Counts: none

Used by: nothing

```
/// A fake for tests and benches.
#[cfg(any(test, feature = "test-support"))]
pub struct Fake;
```

<a id="type.surface.Internal"></a>
### Internal

Counts: none

Used by: nothing

```
/// A crate-only type.
pub(crate) struct Internal;
```

<a id="mod.surface.api"></a>
## api

```
//! The public API.
```

```
/// The version in use.
pub const VERSION: u8 = 1;
```

<a id="mod.surface.api.v1"></a>
## api::v1

```
//! The first version.
```

```
/// Sends a request.
pub fn send(request: Request) -> u8;
```

<a id="type.surface.api.v1.Request"></a>
### api::v1::Request

Counts: none

Used by: [api::v1](#mod.surface.api.v1)

```
/// A request.
pub struct Request;
```

<a id="mod.surface.detail"></a>
## detail

```
//! Hidden helpers.
```

```
/// A helper that stays hidden.
pub fn helper();
```

<a id="type.surface.detail.Handle"></a>
### detail::Handle

Counts: none

Used by: [surface](#mod.surface)

```
/// A handle made public through a re-export.
pub struct Handle;
```

<a id="mod.surface.detail.tools"></a>
## detail::tools

```
//! Tools reached only through a re-export.
```

<a id="type.surface.detail.tools.Tool"></a>
### detail::tools::Tool

Counts: none

Used by: nothing

```
/// A tool.
pub struct Tool;
```

<a id="mod.surface.prelude"></a>
## prelude

```
//! Names a glob brings in.
```

```
/// A function a glob re-exports.
pub fn mark();
```

<a id="type.surface.prelude.Marker"></a>
### prelude::Marker

Counts: none

Used by: nothing

```
/// A marker a glob re-exports.
pub struct Marker;
```

<a id="mod.surface.values"></a>
## values

```
//! Values made public through re-exports.
```

```
pub use self::inner::deep;

/// The largest value.
pub const MAX: u8 = 9;

/// The current value.
pub static CURRENT: u8 = 0;

/// Builds a value.
pub fn build() -> u8;
```

<a id="mod.surface.values.inner"></a>
## values::inner

```
/// A function two re-exports away.
pub fn deep();
```
