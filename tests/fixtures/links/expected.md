<a id="crate.links"></a>
# links

```
//! Source links.
```

- [Unconnected types](#unconnected)
  - [Engine](#type.links.Engine)
  - [parts::Part](#type.links.parts.Part)
- [Other impls](#impls)
- [Modules](#modules)
  - [links](#mod.links)
  - [parts](#mod.links.parts)

**MACROS:** 1 definition
- **MACRO** `macro_rules! noop`, in [links](#mod.links) (source: [src/lib.rs:15](../src/lib.rs#L15))

<a id="unconnected"></a>
## Unconnected types

<a id="type.links.Engine"></a>
### Engine

Source: [src/lib.rs:6](../src/lib.rs#L6), impl [src/lib.rs:10](../src/lib.rs#L10), [src/parts.rs:3](../src/parts.rs#L3)

```
/// A type with an impl in another file.
struct Engine
{
    speed: u32,

    /// Starts the engine.
    start() const;

    /// Stops the engine.
    stop() const;
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

- [parts::Part](#type.links.parts.Part)
