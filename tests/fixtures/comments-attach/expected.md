*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_attach"></a>
# comments_attach

```
//! Where comments attach.
```

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
