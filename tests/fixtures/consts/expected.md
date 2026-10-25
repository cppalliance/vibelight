<a id="crate.consts"></a>
# consts

```
//! Constants and statics.
```

- [Unconnected types](#unconnected)
  - [Config](#type.consts.Config)
- [Modules](#modules)
  - [consts](#mod.consts)

**MACROS:** 1 invocation
- **MACRO** `concat!`, in [consts](#mod.consts)

<a id="unconnected"></a>
## Unconnected types

<a id="type.consts.Config"></a>
### Config

```
/// A type with associated constants.
struct Config
{
    /// A computed value.
    static constexpr DOUBLE: u32 = Self::VERSION * 2;

    /// The version.
    static constexpr VERSION: u32 = 3;
}
```

<a id="modules"></a>
## Modules

<a id="mod.consts"></a>
### consts

- [Config](#type.consts.Config)

```
/// A long value that does not fit.
constexpr GREETING: str&;

/// The default limit.
constexpr LIMIT: usize = 64;

/// A value with a macro call.
// MACRO concat!
constexpr NAME: str&;

/// A counter.
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A mutable static.
static mut STATE: u8 = 0;

private:
constexpr PRIVATE: u8 = 1;
```

**MACRO** concat!
