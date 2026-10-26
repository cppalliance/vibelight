*Updated 2026-01-01T00:00:00Z*

<a id="crate.raw_idents"></a>
# raw_idents

```
//! Raw identifiers.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [r#impl::Inner](#type.raw_idents.impl.Inner)
  - [Keywords](#type.raw_idents.Keywords)
- [Modules](#modules)
  - [raw_idents](#mod.raw_idents)
  - [r#impl](#mod.raw_idents.impl)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [r#fn()](#mod.raw_idents)
- [r#impl](#mod.raw_idents.impl)
- [r#impl::Inner](#type.raw_idents.impl.Inner)
- [Keywords](#type.raw_idents.Keywords)

<a id="unconnected"></a>
## Unconnected types

<a id="type.raw_idents.impl.Inner"></a>
### r#impl::Inner

```
/// Lives in a raw module.
struct Inner;
```

<a id="type.raw_idents.Keywords"></a>
### Keywords

```
/// Raw names in a struct.
struct Keywords
{
    r#match: u8,

    /// A raw field.
    r#type: u8,

    static r#async();

    /// A raw method.
    r#as() -> u8;

private:
    kind: u8,
}
```

<a id="modules"></a>
## Modules

<a id="mod.raw_idents"></a>
### raw_idents

- [Keywords](#type.raw_idents.Keywords)

```
r#fn();
```

<a id="mod.raw_idents.impl"></a>
### r#impl

```
//! A module with a raw name.
```

- [r#impl::Inner](#type.raw_idents.impl.Inner)
