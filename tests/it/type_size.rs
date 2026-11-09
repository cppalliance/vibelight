//! The type-size limits, the same as runvil's: a type's inherent methods,
//! summed across every `impl` block under `src/`, stay at or under
//! `MAX_METHODS`, and a struct's named fields stay at or under
//! `MAX_FIELDS`, unless `ALLOWED` records the type with a reason.
//!
//! The limits sit on types because a file has no length limit: a type's
//! methods may live in any number of files, so the count sums every
//! inherent `impl` block. Trait impls are not counted, because the trait
//! fixes their methods. Clippy holds each function to its own limits.
//!
//! The check reads syntax, so it keys a type by the last segment of the
//! name an `impl` gives it: two types of one name are summed, and an `impl`
//! that a macro writes is not seen. It skips test files by name and every
//! item behind a `cfg` that names `test`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, Fields, ImplItem, Item, Meta, Type};

/// The most inherent methods one type may have, summed across every `impl`
/// block.
const MAX_METHODS: usize = 25;

/// The most named fields one struct may have.
const MAX_FIELDS: usize = 12;

/// The types over a limit today. A new entry is a plan decision with a
/// reason, never a way to make a change pass.
const ALLOWED: &[Allowed] = &[
    Allowed {
        name: "AliasRecord",
        limit: Limit::Fields,
        recorded: 15,
        reason: "One alias record of the markdown's typed schema: each field is one key of the \
                 JSON record, so the count follows the schema.",
    },
    Allowed {
        name: "AnalysisRun",
        limit: Limit::Methods,
        recorded: 39,
        reason: "The run's immutable analysis view: each method answers one question about the \
                 run's items, paths, or uses, in the `analysis_*` file of its topic.",
    },
    Allowed {
        name: "AnalysisRun",
        limit: Limit::Fields,
        recorded: 21,
        reason: "The run's immutable analysis view: each field is one index over the whole run, \
                 built once before any target renders.",
    },
    Allowed {
        name: "BodyFacts",
        limit: Limit::Fields,
        recorded: 18,
        reason: "What one function body holds: each field is one fact the markdown prints for \
                 the body.",
    },
    Allowed {
        name: "Collector",
        limit: Limit::Methods,
        recorded: 60,
        reason: "The syntax walk that collects one target: each method collects one kind of \
                 syntax, in the `collect_*` file of its kind.",
    },
    Allowed {
        name: "Collector",
        limit: Limit::Fields,
        recorded: 18,
        reason: "The syntax walk that collects one target: each field is one piece of the \
                 walk's state or one table it fills.",
    },
    Allowed {
        name: "Crate",
        limit: Limit::Fields,
        recorded: 15,
        reason: "Everything collected from one target: each field is one table of the model \
                 that a later pass reads.",
    },
    Allowed {
        name: "Decl",
        limit: Limit::Fields,
        recorded: 21,
        reason: "One declared type of the model: each field is one property that the markdown \
                 or the HTML prints for it.",
    },
    Allowed {
        name: "FnRecord",
        limit: Limit::Fields,
        recorded: 20,
        reason: "One function record of the markdown's typed schema: each field is one key of \
                 the JSON record, so the count follows the schema.",
    },
    Allowed {
        name: "Impl",
        limit: Limit::Fields,
        recorded: 13,
        reason: "One impl block of the model: each field is one property that routing or an \
                 output reads.",
    },
    Allowed {
        name: "ModRecord",
        limit: Limit::Fields,
        recorded: 14,
        reason: "One module record of the markdown's typed schema: each field is one key of the \
                 JSON record, so the count follows the schema.",
    },
    Allowed {
        name: "Module",
        limit: Limit::Fields,
        recorded: 15,
        reason: "One module of the model: each field is one property that an output prints or \
                 the coupling pass reads.",
    },
    Allowed {
        name: "TraitRecord",
        limit: Limit::Fields,
        recorded: 16,
        reason: "One trait record of the markdown's typed schema: each field is one key of the \
                 JSON record, so the count follows the schema.",
    },
    Allowed {
        name: "Ty",
        limit: Limit::Methods,
        recorded: 26,
        reason: "A type compared as tokens for the leads: each method is one shape question a \
                 detector asks of it.",
    },
    Allowed {
        name: "TypeRecord",
        limit: Limit::Fields,
        recorded: 17,
        reason: "One type record of the markdown's typed schema: each field is one key of the \
                 JSON record, so the count follows the schema.",
    },
    Allowed {
        name: "ValueRecord",
        limit: Limit::Fields,
        recorded: 17,
        reason: "One value record of the markdown's typed schema: each field is one key of the \
                 JSON record, so the count follows the schema.",
    },
];

/// The limit an `Allowed` entry lifts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Limit {
    Methods,
    Fields,
}

impl Limit {
    fn bound(self) -> usize {
        match self {
            Limit::Methods => MAX_METHODS,
            Limit::Fields => MAX_FIELDS,
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Limit::Methods => "inherent methods across its `impl` blocks",
            Limit::Fields => "named fields",
        }
    }
}

/// A type that may pass one limit, up to the count recorded when the entry
/// was written. It fails when it grows past `recorded`, and the entry fails
/// as stale once the type is gone or back within the limit.
struct Allowed {
    name: &'static str,
    limit: Limit,
    recorded: usize,
    /// What the type is and why its size holds, in one sentence.
    reason: &'static str,
}

/// What the sources hold: each type's inherent methods by file, and each
/// struct's named fields with the file that defines it.
#[derive(Default)]
struct Census {
    methods: BTreeMap<String, BTreeMap<String, usize>>,
    fields: BTreeMap<String, (String, usize)>,
}

impl Census {
    /// Parses one source file, `file` relative to the package root.
    fn read(&mut self, file: &str, text: &str) -> Result<(), String> {
        let parsed = syn::parse_file(text).map_err(|error| {
            format!(
                "{file}:{}: unparseable source: {error}",
                error.span().start().line
            )
        })?;
        if !gated_on_test(&parsed.attrs) {
            self.items(&parsed.items, file);
        }
        Ok(())
    }

    /// Records the inherent methods and the structs in `items`, inline
    /// modules included.
    fn items(&mut self, items: &[Item], file: &str) {
        for item in items {
            match item {
                Item::Impl(item) if item.trait_.is_none() && !gated_on_test(&item.attrs) => {
                    let Some(name) = self_type_name(&item.self_ty) else {
                        continue;
                    };
                    let count = (item.items.iter())
                        .filter(|member| {
                            matches!(member, ImplItem::Fn(function) if !gated_on_test(&function.attrs))
                        })
                        .count();
                    let files = self.methods.entry(name).or_default();
                    *files.entry(file.to_owned()).or_default() += count;
                }
                Item::Struct(item) if !gated_on_test(&item.attrs) => {
                    if let Fields::Named(fields) = &item.fields {
                        let entry = (self.fields)
                            .entry(item.ident.to_string())
                            .or_insert_with(|| (file.to_owned(), 0));
                        entry.1 = entry.1.max(fields.named.len());
                    }
                }
                Item::Mod(item) if !gated_on_test(&item.attrs) => {
                    if let Some((_, items)) = &item.content {
                        self.items(items, file);
                    }
                }
                _ => {}
            }
        }
    }

    /// Every type's count under each limit.
    fn counts(&self) -> BTreeMap<(String, Limit), (usize, String)> {
        let mut counts = BTreeMap::new();
        for (name, files) in &self.methods {
            let total = files.values().sum();
            let breakdown: Vec<String> = (files.iter())
                .filter(|(_, count)| **count > 0)
                .map(|(file, count)| format!("{file} {count}"))
                .collect();
            counts.insert(
                (name.clone(), Limit::Methods),
                (total, breakdown.join(", ")),
            );
        }
        for (name, (file, fields)) in &self.fields {
            counts.insert((name.clone(), Limit::Fields), (*fields, file.clone()));
        }
        counts
    }
}

/// Every type over a limit without an entry, every allowed type past its
/// recorded count, and every entry that is stale or has no reason.
fn violations(census: &Census, allowed: &[Allowed]) -> Vec<String> {
    let counts = census.counts();
    let mut found = Vec::new();
    for ((name, limit), (count, place)) in &counts {
        let entry = (allowed.iter()).find(|entry| entry.name == name && entry.limit == *limit);
        match entry {
            None if *count > limit.bound() => found.push(format!(
                "`{name}` has {count} {}, more than {}; split it by responsibility, or record it \
                 in ALLOWED with a reason ({place})",
                limit.noun(),
                limit.bound()
            )),
            Some(entry) if *count > entry.recorded => found.push(format!(
                "`{name}` has {count} {}, more than the {} its ALLOWED entry records ({place})",
                limit.noun(),
                entry.recorded
            )),
            _ => {}
        }
    }
    for entry in allowed {
        if entry.reason.trim().is_empty() {
            found.push(format!(
                "the ALLOWED entry for `{}` has no reason",
                entry.name
            ));
        }
        match counts.get(&(entry.name.to_owned(), entry.limit)) {
            None => found.push(format!(
                "the ALLOWED entry for `{}` is stale: no such type",
                entry.name
            )),
            Some((count, _)) if *count <= entry.limit.bound() => found.push(format!(
                "the ALLOWED entry for `{}` is stale: it has {count} {}, within the limit of {}",
                entry.name,
                entry.limit.noun(),
                entry.limit.bound()
            )),
            Some(_) => {}
        }
    }
    found
}

/// Every `.rs` file under `dir` that is not test code by its name, keyed by
/// its path relative to `root` with forward slashes.
fn sources(root: &Path, dir: &Path, found: &mut BTreeMap<String, String>) {
    let entries = fs::read_dir(dir).expect("the source folder is readable");
    for entry in entries {
        let path = entry.expect("a source folder entry is readable").path();
        if path.is_dir() {
            sources(root, &path, found);
            continue;
        }
        let relative = path.strip_prefix(root).expect("a source is under the root");
        let name = relative.to_string_lossy().replace('\\', "/");
        let stem = path
            .file_stem()
            .map_or(String::new(), |s| s.to_string_lossy().into_owned());
        if path.extension().is_some_and(|ext| ext == "rs") && !is_test_stem(&stem) {
            let text = fs::read_to_string(&path).expect("a source file is readable");
            found.insert(name, text);
        }
    }
}

/// Whether a file stem names test code: `tests`, or a stem that ends in
/// `_tests` or `-tests`, or holds `-tests-`.
fn is_test_stem(stem: &str) -> bool {
    stem == "tests"
        || stem.ends_with("_tests")
        || stem.ends_with("-tests")
        || stem.contains("-tests-")
}

/// Whether `attrs` hold a `cfg` that names `test` outside a `not`, as
/// `cfg(test)` and `cfg(any(test, feature = "..."))` do.
fn gated_on_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && matches!(&attr.meta, Meta::List(list) if names_test(list.tokens.clone()))
    })
}

fn names_test(tokens: TokenStream) -> bool {
    let mut tokens = tokens.into_iter();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(ident) if ident == "not" => {
                tokens.next();
            }
            TokenTree::Ident(ident) if ident == "test" => return true,
            TokenTree::Group(group) if names_test(group.stream()) => return true,
            _ => {}
        }
    }
    false
}

/// The last path segment of an `impl`'s self type, or none for a self type
/// that is not a plain path.
fn self_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            (path.path.segments.last()).map(|segment| segment.ident.to_string())
        }
        Type::Group(group) => self_type_name(&group.elem),
        Type::Paren(paren) => self_type_name(&paren.elem),
        _ => None,
    }
}

/// The census of a set of sources given as text.
fn census_of(files: &[(&str, &str)]) -> Census {
    let mut census = Census::default();
    for (file, text) in files {
        census.read(file, text).expect("a test source parses");
    }
    census
}

#[test]
fn every_type_of_the_package_stays_within_its_limits() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = BTreeMap::new();
    sources(root, &root.join("src"), &mut files);
    assert!(!files.is_empty(), "the check read no source");
    let mut census = Census::default();
    let unreadable: Vec<String> = (files.iter())
        .filter_map(|(file, text)| census.read(file, text).err())
        .collect();
    let mut found = violations(&census, ALLOWED);
    found.extend(unreadable);
    assert!(
        found.is_empty(),
        "type-size violations:\n{}",
        found.join("\n")
    );
}

/// `n` copies of `template`, each with its `#` replaced by the copy's index.
fn numbered(n: usize, template: &str) -> String {
    (0..n)
        .map(|i| template.replace('#', &i.to_string()))
        .collect()
}

#[test]
fn methods_count_across_impl_blocks_and_trait_methods_do_not() {
    let first = format!(
        "struct Big; impl Big {{ {} }}",
        numbered(20, "fn m#(&self) {} ")
    );
    let second = format!(
        "impl Big {{ {} }} impl Clone for Big {{ fn clone(&self) -> Self {{ Big }} }}",
        numbered(6, "fn n#(&self) {} ")
    );
    let census = census_of(&[("src/a.rs", &first), ("src/b.rs", &second)]);
    let found = violations(&census, &[]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("`Big` has 26 inherent methods"),
        "{found:?}"
    );
    assert!(found[0].contains("src/a.rs 20, src/b.rs 6"), "{found:?}");
}

#[test]
fn test_code_does_not_count() {
    let fields = numbered(13, "f#: u8, ");
    let text = format!(
        "#[cfg(test)] struct Wide {{ {fields} }}\n\
         #[cfg(any(test, feature = \"x\"))] mod helpers {{ struct Wider {{ {fields} }} }}\n\
         #[cfg(not(test))] struct Real {{ {fields} }}"
    );
    let found = violations(&census_of(&[("src/a.rs", &text)]), &[]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("`Real` has 13 named fields"),
        "{found:?}"
    );
    assert!(is_test_stem("leads_tests") && is_test_stem("foo-tests") && !is_test_stem("latest"));
}

#[test]
fn an_allowed_type_may_not_grow_and_a_stale_entry_fails() {
    let text = format!(
        "struct Wide {{ {} }} struct Slim {{ {} }}",
        numbered(14, "f#: u8, "),
        numbered(3, "f#: u8, ")
    );
    let census = census_of(&[("src/a.rs", &text)]);
    let entry = |name, recorded| Allowed {
        name,
        limit: Limit::Fields,
        recorded,
        reason: "A test entry.",
    };
    assert_eq!(
        violations(&census, &[entry("Wide", 14)]),
        Vec::<String>::new()
    );
    let grown = violations(&census, &[entry("Wide", 13)]);
    assert!(
        grown[0].contains("more than the 13 its ALLOWED entry records"),
        "{grown:?}"
    );
    let stale = violations(
        &census,
        &[entry("Wide", 14), entry("Slim", 20), entry("Gone", 20)],
    );
    assert_eq!(stale.len(), 2, "{stale:?}");
    assert!(
        stale.iter().any(|v| v.contains("`Slim` is stale")),
        "{stale:?}"
    );
    assert!(
        stale.iter().any(|v| v.contains("`Gone` is stale")),
        "{stale:?}"
    );
}
