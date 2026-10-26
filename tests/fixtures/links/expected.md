*Updated 2026-01-01T00:00:00Z*

<a id="crate.links"></a>
# links

```
//! Source links.
```

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

Source: [src/lib.rs:6](../src/lib.rs#L6), impl [src/lib.rs:10](../src/lib.rs#L10), [src/parts.rs:3](../src/parts.rs#L3)

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
