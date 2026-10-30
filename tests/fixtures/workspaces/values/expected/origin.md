*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"origin","target":"lib","dir":"crates/origin"}
{"k":"modules","items":[{"path":"crate","from":8,"to":9},{"path":"gauge","from":10,"to":17}]}
{"k":"surface","path":"gauge","kind":"mod"}
{"k":"surface","path":"gauge::LIMIT","kind":"value"}
{"k":"surface","path":"gauge::measure","kind":"fn"}
{"k":"surface","path":"gauge::TOTAL","kind":"value"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0"}
{"k":"doc","path":"crate","text":"Origin: defines the values other crates re-export."}
{"k":"mod","path":"gauge","loc":"src/lib.rs:0","vis":"public","functions":{"public":1}}
{"k":"doc","path":"gauge","text":"Gauge values."}
{"k":"const","path":"gauge::LIMIT","rust":"pub const LIMIT: u32","value":"8","loc":"src/lib.rs:0","reexported_by":["::front","::relay"],"crates":["front","relay"],"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"gauge::LIMIT","text":"The highest reading."}
{"k":"static","path":"gauge::TOTAL","rust":"pub static TOTAL: u32","value":"0","loc":"src/lib.rs:0","reexported_by":["::front","::relay"],"crates":["front","relay"],"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"gauge::TOTAL","text":"A running total."}
{"k":"fn","path":"gauge::measure","sig":"pub fn measure() -> u32","loc":"src/lib.rs:0","direct_callers":["::front::sample","::relay::read"],"reexported_by":["::front","::relay"],"crates":["front","relay"],"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"gauge::measure","text":"Takes one reading."}
