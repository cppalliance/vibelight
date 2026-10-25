<a id="crate.base_lists"></a>
# base_lists

```
//! Base lists: short names, long paths, wrapping, collisions, and supertraits.
```

- [Level 1: top-level types](#level.1)
  - [Error](#type.base_lists.Error)
  - [Holder](#type.base_lists.Holder)
  - [Labeled](#type.base_lists.Labeled)
  - [Local](#type.base_lists.Local)
  - [Mixed](#type.base_lists.Mixed)
- [Level 2](#level.2)
  - [Named](#type.base_lists.Named)
  - [Clone](#type.base_lists.Clone)
- [Unconnected types](#unconnected)
  - [Bits](#type.base_lists.Bits)
  - [Marker](#type.base_lists.Marker)
  - [Plain](#type.base_lists.Plain)
  - [Run](#type.base_lists.Run)
  - [RunError](#type.base_lists.RunError)
  - [Secret](#type.base_lists.Secret)
  - [Sink](#type.base_lists.Sink)
  - [Wide](#type.base_lists.Wide)
  - [Wrapped](#type.base_lists.Wrapped)
- [Modules](#modules)
  - [base_lists](#mod.base_lists)

**MACROS:** 1 derive list
- **MACRO** `derive: From`, in [Wrapped](#type.base_lists.Wrapped)

<a id="level.1"></a>
## Level 1: top-level types

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

<a id="type.base_lists.Marker"></a>
### Marker

```
/// A unit struct whose list fits on the head line.
struct Marker : Debug, Eq, Hash;
```

<a id="type.base_lists.Plain"></a>
### Plain

```
/// Standard supertraits and a lifetime bound.
trait Plain : 'static, Send, Sync
{
}
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
- [Plain](#type.base_lists.Plain)
- [Run](#type.base_lists.Run)
- [RunError](#type.base_lists.RunError)
- [Secret](#type.base_lists.Secret)
- [Sink](#type.base_lists.Sink)
- [Wide](#type.base_lists.Wide)
- [Wrapped](#type.base_lists.Wrapped)
