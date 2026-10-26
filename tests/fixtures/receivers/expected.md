*Updated 2026-01-01T00:00:00Z*

<a id="crate.receivers"></a>
# receivers

```
//! Receivers.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Node](#type.receivers.Node)
- [Modules](#modules)
  - [receivers](#mod.receivers)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [free()](#mod.receivers)
- [Node](#type.receivers.Node)

<a id="unconnected"></a>
## Unconnected types

<a id="type.receivers.Node"></a>
### Node

```
/// Every receiver form.
struct Node
{
    static none();

    boxed(self: Box<Self>);

    consume(self);

    consume_mut(mut self);

    counted(self: Rc<Self>);

    exclusive() mut;

    explicit() -> u8;

    explicit_mut() mut;

    lifetime<'a>() -> str& 'a;

    pinned(self: Pin<Self mut&>);

    shared() -> u8;
}
```

<a id="modules"></a>
## Modules

<a id="mod.receivers"></a>
### receivers

- [Node](#type.receivers.Node)

```
/// A free function never gets static.
free(node: Node&);
```
