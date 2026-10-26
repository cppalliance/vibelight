*Updated 2026-01-01T00:00:00Z*

<a id="crate.macros"></a>
# macros

```
//! Macros of every kind.
```

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
