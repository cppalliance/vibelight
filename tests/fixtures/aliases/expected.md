*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"aliases","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":8,"to":15}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Callback"],"evidence":"no hit of `Callback` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Callback","kind":"alias"}
{"k":"surface","path":"Error","kind":"struct"}
{"k":"surface","path":"Result","kind":"alias"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":3}
{"k":"doc","path":"crate","text":"Type aliases."}
{"k":"alias","path":"Result","rust":"pub type Result<T> = std::result::Result<T, Error>","loc":"src/lib.rs:0","named_by":["Callback"],"token_hits":2}
{"k":"doc","path":"Result","text":"The crate's result type."}
{"k":"struct","path":"Error","rust":"pub struct Error { message: String }","loc":"src/lib.rs:0","derives":["Debug"],"named_by":["Result"],"token_hits":1}
{"k":"doc","path":"Error","text":"The crate's error."}
{"k":"alias","path":"Callback","rust":"pub type Callback<'a, T> = Box<dyn Fn(T) -> Result<()> + 'a>","loc":"src/lib.rs:0"}
{"k":"doc","path":"Callback","text":"A callback with a bound."}
