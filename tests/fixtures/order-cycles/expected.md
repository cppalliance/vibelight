*Updated 2026-01-01T00:00:00Z*

<a id="crate.order_cycles"></a>
# order_cycles

```
//! Cycles at level 1 and below.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.
- Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [order_cycles](#mod.order_cycles)
  - [Root](#type.order_cycles.Root)
  - [Left](#type.order_cycles.Left)
  - [Right](#type.order_cycles.Right)
- [net](#mod.order_cycles.net)
  - [net::Ping](#type.order_cycles.net.Ping)
  - [net::Pong](#type.order_cycles.net.Pong)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Left](#type.order_cycles.Left)
- [net](#mod.order_cycles.net)
- [net::Ping](#type.order_cycles.net.Ping)
- [net::Pong](#type.order_cycles.net.Pong)
- [Right](#type.order_cycles.Right)
- [Root](#type.order_cycles.Root)

<a id="mod.order_cycles"></a>
## order_cycles

<a id="type.order_cycles.Root"></a>
### Root

Counts: 1 field

Used by: nothing

```
/// Names the cycle below it.
pub struct Root {
    pub ping: net::Ping,
}
```

<a id="type.order_cycles.Left"></a>
### Left

Counts: 1 field (1 Option)

Used by: [Right](#type.order_cycles.Right)

```
/// Half of a cycle that nothing else names.
pub struct Left {
    pub right: Option<Box<Right>>,
}
```

<a id="type.order_cycles.Right"></a>
### Right

Counts: 1 field (1 Option)

Used by: [Left](#type.order_cycles.Left)

```
/// The other half.
pub struct Right {
    pub left: Option<Box<Left>>,
}
```

<a id="mod.order_cycles.net"></a>
## net

```
//! A cycle below level 1.
```

<a id="type.order_cycles.net.Ping"></a>
### net::Ping

Counts: 1 field (1 Option)

Used by: [net::Pong](#type.order_cycles.net.Pong), [Root](#type.order_cycles.Root)

```
/// Names Pong.
pub struct Ping {
    pub pong: Option<Box<Pong>>,
}
```

<a id="type.order_cycles.net.Pong"></a>
### net::Pong

Counts: 1 field (1 Option)

Used by: [net::Ping](#type.order_cycles.net.Ping)

```
/// Names Ping back.
pub struct Pong {
    pub ping: Option<Box<Ping>>,
}
```
