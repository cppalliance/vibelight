*Updated 2026-01-01T00:00:00Z*

<a id="crate.struct_class"></a>
# struct_class

```
//! Struct versus class.
```

- [Unconnected types](#unconnected)
  - [Bare](#type.struct_class.Bare)
  - [Cursor](#type.struct_class.Cursor)
  - [Mixed](#type.struct_class.Mixed)
  - [Toggle](#type.struct_class.Toggle)
  - [Wrapper](#type.struct_class.Wrapper)
- [Modules](#modules)
  - [struct_class](#mod.struct_class)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.struct_class.Bare"></a>
### Bare

```
/// No members at all.
struct Bare
{
}
```

<a id="type.struct_class.Cursor"></a>
### Cursor

```
/// Holds the text the lexer is still reading.
class Cursor
{
    pos: usize,

    text: String,

    static new(text: String) -> Cursor;

    bump() mut;

    into_text(self) -> String;

    peek() -> Option<char>;
}
```

<a id="type.struct_class.Mixed"></a>
### Mixed

```
/// Public and private members.
struct Mixed
{
    open: u8,

private:
    closed: u8,
}
```

<a id="type.struct_class.Toggle"></a>
### Toggle

```
/// An enum with only private methods.
enum Toggle
{
    On,

    Off,

private:
    flip() mut;
}
```

<a id="type.struct_class.Wrapper"></a>
### Wrapper

```
/// A tuple struct with only private fields.
struct Wrapper(u8);
```

<a id="modules"></a>
## Modules

<a id="mod.struct_class"></a>
### struct_class

- [Bare](#type.struct_class.Bare)
- [Cursor](#type.struct_class.Cursor)
- [Mixed](#type.struct_class.Mixed)
- [Toggle](#type.struct_class.Toggle)
- [Wrapper](#type.struct_class.Wrapper)
