<a id="crate.unparsable"></a>
# unparsable

```
//! Constructs the collector cannot read.
```

- [Unconnected types](#unconnected)
  - [Fine](#type.unparsable.Fine)
- [Modules](#modules)
  - [unparsable](#mod.unparsable)
  - [broken](#mod.unparsable.broken)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.unparsable.Fine"></a>
### Fine

```
/// Still collected.
struct Fine
{
    /// Collected normally.
    kept();

    // UNSUPPORTED: verbatim impl item
}
```

**UNSUPPORTED:** verbatim impl item

<a id="modules"></a>
## Modules

<a id="mod.unparsable"></a>
### unparsable

- [Fine](#type.unparsable.Fine)

```
// UNSUPPORTED: verbatim item
```

**UNSUPPORTED:** verbatim item

<a id="mod.unparsable.broken"></a>
### broken

```
// UNSUPPORTED: unparsable file
```

**UNSUPPORTED:** unparsable file
