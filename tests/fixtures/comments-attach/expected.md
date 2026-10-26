*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_attach"></a>
# comments_attach

```
//! Where comments attach.
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
- [Unconnected types](#unconnected)
  - [Config](#type.comments_attach.Config)
  - [Holder](#type.comments_attach.Holder)
- [Modules](#modules)
  - [comments_attach](#mod.comments_attach)

**MACROS:** 1 definition
- **MACRO** `macro_rules! noop`, in [comments_attach](#mod.comments_attach)

<a id="surface"></a>
## Public surface

- [Config](#type.comments_attach.Config)
- [Holder](#type.comments_attach.Holder)
- [PATTERN](#mod.comments_attach)

<a id="unconnected"></a>
## Unconnected types

<a id="type.comments_attach.Config"></a>
### Config

```
// Attached to the struct below.
/// Docs for Config.
struct Config
{
    // Directly above the field.
    first: u8,

    // Separated from the field by a blank line.
    second: u8, // Trailing comment on the same line.

    third: u8,

    /// Takes commented parameters.
    static build(
        // The first input.
        first: u8,
        second: u8) -> Config;

    /// Holds a comment inside its type.
    typed(
        /* the count */
        items: Vec<u32>);

    // Nothing follows this comment in the struct.
}
```

<a id="type.comments_attach.Holder"></a>
### Holder

```
/// A field whose type holds a comment.
struct Holder
{
    /* inner */
    items: Vec<u8>,
}
```

<a id="modules"></a>
## Modules

<a id="mod.comments_attach"></a>
### comments_attach

- [Config](#type.comments_attach.Config)
- [Holder](#type.comments_attach.Holder)

```
// MACRO macro_rules! noop

/// Holds a string with comment-like text.
constexpr PATTERN: str& = "// not a comment";
```

**MACRO** macro_rules! noop
