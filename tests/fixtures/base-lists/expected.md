*Updated 2026-01-01T00:00:00Z*

<a id="crate.base_lists"></a>
# base_lists

```
//! Base lists: short names, long paths, wrapping, collisions, and supertraits.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
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
  - [custom::Homemade](#type.base_lists.custom.Homemade)
  - [Error](#type.base_lists.Error)
  - [Holder](#type.base_lists.Holder)
  - [Labeled](#type.base_lists.Labeled)
  - [Local](#type.base_lists.Local)
  - [Mixed](#type.base_lists.Mixed)
- [Level 2](#level.2)
  - [custom::Display](#type.base_lists.custom.Display)
  - [Named](#type.base_lists.Named)
  - [Clone](#type.base_lists.Clone)
- [Unconnected types](#unconnected)
  - [bare::Imported](#type.base_lists.bare.Imported)
  - [Bits](#type.base_lists.Bits)
  - [globbed::Starred](#type.base_lists.globbed.Starred)
  - [Marker](#type.base_lists.Marker)
  - [Ordered](#type.base_lists.Ordered)
  - [other::Foreign](#type.base_lists.other.Foreign)
  - [Plain](#type.base_lists.Plain)
  - [Renamed](#type.base_lists.Renamed)
  - [Run](#type.base_lists.Run)
  - [RunError](#type.base_lists.RunError)
  - [Secret](#type.base_lists.Secret)
  - [Sink](#type.base_lists.Sink)
  - [Spelled](#type.base_lists.Spelled)
  - [Wide](#type.base_lists.Wide)
  - [Wrapped](#type.base_lists.Wrapped)
- [Modules](#modules)
  - [base_lists](#mod.base_lists)
  - [bare](#mod.base_lists.bare)
  - [custom](#mod.base_lists.custom)
  - [globbed](#mod.base_lists.globbed)
  - [other](#mod.base_lists.other)
- [Spellings](#spellings)

**MACROS:** 1 derive list
- **MACRO** `derive: From`, in [Wrapped](#type.base_lists.Wrapped)

<a id="surface"></a>
## Public surface

- [bare](#mod.base_lists.bare)
- [bare::Imported](#type.base_lists.bare.Imported)
- [Bits](#type.base_lists.Bits)
- [Clone](#type.base_lists.Clone)
- [custom](#mod.base_lists.custom)
- [custom::Display](#type.base_lists.custom.Display)
- [custom::Homemade](#type.base_lists.custom.Homemade)
- [Error](#type.base_lists.Error)
- [globbed](#mod.base_lists.globbed)
- [globbed::Starred](#type.base_lists.globbed.Starred)
- [Holder](#type.base_lists.Holder)
- [Labeled](#type.base_lists.Labeled)
- [Local](#type.base_lists.Local)
- [Marker](#type.base_lists.Marker)
- [Mixed](#type.base_lists.Mixed)
- [Named](#type.base_lists.Named)
- [Ordered](#type.base_lists.Ordered)
- [other](#mod.base_lists.other)
- [other::Foreign](#type.base_lists.other.Foreign)
- [Plain](#type.base_lists.Plain)
- [Renamed](#type.base_lists.Renamed)
- [Run](#type.base_lists.Run)
- [RunError](#type.base_lists.RunError)
- [Secret](#type.base_lists.Secret)
- [Sink](#type.base_lists.Sink)
- [Spelled](#type.base_lists.Spelled)
- [Wide](#type.base_lists.Wide)
- [Wrapped](#type.base_lists.Wrapped)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.base_lists.custom.Homemade"></a>
### custom::Homemade

```
/// The crate's Display, which prints its path on its own line.
struct Homemade
    : custom::Display;
```

<a id="type.base_lists.Error"></a>
### Error

```
/// The crate's own error, whose name collides with std::error::Error.
enum Error
    : Debug, std::error::Error
    , Named
{
    Failed,
}
```

<a id="type.base_lists.Holder"></a>
### Holder

```
/// Generics, a where clause, and a trait of this crate.
struct Holder<T>
    : Debug
    , Named
    where T: Copy
{
    value: T,
}
```

<a id="type.base_lists.Labeled"></a>
### Labeled

```
/// A standard supertrait and a supertrait of this crate.
trait Labeled
    : Display
    , Named
{
}
```

<a id="type.base_lists.Local"></a>
### Local

```
/// The crate's own Clone, a trait nothing declares, and a renamed standard
/// trait.
struct Local
    : Debug
    , Clone
    , Unknown;
```

<a id="type.base_lists.Mixed"></a>
### Mixed

```
/// A trait of this crate and one of another crate take the comma form.
struct Mixed
    : Debug
    , Named
    , serde::Serialize;
```

<a id="level.2"></a>
## Level 2

<a id="type.base_lists.custom.Display"></a>
### custom::Display

```
/// The crate's own Display.
trait Display
{
}
```

<a id="type.base_lists.Named"></a>
### Named

```
/// A trait of this crate.
trait Named
{
    name() -> String;
}
```

<a id="type.base_lists.Clone"></a>
### Clone

```
/// A trait of this crate that shares a prelude name.
trait Clone
{
}
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.base_lists.bare.Imported"></a>
### bare::Imported

```
/// Display through the use, which is the standard one.
struct Imported : Display;
```

<a id="type.base_lists.Bits"></a>
### Bits

```
/// A union with standard derives.
union Bits : Copy, Debug
{
    float: f32,

    int: u32,
}
```

<a id="type.base_lists.globbed.Starred"></a>
### globbed::Starred

```
/// A Display from the glob, which prints as written.
struct Starred
    : Display;
```

<a id="type.base_lists.Marker"></a>
### Marker

```
/// A unit struct whose list fits on the head line.
struct Marker : Debug, Eq, Hash;
```

<a id="type.base_lists.Ordered"></a>
### Ordered

```
/// Standard derives that are not special members, and comparisons with
/// another type written by hand. The crate's own Clone trait never sits on
/// the shared line, so the standard Clone keeps its short name.
struct Ordered(u8)
    : Clone, Eq, Ord, PartialEq, PartialEq<u8>, PartialOrd, PartialOrd<u8>;
```

<a id="type.base_lists.other.Foreign"></a>
### other::Foreign

```
/// The other crate's Display, which stays apart from the standard one.
struct Foreign
    : other_crate::Display;
```

<a id="type.base_lists.Plain"></a>
### Plain

```
/// Standard supertraits and a lifetime bound.
trait Plain : 'static, Send, Sync
{
}
```

<a id="type.base_lists.Renamed"></a>
### Renamed

```
/// The standard Display through a renaming use.
struct Renamed : Display;
```

<a id="type.base_lists.Run"></a>
### Run

```
/// Standard traits only, so the list stays on the head line.
struct Run : Debug, Display
{
    id: u32,
}
```

<a id="type.base_lists.RunError"></a>
### RunError

```
/// The crate's Error makes std::error::Error keep its full path.
struct RunError : Debug, Display, std::error::Error
{
    reason: String,
}
```

<a id="type.base_lists.Secret"></a>
### Secret

```
/// A class with a standard trait.
class Secret : Display
{
    value: u8,
}
```

<a id="type.base_lists.Sink"></a>
### Sink

```
/// Two traits named Write keep their full paths.
struct Sink : std::fmt::Write, std::io::Write;
```

<a id="type.base_lists.Spelled"></a>
### Spelled

```
/// The standard Display written in full, with no use.
struct Spelled : Display;
```

<a id="type.base_lists.Wide"></a>
### Wide

```
/// So many standard traits that the list wraps.
struct Wide(u64)
    : Binary, Copy, Debug, Display, Eq, Hash, LowerExp, LowerHex, Octal, Ord
    , Pointer, UpperExp, UpperHex;
```

<a id="type.base_lists.Wrapped"></a>
### Wrapped

**MACRO** derive: From

```
/// A bare derive of From comes from another crate, so it stays on the mark.
struct Wrapped(u8) : Debug;
```

<a id="modules"></a>
## Modules

<a id="mod.base_lists"></a>
### base_lists

- [Bits](#type.base_lists.Bits)
- [Clone](#type.base_lists.Clone)
- [Error](#type.base_lists.Error)
- [Holder](#type.base_lists.Holder)
- [Labeled](#type.base_lists.Labeled)
- [Local](#type.base_lists.Local)
- [Marker](#type.base_lists.Marker)
- [Mixed](#type.base_lists.Mixed)
- [Named](#type.base_lists.Named)
- [Ordered](#type.base_lists.Ordered)
- [Plain](#type.base_lists.Plain)
- [Renamed](#type.base_lists.Renamed)
- [Run](#type.base_lists.Run)
- [RunError](#type.base_lists.RunError)
- [Secret](#type.base_lists.Secret)
- [Sink](#type.base_lists.Sink)
- [Spelled](#type.base_lists.Spelled)
- [Wide](#type.base_lists.Wide)
- [Wrapped](#type.base_lists.Wrapped)

<a id="mod.base_lists.bare"></a>
### bare

```
//! The standard Display after a use of the trait itself.
```

- [bare::Imported](#type.base_lists.bare.Imported)

<a id="mod.base_lists.custom"></a>
### custom

```
//! A trait of this crate named Display.
```

- [custom::Display](#type.base_lists.custom.Display)
- [custom::Homemade](#type.base_lists.custom.Homemade)

<a id="mod.base_lists.globbed"></a>
### globbed

```
//! A Display that can only come from a glob import.
```

- [globbed::Starred](#type.base_lists.globbed.Starred)

<a id="mod.base_lists.other"></a>
### other

```
//! Another crate's trait that is also named Display.
```

- [other::Foreign](#type.base_lists.other.Foreign)

<a id="spellings"></a>
## Spellings

- `std::fmt::Debug` is written 3 ways; prefer `Debug`:
  - `Dbg`, once
  - `Debug`, 5 times
  - `fmt::Debug`, 4 times
- `std::fmt::Display` is written 4 ways; pick one and search for the rest:
  - `D`, once
  - `Display`, once
  - `fmt::Display`, 5 times
  - `std::fmt::Display`, once
