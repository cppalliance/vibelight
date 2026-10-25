<a id="crate.aliases"></a>
# aliases

```
//! Type aliases.
```

- [Level 1: top-level types](#level.1)
  - [Callback](#type.aliases.Callback)
- [Level 2](#level.2)
  - [Result](#type.aliases.Result)
- [Level 3](#level.3)
  - [Error](#type.aliases.Error)
- [Modules](#modules)
  - [aliases](#mod.aliases)

**MACROS:** 1 derive list
- **MACRO** `derive: Debug`, in [Error](#type.aliases.Error)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.aliases.Callback"></a>
### Callback

```
/// A callback with a bound.
using Callback<'a, T> = Box<dyn Fn(T) -> Result<()> + 'a>;
```

<a id="level.2"></a>
## Level 2

<a id="type.aliases.Result"></a>
### Result

```
/// The crate's result type.
using Result<T> = std::result::Result<T, Error>;
```

<a id="level.3"></a>
## Level 3

<a id="type.aliases.Error"></a>
### Error

**MACRO** derive: Debug

```
/// The crate's error.
class Error
{
    message: String,
}
```

<a id="modules"></a>
## Modules

<a id="mod.aliases"></a>
### aliases

- [Callback](#type.aliases.Callback)
- [Error](#type.aliases.Error)
- [Result](#type.aliases.Result)
