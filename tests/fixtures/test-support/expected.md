*Updated 2026-01-01T00:00:00Z*

<a id="crate.test_support"></a>
# test_support

```
//! Test-support code beside production code.
```

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
