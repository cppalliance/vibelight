*Updated 2026-01-01T00:00:00Z*

<a id="crate.attributes"></a>
# attributes

```
//! Attributes that shape an API or its wire format.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Level](#type.attributes.Level)
  - [Record](#type.attributes.Record)
- [Modules](#modules)
  - [attributes](#mod.attributes)

**MACROS:** 1 derive list
- **MACRO** `derive: serde::Serialize`, in [Record](#type.attributes.Record)

<a id="surface"></a>
## Public surface

- [helper()](#mod.attributes)
- [Level](#type.attributes.Level)
- [Record](#type.attributes.Record)
- [Row](#type.attributes.Record)

<a id="unconnected"></a>
## Unconnected types

<a id="type.attributes.Level"></a>
### Level

```
/// A level.
[[non_exhaustive]]
[[repr(u8)]]
enum Level : Clone, Copy
{
    /// The top.
    [[deprecated(since = "1.2", note = "use High")]]
    Top = 2,

    /// The bottom.
    Low = 0,
}
```

<a id="type.attributes.Record"></a>
### Record

**MACRO** derive: serde::Serialize

```
/// A record with a wire format.
[[serde(rename_all = "camelCase")]]
struct Record
{
    /// Kept out of the docs.
    [[doc(hidden)]]
    cache: u8,

    /// The record's name.
    [[serde(rename = "id")]]
    name: String,

    /// Makes an empty record.
    [[must_use = "a record does nothing until it is stored"]]
    static empty() -> Record;

    /// Reads the name.
    [[must_use]]
    [[track_caller]]
    name() -> str&;

private:
    count: u8,
}
```

<a id="modules"></a>
## Modules

<a id="mod.attributes"></a>
### attributes

- [Level](#type.attributes.Level)
- [Record](#type.attributes.Record)

```
[[doc(hidden)]]
using Row = Record;

/// A hidden helper.
[[deprecated]]
[[doc(hidden)]]
helper();
```
