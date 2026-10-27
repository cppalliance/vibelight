*Updated 2026-01-01T00:00:00Z*

<a id="crate.links"></a>
# links

```
//! Source links.
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
- [links](#mod.links)
  - [Dial](#type.links.Dial)
  - [Engine](#type.links.Engine)
- [parts](#mod.links.parts)
  - [parts::Gauge](#type.links.parts.Gauge)
  - [parts::Part](#type.links.parts.Part)
- [Other impls](#impls)
- [Spellings](#spellings)

**MACROS:** 1 definition

<a id="surface"></a>
## Public surface

- [Dial](#type.links.Dial)
- [Engine](#type.links.Engine)

<a id="levels"></a>
## Levels

- Unconnected types
  - [Dial](#type.links.Dial)
  - [Engine](#type.links.Engine)
  - [parts::Gauge](#type.links.parts.Gauge)
  - [parts::Part](#type.links.parts.Part)

<a id="mod.links"></a>
## links

Source: [src/lib.rs](../src/lib.rs)

```
// MACRO macro_rules! noop
```

**MACRO** macro_rules! noop (source: [src/lib.rs:15](../src/lib.rs#L15))

<a id="type.links.Dial"></a>
### Dial

Source: [src/lib.rs:26](../src/lib.rs#L26)

Counts: 1 field (1 Option)

Used by: nothing

```
/// A dial with an optional limit.
pub struct Dial {
    pub limit: std::option::Option<u32>,
}
```

<a id="type.links.Engine"></a>
### Engine

Source: [src/lib.rs:6](../src/lib.rs#L6), impl [src/lib.rs:10](../src/lib.rs#L10) (start), [src/parts.rs:3](../src/parts.rs#L3) (stop)

Counts: 1 field, 2 functions

Used by: nothing

```
/// A type with an impl in another file.
pub struct Engine {
    pub speed: u32,
}

// impl in src/lib.rs
impl Engine {
    /// Starts the engine.
    pub fn start(&self);
}

// impl in src/parts.rs
impl crate::Engine {
    /// Stops the engine.
    pub fn stop(&self);
}
```

<a id="mod.links.parts"></a>
## parts

Source: [src/parts.rs](../src/parts.rs)

```
//! Parts of the engine.
```

<a id="type.links.parts.Gauge"></a>
### parts::Gauge

Source: [src/parts.rs:12](../src/parts.rs#L12)

Counts: 2 fields (2 Option)

Used by: nothing

```
/// A gauge that writes one standard type two ways.
pub struct Gauge {
    pub reading: Option<u32>,

    pub spare: std::option::Option<u32>,
}
```

<a id="type.links.parts.Part"></a>
### parts::Part

Source: [src/parts.rs:9](../src/parts.rs#L9)

Counts: none

Used by: nothing

```
/// A part.
pub struct Part;
```

<a id="impls"></a>
## Other impls

- `impl Clone for Box<Engine> {}` (source: [src/lib.rs:19](../src/lib.rs#L19))

<a id="spellings"></a>
## Spellings

- `std::option::Option` is written 2 ways; prefer `Option`:
  - `Option`, once
  - `std::option::Option`, 2 times (source: [src/lib.rs:27](../src/lib.rs#L27), [src/parts.rs:14](../src/parts.rs#L14))
