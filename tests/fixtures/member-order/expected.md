*Updated 2026-01-01T00:00:00Z*

<a id="crate.member_order"></a>
# member_order

```
//! Member order inside a block.
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
  - [Based](#type.member_order.Based)
  - [Grouped](#type.member_order.Grouped)
  - [Order](#type.member_order.Order)
  - [Scrambled](#type.member_order.Scrambled)
  - [Triple](#type.member_order.Triple)
- [Modules](#modules)
  - [member_order](#mod.member_order)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Based](#type.member_order.Based)
- [Grouped](#type.member_order.Grouped)
- [Order](#type.member_order.Order)
- [Scrambled](#type.member_order.Scrambled)
- [Triple](#type.member_order.Triple)

<a id="unconnected"></a>
## Unconnected types

<a id="type.member_order.Based"></a>
### Based

```
/// The base list is sorted.
struct Based : AsRef<str>, Clone, Debug
{
    Based();
}
```

<a id="type.member_order.Grouped"></a>
### Grouped

```
/// Trait items in scrambled order.
trait Grouped
{
    using Out;

    static constexpr LIMIT: u8;

    static build() -> Self;

    member();
}
```

<a id="type.member_order.Order"></a>
### Order

```
/// Variants keep their declared order.
enum Order
{
    Zulu,

    Alpha,

    Mike,
}
```

<a id="type.member_order.Scrambled"></a>
### Scrambled

```
/// Members in scrambled order.
struct Scrambled
{
    static constexpr MAX: u8 = 9;

    zeta: u8,

    alpha: u8,

    Alpha: u8,

    static make() -> Scrambled;

    Apply();

    apply();

    zed();

private:
    beta: u8,

    static hidden();
}
```

<a id="type.member_order.Triple"></a>
### Triple

```
/// Tuple fields keep their declared order.
struct Triple(u8, String, bool);
```

<a id="modules"></a>
## Modules

<a id="mod.member_order"></a>
### member_order

- [Based](#type.member_order.Based)
- [Grouped](#type.member_order.Grouped)
- [Order](#type.member_order.Order)
- [Scrambled](#type.member_order.Scrambled)
- [Triple](#type.member_order.Triple)
