*Updated 2026-01-01T00:00:00Z*

<a id="crate.spellings_none"></a>
# spellings_none

```
//! Spellings: each item from outside the crate is written one way, so the
//! outline has no Spellings section. A derive name is never counted, so a
//! derived Debug beside `fmt::Debug` in an impl is still one way.
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
- [spellings_none](#mod.spellings_none)
  - [Line](#type.spellings_none.Line)
  - [Point](#type.spellings_none.Point)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Line](#type.spellings_none.Line)
- [Point](#type.spellings_none.Point)

<a id="mod.spellings_none"></a>
## spellings_none

<a id="type.spellings_none.Line"></a>
### Line

Counts: 2 fields, 1 function

Used by: nothing

```
/// A local type written two ways, which the section leaves out.
pub struct Line {
    pub start: crate::Point,

    pub end: Point,
}

// impl in src/lib.rs
impl Line {
    /// Returns the label; see `std::option::Option`.
    pub fn label<T: Clone>(&self, fallback: T) -> Option<T>;
}

impl fmt::Debug for Line {}
```

<a id="type.spellings_none.Point"></a>
### Point

Counts: 2 fields (1 Option)

Used by: [Line](#type.spellings_none.Line)

```
/// A point whose standard names each have one spelling.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub label: Option<String>,

    pub tags: Vec<String>,
}

impl fmt::Display for Point {}
```
