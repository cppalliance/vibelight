*Updated 2026-01-01T00:00:00Z*

<a id="crate.kill_list"></a>
# kill_list

```
//! A crate with macros: the markdown counts them, and the HTML lists each one.
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
- [kill_list](#mod.kill_list)
  - [Event](#type.kill_list.Event)

**MACROS:** 1 definition, 1 invocation, 1 derive list, 1 attribute list

<a id="surface"></a>
## Public surface

- [Event](#type.kill_list.Event)

<a id="levels"></a>
## Levels

- Unconnected types
  - [Event](#type.kill_list.Event)

<a id="mod.kill_list"></a>
## kill_list

```
// MACRO constant!

/// Defines a constant.
// MACRO macro_rules! constant
```

**MACRO** constant!

**MACRO** macro_rules! constant

<a id="type.kill_list.Event"></a>
### Event

Counts: none

Used by: nothing

**MACRO** derive: serde::Serialize

**MACRO** attribute: tracing::instrument

```
/// A type with a derive from another crate and an attribute macro.
#[derive(Debug, serde::Serialize)]
pub struct Event;
```
