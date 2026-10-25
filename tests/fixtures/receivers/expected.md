<a id="crate.receivers"></a>
# receivers

```
//! Receivers.
```

- [Unconnected types](#unconnected)
  - [Node](#type.receivers.Node)
- [Modules](#modules)
  - [receivers](#mod.receivers)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.receivers.Node"></a>
### Node

```
/// Every receiver form.
struct Node
{
    static boxed(self: Box<Self>);

    static consume(self);

    static consume_mut(mut self);

    static counted(self: Rc<Self>);

    static none();

    static pinned(self: Pin<&mut Self>);

    exclusive();

    explicit() const -> u8;

    explicit_mut();

    lifetime<'a>() const -> &'a str;

    shared() const -> u8;
}
```

<a id="modules"></a>
## Modules

<a id="mod.receivers"></a>
### receivers

- [Node](#type.receivers.Node)

```
/// A free function never gets static.
free(node: &Node);
```
