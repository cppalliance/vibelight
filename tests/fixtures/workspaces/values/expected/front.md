*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":2,"crate":"front","target":"lib","dir":"crates/front"}
{"k":"modules","items":[{"path":"crate","from":12,"to":20}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["level_of"],"evidence":"no hit of `level_of` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["sample"],"evidence":"no hit of `sample` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"level_of","kind":"fn"}
{"k":"surface","path":"MAX","kind":"external","item":"relay::LIMIT"}
{"k":"surface","path":"measure","kind":"external","item":"relay::measure"}
{"k":"surface","path":"sample","kind":"fn"}
{"k":"surface","path":"TOTAL","kind":"external","item":"relay::TOTAL"}
{"k":"external","path":"relay::Reading","uses":1,"named_by":["crate"],"summary":"A reading and how it was taken."}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","functions":2,"reexports":3,"mix":{"pub":5}}
{"k":"doc","path":"crate","text":"Front: re-exports what relay re-exports."}
{"k":"use","path":"MAX","rust":"pub use relay::LIMIT as MAX","loc":"src/lib.rs:0"}
{"k":"use","path":"TOTAL","rust":"pub use relay::TOTAL","loc":"src/lib.rs:0"}
{"k":"use","path":"measure","rust":"pub use relay::measure","loc":"src/lib.rs:0"}
{"k":"fn","path":"sample","sig":"pub fn sample() -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"call measure"}}
{"k":"doc","path":"sample","text":"Takes a reading with [`measure`], which relay re-exports from origin."}
{"k":"fn","path":"level_of","sig":"pub fn level_of(reading: &relay::Reading) -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field level"},"ops":["reading"]}
{"k":"doc","path":"level_of","text":"Returns the level of a reading, a type that front does not re-export."}
