*Updated 2026-01-01T00:00:00Z*

<a id="crate.structs"></a>
# structs

```
//! Struct shapes: named, tuple, unit, and generic.
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
- [structs](#mod.structs)
  - [Buffer](#type.structs.Buffer)
  - [Empty](#type.structs.Empty)
  - [Index](#type.structs.Index)
  - [Marker](#type.structs.Marker)
  - [Meters](#type.structs.Meters)
  - [Pair](#type.structs.Pair)
  - [Point](#type.structs.Point)
  - [Span](#type.structs.Span)
  - [Token](#type.structs.Token)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Buffer](#type.structs.Buffer)
- [Empty](#type.structs.Empty)
- [Index](#type.structs.Index)
- [Marker](#type.structs.Marker)
- [Meters](#type.structs.Meters)
- [Pair](#type.structs.Pair)
- [Point](#type.structs.Point)
- [Span](#type.structs.Span)
- [Token](#type.structs.Token)

<a id="mod.structs"></a>
## structs

<a id="type.structs.Buffer"></a>
### Buffer

Counts: 2 fields

Used by: nothing

```
/// Generics with a lifetime, a bound, a default, and a const generic.
pub struct Buffer<'a, T: Clone = u8, const N: usize = 16> {
    pub data: &'a [T; N],

    len: usize,
}
```

<a id="type.structs.Empty"></a>
### Empty

Counts: none

Used by: nothing

```
/// An empty braced struct.
pub struct Empty {}
```

<a id="type.structs.Index"></a>
### Index

Counts: 1 field, 1 function

Used by: nothing

```
/// A tuple struct with methods.
pub struct Index(usize);

// impl in src/lib.rs
impl Index {
    /// Returns the index.
    pub fn get(&self) -> usize;
}
```

<a id="type.structs.Marker"></a>
### Marker

Counts: none

Used by: nothing

```
/// A unit struct.
pub struct Marker;
```

<a id="type.structs.Meters"></a>
### Meters

Counts: 1 field

Used by: nothing

```
/// A tuple struct.
pub struct Meters(pub f64);
```

<a id="type.structs.Pair"></a>
### Pair

Counts: 2 fields

Used by: nothing

```
/// A tuple struct with two fields, one private.
pub struct Pair(pub u8, u16);
```

<a id="type.structs.Point"></a>
### Point

Counts: 3 fields

Used by: nothing

```
/// A named struct with public and private fields.
pub struct Point {
    /// Horizontal position.
    pub x: i32,

    /// Vertical position.
    pub y: i32,

    label: String,
}
```

<a id="type.structs.Span"></a>
### Span

Counts: 2 fields

Used by: nothing

```
/// A tuple struct whose fields carry comments.
pub struct Span(
    /// The first byte.
    pub usize,
    /// One past the last byte.
    pub usize,
);
```

<a id="type.structs.Token"></a>
### Token

Counts: 1 function

Used by: nothing

```
/// A unit struct with a method.
pub struct Token;

// impl in src/lib.rs
impl Token {
    /// Makes a token.
    pub fn new() -> Token;
}
```
