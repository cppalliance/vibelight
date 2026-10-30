*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"special_members","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":21,"to":56}]}
{"k":"lead","id":"redundant-constructor","mode":"direct-syntax","subjects":["Written","Written::new"],"evidence":"`new()` beside a written Default, and neither calls the other","locs":["src/lib.rs:0","src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Derived","Mode","Mode::Off","Mode::On","Written::get","Written::new"],"evidence":"no hit of `Derived` in the scanned sources beyond its 1 declaration; no hit of `Mode` in the scanned sources beyond its 1 declaration; no hit of `Off` in the scanned sources beyond its 1 declaration; no hit of `On` in the scanned sources beyond its 1 declaration; no hit of `get` in the scanned sources beyond its 1 declaration; no hit of `new` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":6}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Counter"],"evidence":"1 production hits of `Counter` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Handle"],"evidence":"1 production hits of `Handle` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Scheduler"],"evidence":"1 production hits of `Scheduler` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Mode::Off"],"evidence":"no observed path builds `Mode::Off`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Mode::On"],"evidence":"no observed path builds `Mode::On`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Name"],"evidence":"2 observed uses, all inside `special_members`; no other target of the run writes `Name`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Written"],"evidence":"8 observed uses, all inside `special_members`; no other target of the run writes `Written`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"surface","path":"Counter","kind":"struct"}
{"k":"surface","path":"Derived","kind":"struct"}
{"k":"surface","path":"Handle","kind":"struct"}
{"k":"surface","path":"Mode","kind":"enum"}
{"k":"surface","path":"Name","kind":"struct"}
{"k":"surface","path":"Scheduler","kind":"struct"}
{"k":"surface","path":"Written","kind":"struct"}
{"k":"wire-keys","path":"Derived","keys":["value"],"loc":"src/lib.rs:0"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":7}
{"k":"doc","path":"crate","text":"Special members: the standard traits that become C++ constructors and\ndestructors."}
{"k":"struct","path":"Derived","rust":"pub struct Derived { pub value: u8 }","loc":"src/lib.rs:0","derives":["Default","serde::Serialize"]}
{"k":"doc","path":"Derived","text":"The one derivable special member, and a derive from another crate."}
{"k":"struct","path":"Written","rust":"pub struct Written { pub value: u8 }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Written","text":"Every hand-written special member."}
{"k":"impl","path":"Written","rust":"impl Default for Written","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Written::default","sig":"fn default() -> Self","loc":"src/lib.rs:0","impl":"impl Default for Written","body":{"statements":1,"tokens":8}}
{"k":"impl","path":"Written","rust":"impl From<u8> for Written","locs":["src/lib.rs:0"]}
{"k":"doc","path":"Written","text":"Converts from a byte."}
{"k":"fn","path":"Written::from","sig":"fn from(value: u8) -> Self","loc":"src/lib.rs:0","impl":"impl From<u8> for Written","body":{"statements":1,"tokens":6},"ops":["value"]}
{"k":"impl","path":"Written","rust":"impl From<&str> for Written","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Written::from","sig":"fn from(text: &str) -> Self","loc":"src/lib.rs:0","impl":"impl From<&str> for Written","body":{"statements":1,"tokens":14},"ops":["text.len"]}
{"k":"impl","path":"Written","rust":"impl Drop for Written","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Written::drop","sig":"fn drop(&mut self)","loc":"src/lib.rs:0","impl":"impl Drop for Written","body":{"statements":0,"tokens":2}}
{"k":"impl","path":"Written","rust":"impl Written","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Written::new","sig":"pub fn new() -> Self","loc":"src/lib.rs:0","body":{"statements":1,"tokens":8}}
{"k":"fn","path":"Written::get","sig":"pub fn get(&self) -> u8","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field value"},"ops":["self.value"]}
{"k":"struct","path":"Name","rust":"pub struct Name(String)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Name","text":"A name from anything that converts into a String."}
{"k":"impl","path":"Name","rust":"impl<T: Into<String>> From<T> for Name","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Name::from","sig":"fn from(value: T) -> Self","loc":"src/lib.rs:0","impl":"impl<T: Into<String>> From<T> for Name","body":{"statements":1,"tokens":10,"single":"call Name"},"ops":["value.into"]}
{"k":"enum","path":"Mode","rust":"pub enum Mode { On, Off }","loc":"src/lib.rs:0","derives":["Default"]}
{"k":"doc","path":"Mode","text":"An enum with a derived default."}
{"k":"struct","path":"Handle","rust":"pub struct Handle(u32)","loc":"src/lib.rs:0","derives":["Default"]}
{"k":"doc","path":"Handle","text":"A tuple struct with a derived and a hand-written member."}
{"k":"impl","path":"Handle","rust":"impl Drop for Handle","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Handle::drop","sig":"fn drop(&mut self)","loc":"src/lib.rs:0","impl":"impl Drop for Handle","body":{"statements":0,"tokens":2}}
{"k":"struct","path":"Scheduler","rust":"pub struct Scheduler { jobs: Vec<u32> }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Scheduler","text":"A class with a hand-written destructor."}
{"k":"impl","path":"Scheduler","rust":"impl Drop for Scheduler","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Scheduler::drop","sig":"fn drop(&mut self)","loc":"src/lib.rs:0","impl":"impl Drop for Scheduler","body":{"statements":0,"tokens":2}}
{"k":"struct","path":"Counter","rust":"pub struct Counter { count: u32 }","loc":"src/lib.rs:0","derives":["Default"]}
{"k":"doc","path":"Counter","text":"A struct with a private section."}
{"k":"impl","path":"Counter","rust":"impl Counter","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Counter::count","sig":"pub fn count(&self) -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field count"},"ops":["self.count"]}
