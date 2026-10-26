*Updated 2026-01-01T00:00:00Z*

<a id="crate.attributes"></a>
# attributes

```
//! Attributes that shape an API or its wire format.
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
- [Level 1: top-level types](#level.1)
  - [Record](#type.attributes.Record)
- [Unconnected types](#unconnected)
  - [Level](#type.attributes.Level)
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

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.attributes.Record"></a>
### Record

**MACRO** derive: serde::Serialize

```
/// A record with a wire format.
[[serde(rename_all = "camelCase")]]
struct Record
{
    /// The record's name.
    [[serde(rename = "id")]]
    name: String,

    /// Kept out of the docs.
    [[doc(hidden)]]
    cache: u8,

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
