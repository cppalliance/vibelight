*Updated 2026-01-01T00:00:00Z*

<a id="crate.references"></a>
# references

```
//! References, raw pointers, and receivers.
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
- [references](#mod.references)
  - [Marker](#type.references.Marker)
  - [Source](#type.references.Source)
  - [Table](#type.references.Table)
  - [Views](#type.references.Views)
- [Other impls](#impls)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [apply()](#mod.references)
- [BYTES](#mod.references)
- [first()](#mod.references)
- [Marker](#type.references.Marker)
- [NAME](#mod.references)
- [pick()](#mod.references)
- [Source](#type.references.Source)
- [Table](#type.references.Table)
- [Views](#type.references.Views)

<a id="levels"></a>
## Levels

- Unconnected types
  - [Marker](#type.references.Marker)
  - [Source](#type.references.Source)
  - [Table](#type.references.Table)
  - [Views](#type.references.Views)

<a id="mod.references"></a>
## references

```
/// The default name.
pub const NAME: &str = "views";

/// Raw bytes behind a reference.
pub static BYTES: &[u8] = &[1, 2];

/// Applies a function to a string.
pub fn apply<F>(text: &mut String, f: F) -> usize
where
    F: Fn(&mut String) -> usize;

/// Finds the first item.
pub fn first<'a>(items: &'a [&'a str]) -> Option<&'a str>;

/// Picks an item with a closure that returns a reference.
pub fn pick<'a, T>(choose: impl Fn(&T) -> &'a T, items: &'a [T]) -> Option<&'a T>;
```

<a id="type.references.Marker"></a>
### Marker

Counts: none

Used by: nothing

```
/// Marks a type.
pub trait Marker {}
```

<a id="type.references.Source"></a>
### Source

Counts: 2 functions

Used by: nothing

```
/// A source of bytes.
pub trait Source {
    fn read(&mut self, buf: &mut [u8]) -> usize;

    fn name(&self) -> &str;
}
```

<a id="type.references.Table"></a>
### Table

Counts: none

Used by: nothing

```
/// A borrowed table.
pub type Table<'a> = &'a [(&'a str, *const u8)];
```

<a id="type.references.Views"></a>
### Views

Counts: 13 fields, 9 functions

Used by: nothing

Fields:

- `shared`: get
- `unique`: set
- `name`: into_name, rename, same
- `items`: none
- `read`: none
- `write`: none
- `shown`: none
- `check`: none
- `callback`: none
- `both`: none
- `wrapped`: none
- `raw_call`: none
- `step`: none

```
/// Borrowed and raw views of one value.
pub struct Views<'a, T> {
    pub shared: &'a T,

    pub unique: &'a mut T,

    pub name: &'static str,

    pub items: &'a [&'a T],

    pub read: *const T,

    pub write: *mut T,

    pub shown: &'a dyn Display,

    pub check: Box<dyn Fn(&str) -> usize>,

    pub callback: &'a dyn Fn(&str) -> usize,

    pub both: &'a (dyn Display + Send),

    pub wrapped: &'a (dyn Fn(&str) -> usize),

    pub raw_call: *const dyn Fn() -> bool,

    pub step: &'a fn(u8) -> u8,
}

// impl in src/lib.rs
impl<'a, T> Views<'a, T> {
    pub fn get(&self) -> &T;

    pub fn set(&mut self, value: T);

    pub fn into_name(self) -> &'static str;

    pub fn rename(mut self, name: &'static str) -> Self;

    pub fn same(self: &Self, other: &Self) -> bool;

    pub fn clear(self: &mut Self);

    pub fn poll(self: Pin<&mut Self>);

    pub fn render(&self, out: &mut impl Write, items: &[Box<dyn Display>]);

    pub fn lookup(&mut self, keys: &Vec<&'a str>, table: &std::collections::BTreeMap<String, &'a mut T>) -> Option<&'a mut T>;
}
```

<a id="impls"></a>
## Other impls

- `impl<T> Marker for &T {}`
- `impl Marker for *const u8 {}`
