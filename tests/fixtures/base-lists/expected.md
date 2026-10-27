*Updated 2026-01-01T00:00:00Z*

<a id="crate.base_lists"></a>
# base_lists

```
//! Base lists: short names, long paths, wrapping, collisions, and supertraits.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations by level, and Levels indexes them: level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Levels](#levels)
- [base_lists](#mod.base_lists)
  - [Error](#type.base_lists.Error)
  - [Holder](#type.base_lists.Holder)
  - [Labeled](#type.base_lists.Labeled)
  - [Local](#type.base_lists.Local)
  - [Mixed](#type.base_lists.Mixed)
  - [Named](#type.base_lists.Named)
  - [Clone](#type.base_lists.Clone)
  - [Bits](#type.base_lists.Bits)
  - [Marker](#type.base_lists.Marker)
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
- [bare](#mod.base_lists.bare)
  - [bare::Imported](#type.base_lists.bare.Imported)
- [custom](#mod.base_lists.custom)
  - [custom::Homemade](#type.base_lists.custom.Homemade)
  - [custom::Display](#type.base_lists.custom.Display)
- [globbed](#mod.base_lists.globbed)
  - [globbed::Starred](#type.base_lists.globbed.Starred)
- [other](#mod.base_lists.other)
  - [other::Foreign](#type.base_lists.other.Foreign)
- [Spellings](#spellings)

**MACROS:** 1 derive list

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

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [custom::Homemade](#type.base_lists.custom.Homemade)
  - [Error](#type.base_lists.Error)
  - [Holder](#type.base_lists.Holder)
  - [Labeled](#type.base_lists.Labeled)
  - [Local](#type.base_lists.Local)
  - [Mixed](#type.base_lists.Mixed)
- Level 2
  - [custom::Display](#type.base_lists.custom.Display)
  - [Named](#type.base_lists.Named)
  - [Clone](#type.base_lists.Clone)
- Unconnected types
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

<a id="mod.base_lists"></a>
## base_lists

<a id="type.base_lists.Error"></a>
### Error

Counts: 1 variant

Used by: nothing

```
/// The crate's own error, whose name collides with std::error::Error.
#[derive(Debug)]
pub enum Error {
    Failed,
}

impl std::error::Error for Error {}
impl Named for Error {}
```

<a id="type.base_lists.Holder"></a>
### Holder

Counts: 1 field

Used by: nothing

```
/// Generics, a where clause, and a trait of this crate.
pub struct Holder<T>
where
    T: Copy,
{
    pub value: T,
}

impl<T: Copy> fmt::Debug for Holder<T> {}
impl<T> Named for Holder<T> where T: Copy {}
```

<a id="type.base_lists.Labeled"></a>
### Labeled

Counts: none

Used by: nothing

```
/// A standard supertrait and a supertrait of this crate.
pub trait Labeled: fmt::Display + Named {}
```

<a id="type.base_lists.Local"></a>
### Local

Counts: none

Used by: nothing

```
/// The crate's own Clone, a trait nothing declares, and a renamed standard
/// trait.
pub struct Local;

impl Clone for Local {}
impl Unknown for Local {}
impl Dbg for Local {}
```

<a id="type.base_lists.Mixed"></a>
### Mixed

Counts: none

Used by: nothing

```
/// A trait of this crate and one of another crate take the comma form.
pub struct Mixed;

impl fmt::Debug for Mixed {}
impl Named for Mixed {}
impl serde::Serialize for Mixed {}
```

<a id="type.base_lists.Named"></a>
### Named

Counts: 1 function

Used by: [Error](#type.base_lists.Error), [Holder](#type.base_lists.Holder), [Labeled](#type.base_lists.Labeled), [Mixed](#type.base_lists.Mixed)

```
/// A trait of this crate.
pub trait Named {
    fn name(&self) -> String;
}
```

<a id="type.base_lists.Clone"></a>
### Clone

Counts: none

Used by: [Local](#type.base_lists.Local)

```
/// A trait of this crate that shares a prelude name.
pub trait Clone {}
```

<a id="type.base_lists.Bits"></a>
### Bits

Counts: 2 fields

Used by: nothing

```
/// A union with standard derives.
#[derive(Copy, Debug)]
pub union Bits {
    pub int: u32,

    pub float: f32,
}
```

<a id="type.base_lists.Marker"></a>
### Marker

Counts: none

Used by: nothing

```
/// A unit struct whose list fits on the head line.
#[derive(Debug, Eq, Hash)]
pub struct Marker;
```

<a id="type.base_lists.Ordered"></a>
### Ordered

Counts: 1 field

Used by: nothing

```
/// Standard derives that are not special members, and comparisons with
/// another type written by hand. The crate's own Clone trait never sits on
/// the shared line, so the standard Clone keeps its short name.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ordered(u8);

impl PartialEq<u8> for Ordered {}
impl PartialOrd<u8> for Ordered {}
```

<a id="type.base_lists.Plain"></a>
### Plain

Counts: none

Used by: nothing

```
/// Standard supertraits and a lifetime bound.
pub trait Plain: Send + Sync + 'static {}
```

<a id="type.base_lists.Renamed"></a>
### Renamed

Counts: none

Used by: nothing

```
/// The standard Display through a renaming use.
pub struct Renamed;

impl D for Renamed {}
```

<a id="type.base_lists.Run"></a>
### Run

Counts: 1 field

Used by: nothing

```
/// Standard traits only, so the list stays on the head line.
pub struct Run {
    pub id: u32,
}

impl fmt::Debug for Run {}
impl fmt::Display for Run {}
```

<a id="type.base_lists.RunError"></a>
### RunError

Counts: 1 field

Used by: nothing

```
/// The crate's Error makes std::error::Error keep its full path.
pub struct RunError {
    pub reason: String,
}

impl fmt::Debug for RunError {}
impl fmt::Display for RunError {}
impl std::error::Error for RunError {}
```

<a id="type.base_lists.Secret"></a>
### Secret

Counts: 1 field

Used by: nothing

```
/// A class with a standard trait.
pub struct Secret {
    value: u8,
}

impl fmt::Display for Secret {}
```

<a id="type.base_lists.Sink"></a>
### Sink

Counts: none

Used by: nothing

```
/// Two traits named Write keep their full paths.
pub struct Sink;

impl fmt::Write for Sink {}
impl io::Write for Sink {}
```

<a id="type.base_lists.Spelled"></a>
### Spelled

Counts: none

Used by: nothing

```
/// The standard Display written in full, with no use.
pub struct Spelled;

impl std::fmt::Display for Spelled {}
```

<a id="type.base_lists.Wide"></a>
### Wide

Counts: 1 field

Used by: nothing

```
/// So many standard traits that the list wraps.
#[derive(Copy, Debug, Eq, Hash, Ord)]
pub struct Wide(u64);

impl fmt::Binary for Wide {}
impl fmt::Display for Wide {}
impl fmt::LowerExp for Wide {}
impl fmt::LowerHex for Wide {}
impl fmt::Octal for Wide {}
impl fmt::Pointer for Wide {}
impl fmt::UpperExp for Wide {}
impl fmt::UpperHex for Wide {}
```

<a id="type.base_lists.Wrapped"></a>
### Wrapped

Counts: 1 field

Used by: nothing

**MACRO** derive: From

```
/// A bare derive of From comes from another crate, so it stays on the mark.
#[derive(Debug, From)]
pub struct Wrapped(u8);
```

<a id="mod.base_lists.bare"></a>
## bare

```
//! The standard Display after a use of the trait itself.
```

<a id="type.base_lists.bare.Imported"></a>
### bare::Imported

Counts: none

Used by: nothing

```
/// Display through the use, which is the standard one.
pub struct Imported;

impl Display for Imported {}
```

<a id="mod.base_lists.custom"></a>
## custom

```
//! A trait of this crate named Display.
```

<a id="type.base_lists.custom.Homemade"></a>
### custom::Homemade

Counts: none

Used by: nothing

```
/// The crate's Display, which prints its path on its own line.
pub struct Homemade;

impl Display for Homemade {}
```

<a id="type.base_lists.custom.Display"></a>
### custom::Display

Counts: none

Used by: [custom::Homemade](#type.base_lists.custom.Homemade)

```
/// The crate's own Display.
pub trait Display {}
```

<a id="mod.base_lists.globbed"></a>
## globbed

```
//! A Display that can only come from a glob import.
```

<a id="type.base_lists.globbed.Starred"></a>
### globbed::Starred

Counts: none

Used by: nothing

```
/// A Display from the glob, which prints as written.
pub struct Starred;

impl Display for Starred {}
```

<a id="mod.base_lists.other"></a>
## other

```
//! Another crate's trait that is also named Display.
```

<a id="type.base_lists.other.Foreign"></a>
### other::Foreign

Counts: none

Used by: nothing

```
/// The other crate's Display, which stays apart from the standard one.
pub struct Foreign;

impl Display for Foreign {}
```

<a id="spellings"></a>
## Spellings

- `std::fmt::Debug` is written 2 ways; prefer `Debug`, which is not used yet:
  - `Dbg`, once
  - `fmt::Debug`, 4 times
- `std::fmt::Display` is written 4 ways; pick one and search for the rest:
  - `D`, once
  - `Display`, once
  - `fmt::Display`, 5 times
  - `std::fmt::Display`, once
