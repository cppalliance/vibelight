*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"surface","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":41,"to":60},{"path":"api","from":61,"to":64},{"path":"api::v1","from":65,"to":70},{"path":"detail","from":71,"to":76},{"path":"detail::tools","from":77,"to":80},{"path":"prelude","from":81,"to":86},{"path":"values","from":87,"to":95},{"path":"values::inner","from":96,"to":98}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["detail::tools"],"evidence":"holds only `Tool`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["values::inner"],"evidence":"holds only `deep`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["detail::helper"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":["src/lib.rs:0"],"limits":["macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["api::v1::send"],"evidence":"no hit of `send` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["api::VERSION"],"evidence":"no hit of `VERSION` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["CALLS"],"evidence":"no hit of `CALLS` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["detail::tools::Tool"],"evidence":"no hit of `Tool` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Fake"],"evidence":"no hit of `Fake` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["prelude::mark"],"evidence":"no hit of `mark` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["prelude::Marker"],"evidence":"no hit of `Marker` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["start"],"evidence":"no hit of `start` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["detail::Handle"],"evidence":"1 observed uses, all inside `surface`; no other target of the run writes `Handle`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["values::build"],"evidence":"1 observed uses, all inside `surface`; no other target of the run writes `build`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["values::CURRENT"],"evidence":"1 observed uses, all inside `surface`; no other target of the run writes `CURRENT`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["values::inner::deep"],"evidence":"1 observed uses, all inside `surface`; no other target of the run writes `deep`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["values::MAX"],"evidence":"1 observed uses, all inside `surface`; no other target of the run writes `MAX`","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-reexport-use","mode":"lexical","subjects":["BTreeMap","build","CURRENT","deep","Handle","kit","MAX"],"evidence":"`pub use std::collections::BTreeMap`, but no observed path imports `BTreeMap` through `surface`; `pub use values::build`, but no observed path imports `build` through `surface`; `pub use values::CURRENT`, but no observed path imports `CURRENT` through `surface`; `pub use values::deep`, but no observed path imports `deep` through `surface`; `pub use detail::Handle`, but no observed path imports `Handle` through `surface`; `pub use kit = detail::tools`, but no observed path imports `kit` through `surface`; `pub use values::MAX`, but no observed path imports `MAX` through ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","macros"],"module":"crate","count":7}
{"k":"more","path":"BTreeMap","id":"no-observed-reexport-use","field":"evidence","part":2,"text":"`surface`"}
{"k":"surface","path":"api","kind":"mod"}
{"k":"surface","path":"api::v1","kind":"mod"}
{"k":"surface","path":"api::v1::Request","kind":"struct"}
{"k":"surface","path":"api::v1::send","kind":"fn"}
{"k":"surface","path":"api::VERSION","kind":"value"}
{"k":"surface","path":"BTreeMap","kind":"external","item":"std::collections::BTreeMap"}
{"k":"surface","path":"build","kind":"fn","item":"values::build"}
{"k":"surface","path":"CALLS","kind":"value"}
{"k":"surface","path":"CURRENT","kind":"value","item":"values::CURRENT"}
{"k":"surface","path":"deep","kind":"fn","item":"values::inner::deep"}
{"k":"surface","path":"Fake","kind":"struct","test_support":true}
{"k":"surface","path":"Handle","kind":"struct","item":"detail::Handle"}
{"k":"surface","path":"hello","kind":"macro"}
{"k":"surface","path":"kit","kind":"mod","item":"detail::tools"}
{"k":"surface","path":"kit::Tool","kind":"struct","item":"detail::tools::Tool"}
{"k":"surface","path":"mark","kind":"fn","item":"prelude::mark"}
{"k":"surface","path":"Marker","kind":"struct","item":"prelude::Marker"}
{"k":"surface","path":"MAX","kind":"value","item":"values::MAX"}
{"k":"surface","path":"start","kind":"fn"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":2,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"The public surface of a library."}
{"k":"use","path":"Handle","rust":"pub use detail::Handle","loc":"src/lib.rs:0"}
{"k":"use","path":"kit","rust":"pub use detail::tools as kit","loc":"src/lib.rs:0"}
{"k":"use","path":"*","rust":"pub use prelude::*","loc":"src/lib.rs:0"}
{"k":"use","path":"BTreeMap","rust":"pub use std::collections::BTreeMap","loc":"src/lib.rs:0"}
{"k":"use","path":"CURRENT","rust":"pub use values::CURRENT","loc":"src/lib.rs:0"}
{"k":"use","path":"MAX","rust":"pub use values::MAX","loc":"src/lib.rs:0"}
{"k":"use","path":"build","rust":"pub use values::build","loc":"src/lib.rs:0"}
{"k":"use","path":"deep","rust":"pub use values::deep","loc":"src/lib.rs:0"}
{"k":"macro","path":"hello","rust":"macro_rules! hello","loc":"src/lib.rs:0"}
{"k":"doc","path":"hello","text":"Says hello."}
{"k":"static","path":"CALLS","rust":"pub static CALLS: u8","value":"0","loc":"src/lib.rs:0","body":{"statements":1,"tokens":1}}
{"k":"doc","path":"CALLS","text":"The number of calls so far."}
{"k":"fn","path":"start","sig":"pub fn start()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"start","text":"Starts the library."}
{"k":"struct","path":"Internal","rust":"pub(crate) struct Internal","loc":"src/lib.rs:0"}
{"k":"doc","path":"Internal","text":"A crate-only type."}
{"k":"struct","path":"Fake","rust":"pub struct Fake","loc":"src/lib.rs:0","cfgs":["any(test, feature = \"test-support\")"],"test_support":true}
{"k":"doc","path":"Fake","text":"A fake for tests and benches."}
{"k":"mod","path":"api","loc":"src/lib.rs:0","vis":"public"}
{"k":"doc","path":"api","text":"The public API."}
{"k":"const","path":"api::VERSION","rust":"pub const VERSION: u8","value":"1","loc":"src/lib.rs:0","body":{"statements":1,"tokens":1}}
{"k":"doc","path":"api::VERSION","text":"The version in use."}
{"k":"mod","path":"api::v1","loc":"src/lib.rs:0","vis":"public","types":1,"functions":{"public":1}}
{"k":"doc","path":"api::v1","text":"The first version."}
{"k":"struct","path":"api::v1::Request","rust":"pub struct Request","loc":"src/lib.rs:0","named_by":["api::v1"],"token_hits":1}
{"k":"doc","path":"api::v1::Request","text":"A request."}
{"k":"fn","path":"api::v1::send","sig":"pub fn send(request: Request) -> u8","loc":"src/lib.rs:0","body":{"statements":2,"tokens":8},"ops":["request"]}
{"k":"doc","path":"api::v1::send","text":"Sends a request."}
{"k":"mod","path":"detail","loc":"src/lib.rs:0","vis":"private","types":1,"functions":{"public":1}}
{"k":"doc","path":"detail","text":"Hidden helpers."}
{"k":"struct","path":"detail::Handle","rust":"pub struct Handle","loc":"src/lib.rs:0","reexported_by":["crate"],"token_hits":1}
{"k":"doc","path":"detail::Handle","text":"A handle made public through a re-export."}
{"k":"fn","path":"detail::helper","sig":"pub fn helper()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"detail::helper","text":"A helper that stays hidden."}
{"k":"mod","path":"detail::tools","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"detail::tools","text":"Tools reached only through a re-export."}
{"k":"struct","path":"detail::tools::Tool","rust":"pub struct Tool","loc":"src/lib.rs:0"}
{"k":"doc","path":"detail::tools::Tool","text":"A tool."}
{"k":"mod","path":"prelude","loc":"src/lib.rs:0","vis":"private","types":1,"functions":{"public":1}}
{"k":"doc","path":"prelude","text":"Names a glob brings in."}
{"k":"struct","path":"prelude::Marker","rust":"pub struct Marker","loc":"src/lib.rs:0"}
{"k":"doc","path":"prelude::Marker","text":"A marker a glob re-exports."}
{"k":"fn","path":"prelude::mark","sig":"pub fn mark()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"prelude::mark","text":"A function a glob re-exports."}
{"k":"mod","path":"values","loc":"src/lib.rs:0","vis":"private","functions":{"public":1}}
{"k":"doc","path":"values","text":"Values made public through re-exports."}
{"k":"use","path":"values::deep","rust":"pub use self::inner::deep","loc":"src/lib.rs:0"}
{"k":"fn","path":"values::build","sig":"pub fn build() -> u8","loc":"src/lib.rs:0","reexported_by":["crate"],"token_hits":1,"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"values::build","text":"Builds a value."}
{"k":"const","path":"values::MAX","rust":"pub const MAX: u8","value":"9","loc":"src/lib.rs:0","reexported_by":["crate"],"token_hits":1,"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"values::MAX","text":"The largest value."}
{"k":"static","path":"values::CURRENT","rust":"pub static CURRENT: u8","value":"0","loc":"src/lib.rs:0","reexported_by":["crate"],"token_hits":1,"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"values::CURRENT","text":"The current value."}
{"k":"mod","path":"values::inner","loc":"src/lib.rs:0","vis":"private","functions":{"public":1}}
{"k":"fn","path":"values::inner::deep","sig":"pub fn deep()","loc":"src/lib.rs:0","reexported_by":["crate","values"],"token_hits":2,"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"values::inner::deep","text":"A function two re-exports away."}
