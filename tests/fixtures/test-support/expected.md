*Updated 2026-01-01T00:00:00Z*

<a id="crate.test_support"></a>
# test_support

```
//! Test-support code beside production code.
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
- [test_support](#mod.test_support)
  - [Engine](#type.test_support.Engine)
  - [Recording](#type.test_support.Recording)
  - [Model](#type.test_support.Model)
  - [ScriptedChat](#type.test_support.ScriptedChat)
  - [Live](#type.test_support.Live)
- [fakes](#mod.test_support.fakes)
  - [fakes::FakeClock](#type.test_support.fakes.FakeClock)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Engine](#type.test_support.Engine)
- [fakes](#mod.test_support.fakes) (test support)
- [fakes::clock()](#mod.test_support.fakes) (test support)
- [fakes::FakeClock](#type.test_support.fakes.FakeClock) (test support)
- [Live](#type.test_support.Live)
- [Model](#type.test_support.Model)
- [record()](#mod.test_support) (test support)
- [Recording](#type.test_support.Recording)
- [ScriptedChat](#type.test_support.ScriptedChat) (test support)

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [Engine](#type.test_support.Engine)
  - [Recording](#type.test_support.Recording)
- Level 2
  - [Model](#type.test_support.Model)
- Test support
  - [fakes::FakeClock](#type.test_support.fakes.FakeClock)
  - [ScriptedChat](#type.test_support.ScriptedChat)
- Unconnected types
  - [Live](#type.test_support.Live)

<a id="mod.test_support"></a>
## test_support

```
/// Records into the recording.
#[cfg(any(test, feature = "test-support"))]
pub fn record(into: &mut Recording);
```

<a id="type.test_support.Engine"></a>
### Engine

Counts: 1 field

Used by: nothing

```
/// A production engine.
pub struct Engine {
    /// The model it talks to.
    pub model: Model,
}
```

<a id="type.test_support.Recording"></a>
### Recording

Counts: none

Used by: [ScriptedChat](#type.test_support.ScriptedChat), [test_support](#mod.test_support)

```
/// Used only by test support.
pub struct Recording;
```

<a id="type.test_support.Model"></a>
### Model

Counts: none

Used by: [Engine](#type.test_support.Engine), [ScriptedChat](#type.test_support.ScriptedChat)

```
/// A production model.
pub struct Model;
```

<a id="type.test_support.ScriptedChat"></a>
### ScriptedChat

Counts: 2 fields

Used by: nothing

```
/// A chat that plays back a script, for tests and benches.
#[cfg(any(test, feature = "test-support"))]
pub struct ScriptedChat {
    /// The model the script stands in for.
    pub model: Model,

    /// Where it records.
    pub recording: Recording,
}
```

<a id="type.test_support.Live"></a>
### Live

Counts: none

Used by: nothing

```
/// Production only.
#[cfg(not(test))]
pub struct Live;
```

<a id="mod.test_support.fakes"></a>
## fakes

```
//! Fakes for tests and benches.
#![cfg(any(test, feature = "test-support"))]
```

```
/// Makes a fake clock.
pub fn clock() -> FakeClock;
```

<a id="type.test_support.fakes.FakeClock"></a>
### fakes::FakeClock

Counts: none

Used by: [fakes](#mod.test_support.fakes)

```
/// A fake clock.
pub struct FakeClock;
```
