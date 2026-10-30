*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"member_order","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":20,"to":50}]}
{"k":"lead","id":"lint-suppression","mode":"direct-syntax","subjects":["Scrambled"],"evidence":"`#[allow(non_snake_case)]`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Grouped","Grouped::build","Grouped::LIMIT","Grouped::member","Grouped::Out","Order","Order::Alpha","Order::Mike","Order::Zulu","Scrambled::Alpha","Scrambled::alpha","Scrambled::Apply","Scrambled::apply","Scrambled::make","Scrambled::MAX","Scrambled::zed"],"evidence":"no hit of `Grouped` in the scanned sources beyond its 1 declaration; no hit of `build` in the scanned sources beyond its 1 declaration; no hit of `LIMIT` in the scanned sources beyond its 1 declaration; no hit of `member` in the scanned sources beyond its 1 declaration; no hit of `Out` in the scanned sources beyond its 1 declaration; no hit of `Order` in the scanned sources beyond its 1 declaration; no hit of `Alpha` in the scanned sources ","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":18}
{"k":"more","path":"Grouped","id":"no-observed-name-use","field":"subjects","part":2,"items":["Scrambled::zeta","Triple"]}
{"k":"more","path":"Grouped","id":"no-observed-name-use","field":"evidence","part":3,"text":"beyond its 2 declarations; no hit of `Mike` in the scanned sources beyond its 1 declaration; no hit of `Zulu` in the scanned sources beyond its 1 declaration; no hit of `alpha` in the scanned sources beyond its 1 declaration; no hit of `Apply` in the scanned sources beyond its 1 declaration; no hit of `apply` in the scanned sources beyond its 1 declaration; no hit of `make` in the scanned sources beyond its 1 declaration; no hit of `MAX` in the scanned sources beyond its 1 declaration; no hit of `zed` in the scanned sources beyond its 1 declaration; no hit of `zeta` in the scanned sources beyond its 1 declaration; no hit of `Triple` in the scanned sources beyond its 1 declaration"}
{"k":"more","path":"Grouped","id":"no-observed-name-use","field":"locs","part":4,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Order::Alpha"],"evidence":"no observed path builds `Order::Alpha`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Order::Mike"],"evidence":"no observed path builds `Order::Mike`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Order::Zulu"],"evidence":"no observed path builds `Order::Zulu`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Scrambled::Alpha"],"evidence":"0 struct literals of `Scrambled` set it, and no `.Alpha` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Scrambled::alpha"],"evidence":"0 struct literals of `Scrambled` set it, and no `.alpha` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Scrambled::zeta"],"evidence":"0 struct literals of `Scrambled` set it, and no `.zeta` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Based","kind":"struct"}
{"k":"surface","path":"Grouped","kind":"trait"}
{"k":"surface","path":"Order","kind":"enum"}
{"k":"surface","path":"Scrambled","kind":"struct"}
{"k":"surface","path":"Triple","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":5}
{"k":"doc","path":"crate","text":"Member order inside a block."}
{"k":"struct","path":"Scrambled","rust":"pub struct Scrambled { pub zeta: u8, pub alpha: u8, beta: u8, pub Alpha: u8 }","loc":"src/lib.rs:0","lints":["allow(non_snake_case)"]}
{"k":"doc","path":"Scrambled","text":"Members in scrambled order."}
{"k":"impl","path":"Scrambled","rust":"impl Scrambled","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Scrambled::zed","sig":"pub fn zed(&self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"const","path":"Scrambled::MAX","rust":"pub const MAX: u8","value":"9","loc":"src/lib.rs:0","body":{"statements":1,"tokens":1}}
{"k":"fn","path":"Scrambled::make","sig":"pub fn make() -> Scrambled","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3,"panics":1,"macros":["todo"]}}
{"k":"fn","path":"Scrambled::hidden","sig":"fn hidden()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Scrambled::apply","sig":"pub fn apply(&self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Scrambled::Apply","sig":"pub fn Apply(&self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"trait","path":"Grouped","rust":"pub trait Grouped","loc":"src/lib.rs:0"}
{"k":"doc","path":"Grouped","text":"Trait items in scrambled order."}
{"k":"fn","path":"Grouped::member","sig":"fn member(&self)","loc":"src/lib.rs:0"}
{"k":"fn","path":"Grouped::build","sig":"fn build() -> Self","loc":"src/lib.rs:0"}
{"k":"const","path":"Grouped::LIMIT","rust":"const LIMIT: u8","loc":"src/lib.rs:0"}
{"k":"alias","path":"Grouped::Out","rust":"type Out","loc":"src/lib.rs:0"}
{"k":"enum","path":"Order","rust":"pub enum Order { Zulu, Alpha, Mike }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Order","text":"Variants keep their declared order."}
{"k":"struct","path":"Triple","rust":"pub struct Triple(pub u8, pub String, pub bool)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Triple","text":"Tuple fields keep their declared order."}
{"k":"struct","path":"Based","rust":"pub struct Based","loc":"src/lib.rs:0"}
{"k":"doc","path":"Based","text":"The base list is sorted."}
{"k":"impl","path":"Based","rust":"impl Default for Based","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Based::default","sig":"fn default() -> Based","loc":"src/lib.rs:0","impl":"impl Default for Based","body":{"statements":1,"tokens":3}}
{"k":"impl","path":"Based","rust":"impl std::fmt::Debug for Based","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Based::fmt","sig":"fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result","loc":"src/lib.rs:0","impl":"impl std::fmt::Debug for Based","body":{"statements":1,"tokens":7,"single":"call Ok"}}
{"k":"impl","path":"Based","rust":"impl Clone for Based","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Based::clone","sig":"fn clone(&self) -> Based","loc":"src/lib.rs:0","impl":"impl Clone for Based","body":{"statements":1,"tokens":3}}
{"k":"impl","path":"Based","rust":"impl AsRef<str> for Based","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Based::as_ref","sig":"fn as_ref(&self) -> &str","loc":"src/lib.rs:0","impl":"impl AsRef<str> for Based","body":{"statements":1,"tokens":3}}
