*Updated 2026-01-01T00:00:00Z*

<a id="crate.body_uses"></a>
# body_uses

```
//! Types that only function bodies name.
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
- [body_uses](#mod.body_uses)
  - [Listed](#type.body_uses.Listed)
  - [Marker](#type.body_uses.Marker)
  - [Probe](#type.body_uses.Probe)
  - [Lister](#type.body_uses.Lister)
  - [Lonely](#type.body_uses.Lonely)
  - [Runner](#type.body_uses.Runner)
  - [Wrapped](#type.body_uses.Wrapped)
- [Other impls](#impls)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [list()](#mod.body_uses)
- [Listed](#type.body_uses.Listed)
- [Lister](#type.body_uses.Lister)
- [Lonely](#type.body_uses.Lonely)
- [Marker](#type.body_uses.Marker)
- [Probe](#type.body_uses.Probe)
- [Runner](#type.body_uses.Runner)
- [Wrapped](#type.body_uses.Wrapped)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [Listed](#type.body_uses.Listed)
  - [Marker](#type.body_uses.Marker)
  - [Probe](#type.body_uses.Probe)
- Unconnected types
  - [Lister](#type.body_uses.Lister)
  - [Lonely](#type.body_uses.Lonely)
  - [Runner](#type.body_uses.Runner)
  - [Wrapped](#type.body_uses.Wrapped)

<a id="mod.body_uses"></a>
## body_uses

```
/// Lists one thing.
pub fn list();
```

<a id="type.body_uses.Listed"></a>
### Listed

Counts: none

Used by: nothing

```
/// Named only inside a free function's body.
pub struct Listed;
```

<a id="type.body_uses.Marker"></a>
### Marker

Counts: none

Used by: nothing

```
/// Named only in a trait's default method body.
pub struct Marker;
```

<a id="type.body_uses.Probe"></a>
### Probe

Counts: none

Used by: nothing

```
/// Named only inside a method body of another type.
pub struct Probe;
```

<a id="type.body_uses.Lister"></a>
### Lister

Counts: 1 function

Used by: nothing

```
/// Lists things.
pub trait Lister {
    /// Marks.
    fn mark(&self);
}
```

<a id="type.body_uses.Lonely"></a>
### Lonely

Counts: 1 function

Used by: nothing

```
/// Named only inside its own method body, so it stays unconnected.
pub struct Lonely;

// impl in src/lib.rs
impl Lonely {
    /// Makes another one.
    pub fn again();
}
```

<a id="type.body_uses.Runner"></a>
### Runner

Counts: 1 function

Used by: nothing

```
/// Runs things.
pub struct Runner;

// impl in src/lib.rs
impl Runner {
    /// Makes a probe.
    pub fn probe(&self);
}
```

<a id="type.body_uses.Wrapped"></a>
### Wrapped

Counts: none

Used by: nothing

```
/// Named only inside an impl on a wrapper of itself, so it stays
/// unconnected.
pub struct Wrapped;
```

<a id="impls"></a>
## Other impls

- `impl Default for Box<Wrapped> {}`
