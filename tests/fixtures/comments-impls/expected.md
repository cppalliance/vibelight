*Updated 2026-01-01T00:00:00Z*

<a id="crate.comments_impls"></a>
# comments_impls

```
//! Comments on impls and modules.
```

- [Unconnected types](#unconnected)
  - [Engine](#type.comments_impls.Engine)
  - [parts::Part](#type.comments_impls.parts.Part)
- [Other impls](#impls)
- [Modules](#modules)
  - [comments_impls](#mod.comments_impls)
  - [parts](#mod.comments_impls.parts)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.comments_impls.Engine"></a>
### Engine

```
/// The type.
// Above the Clone impl.
/// Above the Display impl.
// Above the inherent impl.
struct Engine : Clone, Display
{
    /// Stays the method's doc comment.
    start();
}
```

<a id="type.comments_impls.parts.Part"></a>
### parts::Part

```
/// A part.
struct Part;
```

<a id="impls"></a>
## Other impls

- `// Above an impl that goes to Other impls.`\
  `implements: Default for Box<Engine>;`

<a id="modules"></a>
## Modules

<a id="mod.comments_impls"></a>
### comments_impls

- [Engine](#type.comments_impls.Engine)

<a id="mod.comments_impls.parts"></a>
### parts

```
// An ordinary comment on a mod line.
//! The outer doc of the module.
//! The inner doc of the module.
```

- [parts::Part](#type.comments_impls.parts.Part)
