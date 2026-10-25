<a id="crate.unions"></a>
# unions

```
//! Unions.
```

- [Unconnected types](#unconnected)
  - [Bits](#type.unions.Bits)
  - [Hidden](#type.unions.Hidden)
- [Modules](#modules)
  - [unions](#mod.unions)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.unions.Bits"></a>
### Bits

```
/// Raw bits viewed two ways.
union Bits
{
    /// As a float.
    float: f32,

    /// As an integer.
    int: u32,

private:
    raw: [u8; 4],
}
```

<a id="type.unions.Hidden"></a>
### Hidden

```
/// A union with only private fields.
union Hidden
{
private:
    a: u8,

    b: i8,
}
```

<a id="modules"></a>
## Modules

<a id="mod.unions"></a>
### unions

- [Bits](#type.unions.Bits)
- [Hidden](#type.unions.Hidden)
