<a id="crate.special_members"></a>
# special_members

```
//! Special members: the standard traits with a C++ equivalent.
```

- [Unconnected types](#unconnected)
  - [Counter](#type.special_members.Counter)
  - [Derived](#type.special_members.Derived)
  - [Handle](#type.special_members.Handle)
  - [Mode](#type.special_members.Mode)
  - [Name](#type.special_members.Name)
  - [Scheduler](#type.special_members.Scheduler)
  - [Written](#type.special_members.Written)
- [Modules](#modules)
  - [special_members](#mod.special_members)

**MACROS:** 1 derive list
- **MACRO** `derive: serde::Serialize`, in [Derived](#type.special_members.Derived)

<a id="unconnected"></a>
## Unconnected types

<a id="type.special_members.Counter"></a>
### Counter

```
/// A struct with a private section.
struct Counter
{
    Counter() = default;

    count() -> u32;

private:
    count: u32,
}
```

<a id="type.special_members.Derived"></a>
### Derived

**MACRO** derive: serde::Serialize

```
/// Every derivable special member, and a derive from another crate.
struct Derived : Debug
{
    value: u8,

    Derived() = default;
    Derived(Derived&) = default;
    bool operator==(Derived&) = default;
    auto operator<=>(Derived&) = default;
}
```

<a id="type.special_members.Handle"></a>
### Handle

```
/// A tuple struct with derived and hand-written members.
struct Handle(u32)
{
    Handle() = default;
    Handle(Handle&) = default;
    ~Handle();
}
```

<a id="type.special_members.Mode"></a>
### Mode

```
/// An enum with derived members.
enum Mode : Copy, Debug, Eq
{
    On,

    Off,

    Mode(Mode&) = default;
    bool operator==(Mode&) = default;
}
```

<a id="type.special_members.Name"></a>
### Name

```
/// A name from anything that converts into a String.
struct Name(String) : From<T>;
```

<a id="type.special_members.Scheduler"></a>
### Scheduler

```
/// A class with a hand-written destructor.
class Scheduler
{
    jobs: Vec<u32>,

    ~Scheduler();
}
```

<a id="type.special_members.Written"></a>
### Written

```
/// Every hand-written special member.
struct Written
{
    value: u8,

    Written();
    Written(Written&);
    Written(str&);

    /// Converts from a byte.
    Written(u8&);

    ~Written();
    bool operator==(Written&);
    auto operator<=>(u8&);

    static new() -> Self;

    get() -> u8;
}
```

<a id="modules"></a>
## Modules

<a id="mod.special_members"></a>
### special_members

- [Counter](#type.special_members.Counter)
- [Derived](#type.special_members.Derived)
- [Handle](#type.special_members.Handle)
- [Mode](#type.special_members.Mode)
- [Name](#type.special_members.Name)
- [Scheduler](#type.special_members.Scheduler)
- [Written](#type.special_members.Written)
