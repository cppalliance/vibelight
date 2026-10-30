*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"comments_groups","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":29,"to":45},{"path":"store","from":46,"to":55},{"path":"traits","from":56,"to":59}]}
{"k":"lead","id":"one-observed-trait-impl","mode":"resolved","subjects":["Handle","traits::Persist"],"evidence":"one impl in the run: `impl traits::Persist for Handle`","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-impls","macros"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["traits"],"evidence":"holds only `Persist`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Handle::get"],"evidence":"no hit of `get` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Handle::open"],"evidence":"no hit of `open` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["traits::Persist::save"],"evidence":"no hit of `save` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["store::Key"],"evidence":"1 observed uses, all inside `comments_groups`; no other target of the run writes `Key`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["store::read"],"evidence":"1 observed uses, all inside `comments_groups`; no other target of the run writes `read`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["store::Value"],"evidence":"1 observed uses, all inside `comments_groups`; no other target of the run writes `Value`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["store::write"],"evidence":"1 observed uses, all inside `comments_groups`; no other target of the run writes `write`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["Key"],"evidence":"`pub use store::Key`, but no observed path imports `Key` through `comments_groups`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["read"],"evidence":"`pub use store::read`, but no observed path imports `read` through `comments_groups`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["Value"],"evidence":"`pub use store::Value`, but no observed path imports `Value` through `comments_groups`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["write"],"evidence":"`pub use store::write`, but no observed path imports `write` through `comments_groups`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"Handle","kind":"struct"}
{"k":"surface","path":"Key","kind":"struct","item":"store::Key"}
{"k":"surface","path":"read","kind":"fn","item":"store::read"}
{"k":"surface","path":"store","kind":"mod"}
{"k":"surface","path":"store::Key","kind":"struct"}
{"k":"surface","path":"store::read","kind":"fn"}
{"k":"surface","path":"store::Value","kind":"struct"}
{"k":"surface","path":"store::write","kind":"fn"}
{"k":"surface","path":"traits","kind":"mod"}
{"k":"surface","path":"traits::Persist","kind":"trait"}
{"k":"surface","path":"Value","kind":"struct","item":"store::Value"}
{"k":"surface","path":"write","kind":"fn","item":"store::write"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":1}
{"k":"doc","path":"crate","text":"Comments on impl blocks and on use groups."}
{"k":"use","path":"Key","rust":"pub use store::Key","loc":{"file":"src/lib.rs","line":0},"comment":"// The key type."}
{"k":"use","path":"Value","rust":"pub use store::Value","loc":{"file":"src/lib.rs","line":0},"comment":"// The value type."}
{"k":"use","path":"read","rust":"pub use store::read","loc":{"file":"src/lib.rs","line":0},"comment":"// Store operations, for scripts."}
{"k":"use","path":"write","rust":"pub use store::write","loc":{"file":"src/lib.rs","line":0},"comment":"// Writes replace the old value."}
{"k":"struct","path":"Handle","rust":"pub struct Handle","loc":{"file":"src/lib.rs","line":0},"token_hits":6}
{"k":"doc","path":"Handle","text":"A handle."}
{"k":"impl","path":"Handle","rust":"impl Handle","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"comment":"// Opening.\n// Reading."}
{"k":"fn","path":"Handle::open","sig":"pub fn open() -> Handle","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"Handle::open","text":"Opens a handle."}
{"k":"fn","path":"Handle::get","sig":"pub fn get(&self) -> u8","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"Handle::get","text":"Reads a byte."}
{"k":"impl","path":"Handle","rust":"impl std::fmt::Display for Handle","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"doc","path":"Handle","text":"Shows the handle."}
{"k":"fn","path":"Handle::fmt","sig":"fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result","loc":{"file":"src/lib.rs","line":0},"impl":"impl std::fmt::Display for Handle","body":{"statements":1,"tokens":8,"single":"method write_str"},"ops":["f.write_str(\"handle\")"]}
{"k":"impl","path":"Handle","rust":"impl traits::Persist for Handle","locs":[{"file":"src/lib.rs","line":0}],"comment":"// Persists the handle."}
{"k":"mod","path":"store","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":2,"functions":{"public":2}}
{"k":"doc","path":"store","text":"Keys and values."}
{"k":"struct","path":"store::Key","rust":"pub struct Key","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":1}
{"k":"doc","path":"store::Key","text":"A key."}
{"k":"struct","path":"store::Value","rust":"pub struct Value","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":1}
{"k":"doc","path":"store::Value","text":"A value."}
{"k":"fn","path":"store::read","sig":"pub fn read()","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":1,"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"store::read","text":"Reads a value."}
{"k":"fn","path":"store::write","sig":"pub fn write()","loc":{"file":"src/lib.rs","line":0},"reexported_by":["crate"],"token_hits":1,"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"store::write","text":"Writes a value."}
{"k":"mod","path":"traits","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":1}
{"k":"trait","path":"traits::Persist","rust":"pub trait Persist","loc":{"file":"src/lib.rs","line":0},"named_by":["Handle"],"token_hits":1}
{"k":"doc","path":"traits::Persist","text":"Something that persists."}
{"k":"fn","path":"traits::Persist::save","sig":"fn save(&self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2}}
