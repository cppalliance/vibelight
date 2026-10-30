*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"comments_attach","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":15,"to":31}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Config::build"],"evidence":"no hit of `build` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Config::typed"],"evidence":"no hit of `typed` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Holder"],"evidence":"no hit of `Holder` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["PATTERN"],"evidence":"no hit of `PATTERN` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Config::first"],"evidence":"1 struct literals of `Config` set it, and no `.first` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Config::second"],"evidence":"1 struct literals of `Config` set it, and no `.second` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Config::third"],"evidence":"1 struct literals of `Config` set it, and no `.third` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Holder::items"],"evidence":"0 struct literals of `Holder` set it, and no `.items` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Config","kind":"struct"}
{"k":"surface","path":"Holder","kind":"struct"}
{"k":"surface","path":"PATTERN","kind":"value"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2}
{"k":"doc","path":"crate","text":"Where comments attach."}
{"k":"struct","path":"Config","rust":"pub struct Config { pub first: u8, pub second: u8, pub third: u8 }","loc":{"file":"src/lib.rs","line":0},"token_hits":3,"comment":"// Attached to the struct below.","dangling":"// Nothing follows this comment in the struct."}
{"k":"doc","path":"Config","text":"Docs for Config."}
{"k":"field","path":"Config::first","rust":"pub first: u8","loc":{"file":"src/lib.rs","line":0},"comment":"// Directly above the field."}
{"k":"field","path":"Config::second","rust":"pub second: u8","loc":{"file":"src/lib.rs","line":0},"comment":"// Separated from the field by a blank line.\n// Trailing comment on the same line."}
{"k":"impl","path":"Config","rust":"impl Config","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Config::build","sig":"pub fn build(first: u8, second: u8) -> Config","loc":{"file":"src/lib.rs","line":0},"param_docs":[{"param":"first","doc":"// The first input."}],"body":{"statements":1,"tokens":10},"ops":["first","second"],"comments":[{"text":"// A comment inside the body is never shown.","line":0}]}
{"k":"doc","path":"Config::build","text":"Takes commented parameters."}
{"k":"fn","path":"Config::typed","sig":"pub fn typed(&self, items: Vec<u32>)","loc":{"file":"src/lib.rs","line":0},"param_docs":[{"param":"items","doc":"/* the count */"}],"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Config::typed","text":"Holds a comment inside its type."}
{"k":"struct","path":"Holder","rust":"pub struct Holder { pub items: Vec<u8> }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Holder","text":"A field whose type holds a comment."}
{"k":"field","path":"Holder::items","rust":"pub items: Vec<u8>","loc":{"file":"src/lib.rs","line":0},"comment":"/* inner */"}
{"k":"macro","path":"noop","rust":"macro_rules! noop","loc":{"file":"src/lib.rs","line":0}}
{"k":"const","path":"PATTERN","rust":"pub const PATTERN: &str","value":"\"// not a comment\"","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"PATTERN","text":"Holds a string with comment-like text."}
