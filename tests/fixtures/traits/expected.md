*Updated 2026-01-01T00:00:00Z*

<a id="crate.traits"></a>
# traits

```
//! Traits.
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
- [traits](#mod.traits)
  - [Convert](#type.traits.Convert)
  - [Lender](#type.traits.Lender)
  - [Observer](#type.traits.Observer)
  - [Zeroable](#type.traits.Zeroable)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Convert](#type.traits.Convert)
- [Lender](#type.traits.Lender)
- [Observer](#type.traits.Observer)
- [Zeroable](#type.traits.Zeroable)

<a id="mod.traits"></a>
## traits

<a id="type.traits.Convert"></a>
### Convert

Counts: 1 function

Used by: nothing

```
/// A generic trait with a where clause.
pub trait Convert<T>
where
    T: Clone,
{
    /// Converts a value.
    fn convert(&self, value: T) -> T;
}
```

<a id="type.traits.Lender"></a>
### Lender

Counts: 1 associated item, 1 function

Used by: nothing

```
/// A trait with a generic associated type.
pub trait Lender {
    /// The borrowed item.
    type Loan<'a> where Self: 'a;

    /// Lends an item.
    fn lend(&mut self) -> Self::Loan<'_>;
}
```

<a id="type.traits.Observer"></a>
### Observer

Counts: 2 associated items, 4 functions

Used by: nothing

```
/// Watches a run.
pub trait Observer: Send + Sync {
    /// The item type.
    type Item: Clone;

    /// The most items kept.
    const LIMIT: usize = 64;

    /// Records one event.
    fn record(&self, event: &str);

    /// Clears the buffer.
    fn flush(&mut self);

    /// Ends the run.
    fn finish(self) -> u32
    where
        Self: Sized;

    /// Makes a default observer.
    fn make() -> Self
    where
        Self: Sized;
}
```

<a id="type.traits.Zeroable"></a>
### Zeroable

Counts: none

Used by: nothing

```
/// A marker that is unsafe to implement.
pub unsafe trait Zeroable {}
```
