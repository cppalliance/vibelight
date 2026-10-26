*Updated 2026-01-01T00:00:00Z*

<a id="crate.extern_block"></a>
# extern_block

```
//! Extern blocks.
```

- [Public surface](#surface)
- [Modules](#modules)
  - [extern_block](#mod.extern_block)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [abs()](#mod.extern_block)
- [errno](#mod.extern_block)
- [exit()](#mod.extern_block)
- [GetTickCount()](#mod.extern_block)

<a id="modules"></a>
## Modules

<a id="mod.extern_block"></a>
### extern_block

```
extern "C"
{
    /// The last error.
    static errno: i32;

    /// Absolute value.
    abs(x: i32) -> i32;

    /// Exits the process.
    exit(code: i32) -> !;
}

extern "system"
{
    GetTickCount() -> u32;
}

private:
extern "C"
{
    secret() -> u8;
}
```
