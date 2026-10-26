*Updated 2026-01-01T00:00:00Z*

<a id="crate.references"></a>
# references

```
//! References, raw pointers, and receivers.
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
  - [Marker](#type.references.Marker)
  - [Source](#type.references.Source)
  - [Table](#type.references.Table)
  - [Views](#type.references.Views)
- [Other impls](#impls)
- [Modules](#modules)
  - [references](#mod.references)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [apply()](#mod.references)
- [BYTES](#mod.references)
- [first()](#mod.references)
- [Marker](#type.references.Marker)
- [NAME](#mod.references)
- [pick()](#mod.references)
- [Source](#type.references.Source)
- [Table](#type.references.Table)
- [Views](#type.references.Views)

<a id="unconnected"></a>
## Unconnected types

<a id="type.references.Marker"></a>
### Marker

```
/// Marks a type.
trait Marker
{
}
```

<a id="type.references.Source"></a>
### Source

```
/// A source of bytes.
trait Source
{
    name() -> str&;

    read(buf: [u8] mut&) mut -> usize;
}
```

<a id="type.references.Table"></a>
### Table

```
/// A borrowed table.
using Table<'a> = [(str& 'a, u8*)]& 'a;
```

<a id="type.references.Views"></a>
### Views

```
/// Borrowed and raw views of one value.
struct Views<'a, T>
{
    shared: T& 'a,

    unique: T mut& 'a,

    name: str& 'static,

    items: [T& 'a]& 'a,

    read: T*,

    write: T mut*,

    shown: dyn Display& 'a,

    check: Box<dyn Fn(str&) -> usize>,

    callback: (dyn Fn(str&) -> usize)& 'a,

    both: (dyn Display + Send)& 'a,

    wrapped: (dyn Fn(str&) -> usize)& 'a,

    raw_call: (dyn Fn() -> bool)*,

    step: (fn(u8) -> u8)& 'a,

    clear() mut;

    get() -> T&;

    into_name(self) -> str& 'static;

    lookup(
        keys: Vec<str& 'a>&,
        table: std::collections::BTreeMap<String, T mut& 'a>&) mut ->
        Option<T mut& 'a>;

    poll(self: Pin<Self mut&>);

    rename(mut self, name: str& 'static) -> Self;

    render(out: impl Write mut&, items: [Box<dyn Display>]&);

    same(other: Self&) -> bool;

    set(value: T) mut;
}
```

<a id="impls"></a>
## Other impls

- `implements: Marker for T&;`
- `implements: Marker for u8*;`

<a id="modules"></a>
## Modules

<a id="mod.references"></a>
### references

- [Marker](#type.references.Marker)
- [Source](#type.references.Source)
- [Table](#type.references.Table)
- [Views](#type.references.Views)

```
/// The default name.
constexpr NAME: str& = "views";

/// Raw bytes behind a reference.
static BYTES: [u8]& = &[1, 2];

/// Applies a function to a string.
apply<F>(text: String mut&, f: F) -> usize where F: Fn(String mut&) -> usize;

/// Finds the first item.
first<'a>(items: [str& 'a]& 'a) -> Option<str& 'a>;

/// Picks an item with a closure that returns a reference.
pick<'a, T>(choose: impl Fn(T&) -> T& 'a, items: [T]& 'a) -> Option<T& 'a>;
```
