<a id="crate.extern_block"></a>
# extern_block

```
//! Extern blocks.
```

- [Modules](#modules)
  - [extern_block](#mod.extern_block)

**MACROS:** none

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
