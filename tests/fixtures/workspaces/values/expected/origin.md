*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":2,"crate":"origin","target":"lib","dir":"crates/origin"}
{"k":"modules","items":[{"path":"crate","from":14,"to":15},{"path":"gauge","vis":"pub","from":16,"to":37}]}
{"k":"lead","id":"pub-mod-surface","mode":"resolved","subjects":["gauge"],"evidence":"`pub mod` on the surface: its `pub` items reach users by its path","locs":["src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["gauge::Reading::level"],"evidence":"no hit of `level` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["gauge::Reading::level"],"evidence":"0 struct literals of `Reading` set it, and no `.level` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"gauge","kind":"mod"}
{"k":"surface","path":"gauge::LIMIT","kind":"value"}
{"k":"surface","path":"gauge::Mark","kind":"struct"}
{"k":"surface","path":"gauge::measure","kind":"fn"}
{"k":"surface","path":"gauge::Reading","kind":"struct"}
{"k":"surface","path":"gauge::Tap","kind":"struct"}
{"k":"surface","path":"gauge::TOTAL","kind":"value"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0"}
{"k":"doc","path":"crate","text":"Origin: defines the values other crates re-export."}
{"k":"mod","path":"gauge","loc":"src/lib.rs:0","types":3,"functions":1,"values":2,"mix":{"pub":6}}
{"k":"doc","path":"gauge","text":"Gauge values."}
{"k":"const","path":"gauge::LIMIT","rust":"pub const LIMIT: u32","value":"8","loc":"src/lib.rs:0","reexported_by":["::front","::relay"],"crates":["front","relay"],"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"gauge::LIMIT","text":"The highest reading."}
{"k":"static","path":"gauge::TOTAL","rust":"pub static TOTAL: u32","value":"0","loc":"src/lib.rs:0","reexported_by":["::front","::relay"],"crates":["front","relay"],"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"gauge::TOTAL","text":"A running total."}
{"k":"fn","path":"gauge::measure","sig":"pub fn measure() -> u32","loc":"src/lib.rs:0","direct_callers":["::front::sample","::relay::read"],"reexported_by":["::front","::relay"],"crates":["front","relay"],"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"gauge::measure","text":"Takes one reading."}
{"k":"struct","path":"gauge::Reading","rust":"pub struct Reading { pub level: u32, tries: u32, tap: Tap, marks: Vec<Mark> }","loc":"src/lib.rs:0","reexported_by":["::relay"],"crates":["relay"]}
{"k":"doc","path":"gauge::Reading","text":"A reading and how it was taken."}
{"k":"field","path":"gauge::Reading::level","rust":"pub level: u32","loc":"src/lib.rs:0"}
{"k":"doc","path":"gauge::Reading::level","text":"The level read."}
{"k":"field","path":"gauge::Reading::tries","rust":"tries: u32","loc":"src/lib.rs:0"}
{"k":"doc","path":"gauge::Reading::tries","text":"How many tries it took."}
{"k":"field","path":"gauge::Reading::tap","rust":"tap: Tap","loc":"src/lib.rs:0"}
{"k":"doc","path":"gauge::Reading::tap","text":"The tap it came from."}
{"k":"field","path":"gauge::Reading::marks","rust":"marks: Vec<Mark>","loc":"src/lib.rs:0"}
{"k":"doc","path":"gauge::Reading::marks","text":"The marks made on the way."}
{"k":"struct","path":"gauge::Tap","rust":"pub struct Tap(u32)","loc":"src/lib.rs:0","named_by":["gauge::Reading"],"reexported_by":["::relay"],"crates":["relay"]}
{"k":"doc","path":"gauge::Tap","text":"Where a reading comes from."}
{"k":"struct","path":"gauge::Mark","rust":"pub struct Mark","loc":"src/lib.rs:0","named_by":["gauge::Reading"]}
{"k":"doc","path":"gauge::Mark","text":"A mark made while reading."}
