*Updated 2026-01-01T00:00:00Z*

<a id="crate.facts"></a>
# facts

```
//! Facts about each declaration: what it holds, what names it, and which
//! types share a shape.
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
- [facts](#mod.facts)
  - [Run](#type.facts.Run)
  - [ToolCallDispatch](#type.facts.ToolCallDispatch)
  - [ChatDispatch](#type.facts.ChatDispatch)
  - [Point](#type.facts.Point)
  - [Size](#type.facts.Size)
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

<a id="levels"></a>
## Levels

- Level 1: top-level types
  - [Run](#type.facts.Run)
  - [ToolCallDispatch](#type.facts.ToolCallDispatch)
- Level 2
  - [ChatDispatch](#type.facts.ChatDispatch)
- Unconnected types
  - [Point](#type.facts.Point)
  - [Size](#type.facts.Size)

<a id="mod.facts"></a>
## facts

```
/// Starts a tool call.
pub fn dispatch(call: ToolCallDispatch, at: chrono::Utc);
```

<a id="type.facts.Run"></a>
### Run

Counts: 4 fields (1 Option), 1 associated item, 1 Default/From/Drop impl, 2 functions

Used by: nothing

```
/// A run of the engine.
pub struct Run {
    /// The name of the run.
    pub name: String,

    /// The limit, when there is one.
    pub limit: Option<u32>,

    /// When the run started.
    pub started: chrono::DateTime<chrono::Utc>,

    chat: ChatDispatch,
}

// impl in src/lib.rs
impl Run {
    /// The largest run.
    pub const MAX: u32 = 9;

    /// Starts a run.
    pub fn start(name: String) -> Run;

    /// Stops it.
    pub fn stop(&mut self);
}

impl Default for Run {}
```

<a id="type.facts.ToolCallDispatch"></a>
### ToolCallDispatch

Counts: 2 variants

Used by: [facts](#mod.facts)

```
/// A tool call request.
pub enum ToolCallDispatch {
    /// Run a tool.
    Send(String),

    /// Halt.
    Stop,
}
```

<a id="type.facts.ChatDispatch"></a>
### ChatDispatch

Counts: 2 variants

Used by: [Run](#type.facts.Run)

```
/// A chat request.
pub enum ChatDispatch {
    /// Send a message.
    Send(String),

    /// Stop the chat.
    Stop,
}
```

<a id="type.facts.Point"></a>
### Point

Counts: 2 fields

Used by: nothing

```
/// A point.
pub struct Point {
    pub x: i32,

    pub y: i32,
}
```

<a id="type.facts.Size"></a>
### Size

Counts: 2 fields

Used by: nothing

```
/// A size.
pub struct Size {
    pub x: i32,

    pub y: i32,
}
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
