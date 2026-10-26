*Updated 2026-01-01T00:00:00Z*

<a id="crate.spellings_none"></a>
# spellings_none

```
//! Spellings: each item from outside the crate is written one way, so the
//! outline has no Spellings section.
```

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
struct Line
{
    end: Point,

    start: crate::Point,

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
struct Point : Clone, Display, PartialEq
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
