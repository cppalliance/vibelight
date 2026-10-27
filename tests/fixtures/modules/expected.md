*Updated 2026-01-01T00:00:00Z*

<a id="crate.modules"></a>
# modules

```
//! Module layouts.
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
- [modules](#mod.modules)
- [files](#mod.modules.files)
  - [files::InFile](#type.modules.files.InFile)
- [inline](#mod.modules.inline)
  - [inline::Inside](#type.modules.inline.Inside)
- [inline::deeper](#mod.modules.inline.deeper)
  - [inline::deeper::Deepest](#type.modules.inline.deeper.Deepest)
- [nested](#mod.modules.nested)
- [nested::child](#mod.modules.nested.child)
- [only_modules](#mod.modules.only_modules)
- [only_modules::a](#mod.modules.only_modules.a)
  - [only_modules::a::A](#type.modules.only_modules.a.A)
- [only_modules::b](#mod.modules.only_modules.b)
  - [only_modules::b::B](#type.modules.only_modules.b.B)
- [renamed](#mod.modules.renamed)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [files](#mod.modules.files)
- [files::InFile](#type.modules.files.InFile)
- [inline](#mod.modules.inline)
- [inline::deeper](#mod.modules.inline.deeper)
- [inline::deeper::Deepest](#type.modules.inline.deeper.Deepest)
- [inline::Inside](#type.modules.inline.Inside)

<a id="mod.modules"></a>
## modules

<a id="mod.modules.files"></a>
## files

```
//! A file module.
```

<a id="type.modules.files.InFile"></a>
### files::InFile

Counts: none

Used by: nothing

```
/// A type in a file module.
pub struct InFile;
```

<a id="mod.modules.inline"></a>
## inline

```
//! An inline module.
//! Inner docs of the inline module.
```

<a id="type.modules.inline.Inside"></a>
### inline::Inside

Counts: none

Used by: nothing

```
/// A type inside the inline module.
pub struct Inside;
```

<a id="mod.modules.inline.deeper"></a>
## inline::deeper

<a id="type.modules.inline.deeper.Deepest"></a>
### inline::deeper::Deepest

Counts: none

Used by: nothing

```
/// The deepest type.
pub struct Deepest;
```

<a id="mod.modules.nested"></a>
## nested

```
//! A module in a mod.rs file.
```

<a id="mod.modules.nested.child"></a>
## nested::child

```
//! A child of a mod.rs module.
```

```
/// A function in a nested file.
pub fn child_fn();
```

<a id="mod.modules.only_modules"></a>
## only_modules

<a id="mod.modules.only_modules.a"></a>
## only_modules::a

<a id="type.modules.only_modules.a.A"></a>
### only_modules::a::A

Counts: none

Used by: nothing

```
/// The first leaf type.
pub struct A;
```

<a id="mod.modules.only_modules.b"></a>
## only_modules::b

<a id="type.modules.only_modules.b.B"></a>
### only_modules::b::B

Counts: none

Used by: nothing

```
/// The second leaf type.
pub struct B;
```

<a id="mod.modules.renamed"></a>
## renamed

```
//! Loaded through a path attribute.
```

```
/// Shows the file was loaded.
pub const LOADED: bool = true;
```
