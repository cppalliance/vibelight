*Updated 2026-01-01T00:00:00Z*

<a id="crate.macros"></a>
# macros

```
//! Macros of every kind.
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
  - [Event](#type.macros.Event)
  - [Started](#type.macros.Started)
- [Modules](#modules)
  - [macros](#mod.macros)

**MACROS:** 1 definition, 4 invocations, 3 attribute lists
- **MACRO** `macro_rules! events`, in [macros](#mod.macros)
- **MACRO** `events!`, in [macros](#mod.macros)
- **MACRO** `forward_methods!`, in [Event](#type.macros.Event)
- **MACRO** `lifecycle_constants!`, in [macros](#mod.macros)
- **MACRO** `stringify!`, in [macros](#mod.macros)
- **MACRO** `attribute: serde_like::tag`, in [Event](#type.macros.Event)
- **MACRO** `attribute: tokio::main`, in [macros](#mod.macros)
- **MACRO** `attribute: tracing::instrument`, in [Event](#type.macros.Event)

<a id="surface"></a>
## Public surface

- [Event](#type.macros.Event)
- [main()](#mod.macros)
- [NAME](#mod.macros)
- [Started](#type.macros.Started)

<a id="unconnected"></a>
## Unconnected types

<a id="type.macros.Event"></a>
### Event

**MACRO** attribute: serde_like::tag

```
/// A type with derives and an attribute macro.
struct Event : Clone, Debug, PartialEq
{
    [[serde(rename = "n")]]
    name: String,

    /// Has an attribute macro.
    // MACRO attribute: tracing::instrument
    emit();

    // MACRO forward_methods!
}
```

**MACRO** attribute: tracing::instrument

**MACRO** forward_methods!

<a id="type.macros.Started"></a>
### Started

```
/// Named by the module-level invocations.
struct Started
{
    // MACRO events!
    // MACRO lifecycle_constants!
}
```

**MACRO** events!

**MACRO** lifecycle_constants!

<a id="modules"></a>
## Modules

<a id="mod.macros"></a>
### macros

- [Event](#type.macros.Event)
- [Started](#type.macros.Started)

```
// MACRO events!

/// Defines the events.
// MACRO macro_rules! events

// MACRO lifecycle_constants!

/// A value with a macro.
// MACRO stringify!
constexpr NAME: str&;

/// The entry point.
// MACRO attribute: tokio::main
async main();
```

**MACRO** events!

**MACRO** macro_rules! events

**MACRO** lifecycle_constants!

**MACRO** stringify!

**MACRO** attribute: tokio::main
