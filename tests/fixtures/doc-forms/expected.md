*Updated 2026-01-01T00:00:00Z*

<a id="crate.doc_forms"></a>
# doc_forms

```
//! Doc comment forms.
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
- [doc_forms](#mod.doc_forms)
  - [Attribute](#type.doc_forms.Attribute)
  - [Block](#type.doc_forms.Block)
  - [Mixed](#type.doc_forms.Mixed)
  - [Starred](#type.doc_forms.Starred)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Attribute](#type.doc_forms.Attribute)
- [Block](#type.doc_forms.Block)
- [Mixed](#type.doc_forms.Mixed)
- [Starred](#type.doc_forms.Starred)

<a id="mod.doc_forms"></a>
## doc_forms

<a id="type.doc_forms.Attribute"></a>
### Attribute

Counts: none

Used by: nothing

```
/// An attribute doc.
pub struct Attribute;
```

<a id="type.doc_forms.Block"></a>
### Block

Counts: none

Used by: nothing

```
/// A block doc comment
/// over two lines.
pub struct Block;
```

<a id="type.doc_forms.Mixed"></a>
### Mixed

Counts: none

Used by: nothing

```
/// Line one.
/// Line two from an attribute.
pub struct Mixed;
```

<a id="type.doc_forms.Starred"></a>
### Starred

Counts: none

Used by: nothing

```
/// A starred block doc.
///
/// With a paragraph.
pub struct Starred;
```
