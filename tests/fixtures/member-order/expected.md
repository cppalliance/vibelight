*Updated 2026-01-01T00:00:00Z*

<a id="crate.member_order"></a>
# member_order

```
//! Member order inside a block.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.
- Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [member_order](#mod.member_order)
  - [Based](#type.member_order.Based)
  - [Grouped](#type.member_order.Grouped)
  - [Order](#type.member_order.Order)
  - [Scrambled](#type.member_order.Scrambled)
  - [Triple](#type.member_order.Triple)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Based](#type.member_order.Based)
- [Grouped](#type.member_order.Grouped)
- [Order](#type.member_order.Order)
- [Scrambled](#type.member_order.Scrambled)
- [Triple](#type.member_order.Triple)

<a id="mod.member_order"></a>
## member_order

<a id="type.member_order.Based"></a>
### Based

Counts: 1 Default/From/Drop impl

Used by: nothing

```
/// The base list is sorted.
pub struct Based;

impl Default for Based {}
impl std::fmt::Debug for Based {}
impl Clone for Based {}
impl AsRef<str> for Based {}
```

<a id="type.member_order.Grouped"></a>
### Grouped

Counts: 2 associated items, 2 functions

Used by: nothing

```
/// Trait items in scrambled order.
pub trait Grouped {
    fn member(&self);

    fn build() -> Self;

    const LIMIT: u8;

    type Out;
}
```

<a id="type.member_order.Order"></a>
### Order

Counts: 3 variants

Used by: nothing

```
/// Variants keep their declared order.
pub enum Order {
    Zulu,

    Alpha,

    Mike,
}
```

<a id="type.member_order.Scrambled"></a>
### Scrambled

Counts: 4 fields, 1 associated item, 5 functions

Used by: nothing

```
/// Members in scrambled order.
pub struct Scrambled {
    pub zeta: u8,

    pub alpha: u8,

    beta: u8,

    pub Alpha: u8,
}

// impl in src/lib.rs
impl Scrambled {
    pub fn zed(&self);

    pub const MAX: u8 = 9;

    pub fn make() -> Scrambled;

    fn hidden();

    pub fn apply(&self);

    pub fn Apply(&self);
}
```

<a id="type.member_order.Triple"></a>
### Triple

Counts: 3 fields

Used by: nothing

```
/// Tuple fields keep their declared order.
pub struct Triple(pub u8, pub String, pub bool);
```
