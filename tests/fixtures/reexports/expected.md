*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"reexports","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":18,"to":26},{"path":"inner","from":27,"to":34}]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["inner::helper"],"evidence":"1 observed uses, all inside `reexports`; no other target of the run writes `helper`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["inner::Other"],"evidence":"2 observed uses, all inside `reexports`; no other target of the run writes `Other`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["inner::Thing"],"evidence":"3 observed uses, all inside `reexports`; no other target of the run writes `Thing`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["assist"],"evidence":"`pub use assist = inner::helper`, but no observed path imports `assist` through `reexports`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["Extra"],"evidence":"`pub use Extra = inner::Other`, but no observed path imports `Extra` through `reexports`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["OldThing"],"evidence":"`pub use OldThing = inner::Thing`, but no observed path imports `OldThing` through `reexports`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["Other"],"evidence":"`pub use inner::Other`, but no observed path imports `Other` through `reexports`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["Thing"],"evidence":"`pub use inner::Thing`, but no observed path imports `Thing` through `reexports`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"assist","kind":"fn","item":"inner::assist"}
{"k":"surface","path":"Extra","kind":"struct","item":"inner::Other"}
{"k":"surface","path":"helper","kind":"fn","item":"inner::helper"}
{"k":"surface","path":"OldThing","kind":"struct","item":"inner::Thing"}
{"k":"surface","path":"Other","kind":"struct","item":"inner::Other"}
{"k":"surface","path":"Thing","kind":"struct","item":"inner::Thing"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"crate","text":"Re-exports."}
{"k":"use","path":"*","rust":"pub use inner::*","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"Other","rust":"pub use inner::Other","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"Extra","rust":"pub use inner::Other as Extra","loc":{"file":"src/lib.rs","line":0},"cfgs":["feature = \"extra\""]}
{"k":"use","path":"Thing","rust":"pub use inner::Thing","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"OldThing","rust":"pub use inner::Thing as OldThing","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"assist","rust":"pub use inner::helper as assist","loc":{"file":"src/lib.rs","line":0}}
{"k":"use","path":"CrateThing","rust":"pub(crate) use inner::Thing as CrateThing","loc":{"file":"src/lib.rs","line":0}}
{"k":"mod","path":"inner","loc":{"file":"src/lib.rs","line":0},"vis":"private","types":2,"functions":{"public":1}}
{"k":"doc","path":"inner","text":"Holds the items."}
{"k":"struct","path":"inner::Thing","rust":"pub struct Thing","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":3}
{"k":"doc","path":"inner::Thing","text":"A thing."}
{"k":"struct","path":"inner::Other","rust":"pub struct Other","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":2}
{"k":"doc","path":"inner::Other","text":"Another thing."}
{"k":"fn","path":"inner::helper","sig":"pub fn helper()","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":1,"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"inner::helper","text":"A helper."}
