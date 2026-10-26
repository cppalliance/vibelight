*Updated 2026-01-01T00:00:00Z*

<a id="crate.consts"></a>
# consts

```
//! Constants and statics.
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
  - [Config](#type.consts.Config)
- [Modules](#modules)
  - [consts](#mod.consts)

**MACROS:** 1 invocation
- **MACRO** `concat!`, in [consts](#mod.consts)

<a id="surface"></a>
## Public surface

- [Config](#type.consts.Config)
- [COUNTER](#mod.consts)
- [GREETING](#mod.consts)
- [LIMIT](#mod.consts)
- [NAME](#mod.consts)
- [STATE](#mod.consts)

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
