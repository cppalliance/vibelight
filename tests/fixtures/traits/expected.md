*Updated 2026-01-01T00:00:00Z*

<a id="crate.traits"></a>
# traits

```
//! Traits.
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
  - [Convert](#type.traits.Convert)
  - [Lender](#type.traits.Lender)
  - [Observer](#type.traits.Observer)
  - [Zeroable](#type.traits.Zeroable)
- [Modules](#modules)
  - [traits](#mod.traits)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Convert](#type.traits.Convert)
- [Lender](#type.traits.Lender)
- [Observer](#type.traits.Observer)
- [Zeroable](#type.traits.Zeroable)

<a id="unconnected"></a>
## Unconnected types

<a id="type.traits.Convert"></a>
### Convert

```
/// A generic trait with a where clause.
trait Convert<T> where T: Clone
{
    /// Converts a value.
    convert(value: T) -> T;
}
```

<a id="type.traits.Lender"></a>
### Lender

```
/// A trait with a generic associated type.
trait Lender
{
    /// The borrowed item.
    using Loan<'a> where Self: 'a;

    /// Lends an item.
    lend() mut -> Self::Loan<'_>;
}
```

<a id="type.traits.Observer"></a>
### Observer

```
/// Watches a run.
trait Observer : Send, Sync
{
    /// The item type.
    using Item: Clone;

    /// The most items kept.
    static constexpr LIMIT: usize = 64;

    /// Makes a default observer.
    static make() -> Self where Self: Sized;

    /// Ends the run.
    finish(self) -> u32 where Self: Sized;

    /// Clears the buffer.
    flush() mut;

    /// Records one event.
    record(event: str&);
}
```

<a id="type.traits.Zeroable"></a>
### Zeroable

```
/// A marker that is unsafe to implement.
unsafe trait Zeroable
{
}
```

<a id="modules"></a>
## Modules

<a id="mod.traits"></a>
### traits

- [Convert](#type.traits.Convert)
- [Lender](#type.traits.Lender)
- [Observer](#type.traits.Observer)
- [Zeroable](#type.traits.Zeroable)
