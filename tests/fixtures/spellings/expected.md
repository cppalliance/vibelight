*Updated 2026-01-01T00:00:00Z*

<a id="crate.spellings"></a>
# spellings

```
//! Spellings: items from outside the crate written in more than one way.
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
- [spellings](#mod.spellings)
  - [Timer](#type.spellings.Timer)
  - [Clock](#type.spellings.Clock)
  - [Duration](#type.spellings.Duration)
  - [Wrapper](#type.spellings.Wrapper)
- [bare](#mod.spellings.bare)
  - [bare::Label](#type.spellings.bare.Label)
- [debug](#mod.spellings.debug)
  - [debug::Dial](#type.spellings.debug.Dial)
  - [debug::Gauge](#type.spellings.debug.Gauge)
  - [debug::Knob](#type.spellings.debug.Knob)
- [other](#mod.spellings.other)
  - [other::Badge](#type.spellings.other.Badge)
  - [other::Pin](#type.spellings.other.Pin)
- [Spellings](#spellings)

**MACROS:** 2 derive lists

<a id="surface"></a>
## Public surface

- [bare](#mod.spellings.bare)
- [bare::Label](#type.spellings.bare.Label)
- [Clock](#type.spellings.Clock)
- [debug](#mod.spellings.debug)
- [debug::Dial](#type.spellings.debug.Dial)
- [debug::Gauge](#type.spellings.debug.Gauge)
- [debug::Knob](#type.spellings.debug.Knob)
- [Duration](#type.spellings.Duration)
- [other](#mod.spellings.other)
- [other::Badge](#type.spellings.other.Badge)
- [other::Pin](#type.spellings.other.Pin)
- [Timer](#type.spellings.Timer)
- [Wrapper](#type.spellings.Wrapper)

<a id="mod.spellings"></a>
## spellings

<a id="type.spellings.Timer"></a>
### Timer

Counts: 3 fields, 2 functions

Used by: nothing

**MACRO** derive: serde::Serialize

```
/// A timer that names its laps.
#[derive(serde::Serialize)]
pub struct Timer {
    pub laps: Map<u32, String>,

    pub names: std::collections::BTreeMap<u32, String>,

    pub period: ::std::time::Duration,
}

// impl in src/lib.rs
impl Timer {
    pub fn next(&self, clock: &crate::Clock) -> std::option::Option<Clock>;

    pub fn wait(&self, extra: std::time::Duration) -> Vec<String>;
}

impl std::fmt::Display for Timer {}
```

<a id="type.spellings.Clock"></a>
### Clock

Counts: 4 fields (2 Option)

Used by: [Timer](#type.spellings.Timer)

**MACRO** derive: Serialize

```
/// A clock with an optional alarm.
#[derive(Serialize)]
pub struct Clock {
    /// When the alarm rings; see `core::option::Option`.
    pub alarm: Option<u8>,

    pub label: String,

    pub snooze: Option<u8>,

    pub step: core::time::Duration,
}

impl fmt::Display for Clock {}
```

<a id="type.spellings.Duration"></a>
### Duration

Counts: none

Used by: nothing

```
/// The crate's own Duration, so the standard one has no bare spelling to
/// prefer.
pub struct Duration;
```

<a id="type.spellings.Wrapper"></a>
### Wrapper

Counts: 1 field, 2 functions

Used by: nothing

```
/// A wrapper whose impl names std::string::String only in its self type,
/// which does not count.
pub struct Wrapper<T> {
    pub inner: T,
}

// impl in src/lib.rs
impl Wrapper<std::string::String> {
    pub fn len(&self) -> usize;

    pub fn is_empty(&self) -> bool;
}
```

<a id="mod.spellings.bare"></a>
## bare

```
//! The standard Display written bare after a use.
```

<a id="type.spellings.bare.Label"></a>
### bare::Label

Counts: none

Used by: nothing

```
/// A label that shows itself through the standard Display.
pub struct Label;

impl Display for Label {}
```

<a id="mod.spellings.debug"></a>
## debug

```
//! Debug by derive and by hand. A derive name is never counted.
```

<a id="type.spellings.debug.Dial"></a>
### debug::Dial

Counts: none

Used by: nothing

```
/// A dial that writes Debug through the fmt module.
pub struct Dial;

impl fmt::Debug for Dial {}
```

<a id="type.spellings.debug.Gauge"></a>
### debug::Gauge

Counts: none

Used by: nothing

```
/// A gauge that derives Debug, which adds no spelling.
#[derive(Debug)]
pub struct Gauge;
```

<a id="type.spellings.debug.Knob"></a>
### debug::Knob

Counts: none

Used by: nothing

```
/// A knob that writes Debug with its full path.
pub struct Knob;

impl std::fmt::Debug for Knob {}
```

<a id="mod.spellings.other"></a>
## other

```
//! Another crate's Display, which never merges with the standard one.
```

<a id="type.spellings.other.Badge"></a>
### other::Badge

Counts: none

Used by: nothing

```
/// A badge that shows itself through the other crate's Display.
pub struct Badge;

impl Display for Badge {}
```

<a id="type.spellings.other.Pin"></a>
### other::Pin

Counts: none

Used by: nothing

```
/// A pin that names the other crate's Display by its full path.
pub struct Pin;

impl other_crate::Display for Pin {}
```

<a id="spellings"></a>
## Spellings

- `other_crate::Display` is written 2 ways; pick one and search for the rest:
  - `Display`, once
  - `other_crate::Display`, once
- `std::collections::BTreeMap` is written 2 ways; prefer `BTreeMap`, which is not used yet:
  - `Map`, once
  - `std::collections::BTreeMap`, once
- `std::fmt::Debug` is written 2 ways; prefer `Debug`, which is not used yet:
  - `fmt::Debug`, once
  - `std::fmt::Debug`, once
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
