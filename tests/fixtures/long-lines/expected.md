*Updated 2026-01-01T00:00:00Z*

<a id="crate.long_lines"></a>
# long_lines

```
//! The long-line rule.
```

- [Unconnected types](#unconnected)
  - [Container](#type.long_lines.Container)
  - [Wide](#type.long_lines.Wide)
- [Modules](#modules)
  - [long_lines](#mod.long_lines)

**MACROS:** none

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
}
```

<a id="modules"></a>
## Modules

<a id="mod.long_lines"></a>
### long_lines

- [Container](#type.long_lines.Container)
- [Wide](#type.long_lines.Wide)
