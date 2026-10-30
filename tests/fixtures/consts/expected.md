*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"consts","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":16,"to":36}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Config::DOUBLE"],"evidence":"no hit of `DOUBLE` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["COUNTER"],"evidence":"no hit of `COUNTER` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["GREETING"],"evidence":"no hit of `GREETING` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["NAME"],"evidence":"no hit of `NAME` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["STATE"],"evidence":"no hit of `STATE` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Config"],"evidence":"1 observed uses, all inside `consts`; no other target of the run writes `Config`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"Config","kind":"struct"}
{"k":"surface","path":"COUNTER","kind":"value"}
{"k":"surface","path":"GREETING","kind":"value"}
{"k":"surface","path":"LIMIT","kind":"value"}
{"k":"surface","path":"NAME","kind":"value"}
{"k":"surface","path":"STATE","kind":"value"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":1}
{"k":"doc","path":"crate","text":"Constants and statics."}
{"k":"const","path":"LIMIT","rust":"pub const LIMIT: usize","value":"64","loc":{"file":"src/lib.rs","line":0},"token_hits":1,"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"LIMIT","text":"The default limit."}
{"k":"const","path":"GREETING","rust":"pub const GREETING: &str","value":"\"a long greeting that will not fit in the outline's eighty column limit\"","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"GREETING","text":"A long value that does not fit."}
{"k":"const","path":"NAME","rust":"pub const NAME: &str","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1,"macros":["concat"]}}
{"k":"doc","path":"NAME","text":"A value with a macro call."}
{"k":"macro","path":"crate","rust":"concat!","loc":{"file":"src/lib.rs","line":0}}
{"k":"static","path":"COUNTER","rust":"pub static COUNTER: AtomicUsize","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":7,"single":"call AtomicUsize::new"}}
{"k":"doc","path":"COUNTER","text":"A counter."}
{"k":"static","path":"STATE","rust":"pub static mut STATE: u8","value":"0","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"STATE","text":"A mutable static."}
{"k":"const","path":"PRIVATE","rust":"const PRIVATE: u8","value":"1","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"struct","path":"Config","rust":"pub struct Config","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Config","text":"A type with associated constants."}
{"k":"impl","path":"Config","rust":"impl Config","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"const","path":"Config::VERSION","rust":"pub const VERSION: u32","value":"3","loc":{"file":"src/lib.rs","line":0},"token_hits":1,"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"Config::VERSION","text":"The version."}
{"k":"const","path":"Config::DOUBLE","rust":"pub const DOUBLE: u32","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":6}}
{"k":"doc","path":"Config::DOUBLE","text":"A computed value."}
