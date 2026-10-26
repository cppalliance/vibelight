*Updated 2026-01-01T00:00:00Z*

<a id="crate.visibility"></a>
# visibility

```
//! Visibility sections: public, internal, and private.
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
- [Unconnected types](#unconnected)
  - [Hidden](#type.visibility.Hidden)
  - [Inner](#type.visibility.Inner)
  - [Mixed](#type.visibility.Mixed)
- [Modules](#modules)
  - [visibility](#mod.visibility)
  - [outer](#mod.visibility.outer)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Hidden](#type.visibility.Hidden)
- [Inner](#type.visibility.Inner)
- [Mixed](#type.visibility.Mixed)
- [outer](#mod.visibility.outer)
- [outer::ext_open()](#mod.visibility.outer)
- [outer::open()](#mod.visibility.outer)

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
