*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"visibility_scopes","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":17,"to":20},{"path":"engine","from":21,"to":28},{"path":"engine::parse","from":29,"to":36},{"path":"engine::parse::inner","from":37,"to":40},{"path":"engine::reading","from":41,"to":41},{"path":"engine::run","from":42,"to":45}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["engine::parse::inner"],"evidence":"holds only `helper`","locs":["src/engine.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["engine::reading"],"evidence":"holds only 1 impls","locs":["src/engine.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["engine::run"],"evidence":"holds only `start`","locs":["src/engine.rs:0"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["engine::Gauge"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/engine.rs:0"],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["engine::parse"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/engine.rs:0"],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["engine::parse::inner"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/engine.rs:0"],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["engine::run"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/engine.rs:0"],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["engine::run::start"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/engine.rs:0"],"limits":["macros"]}
{"k":"lead","id":"visibility-spelling","mode":"direct-syntax","subjects":["engine::parse::inner::helper"],"evidence":"`pub(in crate::engine::parse)` in `engine::parse::inner` reaches what `pub(super)` says","locs":["src/engine.rs:0"]}
{"k":"lead","id":"visibility-spelling","mode":"direct-syntax","subjects":["engine::parse::Token"],"evidence":"`pub(in crate::engine)` in `engine::parse` reaches what `pub(super)` says","locs":["src/engine.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["launch"],"evidence":"no hit of `launch` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"visibility-slack","mode":"resolved","subjects":["engine::parse::local"],"evidence":"`pub(crate)`, but every observed use is inside what `pub(super)` reaches","locs":["src/engine.rs:0"],"limits":["generated-code","macros","method-calls"]}
{"k":"surface","path":"launch","kind":"fn"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","functions":{"public":1}}
{"k":"doc","path":"crate","text":"Visibility scopes: restricted spellings, the reach each one has, and an\nitem whose uses need less reach than it has."}
{"k":"fn","path":"launch","sig":"pub fn launch() -> u8","loc":"src/lib.rs:0","body":{"statements":1,"tokens":11,"single":"call engine::run::start"}}
{"k":"doc","path":"launch","text":"Runs the engine."}
{"k":"mod","path":"engine","loc":"src/engine.rs:0","vis":"private","types":1}
{"k":"doc","path":"engine","text":"The engine: two layers under one group, and a type whose impl is\nwritten in a sibling module."}
{"k":"struct","path":"engine::Gauge","rust":"pub struct Gauge","loc":"src/engine.rs:0"}
{"k":"doc","path":"engine::Gauge","text":"A type with members in an impl of another module."}
{"k":"impl","path":"engine::Gauge","rust":"impl super::Gauge","locs":["src/engine.rs:0"]}
{"k":"fn","path":"engine::Gauge::level","sig":"pub(super) fn level(&self) -> u8","loc":"src/engine.rs:0","body":{"statements":1,"tokens":7,"single":"method private_level"}}
{"k":"doc","path":"engine::Gauge::level","text":"Reachable from the engine."}
{"k":"fn","path":"engine::Gauge::private_level","sig":"fn private_level(&self) -> u8","loc":"src/engine.rs:0","body":{"statements":1,"tokens":3}}
{"k":"mod","path":"engine::parse","loc":"src/engine.rs:0","vis":"public","types":1,"functions":{"internal":2}}
{"k":"doc","path":"engine::parse","text":"The first layer."}
{"k":"struct","path":"engine::parse::Token","rust":"pub(in crate::engine) struct Token","loc":"src/engine.rs:0","named_by":["engine::parse"]}
{"k":"doc","path":"engine::parse::Token","text":"A token, widened to the group so the entry point can match it."}
{"k":"fn","path":"engine::parse::lex","sig":"pub(super) fn lex() -> Token","loc":"src/engine.rs:0","direct_callers":["engine::run::start"],"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"engine::parse::lex","text":"Reads one token."}
{"k":"fn","path":"engine::parse::local","sig":"pub(crate) fn local() -> u8","loc":"src/engine.rs:0","direct_callers":["engine::run::start"],"body":{"statements":1,"tokens":8,"single":"call inner::helper"}}
{"k":"doc","path":"engine::parse::local","text":"Used only by the other layer of the group."}
{"k":"mod","path":"engine::parse::inner","loc":"src/engine.rs:0","vis":"public","functions":{"internal":1}}
{"k":"doc","path":"engine::parse::inner","text":"Deeper."}
{"k":"fn","path":"engine::parse::inner::helper","sig":"pub(in crate::engine::parse) fn helper() -> u8","loc":"src/engine.rs:0","direct_callers":["engine::parse::local"],"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"engine::parse::inner::helper","text":"Spelled for the parent, which `pub(super)` says."}
{"k":"mod","path":"engine::reading","loc":"src/engine.rs:0","vis":"private"}
{"k":"mod","path":"engine::run","loc":"src/engine.rs:0","vis":"public","functions":{"public":1}}
{"k":"doc","path":"engine::run","text":"The second layer."}
{"k":"fn","path":"engine::run::start","sig":"pub fn start() -> u8","loc":"src/engine.rs:0","direct_callers":["launch"],"body":{"statements":2,"tokens":24}}
{"k":"doc","path":"engine::run::start","text":"Starts a run."}
