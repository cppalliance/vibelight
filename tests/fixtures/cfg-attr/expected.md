*Updated 2026-01-01T00:00:00Z*

<a id="crate.cfg_attr"></a>
# cfg_attr

```
//! cfg and cfg_attr.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Plain](#type.cfg_attr.Plain)
  - [Settings](#type.cfg_attr.Settings)
- [Modules](#modules)
  - [cfg_attr](#mod.cfg_attr)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Plain](#type.cfg_attr.Plain)
- [Settings](#type.cfg_attr.Settings)

<a id="unconnected"></a>
## Unconnected types

<a id="type.cfg_attr.Plain"></a>
### Plain

```
/// A test-only cfg_attr is ignored.
struct Plain;
```

<a id="type.cfg_attr.Settings"></a>
### Settings

```
/// Derives through cfg_attr.
struct Settings : Clone, Debug
{
    // cfg: feature = "fast"
    // cfg: unix
    both: u8,

    /// Only with a feature.
    // cfg: feature = "extra"
    extra: u32,

    /// Also gated, with its own cfg.
    // cfg: feature = "methods"
    // cfg: windows
    doubly();

    /// Gated by the impl's cfg.
    // cfg: feature = "methods"
    gated();
}
```

<a id="modules"></a>
## Modules

<a id="mod.cfg_attr"></a>
### cfg_attr

- [Plain](#type.cfg_attr.Plain)
- [Settings](#type.cfg_attr.Settings)
