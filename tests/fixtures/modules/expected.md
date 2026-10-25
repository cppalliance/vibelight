<a id="crate.modules"></a>
# modules

```
//! Module layouts.
```

- [Unconnected types](#unconnected)
  - [files::InFile](#type.modules.files.InFile)
  - [inline::deeper::Deepest](#type.modules.inline.deeper.Deepest)
  - [inline::Inside](#type.modules.inline.Inside)
  - [only_modules::a::A](#type.modules.only_modules.a.A)
  - [only_modules::b::B](#type.modules.only_modules.b.B)
- [Modules](#modules)
  - [modules](#mod.modules)
  - [files](#mod.modules.files)
  - [inline](#mod.modules.inline)
  - [inline::deeper](#mod.modules.inline.deeper)
  - [nested](#mod.modules.nested)
  - [nested::child](#mod.modules.nested.child)
  - [only_modules](#mod.modules.only_modules)
  - [only_modules::a](#mod.modules.only_modules.a)
  - [only_modules::b](#mod.modules.only_modules.b)
  - [renamed](#mod.modules.renamed)

**MACROS:** none

<a id="unconnected"></a>
## Unconnected types

<a id="type.modules.files.InFile"></a>
### files::InFile

```
/// A type in a file module.
struct InFile;
```

<a id="type.modules.inline.deeper.Deepest"></a>
### inline::deeper::Deepest

```
/// The deepest type.
struct Deepest;
```

<a id="type.modules.inline.Inside"></a>
### inline::Inside

```
/// A type inside the inline module.
struct Inside;
```

<a id="type.modules.only_modules.a.A"></a>
### only_modules::a::A

```
/// The first leaf type.
struct A;
```

<a id="type.modules.only_modules.b.B"></a>
### only_modules::b::B

```
/// The second leaf type.
struct B;
```

<a id="modules"></a>
## Modules

<a id="mod.modules"></a>
### modules

<a id="mod.modules.files"></a>
### files

```
//! A file module.
```

- [files::InFile](#type.modules.files.InFile)

<a id="mod.modules.inline"></a>
### inline

```
//! An inline module.
//! Inner docs of the inline module.
```

- [inline::Inside](#type.modules.inline.Inside)

<a id="mod.modules.inline.deeper"></a>
### inline::deeper

- [inline::deeper::Deepest](#type.modules.inline.deeper.Deepest)

<a id="mod.modules.nested"></a>
### nested

```
//! A module in a mod.rs file.
```

<a id="mod.modules.nested.child"></a>
### nested::child

```
//! A child of a mod.rs module.
```

```
/// A function in a nested file.
child_fn();
```

<a id="mod.modules.only_modules"></a>
### only_modules

<a id="mod.modules.only_modules.a"></a>
### only_modules::a

- [only_modules::a::A](#type.modules.only_modules.a.A)

<a id="mod.modules.only_modules.b"></a>
### only_modules::b

- [only_modules::b::B](#type.modules.only_modules.b.B)

<a id="mod.modules.renamed"></a>
### renamed

```
//! Loaded through a path attribute.
```

```
/// Shows the file was loaded.
constexpr LOADED: bool = true;
```
