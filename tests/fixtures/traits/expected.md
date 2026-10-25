<a id="crate.traits"></a>
# traits

```
//! Traits.
```

- [Unconnected types](#unconnected)
  - [Convert](#type.traits.Convert)
  - [Lender](#type.traits.Lender)
  - [Observer](#type.traits.Observer)
  - [Zeroable](#type.traits.Zeroable)
- [Modules](#modules)
  - [traits](#mod.traits)

**MACROS:** none

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
trait Observer: Send + Sync
{
    /// The item type.
    using Item: Clone;

    /// The most items kept.
    static constexpr LIMIT: usize = 64;

    /// Ends the run.
    static finish(self) -> u32 where Self: Sized;

    /// Makes a default observer.
    static make() -> Self where Self: Sized;

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
