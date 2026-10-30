*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"attributes","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":20,"to":41}]}
{"k":"lead","id":"field-and-accessor","mode":"direct-syntax","subjects":["Record::name"],"evidence":"`pub name` and `fn name`","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"compat-shim","mode":"direct-syntax","subjects":["helper"],"evidence":"`#[deprecated]`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"compat-shim","mode":"direct-syntax","subjects":["Level::Top"],"evidence":"`#[deprecated(since = \"1.2\", note = \"use High\")]`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"lint-suppression","mode":"direct-syntax","subjects":["Record::count"],"evidence":"`#[allow(dead_code)]`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["helper"],"evidence":"no hit of `helper` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Level"],"evidence":"no hit of `Level` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Level::Low"],"evidence":"no hit of `Low` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Level::Top"],"evidence":"no hit of `Top` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Record::empty"],"evidence":"no hit of `empty` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Level::Low"],"evidence":"no observed path builds `Level::Low`; 0 patterns or comparisons match it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Level::Top"],"evidence":"no observed path builds `Level::Top`; 0 patterns or comparisons match it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"helper","kind":"fn"}
{"k":"surface","path":"Level","kind":"enum"}
{"k":"surface","path":"Record","kind":"struct"}
{"k":"surface","path":"Row","kind":"struct","item":"Record"}
{"k":"wire-keys","path":"Record","keys":["cache","count","id"],"loc":{"file":"src/lib.rs","line":0}}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Attributes that shape an API or its wire format."}
{"k":"use","path":"Row","rust":"pub use Record as Row","loc":{"file":"src/lib.rs","line":0}}
{"k":"enum","path":"Level","rust":"pub enum Level { #[deprecated(since = \"1.2\", note = \"use High\")] Top = 2, Low = 0 }","loc":{"file":"src/lib.rs","line":0},"attrs":["non_exhaustive","repr(u8)"],"derives":["Clone","Copy"]}
{"k":"doc","path":"Level","text":"A level."}
{"k":"variant","path":"Level::Top","rust":"#[deprecated(since = \"1.2\", note = \"use High\")] Top = 2","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Level::Top","text":"The top."}
{"k":"variant","path":"Level::Low","rust":"Low = 0","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Level::Low","text":"The bottom."}
{"k":"struct","path":"Record","rust":"pub struct Record { #[serde(rename = \"id\")] pub name: String, #[doc(hidden)] pub cache: u8, count: u8 }","loc":{"file":"src/lib.rs","line":0},"attrs":["serde(rename_all = \"camelCase\")"],"derives":["serde::Serialize"],"reexported_by":["crate"],"token_hits":4}
{"k":"doc","path":"Record","text":"A record with a wire format."}
{"k":"field","path":"Record::name","rust":"#[serde(rename = \"id\")] pub name: String","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Record::name","text":"The record's name."}
{"k":"field","path":"Record::cache","rust":"#[doc(hidden)] pub cache: u8","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Record::cache","text":"Kept out of the docs."}
{"k":"impl","path":"Record","rust":"impl Record","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Record::name","sig":"pub fn name(&self) -> &str","loc":{"file":"src/lib.rs","line":0},"attrs":["must_use","track_caller"],"token_hits":2,"body":{"statements":1,"tokens":6,"single":"field name"},"ops":["self.name"]}
{"k":"doc","path":"Record::name","text":"Reads the name."}
{"k":"fn","path":"Record::empty","sig":"pub fn empty() -> Record","loc":{"file":"src/lib.rs","line":0},"attrs":["must_use = \"a record does nothing until it is stored\""],"body":{"statements":1,"tokens":19}}
{"k":"doc","path":"Record::empty","text":"Makes an empty record."}
{"k":"fn","path":"helper","sig":"pub fn helper()","loc":{"file":"src/lib.rs","line":0},"attrs":["deprecated","doc(hidden)"],"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"helper","text":"A hidden helper."}
