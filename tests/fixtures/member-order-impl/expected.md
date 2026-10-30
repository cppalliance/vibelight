*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"member_order_impl","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":10,"to":25},{"path":"more","from":26,"to":27}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["more"],"evidence":"holds only 1 impls","locs":["src/more.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::brake"],"evidence":"no hit of `brake` in the scanned sources beyond its 1 declaration","locs":["src/more.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::new"],"evidence":"no hit of `new` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::start"],"evidence":"no hit of `start` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::stop"],"evidence":"no hit of `stop` in the scanned sources beyond its 1 declaration","locs":["src/more.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Engine","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1}
{"k":"doc","path":"crate","text":"Members grouped by impl."}
{"k":"struct","path":"Engine","rust":"pub struct Engine { speed: u32 }","loc":"src/lib.rs:0","token_hits":4}
{"k":"doc","path":"Engine","text":"A type with methods in two files."}
{"k":"impl","path":"Engine","rust":"impl Engine","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Engine::new","sig":"pub fn new() -> Engine","loc":"src/lib.rs:0","body":{"statements":1,"tokens":8}}
{"k":"doc","path":"Engine::new","text":"Makes one."}
{"k":"fn","path":"Engine::start","sig":"pub fn start(&mut self)","loc":"src/lib.rs:0","body":{"statements":1,"tokens":8},"ops":["self.speed [write]"]}
{"k":"doc","path":"Engine::start","text":"Starts it."}
{"k":"fn","path":"Engine::check","sig":"fn check(&self) -> bool","loc":"src/lib.rs:0","body":{"statements":1,"tokens":7},"ops":["self.speed"]}
{"k":"doc","path":"Engine::check","text":"Checks it."}
{"k":"impl","path":"Engine","rust":"impl crate::Engine","locs":["src/more.rs:0"]}
{"k":"fn","path":"Engine::stop","sig":"pub fn stop(&mut self)","loc":"src/more.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::stop","text":"Stops it."}
{"k":"fn","path":"Engine::brake","sig":"pub fn brake(&mut self)","loc":"src/more.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::brake","text":"Brakes."}
{"k":"mod","path":"more","loc":"src/more.rs:0","vis":"private"}
{"k":"doc","path":"more","text":"More methods."}
