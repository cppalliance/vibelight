<a id="crate.beta"></a>
# beta

```
//! Beta: re-exports a type from one of its modules.
```

- [Unconnected types](#unconnected)
  - [parts::Gauge](#type.beta.parts.Gauge)
  - [parts::Valve](#type.beta.parts.Valve)
- [Modules](#modules)
  - [beta](#mod.beta)
  - [parts](#mod.beta.parts)

**MACROS:** none

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
