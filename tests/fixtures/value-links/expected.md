*Updated 2026-01-01T00:00:00Z*

<a id="crate.value_links"></a>
# value_links

```
//! Links to functions, constants, and statics.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
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
  - [tools::Shape](#type.value_links.tools.Shape)
- [Modules](#modules)
  - [value_links](#mod.value_links)
  - [relay](#mod.value_links.relay)
  - [tools](#mod.value_links.tools)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [backend()](#mod.value_links.tools)
- [GRAND_TOTAL](#mod.value_links.tools)
- [LIMIT](#mod.value_links.tools)
- [measure()](#mod.value_links.tools)
- [Shape](#type.value_links.tools.Shape)
- [tools](#mod.value_links.tools)
- [tools::backend()](#mod.value_links.tools)
- [tools::LIMIT](#mod.value_links.tools)
- [tools::measure()](#mod.value_links.tools)
- [tools::Shape](#type.value_links.tools.Shape)
- [tools::Shape()](#mod.value_links.tools)
- [tools::TOTAL](#mod.value_links.tools)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.value_links.tools.Shape"></a>
### tools::Shape

```
/// A shape that `measure` sizes, also reached as `crate::relay::measure`.
struct Shape
{
    sides: u8,
}
```

<a id="modules"></a>
## Modules

<a id="mod.value_links"></a>
### value_links

```
using tools::backend;
using GRAND_TOTAL = tools::TOTAL;
using tools::LIMIT;
using relay::measure;
using tools::Shape;
```

<a id="mod.value_links.relay"></a>
### relay

```
//! Passes values on.
```

```
using crate::tools::measure;
using crate::tools::TOTAL;
```

<a id="mod.value_links.tools"></a>
### tools

```
//! Defines the values.
```

- [tools::Shape](#type.value_links.tools.Shape)

```
/// The highest reading.
constexpr LIMIT: u32 = 8;

/// A running total.
static TOTAL: u32 = 0;

/// A Windows backend.
// cfg: windows
backend() -> u8;

/// The Unix backend.
// cfg: unix
backend() -> u8;

/// Reads the gauge, never above `LIMIT` and counted in `TOTAL`.
measure() -> u32;

/// Draws a `Shape`, which shares its name.
Shape();
```
