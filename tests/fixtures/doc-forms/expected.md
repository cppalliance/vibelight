*Updated 2026-01-01T00:00:00Z*

<a id="crate.doc_forms"></a>
# doc_forms

```
//! Doc comment forms.
```

- [Public surface](#surface)
- [Unconnected types](#unconnected)
  - [Attribute](#type.doc_forms.Attribute)
  - [Block](#type.doc_forms.Block)
  - [Mixed](#type.doc_forms.Mixed)
  - [Starred](#type.doc_forms.Starred)
- [Modules](#modules)
  - [doc_forms](#mod.doc_forms)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Attribute](#type.doc_forms.Attribute)
- [Block](#type.doc_forms.Block)
- [Mixed](#type.doc_forms.Mixed)
- [Starred](#type.doc_forms.Starred)

<a id="unconnected"></a>
## Unconnected types

<a id="type.doc_forms.Attribute"></a>
### Attribute

```
/// An attribute doc.
struct Attribute;
```

<a id="type.doc_forms.Block"></a>
### Block

```
/// A block doc comment
/// over two lines.
struct Block;
```

<a id="type.doc_forms.Mixed"></a>
### Mixed

```
/// Line one.
/// Line two from an attribute.
struct Mixed;
```

<a id="type.doc_forms.Starred"></a>
### Starred

```
/// A starred block doc.
///
/// With a paragraph.
struct Starred;
```

<a id="modules"></a>
## Modules

<a id="mod.doc_forms"></a>
### doc_forms

- [Attribute](#type.doc_forms.Attribute)
- [Block](#type.doc_forms.Block)
- [Mixed](#type.doc_forms.Mixed)
- [Starred](#type.doc_forms.Starred)
