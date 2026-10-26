*Updated 2026-01-01T00:00:00Z*

<a id="crate.beta"></a>
# beta

```
//! Beta: re-exports a type from one of its modules.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [parts::Gauge](#type.beta.parts.Gauge)
  - [parts::Valve](#type.beta.parts.Valve)
- [Modules](#modules)
  - [beta](#mod.beta)
  - [parts](#mod.beta.parts)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Gauge](#type.beta.parts.Gauge)
- [parts](#mod.beta.parts)
- [parts::Gauge](#type.beta.parts.Gauge)
- [parts::Valve](#type.beta.parts.Valve)

<a id="unconnected"></a>
## Unconnected types

<a id="type.beta.parts.Gauge"></a>
### parts::Gauge

```
/// A gauge.
struct Gauge;
```

<a id="type.beta.parts.Valve"></a>
### parts::Valve

```
/// A valve.
struct Valve;
```

<a id="modules"></a>
## Modules

<a id="mod.beta"></a>
### beta

```
using parts::Gauge;
```

<a id="mod.beta.parts"></a>
### parts

```
//! The parts of a meter.
```

- [parts::Gauge](#type.beta.parts.Gauge)
- [parts::Valve](#type.beta.parts.Valve)
