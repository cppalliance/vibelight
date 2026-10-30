*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"value_links","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":31,"to":37},{"path":"relay","from":38,"to":41},{"path":"tools","from":42,"to":57}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["tools::Shape::sides"],"evidence":"no hit of `sides` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["tools::backend"],"evidence":"1 production hits of `backend` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["tools::backend"],"evidence":"1 production hits of `backend` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["tools::Shape"],"evidence":"1 production hits of `Shape` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["tools::Shape"],"evidence":"1 production hits of `Shape` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["tools::Shape::sides"],"evidence":"0 struct literals of `Shape` set it, and no `.sides` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["tools::LIMIT"],"evidence":"2 observed uses, all inside `value_links`; no other target of the run writes `LIMIT`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["tools::measure"],"evidence":"1 observed uses, all inside `value_links`; no other target of the run writes `measure`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["tools::TOTAL"],"evidence":"2 observed uses, all inside `value_links`; no other target of the run writes `TOTAL`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["backend"],"evidence":"`pub use tools::backend`, but no observed path imports `backend` through `value_links`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["GRAND_TOTAL"],"evidence":"`pub use GRAND_TOTAL = tools::TOTAL`, but no observed path imports `GRAND_TOTAL` through `value_links`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["LIMIT"],"evidence":"`pub use tools::LIMIT`, but no observed path imports `LIMIT` through `value_links`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["measure"],"evidence":"`pub use relay::measure`, but no observed path imports `measure` through `value_links`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["relay::TOTAL"],"evidence":"`pub use crate::tools::TOTAL`, but no observed path imports `TOTAL` through `relay`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["Shape"],"evidence":"`pub use tools::Shape`, but no observed path imports `Shape` through `value_links`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"backend","kind":"fn","item":"tools::backend"}
{"k":"surface","path":"GRAND_TOTAL","kind":"value","item":"tools::GRAND_TOTAL"}
{"k":"surface","path":"LIMIT","kind":"value","item":"tools::LIMIT"}
{"k":"surface","path":"measure","kind":"fn","item":"tools::measure"}
{"k":"surface","path":"Shape","kind":"struct","item":"tools::Shape"}
{"k":"surface","path":"tools","kind":"mod"}
{"k":"surface","path":"tools::backend","kind":"fn"}
{"k":"surface","path":"tools::LIMIT","kind":"value"}
{"k":"surface","path":"tools::measure","kind":"fn"}
{"k":"surface","path":"tools::Shape","kind":"struct"}
{"k":"surface","path":"tools::Shape","kind":"fn"}
{"k":"surface","path":"tools::TOTAL","kind":"value"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"crate","text":"Links to functions, constants, and statics."}
{"k":"use","path":"measure","rust":"pub use relay::measure","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"LIMIT","rust":"pub use tools::LIMIT","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"Shape","rust":"pub use tools::Shape","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"GRAND_TOTAL","rust":"pub use tools::TOTAL as GRAND_TOTAL","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"backend","rust":"pub use tools::backend","loc":{"file":"src/lib.rs","line":0}}
{"k":"mod","path":"relay","loc":{"file":"src/lib.rs","line":0},"vis":"private"}
{"k":"doc","path":"relay","text":"Passes values on."}
{"k":"use","path":"relay::TOTAL","rust":"pub use crate::tools::TOTAL","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"relay::measure","rust":"pub use crate::tools::measure","loc":{"file":"src/lib.rs","line":0}}
{"k":"mod","path":"tools","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":1,"functions":{"public":4}}
{"k":"doc","path":"tools","text":"Defines the values."}
{"k":"const","path":"tools::LIMIT","rust":"pub const LIMIT: u32","value":"8","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":2,"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"tools::LIMIT","text":"The highest reading."}
{"k":"static","path":"tools::TOTAL","rust":"pub static TOTAL: u32","value":"0","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate","relay"],"token_hits":2,"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"tools::TOTAL","text":"A running total."}
{"k":"fn","path":"tools::measure","sig":"pub fn measure() -> u32","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate","relay"],"token_hits":2,"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"tools::measure","text":"Reads the gauge, never above [`LIMIT`] and counted in [`TOTAL`]."}
{"k":"fn","path":"tools::Shape","sig":"pub fn Shape()","loc":{"file":"src/lib.rs","line":0},"token_hits":1,"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"tools::Shape","text":"Draws a [`Shape`], which shares its name."}
{"k":"struct","path":"tools::Shape","rust":"pub struct Shape { pub sides: u8 }","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":1}
{"k":"doc","path":"tools::Shape","text":"A shape that [`measure`] sizes, also reached as [`crate::relay::measure`]."}
{"k":"fn","path":"tools::backend","sig":"pub fn backend() -> u8","loc":{"file":"src/lib.rs","line":0},"cfgs":["unix"],"reexported_by":["crate"],"token_hits":1,"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"tools::backend","text":"The Unix backend."}
{"k":"fn","path":"tools::backend","sig":"pub fn backend() -> u8","loc":{"file":"src/lib.rs","line":0},"cfgs":["windows"],"token_hits":1,"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"tools::backend","text":"A Windows backend."}
