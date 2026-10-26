*Updated 2026-01-01T00:00:00Z*

<a id="crate.modules"></a>
# modules

```
//! Module layouts.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- A type's block holds the members of all its inherent impls, wherever they are written, so a module that holds only impls shows nothing but its docs.
- Members sit under `public:`, `internal:` (for `pub(crate)`, `pub(super)`, and `pub(in path)`), and `private:`; the first label is left out when it is the default.
- A `// internal` or `// private` line above a type's header gives the type's own visibility; a public type has none.
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

<a id="surface"></a>
## Public surface

- [files](#mod.modules.files)
- [files::InFile](#type.modules.files.InFile)
- [inline](#mod.modules.inline)
- [inline::deeper](#mod.modules.inline.deeper)
- [inline::deeper::Deepest](#type.modules.inline.deeper.Deepest)
- [inline::Inside](#type.modules.inline.Inside)

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
