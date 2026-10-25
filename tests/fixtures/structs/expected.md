<a id="crate.structs"></a>
# structs

```
//! Struct shapes: named, tuple, unit, and generic.
```

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
