*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":2,"crate":"relay","target":"lib","dir":"crates/relay"}
{"k":"modules","items":[{"path":"crate","from":11,"to":19}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["read"],"evidence":"no hit of `read` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"LIMIT","kind":"external","item":"origin::gauge::LIMIT"}
{"k":"surface","path":"measure","kind":"external","item":"origin::gauge::measure"}
{"k":"surface","path":"read","kind":"fn"}
{"k":"surface","path":"Reading","kind":"external","item":"origin::gauge::Reading"}
{"k":"surface","path":"Tap","kind":"external","item":"origin::gauge::Tap"}
{"k":"surface","path":"TOTAL","kind":"external","item":"origin::gauge::TOTAL"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","functions":1,"reexports":5,"mix":{"pub":6}}
{"k":"doc","path":"crate","text":"Relay: re-exports origin's values and links to one in a comment."}
{"k":"use","path":"LIMIT","rust":"pub use origin::gauge::LIMIT","loc":"src/lib.rs:0"}
{"k":"use","path":"Reading","rust":"pub use origin::gauge::Reading","loc":"src/lib.rs:0"}
{"k":"use","path":"TOTAL","rust":"pub use origin::gauge::TOTAL","loc":"src/lib.rs:0"}
{"k":"use","path":"Tap","rust":"pub use origin::gauge::Tap","loc":"src/lib.rs:0"}
{"k":"use","path":"measure","rust":"pub use origin::gauge::measure","loc":"src/lib.rs:0"}
{"k":"fn","path":"read","sig":"pub fn read() -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"call measure"}}
{"k":"doc","path":"read","text":"Takes a reading through [`origin::gauge::measure`]."}
