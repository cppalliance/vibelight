*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"order_cycles","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":21,"to":28},{"path":"net","from":29,"to":34}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Left::right"],"evidence":"no hit of `right` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["net::Ping::pong"],"evidence":"no hit of `pong` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["net::Pong::ping"],"evidence":"no hit of `ping` in the scanned sources beyond its 2 declarations","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Right::left"],"evidence":"no hit of `left` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Root"],"evidence":"no hit of `Root` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Root::ping"],"evidence":"no hit of `ping` in the scanned sources beyond its 2 declarations","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Left::right"],"evidence":"0 struct literals of `Left` set it, and no `.right` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["net::Ping::pong"],"evidence":"0 struct literals of `Ping` set it, and no `.pong` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["net::Pong::ping"],"evidence":"0 struct literals of `Pong` set it, and no `.ping` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Right::left"],"evidence":"0 struct literals of `Right` set it, and no `.left` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Root::ping"],"evidence":"0 struct literals of `Root` set it, and no `.ping` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Left","kind":"struct"}
{"k":"surface","path":"net","kind":"mod"}
{"k":"surface","path":"net::Ping","kind":"struct"}
{"k":"surface","path":"net::Pong","kind":"struct"}
{"k":"surface","path":"Right","kind":"struct"}
{"k":"surface","path":"Root","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":3}
{"k":"doc","path":"crate","text":"Cycles at level 1 and below."}
{"k":"struct","path":"Root","rust":"pub struct Root { pub ping: net::Ping }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Root","text":"Names the cycle below it."}
{"k":"struct","path":"Left","rust":"pub struct Left { pub right: Option<Box<Right>> }","loc":{"file":"src/lib.rs","line":0},"named_by":["Right"],"token_hits":1}
{"k":"doc","path":"Left","text":"Half of a cycle that nothing else names."}
{"k":"struct","path":"Right","rust":"pub struct Right { pub left: Option<Box<Left>> }","loc":{"file":"src/lib.rs","line":0},"named_by":["Left"],"token_hits":1}
{"k":"doc","path":"Right","text":"The other half."}
{"k":"mod","path":"net","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":2}
{"k":"doc","path":"net","text":"A cycle below level 1."}
{"k":"struct","path":"net::Ping","rust":"pub struct Ping { pub pong: Option<Box<Pong>> }","loc":{"file":"src/lib.rs","line":0},"named_by":["net::Pong","Root"],"token_hits":2}
{"k":"doc","path":"net::Ping","text":"Names Pong."}
{"k":"struct","path":"net::Pong","rust":"pub struct Pong { pub ping: Option<Box<Ping>> }","loc":{"file":"src/lib.rs","line":0},"named_by":["net::Ping"],"token_hits":1}
{"k":"doc","path":"net::Pong","text":"Names Ping back."}
