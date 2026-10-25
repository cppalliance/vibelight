<a id="crate.cfg_test"></a>
# cfg_test

```
//! Test-only items are skipped.
```

- [Unconnected types](#unconnected)
  - [Kept](#type.cfg_test.Kept)
  - [Mode](#type.cfg_test.Mode)
  - [Probe](#type.cfg_test.Probe)
- [Modules](#modules)
  - [cfg_test](#mod.cfg_test)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.cfg_test.Kept"></a>
### Kept

```
/// Kept.
struct Kept
{
    /// A kept field.
    value: u32,

    /// A kept method.
    get() const -> u32;
}
```

<a id="type.cfg_test.Mode"></a>
### Mode

```
/// A kept enum.
enum Mode
{
    On,

    // cfg: any(test, feature = "debug")
    Debug,
}
```

<a id="type.cfg_test.Probe"></a>
### Probe

```
/// A test-only trait item stays out.
trait Probe
{
    /// Kept.
    check() const;
}
```

<a id="modules"></a>
## Modules

<a id="mod.cfg_test"></a>
### cfg_test

- [Kept](#type.cfg_test.Kept)
- [Mode](#type.cfg_test.Mode)
- [Probe](#type.cfg_test.Probe)

```
// cfg: not(test)
production_only();
```
