*Updated 2026-01-01T00:00:00Z*

<a id="crate.value_links"></a>
# value_links

```
//! Links to functions, constants, and statics.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.
- Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [value_links](#mod.value_links)
- [relay](#mod.value_links.relay)
- [tools](#mod.value_links.tools)
  - [tools::Shape](#type.value_links.tools.Shape)

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

<a id="mod.value_links"></a>
## value_links

```
pub use tools::backend;
pub use tools::TOTAL as GRAND_TOTAL;
pub use tools::LIMIT;
pub use relay::measure;
pub use tools::Shape;
```

<a id="mod.value_links.relay"></a>
## relay

```
//! Passes values on.
```

```
pub use crate::tools::measure;
pub use crate::tools::TOTAL;
```

<a id="mod.value_links.tools"></a>
## tools

```
//! Defines the values.
```

```
/// The highest reading.
pub const LIMIT: u32 = 8;

/// A running total.
pub static TOTAL: u32 = 0;

/// A Windows backend.
#[cfg(windows)]
pub fn backend() -> u8;

/// The Unix backend.
#[cfg(unix)]
pub fn backend() -> u8;

/// Reads the gauge, never above `LIMIT` and counted in `TOTAL`.
pub fn measure() -> u32;

/// Draws a `Shape`, which shares its name.
pub fn Shape();
```

<a id="type.value_links.tools.Shape"></a>
### tools::Shape

Counts: 1 field

Used by: [value_links](#mod.value_links)

```
/// A shape that `measure` sizes, also reached as `crate::relay::measure`.
pub struct Shape {
    pub sides: u8,
}
```
