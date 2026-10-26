*Updated 2026-01-01T00:00:00Z*

<a id="crate.member_order"></a>
# member_order

```
//! Member order inside a block.
```

- [Unconnected types](#unconnected)
  - [Based](#type.member_order.Based)
  - [Grouped](#type.member_order.Grouped)
  - [Order](#type.member_order.Order)
  - [Scrambled](#type.member_order.Scrambled)
  - [Triple](#type.member_order.Triple)
- [Modules](#modules)
  - [member_order](#mod.member_order)

**MACROS:** none

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

    Alpha: u8,

    alpha: u8,

    zeta: u8,

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
