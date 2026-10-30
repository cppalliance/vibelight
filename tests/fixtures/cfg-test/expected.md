*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"cfg_test","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":11,"to":26}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Kept::get","Mode","Mode::Debug","Mode::On","Probe","Probe::check","production_only"],"evidence":"no hit of `get` in the scanned sources beyond its 1 declaration; no hit of `Mode` in the scanned sources beyond its 1 declaration; no hit of `Debug` in the scanned sources beyond its 1 declaration; no hit of `On` in the scanned sources beyond its 1 declaration; no hit of `Probe` in the scanned sources beyond its 1 declaration; no hit of `check` in the scanned sources beyond its 1 declaration; no hit of `production_only` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":7}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Mode::Debug"],"evidence":"no observed path builds `Mode::Debug`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Mode::On"],"evidence":"no observed path builds `Mode::On`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Kept","kind":"struct"}
{"k":"surface","path":"Mode","kind":"enum"}
{"k":"surface","path":"Probe","kind":"trait"}
{"k":"surface","path":"production_only","kind":"fn"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":3,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Test-only items are skipped."}
{"k":"struct","path":"Kept","rust":"pub struct Kept { pub value: u32 }","loc":"src/lib.rs:0","token_hits":2}
{"k":"doc","path":"Kept","text":"Kept."}
{"k":"field","path":"Kept::value","rust":"pub value: u32","loc":"src/lib.rs:0"}
{"k":"doc","path":"Kept::value","text":"A kept field."}
{"k":"enum","path":"Mode","rust":"pub enum Mode { On, #[cfg(any(test, feature = \"debug\"))] Debug }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Mode","text":"A kept enum."}
{"k":"impl","path":"Kept","rust":"impl Kept","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Kept::get","sig":"pub fn get(&self) -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field value"},"ops":["self.value"]}
{"k":"doc","path":"Kept::get","text":"A kept method."}
{"k":"trait","path":"Probe","rust":"pub trait Probe","loc":"src/lib.rs:0"}
{"k":"doc","path":"Probe","text":"A test-only trait item stays out."}
{"k":"fn","path":"Probe::check","sig":"fn check(&self)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Probe::check","text":"Kept."}
{"k":"fn","path":"production_only","sig":"pub fn production_only()","loc":"src/lib.rs:0","cfgs":["not(test)"],"body":{"statements":0,"tokens":2}}
