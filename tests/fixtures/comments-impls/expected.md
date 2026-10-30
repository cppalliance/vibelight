*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"comments_impls","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":10,"to":23},{"path":"parts","from":24,"to":27}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["parts"],"evidence":"holds only `Part`","locs":["src/parts.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::start"],"evidence":"no hit of `start` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["parts::Part"],"evidence":"no hit of `Part` in the scanned sources beyond its 1 declaration","locs":["src/parts.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Engine","kind":"struct"}
{"k":"surface","path":"parts","kind":"mod"}
{"k":"surface","path":"parts::Part","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1}
{"k":"doc","path":"crate","text":"Comments on impls and modules."}
{"k":"struct","path":"Engine","rust":"pub struct Engine","loc":"src/lib.rs:0","token_hits":7}
{"k":"doc","path":"Engine","text":"The type."}
{"k":"impl","path":"Engine","rust":"impl Engine","locs":["src/lib.rs:0"],"comment":"// Above the inherent impl."}
{"k":"fn","path":"Engine::start","sig":"pub fn start(&self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::start","text":"Stays the method's doc comment."}
{"k":"impl","path":"Engine","rust":"impl std::fmt::Display for Engine","locs":["src/lib.rs:0"]}
{"k":"doc","path":"Engine","text":"Above the Display impl."}
{"k":"fn","path":"Engine::fmt","sig":"fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result","loc":"src/lib.rs:0","impl":"impl std::fmt::Display for Engine","body":{"statements":1,"tokens":7,"single":"call Ok"},"comment":"// A comment on a trait impl member is not shown."}
{"k":"impl","path":"Engine","rust":"impl Clone for Engine","locs":["src/lib.rs:0"],"comment":"// Above the Clone impl."}
{"k":"fn","path":"Engine::clone","sig":"fn clone(&self) -> Engine","loc":"src/lib.rs:0","impl":"impl Clone for Engine","body":{"statements":1,"tokens":3}}
{"k":"impl","path":"Box<Engine>","rust":"impl Default for Box<Engine>","locs":["src/lib.rs:0"],"comment":"// Above an impl that goes to Other impls."}
{"k":"fn","path":"Box<Engine>::default","sig":"fn default() -> Self","loc":"src/lib.rs:0","impl":"impl Default for Box<Engine>","body":{"statements":1,"tokens":9,"single":"call Box::new"}}
{"k":"mod","path":"parts","loc":"src/parts.rs:0","vis":"public","types":1,"comment":"// An ordinary comment on a mod line."}
{"k":"doc","path":"parts","text":"The outer doc of the module.\nThe inner doc of the module."}
{"k":"struct","path":"parts::Part","rust":"pub struct Part","loc":"src/parts.rs:0"}
{"k":"doc","path":"parts::Part","text":"A part."}
