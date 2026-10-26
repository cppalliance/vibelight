*Updated 2026-01-01T00:00:00Z*

<a id="crate.functions"></a>
# functions

```
//! Functions and signatures.
```

- [Modules](#modules)
  - [functions](#mod.functions)

**MACROS:** none

<a id="modules"></a>
## Modules

<a id="mod.functions"></a>
### functions

```
/// Every qualifier at once.
constexpr unsafe extern "C" all_qualifiers();

/// Uses a higher-ranked bound.
apply<F>(f: F) -> usize where F: for<'a> Fn(str& 'a) -> usize;

/// Exported to C.
extern "C" callback(code: i32) -> i32;

/// Has a where clause on several lines.
merge<A, B, C>(a: A, b: B) ->
    C where A: Into<C>, B: Into<C>, C: std::ops::Add<Output = C>;

/// A rustfmt-style signature over several lines.
parse(input: str&, execution: str&) ->
    (std::result::Result<String, String>, Vec<String>);

/// Reads raw memory.
///
/// # Safety
/// The pointer must be valid.
unsafe peek(ptr: u8*) -> u8;

/// Takes impl and dyn traits.
render(out: impl std::io::Write mut&, items: [Box<dyn std::fmt::Display>]&);

/// Computes at compile time.
constexpr square(x: u32) -> u32;

/// Waits for input.
async wait(seconds: u64);

private:
private_helper();
```
