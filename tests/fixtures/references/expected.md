*Updated 2026-01-01T00:00:00Z*

<a id="crate.references"></a>
# references

```
//! References, raw pointers, and receivers.
```

- [Unconnected types](#unconnected)
  - [Marker](#type.references.Marker)
  - [Source](#type.references.Source)
  - [Table](#type.references.Table)
  - [Views](#type.references.Views)
- [Other impls](#impls)
- [Modules](#modules)
  - [references](#mod.references)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.references.Marker"></a>
### Marker

```
/// Marks a type.
trait Marker
{
}
```

<a id="type.references.Source"></a>
### Source

```
/// A source of bytes.
trait Source
{
    name() -> str&;

    read(buf: [u8] mut&) mut -> usize;
}
```

<a id="type.references.Table"></a>
### Table

```
/// A borrowed table.
using Table<'a> = [(str& 'a, u8*)]& 'a;
```

<a id="type.references.Views"></a>
### Views

```
/// Borrowed and raw views of one value.
struct Views<'a, T>
{
    both: (dyn Display + Send)& 'a,

    callback: (dyn Fn(str&) -> usize)& 'a,

    check: Box<dyn Fn(str&) -> usize>,

    items: [T& 'a]& 'a,

    name: str& 'static,

    raw_call: (dyn Fn() -> bool)*,

    read: T*,

    shared: T& 'a,

    shown: dyn Display& 'a,

    step: (fn(u8) -> u8)& 'a,

    unique: T mut& 'a,

    wrapped: (dyn Fn(str&) -> usize)& 'a,

    write: T mut*,

    clear() mut;

    get() -> T&;

    into_name(self) -> str& 'static;

    lookup(
        keys: Vec<str& 'a>&,
        table: std::collections::BTreeMap<String, T mut& 'a>&) mut ->
        Option<T mut& 'a>;

    poll(self: Pin<Self mut&>);

    rename(mut self, name: str& 'static) -> Self;

    render(out: impl Write mut&, items: [Box<dyn Display>]&);

    same(other: Self&) -> bool;

    set(value: T) mut;
}
```

<a id="impls"></a>
## Other impls

- `implements: Marker for T&;`
- `implements: Marker for u8*;`

<a id="modules"></a>
## Modules

<a id="mod.references"></a>
### references

- [Marker](#type.references.Marker)
- [Source](#type.references.Source)
- [Table](#type.references.Table)
- [Views](#type.references.Views)

```
/// The default name.
constexpr NAME: str& = "views";

/// Raw bytes behind a reference.
static BYTES: [u8]& = &[1, 2];

/// Applies a function to a string.
apply<F>(text: String mut&, f: F) -> usize where F: Fn(String mut&) -> usize;

/// Finds the first item.
first<'a>(items: [str& 'a]& 'a) -> Option<str& 'a>;

/// Picks an item with a closure that returns a reference.
pick<'a, T>(choose: impl Fn(T&) -> T& 'a, items: [T]& 'a) -> Option<T& 'a>;
```
