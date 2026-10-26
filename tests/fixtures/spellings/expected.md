*Updated 2026-01-01T00:00:00Z*

<a id="crate.spellings"></a>
# spellings

```
//! Spellings: items from outside the crate written in more than one way.
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
  - [Timer](#type.spellings.Timer)
- [Level 2](#level.2)
  - [Clock](#type.spellings.Clock)
- [Unconnected types](#unconnected)
  - [bare::Label](#type.spellings.bare.Label)
  - [Duration](#type.spellings.Duration)
  - [other::Badge](#type.spellings.other.Badge)
  - [Wrapper](#type.spellings.Wrapper)
- [Modules](#modules)
  - [spellings](#mod.spellings)
  - [bare](#mod.spellings.bare)
  - [other](#mod.spellings.other)
- [Spellings](#spellings)

**MACROS:** 2 derive lists
- **MACRO** `derive: serde::Serialize`, in [Timer](#type.spellings.Timer)
- **MACRO** `derive: Serialize`, in [Clock](#type.spellings.Clock)

<a id="surface"></a>
## Public surface

- [bare](#mod.spellings.bare)
- [bare::Label](#type.spellings.bare.Label)
- [Clock](#type.spellings.Clock)
- [Duration](#type.spellings.Duration)
- [other](#mod.spellings.other)
- [other::Badge](#type.spellings.other.Badge)
- [Timer](#type.spellings.Timer)
- [Wrapper](#type.spellings.Wrapper)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.spellings.Timer"></a>
### Timer

**MACRO** derive: serde::Serialize

```
/// A timer that names its laps.
struct Timer : Display
{
    laps: Map<u32, String>,

    names: std::collections::BTreeMap<u32, String>,

    period: ::std::time::Duration,

    next(clock: crate::Clock&) -> std::option::Option<Clock>;

    wait(extra: std::time::Duration) -> Vec<String>;
}
```

<a id="level.2"></a>
## Level 2

<a id="type.spellings.Clock"></a>
### Clock

**MACRO** derive: Serialize

```
/// A clock with an optional alarm.
struct Clock : Display
{
    /// When the alarm rings; see `core::option::Option`.
    alarm: Option<u8>,

    label: String,

    snooze: Option<u8>,

    step: core::time::Duration,
}
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.spellings.bare.Label"></a>
### bare::Label

```
/// A label that shows itself through the standard Display.
struct Label : Display;
```

<a id="type.spellings.Duration"></a>
### Duration

```
/// The crate's own Duration, so the standard one has no bare spelling to
/// prefer.
struct Duration;
```

<a id="type.spellings.other.Badge"></a>
### other::Badge

```
/// A badge that shows itself through the other crate's Display.
struct Badge
    : other_crate::Display;
```

<a id="type.spellings.Wrapper"></a>
### Wrapper

```
/// A wrapper whose impl names std::string::String only in its self type,
/// which does not count.
struct Wrapper<T>
{
    inner: T,

    is_empty() -> bool;

    len() -> usize;
}
```

<a id="modules"></a>
## Modules

<a id="mod.spellings"></a>
### spellings

- [Clock](#type.spellings.Clock)
- [Duration](#type.spellings.Duration)
- [Timer](#type.spellings.Timer)
- [Wrapper](#type.spellings.Wrapper)

<a id="mod.spellings.bare"></a>
### bare

```
//! The standard Display written bare after a use.
```

- [bare::Label](#type.spellings.bare.Label)

<a id="mod.spellings.other"></a>
### other

```
//! Another crate's Display, which never merges with the standard one.
```

- [other::Badge](#type.spellings.other.Badge)

<a id="spellings"></a>
## Spellings

- `serde::Serialize` is written 2 ways; pick one and search for the rest:
  - `serde::Serialize`, once
  - `Serialize`, once
- `std::collections::BTreeMap` is written 2 ways; prefer `BTreeMap`, which is not used yet:
  - `Map`, once
  - `std::collections::BTreeMap`, once
- `std::fmt::Display` is written 3 ways; prefer `Display`:
  - `Display`, once
  - `fmt::Display`, once
  - `std::fmt::Display`, once
- `std::option::Option` is written 2 ways; prefer `Option`:
  - `Option`, 2 times
  - `std::option::Option`, once
- `std::time::Duration` is written 3 ways; pick one and search for the rest:
  - `::std::time::Duration`, once
  - `core::time::Duration`, once
  - `std::time::Duration`, once
