*Updated 2026-01-01T00:00:00Z*

<a id="crate.prelude_order"></a>
# prelude_order

```
//! Prelude order: a bare name is the prelude's unless the module declares
//! it, imports it, or a glob brings it in.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.
- Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [prelude_order](#mod.prelude_order)
- [alpha](#mod.prelude_order.alpha)
  - [alpha::Result](#type.prelude_order.alpha.Result)
  - [alpha::AlphaError](#type.prelude_order.alpha.AlphaError)
- [beta](#mod.prelude_order.beta)
  - [beta::Result](#type.prelude_order.beta.Result)
  - [beta::BetaError](#type.prelude_order.beta.BetaError)
- [own](#mod.prelude_order.own)
  - [own::Owned](#type.prelude_order.own.Owned)
  - [own::Result](#type.prelude_order.own.Result)
  - [own::OwnError](#type.prelude_order.own.OwnError)
- [plain](#mod.prelude_order.plain)
  - [plain::Report](#type.prelude_order.plain.Report)
- [starred](#mod.prelude_order.starred)
  - [starred::Starred](#type.prelude_order.starred.Starred)
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

<a id="mod.prelude_order"></a>
## prelude_order

<a id="mod.prelude_order.alpha"></a>
## alpha

```
//! The first module with a private Result alias.
```

<a id="type.prelude_order.alpha.Result"></a>
### alpha::Result

Counts: none

Used by: nothing

```
/// The results of alpha.
type Result<T> = std::result::Result<T, AlphaError>;
```

<a id="type.prelude_order.alpha.AlphaError"></a>
### alpha::AlphaError

Counts: none

Used by: [alpha::Result](#type.prelude_order.alpha.Result)

```
/// Alpha's error.
pub struct AlphaError;
```

<a id="mod.prelude_order.beta"></a>
## beta

```
//! The second module with a private Result alias.
```

<a id="type.prelude_order.beta.Result"></a>
### beta::Result

Counts: none

Used by: nothing

```
/// The results of beta.
type Result<T> = std::result::Result<T, BetaError>;
```

<a id="type.prelude_order.beta.BetaError"></a>
### beta::BetaError

Counts: none

Used by: [beta::Result](#type.prelude_order.beta.Result)

```
/// Beta's error.
pub struct BetaError;
```

<a id="mod.prelude_order.own"></a>
## own

```
//! A module that declares its own Result and uses it.
```

<a id="type.prelude_order.own.Owned"></a>
### own::Owned

Counts: 1 field

Used by: nothing

```
/// A value in this module's Result.
pub struct Owned {
    pub value: Result<u8>,
}
```

<a id="type.prelude_order.own.Result"></a>
### own::Result

Counts: none

Used by: [own::Owned](#type.prelude_order.own.Owned), [starred::Starred](#type.prelude_order.starred.Starred)

```
/// The results of this module.
pub type Result<T> = std::result::Result<T, OwnError>;
```

<a id="type.prelude_order.own.OwnError"></a>
### own::OwnError

Counts: none

Used by: [own::Result](#type.prelude_order.own.Result)

```
/// The error of this module.
pub struct OwnError;
```

<a id="mod.prelude_order.plain"></a>
## plain

```
//! A module that names the prelude Result and imports nothing.
```

<a id="type.prelude_order.plain.Report"></a>
### plain::Report

Counts: 1 field

Used by: nothing

```
/// A report whose outcome is the standard Result.
pub struct Report {
    pub outcome: Result<u8, String>,
}
```

<a id="mod.prelude_order.starred"></a>
## starred

```
//! A module whose glob import brings in the Result of own.
```

<a id="type.prelude_order.starred.Starred"></a>
### starred::Starred

Counts: 1 field

Used by: nothing

```
/// A value in the Result that the glob brings in.
pub struct Starred {
    pub value: Result<u16>,
}
```

<a id="spellings"></a>
## Spellings

- `std::result::Result` is written 2 ways; pick one and search for the rest:
  - `Result`, once
  - `std::result::Result`, 3 times
