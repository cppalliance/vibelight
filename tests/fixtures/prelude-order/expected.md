*Updated 2026-01-01T00:00:00Z*

<a id="crate.prelude_order"></a>
# prelude_order

```
//! Prelude order: a bare name is the prelude's unless the module declares
//! it, imports it, or a glob brings it in.
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
  - [own::Owned](#type.prelude_order.own.Owned)
  - [starred::Starred](#type.prelude_order.starred.Starred)
  - [alpha::Result](#type.prelude_order.alpha.Result)
  - [beta::Result](#type.prelude_order.beta.Result)
- [Level 2](#level.2)
  - [own::Result](#type.prelude_order.own.Result)
  - [alpha::AlphaError](#type.prelude_order.alpha.AlphaError)
  - [beta::BetaError](#type.prelude_order.beta.BetaError)
- [Level 3](#level.3)
  - [own::OwnError](#type.prelude_order.own.OwnError)
- [Unconnected types](#unconnected)
  - [plain::Report](#type.prelude_order.plain.Report)
- [Modules](#modules)
  - [prelude_order](#mod.prelude_order)
  - [alpha](#mod.prelude_order.alpha)
  - [beta](#mod.prelude_order.beta)
  - [own](#mod.prelude_order.own)
  - [plain](#mod.prelude_order.plain)
  - [starred](#mod.prelude_order.starred)
- [Spellings](#spellings)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [alpha](#mod.prelude_order.alpha)
- [alpha::AlphaError](#type.prelude_order.alpha.AlphaError)
- [beta](#mod.prelude_order.beta)
- [beta::BetaError](#type.prelude_order.beta.BetaError)
- [own](#mod.prelude_order.own)
- [own::Owned](#type.prelude_order.own.Owned)
- [own::OwnError](#type.prelude_order.own.OwnError)
- [own::Result](#type.prelude_order.own.Result)
- [plain](#mod.prelude_order.plain)
- [plain::Report](#type.prelude_order.plain.Report)
- [starred](#mod.prelude_order.starred)
- [starred::Starred](#type.prelude_order.starred.Starred)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.prelude_order.own.Owned"></a>
### own::Owned

```
/// A value in this module's Result.
struct Owned
{
    value: Result<u8>,
}
```

<a id="type.prelude_order.starred.Starred"></a>
### starred::Starred

```
/// A value in the Result that the glob brings in.
struct Starred
{
    value: Result<u16>,
}
```

<a id="type.prelude_order.alpha.Result"></a>
### alpha::Result

```
/// The results of alpha.
// private
using Result<T> = std::result::Result<T, AlphaError>;
```

<a id="type.prelude_order.beta.Result"></a>
### beta::Result

```
/// The results of beta.
// private
using Result<T> = std::result::Result<T, BetaError>;
```

<a id="level.2"></a>
## Level 2

<a id="type.prelude_order.own.Result"></a>
### own::Result

```
/// The results of this module.
using Result<T> = std::result::Result<T, OwnError>;
```

<a id="type.prelude_order.alpha.AlphaError"></a>
### alpha::AlphaError

```
/// Alpha's error.
struct AlphaError;
```

<a id="type.prelude_order.beta.BetaError"></a>
### beta::BetaError

```
/// Beta's error.
struct BetaError;
```

<a id="level.3"></a>
## Level 3

<a id="type.prelude_order.own.OwnError"></a>
### own::OwnError

```
/// The error of this module.
struct OwnError;
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.prelude_order.plain.Report"></a>
### plain::Report

```
/// A report whose outcome is the standard Result.
struct Report
{
    outcome: Result<u8, String>,
}
```

<a id="modules"></a>
## Modules

<a id="mod.prelude_order"></a>
### prelude_order

<a id="mod.prelude_order.alpha"></a>
### alpha

```
//! The first module with a private Result alias.
```

- [alpha::AlphaError](#type.prelude_order.alpha.AlphaError)
- [alpha::Result](#type.prelude_order.alpha.Result)

<a id="mod.prelude_order.beta"></a>
### beta

```
//! The second module with a private Result alias.
```

- [beta::BetaError](#type.prelude_order.beta.BetaError)
- [beta::Result](#type.prelude_order.beta.Result)

<a id="mod.prelude_order.own"></a>
### own

```
//! A module that declares its own Result and uses it.
```

- [own::Owned](#type.prelude_order.own.Owned)
- [own::OwnError](#type.prelude_order.own.OwnError)
- [own::Result](#type.prelude_order.own.Result)

<a id="mod.prelude_order.plain"></a>
### plain

```
//! A module that names the prelude Result and imports nothing.
```

- [plain::Report](#type.prelude_order.plain.Report)

<a id="mod.prelude_order.starred"></a>
### starred

```
//! A module whose glob import brings in the Result of own.
```

- [starred::Starred](#type.prelude_order.starred.Starred)

<a id="spellings"></a>
## Spellings

- `std::result::Result` is written 2 ways; pick one and search for the rest:
  - `Result`, once
  - `std::result::Result`, 3 times
