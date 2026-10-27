*Updated 2026-01-01T00:00:00Z*

<a id="crate.functions"></a>
# functions

```
//! Functions and signatures.
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
- [functions](#mod.functions)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [all_qualifiers()](#mod.functions)
- [apply()](#mod.functions)
- [callback()](#mod.functions)
- [merge()](#mod.functions)
- [parse()](#mod.functions)
- [peek()](#mod.functions)
- [render()](#mod.functions)
- [square()](#mod.functions)
- [wait()](#mod.functions)

<a id="mod.functions"></a>
## functions

```
/// Every qualifier at once.
pub const unsafe extern "C" fn all_qualifiers();

/// Uses a higher-ranked bound.
pub fn apply<F>(f: F) -> usize
where
    F: for<'a> Fn(&'a str) -> usize;

/// Exported to C.
pub extern "C" fn callback(code: i32) -> i32;

/// Has a where clause on several lines.
pub fn merge<A, B, C>(a: A, b: B) -> C
where
    A: Into<C>,
    B: Into<C>,
    C: std::ops::Add<Output = C>;

/// A rustfmt-style signature over several lines.
pub fn parse(input: &str, execution: &str) -> (std::result::Result<String, String>, Vec<String>);

/// Reads raw memory.
///
/// # Safety
/// The pointer must be valid.
pub unsafe fn peek(ptr: *const u8) -> u8;

/// Takes impl and dyn traits.
pub fn render(out: &mut impl std::io::Write, items: &[Box<dyn std::fmt::Display>]);

/// Computes at compile time.
pub const fn square(x: u32) -> u32;

/// Waits for input.
pub async fn wait(seconds: u64);

fn private_helper();
```
