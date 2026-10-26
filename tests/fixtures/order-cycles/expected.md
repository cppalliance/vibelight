*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_cycles"></a>
# order_cycles

```
//! Cycles at level 1 and below.
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
- [Level 1: top-level types](#level.1)
  - [Root](#type.order_cycles.Root)
  - [Left](#type.order_cycles.Left)
  - [Right](#type.order_cycles.Right)
- [Level 2](#level.2)
  - [net::Ping](#type.order_cycles.net.Ping)
  - [net::Pong](#type.order_cycles.net.Pong)
- [Modules](#modules)
  - [order_cycles](#mod.order_cycles)
  - [net](#mod.order_cycles.net)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Left](#type.order_cycles.Left)
- [net](#mod.order_cycles.net)
- [net::Ping](#type.order_cycles.net.Ping)
- [net::Pong](#type.order_cycles.net.Pong)
- [Right](#type.order_cycles.Right)
- [Root](#type.order_cycles.Root)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.order_cycles.Root"></a>
### Root

```
/// Names the cycle below it.
struct Root
{
    ping: net::Ping,
}
```

<a id="type.order_cycles.Left"></a>
### Left

```
/// Half of a cycle that nothing else names.
struct Left
{
    right: Option<Box<Right>>,
}
```

<a id="type.order_cycles.Right"></a>
### Right

```
/// The other half.
struct Right
{
    left: Option<Box<Left>>,
}
```

<a id="level.2"></a>
## Level 2

<a id="type.order_cycles.net.Ping"></a>
### net::Ping

```
/// Names Pong.
struct Ping
{
    pong: Option<Box<Pong>>,
}
```

<a id="type.order_cycles.net.Pong"></a>
### net::Pong

```
/// Names Ping back.
struct Pong
{
    ping: Option<Box<Ping>>,
}
```

<a id="modules"></a>
## Modules

<a id="mod.order_cycles"></a>
### order_cycles

- [Left](#type.order_cycles.Left)
- [Right](#type.order_cycles.Right)
- [Root](#type.order_cycles.Root)

<a id="mod.order_cycles.net"></a>
### net

```
//! A cycle below level 1.
```

- [net::Ping](#type.order_cycles.net.Ping)
- [net::Pong](#type.order_cycles.net.Pong)
