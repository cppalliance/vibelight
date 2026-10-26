*Updated 2026-01-01T00:00:00Z*

<a id="crate.alpha"></a>
# alpha

```
//! Alpha: a crate that names types of its dependencies.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Meter](#type.alpha.Meter)
  - [Reads](#type.alpha.Reads)
- [Modules](#modules)
  - [alpha](#mod.alpha)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Meter](#type.alpha.Meter)
- [Reads](#type.alpha.Reads)

<a id="unconnected"></a>
## Unconnected types

<a id="type.alpha.Meter"></a>
### Meter

```
/// A meter that reads a gauge of beta.
struct Meter
{
    /// Through a use of the re-export at beta's root.
    gauge: Gauge,

    /// A crate outside the run stays plain.
    stray: delta::Stray,

    /// Through beta's module path.
    valve: b::parts::Valve,

    /// A crate that no follow glob allows stays plain.
    vendored: gamma::Vendored,
}
```

<a id="type.alpha.Reads"></a>
### Reads

```
/// A trait other crates implement.
trait Reads
{
    read() -> u32;
}
```

<a id="modules"></a>
## Modules

<a id="mod.alpha"></a>
### alpha

- [Meter](#type.alpha.Meter)
- [Reads](#type.alpha.Reads)
