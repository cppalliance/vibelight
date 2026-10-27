*Updated 2026-01-01T00:00:00Z*

<a id="crate.beta"></a>
# beta

```
//! Beta: re-exports a type from one of its modules.
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
- [beta](#mod.beta)
- [parts](#mod.beta.parts)
  - [parts::Gauge](#type.beta.parts.Gauge)
  - [parts::Valve](#type.beta.parts.Valve)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Gauge](#type.beta.parts.Gauge)
- [parts](#mod.beta.parts)
- [parts::Gauge](#type.beta.parts.Gauge)
- [parts::Valve](#type.beta.parts.Valve)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [parts::Gauge](#type.beta.parts.Gauge)
- Unconnected types
  - [parts::Valve](#type.beta.parts.Valve)

<a id="mod.beta"></a>
## beta

```
pub use parts::Gauge;
```

<a id="mod.beta.parts"></a>
## parts

```
//! The parts of a meter.
```

<a id="type.beta.parts.Gauge"></a>
### parts::Gauge

Counts: none

Used by: [beta](#mod.beta)

```
/// A gauge.
pub struct Gauge;
```

<a id="type.beta.parts.Valve"></a>
### parts::Valve

Counts: none

Used by: nothing

```
/// A valve.
pub struct Valve;
```
