*Updated 2026-01-01T00:00:00Z*

<a id="crate.struct_class"></a>
# struct_class

```
//! Struct versus class.
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
- [struct_class](#mod.struct_class)
  - [Bare](#type.struct_class.Bare)
  - [Cursor](#type.struct_class.Cursor)
  - [Mixed](#type.struct_class.Mixed)
  - [Toggle](#type.struct_class.Toggle)
  - [Wrapper](#type.struct_class.Wrapper)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Bare](#type.struct_class.Bare)
- [Cursor](#type.struct_class.Cursor)
- [Mixed](#type.struct_class.Mixed)
- [Toggle](#type.struct_class.Toggle)
- [Wrapper](#type.struct_class.Wrapper)

<a id="mod.struct_class"></a>
## struct_class

<a id="type.struct_class.Bare"></a>
### Bare

Counts: none

Used by: nothing

```
/// No members at all.
pub struct Bare {}
```

<a id="type.struct_class.Cursor"></a>
### Cursor

Counts: 2 fields, 4 functions

Used by: nothing

```
/// Holds the text the lexer is still reading.
pub struct Cursor {
    text: String,

    pos: usize,
}

// impl in src/lib.rs
impl Cursor {
    fn new(text: String) -> Cursor;

    fn peek(&self) -> Option<char>;

    fn bump(&mut self);

    fn into_text(self) -> String;
}
```

<a id="type.struct_class.Mixed"></a>
### Mixed

Counts: 2 fields

Used by: nothing

```
/// Public and private members.
pub struct Mixed {
    pub open: u8,

    closed: u8,
}
```

<a id="type.struct_class.Toggle"></a>
### Toggle

Counts: 2 variants, 1 function

Used by: nothing

```
/// An enum with only private methods.
pub enum Toggle {
    On,

    Off,
}

// impl in src/lib.rs
impl Toggle {
    fn flip(&mut self);
}
```

<a id="type.struct_class.Wrapper"></a>
### Wrapper

Counts: 1 field

Used by: nothing

```
/// A tuple struct with only private fields.
pub struct Wrapper(u8);
```
