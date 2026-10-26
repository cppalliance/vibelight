*Updated 2026-01-01T00:00:00Z*

<a id="crate.member_order_impl"></a>
# member_order_impl

```
//! Members grouped by impl.
```

<a id="legend"></a>
## Legend

- Each type is one C++-style block: a `struct` has a public member, and a `class` has none.
- A type's block holds the members of all its inherent impls, wherever they are written, so a module that holds only impls shows nothing but its docs.
- A `// impl in` line starts the functions one inherent impl adds, in source order, and names the file that holds the impl.
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
  - [Engine](#type.member_order_impl.Engine)
- [Modules](#modules)
  - [member_order_impl](#mod.member_order_impl)
  - [more](#mod.member_order_impl.more)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Engine](#type.member_order_impl.Engine)

<a id="unconnected"></a>
## Unconnected types

<a id="type.member_order_impl.Engine"></a>
### Engine

```
/// A type with methods in two files.
struct Engine
{
    // impl in src/lib.rs
    /// Makes one.
    static new() -> Engine;

    /// Starts it.
    start() mut;

    // impl in src/more.rs
    /// Stops it.
    stop() mut;

    /// Brakes.
    brake() mut;

private:
    speed: u32,

    // impl in src/lib.rs
    /// Checks it.
    check() -> bool;
}
```

<a id="modules"></a>
## Modules

<a id="mod.member_order_impl"></a>
### member_order_impl

- [Engine](#type.member_order_impl.Engine)

<a id="mod.member_order_impl.more"></a>
### more

```
//! More methods.
```
