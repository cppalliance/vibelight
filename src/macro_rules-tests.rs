//! Tests the `macro_rules!` engine: parsing definitions, matching each
//! fragment kind and repetition, transcribing, and expanding nested
//! invocations into items.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::{Expr, Fields, ImplItem, Item, Member, Pat, Stmt};

use crate::macro_rules::{Budget, Failure, MacroRules, expand_items};
use crate::macros::path_text;

/// The `events!` macro of runvil's `event.rs`, copied verbatim.
const EVENTS: &str = r#"macro_rules! events {
    (
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident {
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident : $ty:ty
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        #[non_exhaustive]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant {
                    /// The caller-chosen run identifier.
                    execution: String,
                    /// The reporting scope: the prompt's H2 heading text,
                    /// or an agent's name.
                    section: String,
                    /// The replay key: the nearest enclosing task and this
                    /// event's position within it.
                    provenance: Provenance,
                    $(
                        $(#[$field_meta])*
                        $field: $ty,
                    )*
                },
            )*
        }

        impl $name {
            /// The caller-chosen run identifier this event belongs to.
            #[must_use]
            pub fn execution(&self) -> &str {
                match self {
                    $( $name::$variant { execution, .. } )|* => execution,
                }
            }

            /// The reporting scope this event was reported under.
            #[must_use]
            pub fn section(&self) -> &str {
                match self {
                    $( $name::$variant { section, .. } )|* => section,
                }
            }

            /// The replay key: the nearest enclosing task and this event's
            /// position within it.
            #[must_use]
            pub fn provenance(&self) -> &Provenance {
                match self {
                    $( $name::$variant { provenance, .. } )|* => provenance,
                }
            }
        }
    };
}"#;

/// A tt-muncher that builds a struct one field at a time through internal
/// `@inner` rules.
const FIELDS: &str = "macro_rules! fields {
    (@inner $name:ident [$($done:tt)*]) => { pub struct $name { $($done)* } };
    (@inner $name:ident [$($done:tt)*] $field:ident $($rest:ident)*) => {
        fields!(@inner $name [$($done)* pub $field: u8,] $($rest)*);
    };
    ($name:ident { $($field:ident)* }) => { fields!(@inner $name [] $($field)*); };
}";

/// Parses a `macro_rules!` definition from its source.
pub(crate) fn rules(source: &str) -> MacroRules {
    let definition: syn::ItemMacro = syn::parse_str(source).unwrap();
    MacroRules::parse(&definition.mac.tokens).unwrap()
}

/// Lexes source into tokens.
fn tokens(source: &str) -> TokenStream {
    source.parse().unwrap()
}

/// Applies `rules` to the tokens of `input` with a fresh budget.
pub(crate) fn apply(rules: &MacroRules, input: &str) -> Result<TokenStream, Failure> {
    rules.apply(&tokens(input), &mut Budget::new())
}

/// Asserts that `rules` turn `input` into the tokens of `expected`, spacing
/// aside.
pub(crate) fn expands(rules: &MacroRules, input: &str, expected: &str) {
    let output = apply(rules, input).unwrap();
    assert_eq!(words(output), words(tokens(expected)), "{input}");
}

/// Returns the text of each token, with each group's delimiters around its
/// tokens and invisible delimiters left out.
fn words(stream: TokenStream) -> Vec<String> {
    let mut out = Vec::new();
    for tree in stream {
        let TokenTree::Group(group) = tree else {
            out.push(tree.to_string());
            continue;
        };
        let (open, close) = match group.delimiter() {
            Delimiter::Parenthesis => ("(", ")"),
            Delimiter::Brace => ("{", "}"),
            Delimiter::Bracket => ("[", "]"),
            Delimiter::None => ("", ""),
        };
        out.push(open.to_owned());
        out.extend(words(group.stream()));
        out.push(close.to_owned());
    }
    out.retain(|word| !word.is_empty());
    out
}

/// Returns the names of named fields, in order.
fn field_names(fields: &Fields) -> Vec<String> {
    fields
        .iter()
        .filter_map(|field| field.ident.as_ref().map(ToString::to_string))
        .collect()
}

#[test]
fn the_events_macro_stamps_the_coordinates_on_every_variant() {
    let input = "/// One thing that happened during a run.
        pub enum Event {
            /// Prompt parsing began.
            ParseStarted {},
            RunStarted {},
            TaskAbandoned {
                /// The task that stopped.
                task: TaskId,
                #[serde(default, skip_serializing_if = \"Option::is_none\")]
                reason: Option<AbandonReason>,
            },
        }";
    let items = expand_items(&rules(EVENTS), &tokens(input), &|_| None).unwrap();
    let [Item::Enum(event), Item::Impl(accessors)] = items.as_slice() else {
        panic!("the expansion is one enum and one impl");
    };
    let attrs: Vec<String> = event.attrs.iter().map(|a| path_text(a.path())).collect();
    assert_eq!(attrs, ["doc", "derive", "serde", "non_exhaustive"]);
    let coordinates = ["execution", "section", "provenance"];
    let fields = |payload: &[&str]| -> Vec<String> {
        coordinates
            .iter()
            .chain(payload)
            .map(|name| (*name).to_owned())
            .collect()
    };
    let variants: Vec<(String, usize, Vec<String>)> = event
        .variants
        .iter()
        .map(|v| (v.ident.to_string(), v.attrs.len(), field_names(&v.fields)))
        .collect();
    let expected = [
        ("ParseStarted".to_owned(), 1, fields(&[])),
        ("RunStarted".to_owned(), 0, fields(&[])),
        ("TaskAbandoned".to_owned(), 0, fields(&["task", "reason"])),
    ];
    assert_eq!(variants, expected);
    let payload: Vec<Vec<String>> = (event.variants[2].fields.iter().skip(3))
        .map(|field| field.attrs.iter().map(|a| path_text(a.path())).collect())
        .collect();
    assert_eq!(payload, [["doc"], ["serde"]]);
    let paths = [
        "Event::ParseStarted",
        "Event::RunStarted",
        "Event::TaskAbandoned",
    ];
    let mut methods = Vec::new();
    for item in &accessors.items {
        let ImplItem::Fn(method) = item else {
            panic!("every accessor is a method");
        };
        methods.push(method.sig.ident.to_string());
        let [Stmt::Expr(Expr::Match(body), None)] = method.block.stmts.as_slice() else {
            panic!("an accessor's body is one match");
        };
        let [arm] = body.arms.as_slice() else {
            panic!("the match has one arm");
        };
        let Pat::Or(or) = &arm.pat else {
            panic!("the arm is an or-pattern");
        };
        let named: Vec<String> = or
            .cases
            .iter()
            .map(|case| match case {
                Pat::Struct(variant) => {
                    let bound: Vec<String> = (variant.fields.iter())
                        .map(|field| match &field.member {
                            Member::Named(name) => name.to_string(),
                            Member::Unnamed(_) => String::new(),
                        })
                        .collect();
                    assert_eq!(
                        bound,
                        [method.sig.ident.to_string()],
                        "an arm binds its field"
                    );
                    path_text(&variant.path)
                }
                _ => String::new(),
            })
            .collect();
        assert_eq!(named, paths);
    }
    assert_eq!(methods, coordinates);
}

#[test]
fn every_fragment_kind_matches_a_typical_input() {
    let cases = [
        ("block", "{ let a = 1; a }"),
        ("expr", "a + b * 2"),
        ("expr_2021", "f(x, y)?"),
        ("ident", "name"),
        ("item", "pub struct Point { x: u8 }"),
        ("lifetime", "'a"),
        ("literal", "-1.5"),
        ("meta", "serde(rename = \"kind\")"),
        ("pat", "Some(1 | 2) | None"),
        ("pat_param", "(first, ..)"),
        ("path", "std::vec::Vec<u8>"),
        ("stmt", "let x = y"),
        ("tt", "[a, b]"),
        ("ty", "&'a mut [Option<u8>]"),
        ("vis", "pub(crate)"),
    ];
    for (kind, input) in cases {
        let rules = rules(&format!(
            "macro_rules! m {{ ($x:{kind} , end) => {{ $x }}; }}"
        ));
        let output = apply(&rules, &format!("{input} , end")).unwrap();
        let trees: Vec<TokenTree> = output.clone().into_iter().collect();
        let invisible = matches!(trees.as_slice(),
            [TokenTree::Group(group)] if group.delimiter() == Delimiter::None);
        assert_eq!(invisible, kind.starts_with("expr"), "{kind}");
        assert_eq!(words(output), words(tokens(input)), "{kind}");
    }
    let vis = rules("macro_rules! m { ($v:vis fn) => { $v struct S; }; }");
    expands(&vis, "fn", "struct S;");
    let ident = rules("macro_rules! m { ($i:ident) => {}; }");
    assert_eq!(apply(&ident, "_").err(), Some(Failure::NoRuleMatched));
    let literal = rules("macro_rules! m { ($l:literal) => { const B: bool = $l; }; }");
    expands(&literal, "true", "const B: bool = true;");
}

#[test]
fn separators_split_repetitions_and_plus_needs_one_iteration() {
    let commas = rules("macro_rules! m { ($($x:ident),* $(,)?) => { $(struct $x;)* }; }");
    expands(&commas, "A, B, C", "struct A; struct B; struct C;");
    expands(&commas, "A, B,", "struct A; struct B;");
    expands(&commas, "", "");
    assert_eq!(apply(&commas, "A B").err(), Some(Failure::NoRuleMatched));
    let bars = rules("macro_rules! m { ($($x:ident)|*) => { enum E { $($x),* } }; }");
    expands(&bars, "A | B | C", "enum E { A, B, C }");
    let plus = rules("macro_rules! m { ($($x:ident)+) => { $(struct $x;)+ }; }");
    expands(&plus, "A B", "struct A; struct B;");
    assert_eq!(apply(&plus, "").err(), Some(Failure::NoRuleMatched));
}

#[test]
fn the_first_matching_rule_wins() {
    let rules = rules(
        "macro_rules! m {
            (special) => { struct First; };
            ($x:ident) => { struct Second; };
            ($x:ident) => { struct Third; };
        }",
    );
    expands(&rules, "special", "struct First;");
    expands(&rules, "other", "struct Second;");
    assert_eq!(apply(&rules, "1").err(), Some(Failure::NoRuleMatched));
}

#[test]
fn a_recursive_macro_with_inner_rules_expands_through_the_lookup() {
    let rules = rules(FIELDS);
    let lookup = |path: &syn::Path| path.is_ident("fields").then_some(&rules);
    let items = expand_items(&rules, &tokens("Point { x y z }"), &lookup).unwrap();
    let [Item::Struct(point)] = items.as_slice() else {
        panic!("the expansion is one struct");
    };
    assert_eq!(point.ident, "Point");
    assert_eq!(field_names(&point.fields), ["x", "y", "z"]);
}

#[test]
fn a_macro_that_invokes_itself_forever_stops_at_the_depth_limit() {
    let forever = rules("macro_rules! forever { () => { forever! {} }; }");
    let lookup = |path: &syn::Path| path.is_ident("forever").then_some(&forever);
    let found = expand_items(&forever, &TokenStream::new(), &lookup).err();
    assert_eq!(found, Some(Failure::DepthLimit));
    let munch = rules(
        "macro_rules! munch { () => {}; ($first:tt $($rest:tt)*) => { munch! { $($rest)* } }; }",
    );
    let lookup = |path: &syn::Path| path.is_ident("munch").then_some(&munch);
    let nested = |depth: usize| expand_items(&munch, &tokens(&"x ".repeat(depth - 1)), &lookup);
    assert!(nested(128).is_ok(), "128 nested expansions are allowed");
    assert_eq!(nested(129).err(), Some(Failure::DepthLimit));
}

#[test]
fn dollar_crate_becomes_crate() {
    let rules = rules("macro_rules! m { () => { use $crate::x; }; }");
    expands(&rules, "", "use crate::x;");
}

#[test]
fn fat_arrows_and_path_separators_match_only_when_joined() {
    let rules =
        rules("macro_rules! m { ($a:ident => $b:ident :: $c:ident) => { use $b::$c as $a; }; }");
    expands(&rules, "short => std::mem", "use std::mem as short;");
    let output = apply(&rules, "short => std::mem").unwrap();
    assert!(
        syn::parse2::<syn::ItemUse>(output).is_ok(),
        "`::` stays joined"
    );
    assert_eq!(
        apply(&rules, "short = > std::mem").err(),
        Some(Failure::NoRuleMatched)
    );
    assert_eq!(
        apply(&rules, "short => std: :mem").err(),
        Some(Failure::NoRuleMatched)
    );
}

#[test]
fn nested_repetitions_capture_sequences_of_sequences() {
    let rules = rules(
        "macro_rules! m {
            ($kind:ident: $($name:ident { $($field:ident),* });*) => {
                $(struct $name { $($field: $kind),* })*
            };
        }",
    );
    let expected = "struct A { x: u8, y: u8 } struct B {}";
    expands(&rules, "u8: A { x, y }; B {}", expected);
}

#[test]
fn repetitions_need_metavariables_that_repeat_alike() {
    let pairs =
        rules("macro_rules! m { ($($a:ident)* ; $($b:ident)*) => { $(const $a: u8 = $b;)* }; }");
    expands(&pairs, "x y ; z w", "const x: u8 = z; const y: u8 = w;");
    assert!(matches!(
        apply(&pairs, "x y ; z").err(),
        Some(Failure::Unsupported(_))
    ));
    let none = rules("macro_rules! m { ($x:ident) => { $(struct $x;)* }; }");
    assert!(matches!(
        apply(&none, "A").err(),
        Some(Failure::Unsupported(_))
    ));
    let shallow = rules("macro_rules! m { ($($x:ident)*) => { struct $x; }; }");
    assert!(matches!(
        apply(&shallow, "A").err(),
        Some(Failure::Unsupported(_))
    ));
}

#[test]
fn a_tt_takes_a_whole_compound_operator_or_lifetime() {
    let rules = rules(
        "macro_rules! m {
            ($op:tt $lt:tt $range:tt) => { fn f<$lt>(a: &$lt mut u8) { *a $op 1; } };
        }",
    );
    expands(&rules, "+= 'a ..=", "fn f<'a>(a: &'a mut u8) { *a += 1; }");
}

#[test]
fn definitions_and_unresolved_invocations_stay_items() {
    let rules = rules(
        "macro_rules! make {
            ($name:ident) => { macro_rules! $name { ($x:expr) => { $x }; } other! {} };
        }",
    );
    let lookup = |path: &syn::Path| {
        (path.is_ident("make") || path.is_ident("macro_rules")).then_some(&rules)
    };
    let items = expand_items(&rules, &tokens("inner"), &lookup).unwrap();
    let [Item::Macro(definition), Item::Macro(other)] = items.as_slice() else {
        panic!("the expansion is a definition and an invocation");
    };
    assert!(
        definition
            .ident
            .as_ref()
            .is_some_and(|ident| ident == "inner")
    );
    assert_eq!(path_text(&other.mac.path), "other");
    let inner = MacroRules::parse(&definition.mac.tokens).unwrap();
    expands(&inner, "1 + 2", "1 + 2");
}

#[test]
fn runaway_expansions_stop_at_the_budget_and_non_items_fail() {
    let dup = rules("macro_rules! dup { ($($t:tt)*) => { dup! { $($t)* $($t)* } }; }");
    let lookup = |path: &syn::Path| path.is_ident("dup").then_some(&dup);
    let found = expand_items(&dup, &tokens("a"), &lookup).err();
    assert_eq!(found, Some(Failure::TooLarge));
    let slow = rules("macro_rules! slow { ($($a:tt)* $($b:tt)* $($c:tt)* !) => {}; }");
    assert_eq!(
        apply(&slow, &"x ".repeat(300)).err(),
        Some(Failure::StepLimit)
    );
    let sum = rules("macro_rules! sum { () => { 1 + 1 }; }");
    let found = expand_items(&sum, &TokenStream::new(), &|_| None).err();
    assert!(matches!(found, Some(Failure::NotItems(_))));
}

#[test]
fn an_unsupported_rule_fails_only_the_expansions_that_reach_it() {
    let rules = rules(
        "macro_rules! m {
            (a) => { struct A; };
            (b) => { ${count(x)} };
            ($x:block2) => {};
        }",
    );
    expands(&rules, "a", "struct A;");
    assert!(matches!(
        apply(&rules, "b").err(),
        Some(Failure::Unsupported(_))
    ));
    assert!(matches!(
        apply(&rules, "c").err(),
        Some(Failure::Unsupported(_))
    ));
    assert!(MacroRules::parse(&tokens("(a) => {} junk")).is_err());
}

#[test]
fn fragments_that_reach_past_a_comma_or_brace_still_parse_whole() {
    let items = rules("macro_rules! m { ($($item:item)*) => { $(#[cfg(unix)] $item)* }; }");
    let input = "const P: Point = Point { x: 1 }; fn f() -> u8 { 1 } struct S;";
    let found = expand_items(&items, &tokens(input), &|_| None).unwrap();
    assert!(matches!(
        found.as_slice(),
        [Item::Const(_), Item::Fn(_), Item::Struct(_)]
    ));
    let exprs = rules("macro_rules! m { ($($e:expr),*) => { const A: () = ($($e,)*); }; }");
    let expected = "const A: () = (f::<u8, u16>(), |a, b| a, x < y,);";
    expands(&exprs, "f::<u8, u16>(), |a, b| a, x < y", expected);
}

#[test]
fn failures_read_as_short_lowercase_reasons() {
    let cases = [
        (
            Failure::Unsupported("a reason".to_owned()),
            "unsupported: a reason",
        ),
        (Failure::NoRuleMatched, "no rule matched"),
        (Failure::StepLimit, "the step limit was reached"),
        (Failure::DepthLimit, "the depth limit was reached"),
        (Failure::TooLarge, "the expansion is too large"),
        (
            Failure::NotItems("expected item".to_owned()),
            "the expansion is not items: expected item",
        ),
    ];
    for (failure, text) in cases {
        assert_eq!(failure.to_string(), text);
    }
}
