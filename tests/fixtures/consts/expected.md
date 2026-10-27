*Updated 2026-01-01T00:00:00Z*

<a id="crate.consts"></a>
# consts

```
//! Constants and statics.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations by level, and Levels indexes them: level 1 holds the types no other type names, and each type sits one level below the deepest type that names it.
- Test support holds types behind a `cfg` that lists `test` inside `any(...)`, and Unconnected types holds types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Levels](#levels)
- [consts](#mod.consts)
  - [Config](#type.consts.Config)

**MACROS:** 1 invocation

<a id="surface"></a>
## Public surface

- [Config](#type.consts.Config)
- [COUNTER](#mod.consts)
- [GREETING](#mod.consts)
- [LIMIT](#mod.consts)
- [NAME](#mod.consts)
- [STATE](#mod.consts)

<a id="levels"></a>
## Levels

- Unconnected types
  - [Config](#type.consts.Config)

<a id="mod.consts"></a>
## consts

```
/// A long value that does not fit.
pub const GREETING: &str = "a long greeting that will not fit in the outline's eighty column limit";

/// The default limit.
pub const LIMIT: usize = 64;

/// A value with a macro call.
// MACRO concat!
pub const NAME: &str;

/// A counter.
pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A mutable static.
pub static mut STATE: u8 = 0;

const PRIVATE: u8 = 1;
```

**MACRO** concat!

<a id="type.consts.Config"></a>
### Config

Counts: 2 associated items

Used by: nothing

```
/// A type with associated constants.
pub struct Config;

// impl in src/lib.rs
impl Config {
    /// The version.
    pub const VERSION: u32 = 3;

    /// A computed value.
    pub const DOUBLE: u32 = Self::VERSION * 2;
}
```
