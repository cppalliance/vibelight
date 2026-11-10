//! Tests the matcher at its edges: where a `stmt` ends, the spacing a
//! punctuation must have, separators of more than one character, an empty
//! iteration before a separator, shallower metavariables inside deeper
//! repetitions, lifetimes and negative literals in repetitions, `$` at the
//! end of a matcher, `expr_2021`, and the budget on long lists.

use crate::macro_rules::Failure;
use crate::macro_rules::tests::{apply, expands, rules};

#[test]
fn a_stmt_ends_before_a_semicolon_it_does_not_own() {
    let list = rules("macro_rules! m { ($($s:stmt);*) => { fn f() { $($s;)* } }; }");
    expands(
        &list,
        "let a = 1; let b = 2",
        "fn f() { let a = 1; let b = 2; }",
    );
    let one = rules("macro_rules! m { ($s:stmt) => { fn f() { $s; } }; }");
    expands(&one, "x + 1", "fn f() { x + 1; }");
    let closed = rules("macro_rules! m { ($s:stmt ;) => { fn f() { $s; } }; }");
    expands(&closed, "let a = 1;", "fn f() { let a = 1; }");
    let item = rules("macro_rules! m { ($s:stmt) => { $s }; }");
    expands(&item, "struct S;", "struct S;");
    let comma = rules("macro_rules! m { ($s:stmt , end) => { fn f() { $s; } }; }");
    let generic = "let m: Map<K, V> = make() , end";
    expands(&comma, generic, "fn f() { let m: Map<K, V> = make(); }");
}

#[test]
fn a_lone_punctuation_never_matches_half_of_a_compound() {
    let colons = rules(
        "macro_rules! m {
            ($a:ident : $($r:tt)*) => { struct Colon; };
            ($a:ident :: $b:ident) => { struct Path; };
        }",
    );
    expands(&colons, "x::y", "struct Path;");
    expands(&colons, "x: y", "struct Colon;");
    let split = rules("macro_rules! m { ($a:ident > > $b:ident) => { struct Two; }; }");
    assert_eq!(apply(&split, "x >> y").err(), Some(Failure::NoRuleMatched));
    expands(&split, "x > > y", "struct Two;");
}

#[test]
fn separators_of_more_than_one_character_split_repetitions() {
    let arrows = rules("macro_rules! m { ($($a:ident)=>*) => { $(struct $a;)* }; }");
    expands(&arrows, "A => B => C", "struct A; struct B; struct C;");
    assert_eq!(
        apply(&arrows, "A = > B").err(),
        Some(Failure::NoRuleMatched)
    );
    let paths = rules("macro_rules! m { ($($a:ident)::+) => { use $($a)::+; }; }");
    expands(&paths, "std::mem::swap", "use std::mem::swap;");
}

#[test]
fn an_empty_iteration_goes_on_past_a_separator() {
    let vis = rules("macro_rules! m { ($($v:vis),*) => { $($v struct S;)* }; }");
    expands(&vis, ", pub", "struct S; pub struct S;");
    expands(&vis, "pub(crate),", "pub(crate) struct S; struct S;");
}

#[test]
fn a_shallower_metavariable_repeats_inside_a_deeper_repetition() {
    let rules = rules(
        "macro_rules! m {
            ($($group:ident ($($item:ident),*));*) => { $($(fn $item() -> $group {})*)* };
        }",
    );
    let expected = "fn x() -> A {} fn y() -> A {} fn z() -> B {}";
    expands(&rules, "A (x, y); B (z)", expected);
}

#[test]
fn lifetimes_and_negative_literals_repeat() {
    let rules = rules(
        "macro_rules! m {
            ($($l:lifetime),* ; $($n:literal),*) => { fn f<$($l),*>() -> [i8; 2] { [$($n),*] } };
        }",
    );
    let expected = "fn f<'a, 'b>() -> [i8; 2] { [-1, 2] }";
    expands(&rules, "'a, 'b ; -1, 2", expected);
}

#[test]
fn a_trailing_dollar_is_a_literal_and_a_double_dollar_is_unsupported() {
    let trailing = rules("macro_rules! m { (a $) => { struct D; }; }");
    expands(&trailing, "a $", "struct D;");
    let double = rules("macro_rules! m { ($$) => {}; }");
    assert!(matches!(apply(&double, "$"), Err(Failure::Unsupported(_))));
}

#[test]
fn expr_2021_leaves_out_an_underscore_and_a_const_block() {
    let old = rules(
        "macro_rules! m { ($e:expr_2021) => { struct Old; }; ($($t:tt)*) => { struct Other; }; }",
    );
    expands(&old, "a + 1", "struct Old;");
    expands(&old, "_", "struct Other;");
    expands(&old, "const { 1 }", "struct Other;");
    let new = rules("macro_rules! m { ($e:expr) => { struct New; }; }");
    expands(&new, "_", "struct New;");
}

#[test]
fn long_lists_stay_within_the_budget() {
    let types = rules("macro_rules! m { ($($t:ty)|*) => { $(impl Marker for $t {})* }; }");
    let names: Vec<String> = (0..8000).map(|n| format!("T{n}")).collect();
    assert!(apply(&types, &names.join(" | ")).is_ok());
    let paths = rules("macro_rules! m { ($($p:path)+) => {}; }");
    let found = apply(&paths, &"a ".repeat(4000)).err();
    assert_eq!(
        found,
        Some(Failure::StepLimit),
        "each parse spends a step per token read"
    );
}
