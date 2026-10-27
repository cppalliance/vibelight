*Updated 2026-01-01T00:00:00Z*

<a id="crate.long_lines"></a>
# long_lines

```
//! The long-line rule.
```

<a id="legend"></a>
## Legend

- Each type is one block of Rust with the bodies left out: the item with its attributes and its fields, variants, or trait items; then each inherent impl that adds members, under a `// impl in` line that names the file holding it; then one line for each trait impl.
- Visibilities, signatures, and impl headers are written as in the source, without their comments; a constant's value shows as `...` when it is long, and is left out when it calls a macro or holds a comment.
- `#[derive(...)]` lists every derived trait, and `#[cfg(...)]` and the attributes that shape an API or its wire format print as written; other attributes are left out.
- Each impl prints with its type, so a module that holds only impls shows nothing but its docs.
- `MACRO` marks each macro definition, invocation, derive from another crate, and attribute macro; the kill list near the top counts them.
- Public surface lists every path a user of the crate can name, and `(test support)` marks one that is test support.
- Each module has a section that holds its declarations in dependency order: a type that no other type names comes first, a type that only one other type names follows it, and a type that several name follows the last of them.
- Last in a module's section come the types behind a `cfg` that lists `test` inside `any(...)`, which are test support, and then the types that name nothing and that nothing names.
- A `Source:` line links to the definition and to each impl block, with the members each inherent impl adds.
- A `Counts:` line gives a type's variants, fields (and how many are an `Option`), associated items, `Default`, `From`, and `Drop` impls, and functions; a `Used by:` line links what names it; a `Fields:` list, for a type with 8 or more functions, names the functions that reach each field through `self`, and gives only a count for a hub that more than half of them reach; External items lists the items of other crates that signatures name; and Same shapes groups types with the same variants or fields.
- The first line is the UTC time the file's content last changed.

- [Legend](#legend)
- [Public surface](#surface)
- [long_lines](#mod.long_lines)
  - [Container](#type.long_lines.Container)
  - [Registry](#type.long_lines.Registry)
  - [Wide](#type.long_lines.Wide)

**MACROS:** none

<a id="surface"></a>
## Public surface

- [Container](#type.long_lines.Container)
- [Registry](#type.long_lines.Registry)
- [Wide](#type.long_lines.Wide)

<a id="mod.long_lines"></a>
## long_lines

<a id="type.long_lines.Container"></a>
### Container

Counts: 2 fields

Used by: nothing

```
/// A header whose where clause moves.
pub struct Container<K, V>
where
    K: Ord + Clone + std::fmt::Debug + Send + Sync,
    V: Clone + Send,
{
    keys: Vec<K>,

    values: Vec<V>,
}
```

<a id="type.long_lines.Registry"></a>
### Registry

Counts: 3 fields

Used by: nothing

```
/// A header whose where clause breaks one predicate per line.
pub struct Registry<K, V, S>
where
    K: Ord + Clone + std::fmt::Debug + Send + Sync + 'static,
    V: Clone + Send + Sync + std::fmt::Debug,
    S: std::hash::BuildHasher + Default,
{
    keys: Vec<K>,

    state: S,

    values: Vec<V>,
}
```

<a id="type.long_lines.Wide"></a>
### Wide

Counts: 1 field, 9 functions

Used by: nothing

Fields:

- `a_field_with_a_very_long_type`: none

```
/// Members that test the long-line rule.
pub struct Wide {
    /// This doc comment line is deliberately longer than eighty columns so it shows comments never wrap.
    pub a_field_with_a_very_long_type: std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>,
}

// impl in src/lib.rs
impl Wide {
    pub fn exactly_at_80(first_value: u64, second_value: u64) -> Option<String>;

    pub fn one_over_limit(first_value: u64, second_value: u64) -> Option<String>;

    pub fn long_first_line(argument_number_one: u64, argument_number_two: u64) -> u64;

    pub fn tail_also_breaks(argument_number_one: u64, argument_number_two: u64) -> std::collections::BTreeMap<String, Vec<u64>>;

    pub fn register(&mut self, name_of_the_thing: &str, handler_for_the_thing: Box<dyn Fn(u32)>, priority: u8);

    pub fn register_bounded<T>(&mut self, name: &str, value: T)
    where
        T: Clone + Send + Sync + std::fmt::Debug + std::hash::Hash;

    pub fn route<R, L>(&self, read: R, load: L)
    where
        R: Fn(String) -> Option<String> + Send + Sync + 'static,
        L: Fn(String, Option<u32>) -> Result<String, u8> + Send;

    pub fn huge(an_extremely_long_parameter_name_for_testing: std::collections::BTreeMap<String, Vec<u64>>);

    pub fn fits_with_comment(value: u64) -> u64; // a trailing comment that pushes past eighty
}
```
