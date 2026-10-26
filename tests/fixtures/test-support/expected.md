*Updated 2026-01-01T00:00:00Z*

<a id="crate.test_support"></a>
# test_support

```
//! Test-support code beside production code.
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
- [Level 1: top-level types](#level.1)
  - [Engine](#type.test_support.Engine)
  - [Recording](#type.test_support.Recording)
- [Level 2](#level.2)
  - [Model](#type.test_support.Model)
- [Test support](#test-support)
  - [fakes::FakeClock](#type.test_support.fakes.FakeClock)
  - [ScriptedChat](#type.test_support.ScriptedChat)
- [Unconnected types](#unconnected)
  - [Live](#type.test_support.Live)
- [Modules](#modules)
  - [test_support](#mod.test_support)
  - [fakes](#mod.test_support.fakes)

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

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.test_support.Engine"></a>
### Engine

```
/// A production engine.
struct Engine
{
    /// The model it talks to.
    model: Model,
}
```

<a id="type.test_support.Recording"></a>
### Recording

```
/// Used only by test support.
struct Recording;
```

<a id="level.2"></a>
## Level 2

<a id="type.test_support.Model"></a>
### Model

```
/// A production model.
struct Model;
```

<a id="test-support"></a>
## Test support

<a id="type.test_support.fakes.FakeClock"></a>
### fakes::FakeClock

```
/// A fake clock.
struct FakeClock;
```

<a id="type.test_support.ScriptedChat"></a>
### ScriptedChat

```
/// A chat that plays back a script, for tests and benches.
// cfg: any(test, feature = "test-support")
struct ScriptedChat
{
    /// The model the script stands in for.
    model: Model,

    /// Where it records.
    recording: Recording,
}
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.test_support.Live"></a>
### Live

```
/// Production only.
// cfg: not(test)
struct Live;
```

<a id="modules"></a>
## Modules

<a id="mod.test_support"></a>
### test_support

- [Engine](#type.test_support.Engine)
- [Live](#type.test_support.Live)
- [Model](#type.test_support.Model)
- [Recording](#type.test_support.Recording)
- [ScriptedChat](#type.test_support.ScriptedChat)

```
/// Records into the recording.
// cfg: any(test, feature = "test-support")
record(into: Recording mut&);
```

<a id="mod.test_support.fakes"></a>
### fakes

```
//! Fakes for tests and benches.
// cfg: any(test, feature = "test-support")
```

- [fakes::FakeClock](#type.test_support.fakes.FakeClock)

```
/// Makes a fake clock.
clock() -> FakeClock;
```
