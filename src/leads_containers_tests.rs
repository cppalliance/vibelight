//! Tests each detector of containers and access profiles on a small
//! library where it finds its lead, and on a close one where it finds
//! none.

use crate::leads_tests::lib;

/// Asserts what `id` finds in a library and that it finds nothing in a
/// close one.
fn finds(id: &str, positive: &str, expected: &[&str], negative: &str) {
    assert_eq!(lib(positive, id), expected, "{id} on its case");
    assert_eq!(
        lib(negative, id),
        Vec::<String>::new(),
        "{id} on the close case"
    );
}

/// The imports the cases share.
const USES: &str = "use std::cell::RefCell;\n\
    use std::collections::{BTreeMap, HashMap, HashSet, LinkedList, VecDeque};\n\
    use std::rc::Rc;\n\
    use std::sync::Arc;\n\
    pub struct Id(u32);\n\
    pub struct Job;\n";

/// Returns a library of the shared imports and `rest`.
fn with_uses(rest: &str) -> String {
    format!("{USES}{rest}")
}

#[test]
fn a_linked_list_a_list_of_pairs_a_map_of_maps_and_a_shared_cell_smell() {
    finds(
        "container-smell",
        &with_uses(
            "pub struct A { pub queue: LinkedList<u8>, pub names: Vec<(String, Job)>,\n\
             pub nested: HashMap<String, BTreeMap<u8, u8>>, pub shared: Rc<RefCell<Job>> }\n",
        ),
        &["A::names", "A::nested", "A::queue", "A::shared"],
        &with_uses(
            "pub struct A { pub queue: VecDeque<u8>, pub spans: Vec<(usize, usize)>,\n\
             pub nested: HashMap<String, Vec<u8>>, pub shared: Rc<Job>, pub cell: RefCell<Job> }\n",
        ),
    );
}

#[test]
fn collections_of_one_type_keyed_by_one_identity_are_parallel() {
    finds(
        "parallel-collections",
        &with_uses(
            "pub struct Table { pub jobs: HashMap<Id, Job>, pub done: Vec<Id>, pub held: HashSet<Id> }\n",
        ),
        &["Table::done, Table::held, Table::jobs"],
        &with_uses(
            "pub struct Lists { pub pending: Vec<Job>, pub done: Vec<Job>,\n\
             pub names: HashMap<String, u8>, pub ages: HashMap<String, u32> }\n",
        ),
    );
}

#[test]
fn a_key_held_across_owned_types_is_found_once_on_the_outermost_owner() {
    let types = |queue: &str| {
        with_uses(&format!(
            "pub struct Table {{ jobs: HashMap<Id, Job> }}\n\
             pub struct Queue {{ ready: VecDeque<Id> }}\n\
             pub struct Engine {{ table: Table, queue: {queue} }}\n\
             pub struct Outer {{ engine: Box<Engine> }}\n"
        ))
    };
    finds(
        "shared-key",
        &types("Queue"),
        &["Outer, Queue::ready, Table::jobs"],
        &types("Arc<Queue>"),
    );
}

#[test]
fn a_list_that_methods_scan_or_a_loop_searches_is_a_linear_search() {
    let store = "pub struct Item { pub id: u32 }\npub struct Store { items: Vec<Item> }\n";
    finds(
        "linear-search",
        &format!(
            "{store}impl Store {{\n\
             pub fn has(&self, id: u32) -> bool {{ self.items.iter().any(|i| i.id == id) }}\n\
             pub fn get(&self, id: u32) -> Option<&Item> {{ self.items.iter().find(|i| i.id == id) }}\n\
             pub fn count(&self, wanted: &[u32], all: &[u32]) -> usize {{\n\
             let mut n = 0; for t in all {{ if wanted.contains(t) {{ n += 1; }} }} n }}\n\
             }}\n"
        ),
        &["Store::count", "Store::get, Store::has, Store::items"],
        &format!(
            "{store}impl Store {{\n\
             pub fn has(&self, id: u32) -> bool {{ self.items.iter().any(|i| i.id == id) }}\n\
             pub fn known(&self, wanted: &[u32], t: u32) -> bool {{ wanted.contains(&t) }}\n\
             }}\n"
        ),
    );
}

#[test]
fn removing_or_inserting_at_the_front_of_a_vec_is_a_lead() {
    finds(
        "front-ops-on-vec",
        "pub struct Queue { items: Vec<u8> }\n\
         impl Queue {\n\
         pub fn pop(&mut self) -> u8 { self.items.remove(0) }\n\
         pub fn shift(&mut self, x: u8) { self.items.insert(0, x); }\n\
         }\n\
         pub fn first(v: &mut Vec<u8>) -> u8 { v.remove(0) }\n",
        &[
            "first",
            "Queue::items, Queue::pop",
            "Queue::items, Queue::shift",
        ],
        "pub struct Queue { items: std::collections::VecDeque<u8>, list: Vec<u8> }\n\
         impl Queue {\n\
         pub fn pop(&mut self) -> Option<u8> { self.items.remove(0) }\n\
         pub fn second(&mut self) -> u8 { self.list.remove(1) }\n\
         }\n",
    );
}

#[test]
fn a_vec_sorted_after_every_push_is_a_lead_but_a_batch_is_not() {
    let board = "pub struct Board { scores: Vec<u32> }\n";
    finds(
        "sort-after-push",
        &format!(
            "{board}impl Board {{ pub fn add(&mut self, s: u32) {{ self.scores.push(s); self.scores.sort(); }} }}\n"
        ),
        &["Board::add, Board::scores"],
        &format!(
            "{board}impl Board {{ pub fn fill(&mut self, all: &[u32]) {{\n\
             for &s in all {{ self.scores.push(s); }} self.scores.sort(); }} }}\n"
        ),
    );
}

#[test]
fn a_hash_map_iterated_in_sorted_order_is_a_lead() {
    finds(
        "sorted-unordered-map",
        &with_uses(
            "pub struct Index { map: HashMap<String, u32> }\n\
             impl Index { pub fn names(&self) -> Vec<&String> { self.map.keys().sorted().collect() } }\n",
        ),
        &["Index::map, Index::names"],
        &with_uses(
            "pub struct Index { map: HashMap<String, u32>, tree: BTreeMap<String, u32> }\n\
             impl Index {\n\
             pub fn names(&self) -> Vec<&String> { self.map.keys().collect() }\n\
             pub fn ordered(&self) -> Vec<&String> { self.tree.keys().sorted().collect() }\n\
             }\n",
        ),
    );
}

#[test]
fn collections_that_the_same_methods_mutate_are_co_mutated() {
    let methods = "pub fn add(&mut self, id: u32, name: String, age: u8) {\n\
         self.names.insert(id, name); self.ages.insert(id, age); self.log.push(id); }\n\
         pub fn drop_id(&mut self, id: u32) { self.names.remove(&id); self.ages.remove(&id); }\n";
    let reg = |more: &str| {
        with_uses(&format!(
            "pub struct Reg {{ names: HashMap<u32, String>, ages: HashMap<u32, u8>, log: Vec<u32> }}\n\
             impl Reg {{\n{methods}{more}}}\n"
        ))
    };
    finds(
        "co-mutated",
        &reg(""),
        &["Reg::ages, Reg::names"],
        &reg("pub fn rename(&mut self, id: u32, name: String) { self.names.insert(id, name); }\n"),
    );
}
