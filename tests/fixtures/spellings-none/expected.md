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

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- A type's block holds the members of all its inherent impls, wherever they are written, so a module that holds only impls shows nothing but its docs.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A `// internal` or `// private` line above a type's header gives the type's own visibility; a public type has none.
- A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.
- References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.
- `constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.
- `Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.
- A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top gathers them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [Line](#type.spellings_none.Line)
- [Level 2](#level.2)
  - [Point](#type.spellings_none.Point)
- [Modules](#modules)
  - [spellings_none](#mod.spellings_none)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Line](#type.spellings_none.Line)
- [Point](#type.spellings_none.Point)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.spellings_none.Line"></a>
### Line

```
/// A local type written two ways, which the section leaves out.
struct Line : Debug
{
    start: crate::Point,

    end: Point,

    /// Returns the label; see `std::option::Option`.
    label<T: Clone>(fallback: T) -> Option<T>;
}
```

<a id="level.2"></a>
## Level 2

<a id="type.spellings_none.Point"></a>
### Point

```
/// A point whose standard names each have one spelling.
struct Point : Clone, Debug, Display, PartialEq
{
    label: Option<String>,

    tags: Vec<String>,
}
```

<a id="modules"></a>
## Modules

<a id="mod.spellings_none"></a>
### spellings_none

- [Line](#type.spellings_none.Line)
- [Point](#type.spellings_none.Point)
