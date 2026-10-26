*Updated 2026-01-01T00:00:00Z*

<a id="crate.facts"></a>
# facts

```
//! Facts about each declaration: what it holds, what names it, and which
//! types share a shape.
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
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, special members, and functions; a `Used by:` line links what names it; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [Level 1: top-level types](#level.1)
  - [Run](#type.facts.Run)
  - [ToolCallDispatch](#type.facts.ToolCallDispatch)
- [Level 2](#level.2)
  - [ChatDispatch](#type.facts.ChatDispatch)
- [Unconnected types](#unconnected)
  - [Point](#type.facts.Point)
  - [Size](#type.facts.Size)
- [Modules](#modules)
  - [facts](#mod.facts)
- [External items](#external)
- [Same shapes](#shapes)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [ChatDispatch](#type.facts.ChatDispatch)
- [dispatch()](#mod.facts)
- [Point](#type.facts.Point)
- [Run](#type.facts.Run)
- [Size](#type.facts.Size)
- [ToolCallDispatch](#type.facts.ToolCallDispatch)

<a id="level.1"></a>
## Level 1: top-level types

<a id="type.facts.Run"></a>
### Run

Counts: 4 fields (1 Option), 1 associated item, 1 special member, 2 functions

Used by: nothing

```
/// A run of the engine.
struct Run
{
    /// The largest run.
    static constexpr MAX: u32 = 9;

    /// The name of the run.
    name: String,

    /// The limit, when there is one.
    limit: Option<u32>,

    /// When the run started.
    started: chrono::DateTime<chrono::Utc>,

    Run();

    /// Starts a run.
    static start(name: String) -> Run;

    /// Stops it.
    stop() mut;

private:
    chat: ChatDispatch,
}
```

<a id="type.facts.ToolCallDispatch"></a>
### ToolCallDispatch

Counts: 2 variants

Used by: [facts](#mod.facts)

```
/// A tool call request.
enum ToolCallDispatch
{
    /// Run a tool.
    Send(String),

    /// Halt.
    Stop,
}
```

<a id="level.2"></a>
## Level 2

<a id="type.facts.ChatDispatch"></a>
### ChatDispatch

Counts: 2 variants

Used by: [Run](#type.facts.Run)

```
/// A chat request.
enum ChatDispatch
{
    /// Send a message.
    Send(String),

    /// Stop the chat.
    Stop,
}
```

<a id="unconnected"></a>
## Unconnected types

<a id="type.facts.Point"></a>
### Point

Counts: 2 fields

Used by: nothing

```
/// A point.
struct Point
{
    x: i32,

    y: i32,
}
```

<a id="type.facts.Size"></a>
### Size

Counts: 2 fields

Used by: nothing

```
/// A size.
struct Size
{
    x: i32,

    y: i32,
}
```

<a id="modules"></a>
## Modules

<a id="mod.facts"></a>
### facts

- [ChatDispatch](#type.facts.ChatDispatch)
- [Point](#type.facts.Point)
- [Run](#type.facts.Run)
- [Size](#type.facts.Size)
- [ToolCallDispatch](#type.facts.ToolCallDispatch)

```
/// Starts a tool call.
dispatch(call: ToolCallDispatch, at: chrono::Utc);
```

<a id="external"></a>
## External items

- `chrono`
  - `chrono::DateTime`, 1 use by [Run](#type.facts.Run)
  - `chrono::Utc`, 2 uses by [Run](#type.facts.Run), [facts](#mod.facts)

<a id="shapes"></a>
## Same shapes

- enums with the same 2 variants: [ChatDispatch](#type.facts.ChatDispatch), [ToolCallDispatch](#type.facts.ToolCallDispatch)
- structs with the same 2 fields: [Point](#type.facts.Point), [Size](#type.facts.Size)
