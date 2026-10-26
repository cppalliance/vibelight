*Updated 2026-01-01T00:00:00Z*

<a id="crate.alpha"></a>
# alpha

```
//! Alpha: a crate that names types of its dependencies.
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
  - [Meter](#type.alpha.Meter)
  - [Reads](#type.alpha.Reads)
- [Modules](#modules)
  - [alpha](#mod.alpha)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Meter](#type.alpha.Meter)
- [Reads](#type.alpha.Reads)

<a id="unconnected"></a>
## Unconnected types

<a id="type.alpha.Meter"></a>
### Meter

```
/// A meter that reads a gauge of beta.
struct Meter
{
    /// Through a use of the re-export at beta's root.
    gauge: Gauge,

    /// A crate outside the run stays plain.
    stray: delta::Stray,

    /// Through beta's module path.
    valve: b::parts::Valve,

    /// A crate that no follow glob allows stays plain.
    vendored: gamma::Vendored,
}
```

<a id="type.alpha.Reads"></a>
### Reads

```
/// A trait other crates implement.
trait Reads
{
    read() -> u32;
}
```

<a id="modules"></a>
## Modules

<a id="mod.alpha"></a>
### alpha

- [Meter](#type.alpha.Meter)
- [Reads](#type.alpha.Reads)
