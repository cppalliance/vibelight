*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"relay","target":"lib","dir":"crates/relay"}
{"k":"modules","items":[{"path":"crate","from":9,"to":15}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["read"],"evidence":"no hit of `read` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"LIMIT","kind":"external","item":"origin::gauge::LIMIT"}
{"k":"surface","path":"measure","kind":"external","item":"origin::gauge::measure"}
{"k":"surface","path":"read","kind":"fn"}
{"k":"surface","path":"TOTAL","kind":"external","item":"origin::gauge::TOTAL"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Relay: re-exports origin's values and links to one in a comment."}
{"k":"use","path":"LIMIT","rust":"pub use origin::gauge::LIMIT","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"TOTAL","rust":"pub use origin::gauge::TOTAL","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"measure","rust":"pub use origin::gauge::measure","loc":{"file":"src/lib.rs","line":0}}
{"k":"fn","path":"read","sig":"pub fn read() -> u32","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":5,"single":"call measure"}}
{"k":"doc","path":"read","text":"Takes a reading through [`origin::gauge::measure`]."}
