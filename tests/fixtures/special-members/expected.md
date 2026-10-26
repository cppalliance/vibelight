*Updated 2026-01-01T00:00:00Z*

<a id="crate.special_members"></a>
# special_members

```
//! Special members: the standard traits that become C++ constructors and
//! destructors.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A member function drops `&self`, shows `&mut self` as a trailing `mut`, and keeps any other receiver, such as `self`, as its first parameter; `static` means no receiver.
- References and pointers put the sigil after the type: `T&` is `&T`, `T mut&` is `&mut T`, `T*` is `*const T`, and `T mut*` is `*mut T`.
- `constexpr` marks a `const fn` or a constant, and `using` marks a type alias, an associated type, or a re-export.
- `Default`, `From<T>`, and `Drop` print as constructors and a destructor; every other trait goes in the base list after the type's name.
- A `// cfg:` line shows a condition, a `[[...]]` line shows an attribute that shapes the API, and a `// impl` line starts an impl block's comments.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top gathers them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- With source links on, a `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
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

<a id="surface"></a>
## Public surface

- [Counter](#type.special_members.Counter)
- [Derived](#type.special_members.Derived)
- [Handle](#type.special_members.Handle)
- [Mode](#type.special_members.Mode)
- [Name](#type.special_members.Name)
- [Scheduler](#type.special_members.Scheduler)
- [Written](#type.special_members.Written)

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
/// The one derivable special member, and a derive from another crate.
struct Derived
{
    value: u8,

    Derived() = default;
}
```

<a id="type.special_members.Handle"></a>
### Handle

```
/// A tuple struct with a derived and a hand-written member.
struct Handle(u32)
{
    Handle() = default;
    ~Handle();
}
```

<a id="type.special_members.Mode"></a>
### Mode

```
/// An enum with a derived default.
enum Mode
{
    On,

    Off,

    Mode() = default;
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
    Written(str&);

    /// Converts from a byte.
    Written(u8&);

    ~Written();

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
