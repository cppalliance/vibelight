*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"front","target":"lib","dir":"crates/front"}
{"k":"modules","items":[{"path":"crate","from":9,"to":15}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["sample"],"evidence":"no hit of `sample` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"MAX","kind":"external","item":"relay::LIMIT"}
{"k":"surface","path":"measure","kind":"external","item":"relay::measure"}
{"k":"surface","path":"sample","kind":"fn"}
{"k":"surface","path":"TOTAL","kind":"external","item":"relay::TOTAL"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Front: re-exports what relay re-exports."}
{"k":"use","path":"MAX","rust":"pub use relay::LIMIT as MAX","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"TOTAL","rust":"pub use relay::TOTAL","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"measure","rust":"pub use relay::measure","loc":{"file":"src/lib.rs","line":0}}
{"k":"fn","path":"sample","sig":"pub fn sample() -> u32","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":5,"single":"call measure"}}
{"k":"doc","path":"sample","text":"Takes a reading with [`measure`], which relay re-exports from origin."}
