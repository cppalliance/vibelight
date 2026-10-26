*Updated 2026-01-01T00:00:00Z*

<a id="crate.profile"></a>
# profile

```
//! A crate outlined with the analysis profile.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- A type's block holds the members of all its inherent impls, wherever they are written, so a module that holds only impls shows nothing but its docs.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A `// internal` or `// private` line above a type's header gives the type's own visibility; a public type has none.
- A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.
- References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.
- `constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.
- `Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.
- A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations by level, and Levels indexes them: level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, special members, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Levels](#levels)
- [profile](#mod.profile)
  - [Canvas](#type.profile.Canvas)
- [shapes](#mod.profile.shapes)
  - [shapes::Point](#type.profile.shapes.Point)
  - [shapes::Size](#type.profile.shapes.Size)
- [Same shapes](#shapes)

**MACROS:** 1 definition, 1 invocation

<a id="surface"></a>
## Public surface

- [Canvas](#type.profile.Canvas)
- [shapes](#mod.profile.shapes)
- [shapes::Point](#type.profile.shapes.Point)
- [shapes::Size](#type.profile.shapes.Size)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [Canvas](#type.profile.Canvas)
- Level 2
  - [shapes::Point](#type.profile.shapes.Point)
- Unconnected types
  - [shapes::Size](#type.profile.shapes.Size)

<a id="mod.profile"></a>
## profile

```
// MACRO constant!

/// Defines a constant.
// MACRO macro_rules! constant
```

**MACRO** constant!

**MACRO** macro_rules! constant

<a id="type.profile.Canvas"></a>
### Canvas

Counts: 1 field

Used by: nothing

```
/// A canvas.
struct Canvas
{
    /// Its corner.
    corner: shapes::Point,
}
```

<a id="mod.profile.shapes"></a>
## shapes

```
//! Shapes.
```

<a id="type.profile.shapes.Point"></a>
### shapes::Point

Counts: 2 fields

Used by: [Canvas](#type.profile.Canvas)

```
/// A point.
struct Point
{
    x: i32,

    y: i32,
}
```

<a id="type.profile.shapes.Size"></a>
### shapes::Size

Counts: 2 fields

Used by: nothing

```
/// A size.
struct Size
{
    x: i32,

    y: i32,
}
```

<a id="shapes"></a>
## Same shapes

- structs with the same 2 fields: [shapes::Point](#type.profile.shapes.Point), [shapes::Size](#type.profile.shapes.Size)
