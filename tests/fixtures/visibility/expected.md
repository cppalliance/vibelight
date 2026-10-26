*Updated 2026-01-01T00:00:00Z*

<a id="crate.visibility"></a>
# visibility

```
//! Visibility sections: public, internal, and private.
```

- [Unconnected types](#unconnected)
  - [Hidden](#type.visibility.Hidden)
  - [Inner](#type.visibility.Inner)
  - [Mixed](#type.visibility.Mixed)
- [Modules](#modules)
  - [visibility](#mod.visibility)
  - [outer](#mod.visibility.outer)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.visibility.Hidden"></a>
### Hidden

```
/// A class with private members only.
class Hidden
{
    value: u8,
}
```

<a id="type.visibility.Inner"></a>
### Inner

```
/// A class with internal and private members.
class Inner
{
internal:
    /// Seen in the crate.
    id: u32,

private:
    /// Seen only here.
    state: u8,
}
```

<a id="type.visibility.Mixed"></a>
### Mixed

```
/// A struct with every section.
struct Mixed
{
    /// A public field.
    name: String,

    /// Makes one.
    static new() -> Mixed;

    /// Takes the name out.
    into_name(self) -> String;

internal:
    /// An internal field.
    size: usize,

    /// Grows it.
    grow() mut;

private:
    /// A private field.
    secret: u8,

    /// Checks it.
    check() -> bool;
}
```

<a id="modules"></a>
## Modules

<a id="mod.visibility"></a>
### visibility

- [Hidden](#type.visibility.Hidden)
- [Inner](#type.visibility.Inner)
- [Mixed](#type.visibility.Mixed)

<a id="mod.visibility.outer"></a>
### outer

```
//! Module items in every section.
```

```
/// Visible everywhere.
open();

extern "C"
{
    /// Opens a stream.
    ext_open() -> i32;
}

internal:
/// Visible in the crate, through a path.
constexpr LIMIT: u8 = 4;

/// A shared counter.
static COUNT: u8 = 0;

/// Visible in the parent module.
parent();

/// Visible in the crate.
shared();

extern "C"
{
    /// Shares a stream.
    ext_share() -> i32;
}

private:
/// Visible only here.
hidden();

/// Also visible only here.
own();
```
