*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_attach"></a>
# comments_attach

```
//! Where comments attach.
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
- [comments_attach](#mod.comments_attach)
  - [Config](#type.comments_attach.Config)
  - [Holder](#type.comments_attach.Holder)

**MACROS:** 1 definition

<a id="surface"></a>
## Public surface

- [Config](#type.comments_attach.Config)
- [Holder](#type.comments_attach.Holder)
- [PATTERN](#mod.comments_attach)

<a id="levels"></a>
## Levels

- Unconnected types
  - [Config](#type.comments_attach.Config)
  - [Holder](#type.comments_attach.Holder)

<a id="mod.comments_attach"></a>
## comments_attach

```
// MACRO macro_rules! noop

/// Holds a string with comment-like text.
pub const PATTERN: &str = "// not a comment";
```

**MACRO** macro_rules! noop

<a id="type.comments_attach.Config"></a>
### Config

Counts: 3 fields, 2 functions

Used by: nothing

```
// Attached to the struct below.
/// Docs for Config.
pub struct Config {
    // Directly above the field.
    pub first: u8,

    // Separated from the field by a blank line.
    pub second: u8, // Trailing comment on the same line.

    pub third: u8,

    // Nothing follows this comment in the struct.
}

// impl in src/lib.rs
impl Config {
    /// Takes commented parameters.
    pub fn build(
        // The first input.
        first: u8,
        second: u8,
    ) -> Config;

    /// Holds a comment inside its type.
    pub fn typed(
        &self,
        /* the count */
        items: Vec<u32>,
    );
}
```

<a id="type.comments_attach.Holder"></a>
### Holder

Counts: 1 field

Used by: nothing

```
/// A field whose type holds a comment.
pub struct Holder {
    /* inner */
    pub items: Vec<u8>,
}
```
