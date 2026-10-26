*Updated 2026-01-01T00:00:00Z*

<a id="crate.structs"></a>
# structs

```
//! Struct shapes: named, tuple, unit, and generic.
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
  - [Buffer](#type.structs.Buffer)
  - [Empty](#type.structs.Empty)
  - [Index](#type.structs.Index)
  - [Marker](#type.structs.Marker)
  - [Meters](#type.structs.Meters)
  - [Pair](#type.structs.Pair)
  - [Point](#type.structs.Point)
  - [Span](#type.structs.Span)
  - [Token](#type.structs.Token)
- [Modules](#modules)
  - [structs](#mod.structs)

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

<a id="unconnected"></a>
## Unconnected types

<a id="type.structs.Buffer"></a>
### Buffer

```
/// Generics with a lifetime, a bound, a default, and a const generic.
struct Buffer<'a, T: Clone = u8, const N: usize = 16>
{
    data: [T; N]& 'a,

private:
    len: usize,
}
```

<a id="type.structs.Empty"></a>
### Empty

```
/// An empty braced struct.
struct Empty
{
}
```

<a id="type.structs.Index"></a>
### Index

```
/// A tuple struct with methods.
struct Index(usize)
{
    /// Returns the index.
    get() -> usize;
}
```

<a id="type.structs.Marker"></a>
### Marker

```
/// A unit struct.
struct Marker;
```

<a id="type.structs.Meters"></a>
### Meters

```
/// A tuple struct.
struct Meters(f64);
```

<a id="type.structs.Pair"></a>
### Pair

```
/// A tuple struct with two fields, one private.
struct Pair(u8, u16);
```

<a id="type.structs.Point"></a>
### Point

```
/// A named struct with public and private fields.
struct Point
{
    /// Horizontal position.
    x: i32,

    /// Vertical position.
    y: i32,

private:
    label: String,
}
```

<a id="type.structs.Span"></a>
### Span

```
/// A tuple struct whose fields carry comments.
struct Span(
    /// The first byte.
    usize,

    /// One past the last byte.
    usize);
```

<a id="type.structs.Token"></a>
### Token

```
/// A unit struct with a method.
struct Token
{
    /// Makes a token.
    static new() -> Token;
}
```

<a id="modules"></a>
## Modules

<a id="mod.structs"></a>
### structs

- [Buffer](#type.structs.Buffer)
- [Empty](#type.structs.Empty)
- [Index](#type.structs.Index)
- [Marker](#type.structs.Marker)
- [Meters](#type.structs.Meters)
- [Pair](#type.structs.Pair)
- [Point](#type.structs.Point)
- [Span](#type.structs.Span)
- [Token](#type.structs.Token)
