*Updated 2026-01-01T00:00:00Z*

<a id="crate.raw_idents"></a>
# raw_idents

```
//! Raw identifiers.
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
    /// A raw field.
    r#type: u8,

    r#match: u8,

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
