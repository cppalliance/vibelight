*Updated 2026-01-01T00:00:00Z*

<a id="crate.impls"></a>
# impls

```
//! Impl blocks of every kind.
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
- [impls](#mod.impls)
  - [Describe](#type.impls.Describe)
  - [Holder](#type.impls.Holder)
  - [Render](#type.impls.Render)
  - [Widget](#type.impls.Widget)
- [Other impls](#impls)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Describe](#type.impls.Describe)
- [Holder](#type.impls.Holder)
- [Render](#type.impls.Render)
- [Widget](#type.impls.Widget)

<a id="mod.impls"></a>
## impls

<a id="type.impls.Describe"></a>
### Describe

Counts: 1 function

Used by: nothing

```
/// Describes itself.
pub trait Describe {
    /// Returns a description.
    fn describe(&self) -> String;
}
```

<a id="type.impls.Holder"></a>
### Holder

Counts: 1 field, 1 function

Used by: nothing

```
/// A generic holder.
pub struct Holder<'a, T> {
    item: &'a T,
}

// impl in src/lib.rs
impl<'a, T> Holder<'a, T> {
    /// Returns the held item.
    pub fn item(&self) -> &'a T;
}
```

<a id="type.impls.Render"></a>
### Render

Counts: 1 function

Used by: nothing

```
/// Renders itself.
pub trait Render {
    /// Renders to a string.
    fn render(&self) -> String;
}
```

<a id="type.impls.Widget"></a>
### Widget

Counts: 1 field, 2 functions

Used by: nothing

```
/// A type with split impls and trait impls.
pub struct Widget {
    id: u32,
}

// impl in src/lib.rs
impl Widget {
    /// Makes a widget.
    pub fn new(id: u32) -> Widget;
}

// impl in src/lib.rs
impl Widget {
    /// Returns the identifier.
    pub fn id(&self) -> u32;
}

impl fmt::Display for Widget {}
unsafe impl Send for Widget {}
impl !Sync for Widget {}
impl<'a> From<&'a str> for Widget {}
```

<a id="impls"></a>
## Other impls

- `impl<T: fmt::Debug> Describe for T {}`
- `impl Render for (u8, u8) {}`
- `impl Render for [Widget] {}`
- `impl Render for Box<Widget> {}`
- `impl Render for &Widget {}`
