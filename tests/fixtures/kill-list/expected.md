*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":2,"crate":"kill_list","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":6,"to":13}]}
{"k":"surface","path":"Event","kind":"struct"}
{"k":"surface","path":"ONE","kind":"value"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1,"values":1,"mix":{"pub":2}}
{"k":"doc","path":"crate","text":"A crate with macros: the markdown counts them, and the HTML lists each one."}
{"k":"macro","path":"constant","rust":"macro_rules! constant","loc":"src/lib.rs:0"}
{"k":"doc","path":"constant","text":"Defines a constant."}
{"k":"const","path":"ONE","rust":"pub const ONE: u8","value":"1","loc":"src/lib.rs:0","via":"constant!","body":{"statements":1,"tokens":1}}
{"k":"macro","path":"crate","rust":"constant!","loc":"src/lib.rs:0"}
{"k":"struct","path":"Event","rust":"pub struct Event","loc":"src/lib.rs:0","derives":["Debug","serde::Serialize"],"attributes":["tracing::instrument"]}
{"k":"doc","path":"Event","text":"A type with a derive from another crate and an attribute macro."}
