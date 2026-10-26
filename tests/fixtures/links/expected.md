*Updated 2026-01-01T00:00:00Z*

<a id="crate.links"></a>
# links

```
//! Source links.
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
- [Unconnected types](#unconnected)
  - [Dial](#type.links.Dial)
  - [Engine](#type.links.Engine)
  - [parts::Gauge](#type.links.parts.Gauge)
  - [parts::Part](#type.links.parts.Part)
- [Other impls](#impls)
- [Modules](#modules)
  - [links](#mod.links)
  - [parts](#mod.links.parts)
- [Spellings](#spellings)

**MACROS:** 1 definition
- **MACRO** `macro_rules! noop`, in [links](#mod.links) (source: [src/lib.rs:15](../src/lib.rs#L15))

<a id="surface"></a>
## Public surface

- [Dial](#type.links.Dial)
- [Engine](#type.links.Engine)

<a id="unconnected"></a>
## Unconnected types

<a id="type.links.Dial"></a>
### Dial

Source: [src/lib.rs:26](../src/lib.rs#L26)

```
/// A dial with an optional limit.
struct Dial
{
    limit: std::option::Option<u32>,
}
```

<a id="type.links.Engine"></a>
### Engine

Source: [src/lib.rs:6](../src/lib.rs#L6), impl [src/lib.rs:10](../src/lib.rs#L10) (start), [src/parts.rs:3](../src/parts.rs#L3) (stop)

```
/// A type with an impl in another file.
struct Engine
{
    speed: u32,

    /// Starts the engine.
    start();

    /// Stops the engine.
    stop();
}
```

<a id="type.links.parts.Gauge"></a>
### parts::Gauge

Source: [src/parts.rs:12](../src/parts.rs#L12)

```
/// A gauge that writes one standard type two ways.
struct Gauge
{
    reading: Option<u32>,

    spare: std::option::Option<u32>,
}
```

<a id="type.links.parts.Part"></a>
### parts::Part

Source: [src/parts.rs:9](../src/parts.rs#L9)

```
/// A part.
struct Part;
```

<a id="impls"></a>
## Other impls

- `implements: Clone for Box<Engine>;` (source: [src/lib.rs:19](../src/lib.rs#L19))

<a id="modules"></a>
## Modules

<a id="mod.links"></a>
### links

Source: [src/lib.rs](../src/lib.rs)

- [Dial](#type.links.Dial)
- [Engine](#type.links.Engine)

```
// MACRO macro_rules! noop
```

**MACRO** macro_rules! noop (source: [src/lib.rs:15](../src/lib.rs#L15))

<a id="mod.links.parts"></a>
### parts

Source: [src/parts.rs](../src/parts.rs)

```
//! Parts of the engine.
```

- [parts::Gauge](#type.links.parts.Gauge)
- [parts::Part](#type.links.parts.Part)

<a id="spellings"></a>
## Spellings

- `std::option::Option` is written 2 ways; prefer `Option`:
  - `Option`, once
  - `std::option::Option`, 2 times (source: [src/lib.rs:27](../src/lib.rs#L27), [src/parts.rs:14](../src/parts.rs#L14))
