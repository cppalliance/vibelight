*Updated 2026-01-01T00:00:00Z*

<a id="crate.facade"></a>
# facade

```
//! Facade: re-exports the workspace's public types.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Panel](#type.facade.Panel)
- [Modules](#modules)
  - [facade](#mod.facade)

**MACROS:** none

<a id="surface"></a>
## Public surface

- Gauge (beta::Gauge)
- Meter (alpha::Meter)
- [Panel](#type.facade.Panel)
- Reads (alpha::Reads)
- [show()](#mod.facade)

<a id="unconnected"></a>
## Unconnected types

<a id="type.facade.Panel"></a>
### Panel

```
/// A panel that reads through alpha's trait.
struct Panel
    : alpha::Reads
{
    /// A meter from alpha.
    meter: alpha::Meter,
}
```

<a id="modules"></a>
## Modules

<a id="mod.facade"></a>
### facade

- [Panel](#type.facade.Panel)

```
using beta::Gauge;

using alpha::Meter;

using alpha::Reads;

/// Shows any reader.
show<T: alpha::Reads>(reader: T&) -> u32;
```
