*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_kinds"></a>
# comments_kinds

```
//! Comment kinds and their order.
//!
//! Crate docs print as `//!` lines.
```

- [Unconnected types](#unconnected)
  - [BlockComment](#type.comments_kinds.BlockComment)
  - [DocFirst](#type.comments_kinds.DocFirst)
  - [Fenced](#type.comments_kinds.Fenced)
  - [Linked](#type.comments_kinds.Linked)
  - [PlainFirst](#type.comments_kinds.PlainFirst)
- [Modules](#modules)
  - [comments_kinds](#mod.comments_kinds)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.comments_kinds.BlockComment"></a>
### BlockComment

```
/* A block comment
kept as written. */
struct BlockComment;
```

<a id="type.comments_kinds.DocFirst"></a>
### DocFirst

```
/// A doc comment first.
// Then an ordinary comment.
struct DocFirst;
```

<a id="type.comments_kinds.Fenced"></a>
### Fenced

```
/// Docs with their own code fence:
///
/// ```
/// let x = [1, 2];
/// ```
///
/// And a ```` four-backtick run inside.
struct Fenced;
```

<a id="type.comments_kinds.Linked"></a>
### Linked

```
/// Links: `DocFirst`, the plain one, and `inline code`.
struct Linked;
```

<a id="type.comments_kinds.PlainFirst"></a>
### PlainFirst

```
// An ordinary comment first.
/// Then a doc comment.
struct PlainFirst;
```

<a id="modules"></a>
## Modules

<a id="mod.comments_kinds"></a>
### comments_kinds

- [BlockComment](#type.comments_kinds.BlockComment)
- [DocFirst](#type.comments_kinds.DocFirst)
- [Fenced](#type.comments_kinds.Fenced)
- [Linked](#type.comments_kinds.Linked)
- [PlainFirst](#type.comments_kinds.PlainFirst)

````
/*
```
not code
```
*/
after_block();
````
