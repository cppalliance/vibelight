<a id="crate.order_cycles"></a>
# order_cycles

```
//! Cycles at level 1 and below.
```

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
