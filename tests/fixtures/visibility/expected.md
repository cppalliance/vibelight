*Updated 2026-01-01T00:00:00Z*

<a id="crate.visibility"></a>
# visibility

```
//! Visibility sections: public, internal, and private.
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
- [visibility](#mod.visibility)
  - [Count](#type.visibility.Count)
  - [Hidden](#type.visibility.Hidden)
  - [Inner](#type.visibility.Inner)
  - [Mixed](#type.visibility.Mixed)
  - [Mode](#type.visibility.Mode)
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

<a id="levels"></a>
## Levels

- Unconnected types
  - [Count](#type.visibility.Count)
  - [Hidden](#type.visibility.Hidden)
  - [Inner](#type.visibility.Inner)
  - [Mixed](#type.visibility.Mixed)
  - [Mode](#type.visibility.Mode)

<a id="mod.visibility"></a>
## visibility

<a id="type.visibility.Count"></a>
### Count

Counts: none

Used by: nothing

```
/// A type alias seen only here.
type Count = u32;
```

<a id="type.visibility.Hidden"></a>
### Hidden

Counts: 1 field

Used by: nothing

```
/// A class with private members only.
pub struct Hidden {
    value: u8,
}
```

<a id="type.visibility.Inner"></a>
### Inner

Counts: 2 fields

Used by: nothing

```
/// A class with internal and private members.
pub struct Inner {
    /// Seen in the crate.
    pub(crate) id: u32,

    /// Seen only here.
    state: u8,
}
```

<a id="type.visibility.Mixed"></a>
### Mixed

Counts: 3 fields, 4 functions

Used by: nothing

```
/// A struct with every section.
pub struct Mixed {
    /// A public field.
    pub name: String,

    /// An internal field.
    pub(crate) size: usize,

    /// A private field.
    secret: u8,
}

// impl in src/lib.rs
impl Mixed {
    /// Makes one.
    pub fn new() -> Mixed;

    /// Takes the name out.
    pub fn into_name(self) -> String;

    /// Grows it.
    pub(crate) fn grow(&mut self);

    /// Checks it.
    fn check(&self) -> bool;
}
```

<a id="type.visibility.Mode"></a>
### Mode

Counts: 2 variants

Used by: nothing

```
/// An enum seen only in the crate.
#[non_exhaustive]
pub(crate) enum Mode {
    Fast,

    Slow,
}
```

<a id="mod.visibility.outer"></a>
## outer

```
//! Module items in every section.
```

```
/// Visible everywhere.
pub fn open();

extern "C" {
    /// Opens a stream.
    pub fn ext_open() -> i32;
}

/// Visible in the crate, through a path.
pub(crate) const LIMIT: u8 = 4;

/// A shared counter.
pub(crate) static COUNT: u8 = 0;

/// Visible in the parent module.
pub(super) fn parent();

/// Visible in the crate.
pub(crate) fn shared();

extern "C" {
    /// Shares a stream.
    pub(crate) fn ext_share() -> i32;
}

/// Visible only here.
fn hidden();

/// Also visible only here.
pub(self) fn own();
```
