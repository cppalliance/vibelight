<a id="crate.enums"></a>
# enums

```
//! Enum shapes.
```

- [Unconnected types](#unconnected)
  - [Event](#type.enums.Event)
  - [Level](#type.enums.Level)
  - [Shape](#type.enums.Shape)
- [Modules](#modules)
  - [enums](#mod.enums)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.enums.Event"></a>
### Event

```
/// Struct-like variants that expand.
enum Event
{
    /// Expands because a field has a comment.
    Started
    {
        /// The run identifier.
        id: u64,

        name: String,
    },

    /// Expands because its one-line form is too wide.
    Finished
    {
        identifier: u64,

        summary: String,

        duration_in_milliseconds: u128,
    },
}
```

<a id="type.enums.Level"></a>
### Level

```
/// An enum with methods.
enum Level
{
    Low,

    High,

    /// Returns the highest level.
    static max() -> Level;

private:
    rank() const -> u8;
}
```

<a id="type.enums.Shape"></a>
### Shape

```
/// Every variant shape.
enum Shape
{
    /// A unit variant.
    Unit,

    /// A tuple variant.
    Tuple(u8, String),

    /// A struct-like variant.
    Named { x: i32, y: i32 },

    /// A variant with a discriminant.
    Valued = 4,
}
```

<a id="modules"></a>
## Modules

<a id="mod.enums"></a>
### enums

- [Event](#type.enums.Event)
- [Level](#type.enums.Level)
- [Shape](#type.enums.Shape)
