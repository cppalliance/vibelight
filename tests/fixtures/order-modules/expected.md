*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"order_modules","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":18,"to":23},{"path":"engine","from":24,"to":31},{"path":"plan","from":32,"to":41}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["engine::Runner"],"evidence":"no hit of `Runner` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["engine::start"],"evidence":"no hit of `start` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["plan::FakeStep"],"evidence":"no hit of `FakeStep` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["plan::Plan::steps"],"evidence":"no hit of `steps` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["engine::Runner::plan"],"evidence":"0 struct literals of `Runner` set it, and no `.plan` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["plan::Plan::steps"],"evidence":"0 struct literals of `Plan` set it, and no `.steps` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"engine","kind":"mod"}
{"k":"surface","path":"engine::Runner","kind":"struct"}
{"k":"surface","path":"engine::start","kind":"fn"}
{"k":"surface","path":"Orphan","kind":"struct"}
{"k":"surface","path":"plan","kind":"mod"}
{"k":"surface","path":"plan::FakeStep","kind":"struct","test_support":true}
{"k":"surface","path":"plan::Plan","kind":"struct"}
{"k":"surface","path":"plan::Step","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1}
{"k":"doc","path":"crate","text":"Declarations shown one module at a time."}
{"k":"struct","path":"Orphan","rust":"pub struct Orphan","loc":"src/lib.rs:0"}
{"k":"doc","path":"Orphan","text":"A value nothing uses."}
{"k":"impl","path":"Box<Orphan>","rust":"impl Clone for Box<Orphan>","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Box<Orphan>::clone","sig":"fn clone(&self) -> Self","loc":"src/lib.rs:0","impl":"impl Clone for Box<Orphan>","body":{"statements":1,"tokens":9,"single":"call Box::new"}}
{"k":"mod","path":"engine","loc":"src/lib.rs:0","vis":"public","types":1,"functions":{"public":1}}
{"k":"doc","path":"engine","text":"Runs things."}
{"k":"struct","path":"engine::Runner","rust":"pub struct Runner { pub plan: crate::plan::Plan }","loc":"src/lib.rs:0"}
{"k":"doc","path":"engine::Runner","text":"A runner."}
{"k":"field","path":"engine::Runner::plan","rust":"pub plan: crate::plan::Plan","loc":"src/lib.rs:0"}
{"k":"doc","path":"engine::Runner::plan","text":"Its plan."}
{"k":"fn","path":"engine::start","sig":"pub fn start()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"engine::start","text":"Starts the engine."}
{"k":"mod","path":"plan","loc":"src/lib.rs:0","vis":"public","types":3}
{"k":"doc","path":"plan","text":"Plans runs."}
{"k":"struct","path":"plan::Plan","rust":"pub struct Plan { pub steps: Vec<Step> }","loc":"src/lib.rs:0","named_by":["engine::Runner"]}
{"k":"doc","path":"plan::Plan","text":"A plan."}
{"k":"field","path":"plan::Plan::steps","rust":"pub steps: Vec<Step>","loc":"src/lib.rs:0"}
{"k":"doc","path":"plan::Plan::steps","text":"Its steps."}
{"k":"struct","path":"plan::Step","rust":"pub struct Step","loc":"src/lib.rs:0","named_by":["plan::Plan"]}
{"k":"doc","path":"plan::Step","text":"One step."}
{"k":"struct","path":"plan::FakeStep","rust":"pub struct FakeStep","loc":"src/lib.rs:0","cfgs":["any(test, feature = \"test-support\")"],"test_support":true}
{"k":"doc","path":"plan::FakeStep","text":"A step for tests and benches."}
