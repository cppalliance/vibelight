*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"macros","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":9,"to":27}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["NAME"],"evidence":"no hit of `NAME` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Event","kind":"struct"}
{"k":"surface","path":"main","kind":"fn"}
{"k":"surface","path":"NAME","kind":"value"}
{"k":"surface","path":"Started","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Macros of every kind."}
{"k":"macro","path":"events","rust":"macro_rules! events","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"events","text":"Defines the events."}
{"k":"macro","path":"crate","rust":"events!","loc":{"file":"src/lib.rs","line":0}}
{"k":"struct","path":"Event","rust":"pub struct Event { #[serde(rename = \"n\")] pub name: String }","loc":{"file":"src/lib.rs","line":0},"derives":["Clone","Debug","PartialEq"],"attributes":["serde_like::tag"],"token_hits":1}
{"k":"doc","path":"Event","text":"A type with derives and an attribute macro."}
{"k":"struct","path":"Started","rust":"pub struct Started","loc":{"file":"src/lib.rs","line":0},"token_hits":2}
{"k":"doc","path":"Started","text":"Named by the module-level invocations."}
{"k":"impl","path":"Event","rust":"impl Event","locs":[{"file":"src/lib.rs","line":0}],"macros":["forward_methods!"]}
{"k":"macro","path":"Event","rust":"forward_methods!","loc":{"file":"src/lib.rs","line":0}}
{"k":"fn","path":"Event::emit","sig":"pub fn emit(&self)","loc":{"file":"src/lib.rs","line":0},"attributes":["tracing::instrument"],"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Event::emit","text":"Has an attribute macro."}
{"k":"macro","path":"crate","rust":"lifecycle_constants!","loc":{"file":"src/lib.rs","line":0}}
{"k":"fn","path":"main","sig":"pub async fn main()","loc":{"file":"src/lib.rs","line":0},"attributes":["tokio::main"],"token_hits":1,"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"main","text":"The entry point."}
{"k":"const","path":"NAME","rust":"pub const NAME: &str","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1,"macros":["stringify"]}}
{"k":"doc","path":"NAME","text":"A value with a macro."}
{"k":"macro","path":"crate","rust":"stringify!","loc":{"file":"src/lib.rs","line":0}}
