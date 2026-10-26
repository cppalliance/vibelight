*Updated 2026-01-01T00:00:00Z*

<a id="crate.long_lines"></a>
# long_lines

```
//! The long-line rule.
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
  - [Container](#type.long_lines.Container)
  - [Registry](#type.long_lines.Registry)
  - [Wide](#type.long_lines.Wide)
- [Modules](#modules)
  - [long_lines](#mod.long_lines)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Container](#type.long_lines.Container)
- [Registry](#type.long_lines.Registry)
- [Wide](#type.long_lines.Wide)

<a id="unconnected"></a>
## Unconnected types

<a id="type.long_lines.Container"></a>
### Container

```
/// A header whose where clause moves.
class Container<K, V>
    where K: Ord + Clone + std::fmt::Debug + Send + Sync, V: Clone + Send
{
    keys: Vec<K>,

    values: Vec<V>,
}
```

<a id="type.long_lines.Registry"></a>
### Registry

```
/// A header whose where clause breaks one predicate per line.
class Registry<K, V, S>
    where
        K: Ord + Clone + std::fmt::Debug + Send + Sync + 'static,
        V: Clone + Send + Sync + std::fmt::Debug,
        S: std::hash::BuildHasher + Default
{
    keys: Vec<K>,

    state: S,

    values: Vec<V>,
}
```

<a id="type.long_lines.Wide"></a>
### Wide

```
/// Members that test the long-line rule.
struct Wide
{
    /// This doc comment line is deliberately longer than eighty columns so it shows comments never wrap.
    a_field_with_a_very_long_type: std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>,

    static exactly_at_80(first_value: u64, second_value: u64) -> Option<String>;

    static fits_with_comment(value: u64) -> u64; // a trailing comment that pushes past eighty

    static huge(
        an_extremely_long_parameter_name_for_testing: std::collections::BTreeMap<String, Vec<u64>>);

    static long_first_line(
        argument_number_one: u64,
        argument_number_two: u64) -> u64;

    static one_over_limit(first_value: u64, second_value: u64) ->
        Option<String>;

    static tail_also_breaks(
        argument_number_one: u64,
        argument_number_two: u64) ->
        std::collections::BTreeMap<String, Vec<u64>>;

    register(
        name_of_the_thing: str&,
        handler_for_the_thing: Box<dyn Fn(u32)>,
        priority: u8) mut;

    register_bounded<T>(
        name: str&,
        value: T) mut
        where T: Clone + Send + Sync + std::fmt::Debug + std::hash::Hash;

    route<R, L>(
        read: R,
        load: L)
        where
            R: Fn(String) -> Option<String> + Send + Sync + 'static,
            L: Fn(String, Option<u32>) -> Result<String, u8> + Send;
}
```

<a id="modules"></a>
## Modules

<a id="mod.long_lines"></a>
### long_lines

- [Container](#type.long_lines.Container)
- [Registry](#type.long_lines.Registry)
- [Wide](#type.long_lines.Wide)
