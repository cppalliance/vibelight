<a id="crate.impls"></a>
# impls

```
//! Impl blocks of every kind.
```

- [Unconnected types](#unconnected)
  - [Describe](#type.impls.Describe)
  - [Holder](#type.impls.Holder)
  - [Render](#type.impls.Render)
  - [Widget](#type.impls.Widget)
- [Other impls](#impls)
- [Modules](#modules)
  - [impls](#mod.impls)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.impls.Describe"></a>
### Describe

```
/// Describes itself.
trait Describe
{
    /// Returns a description.
    describe() -> String;
}
```

<a id="type.impls.Holder"></a>
### Holder

```
/// A generic holder.
struct Holder<'a, T>
{
    /// Returns the held item.
    item() -> T& 'a;

private:
    item: T& 'a,
}
```

<a id="type.impls.Render"></a>
### Render

```
/// Renders itself.
trait Render
{
    /// Renders to a string.
    render() -> String;
}
```

<a id="type.impls.Widget"></a>
### Widget

```
/// A type with split impls and trait impls.
struct Widget : Display, From<str& 'a>, unsafe Send, !Sync
{
    /// Makes a widget.
    static new(id: u32) -> Widget;

    /// Returns the identifier.
    id() -> u32;

private:
    id: u32,
}
```

<a id="impls"></a>
## Other impls

- `implements: Describe for T;`
- `implements: Render for (u8, u8);`
- `implements: Render for [Widget];`
- `implements: Render for Box<Widget>;`
- `implements: Render for Widget&;`

<a id="modules"></a>
## Modules

<a id="mod.impls"></a>
### impls

- [Describe](#type.impls.Describe)
- [Holder](#type.impls.Holder)
- [Render](#type.impls.Render)
- [Widget](#type.impls.Widget)
