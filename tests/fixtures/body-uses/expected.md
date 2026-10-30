*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"body_uses","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":22,"to":49}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["list"],"evidence":"no hit of `list` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Lister"],"evidence":"no hit of `Lister` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Lister::mark"],"evidence":"no hit of `mark` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Lonely::again"],"evidence":"no hit of `again` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Runner::probe"],"evidence":"no hit of `probe` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Runner"],"evidence":"1 production hits of `Runner` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Listed"],"evidence":"2 observed uses, all inside `body_uses`; no other target of the run writes `Listed`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Lonely"],"evidence":"2 observed uses, all inside `body_uses`; no other target of the run writes `Lonely`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Marker"],"evidence":"2 observed uses, all inside `body_uses`; no other target of the run writes `Marker`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-external-use","mode":"lexical","subjects":["Probe"],"evidence":"2 observed uses, all inside `body_uses`; no other target of the run writes `Probe`","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"list","kind":"fn"}
{"k":"surface","path":"Listed","kind":"struct"}
{"k":"surface","path":"Lister","kind":"trait"}
{"k":"surface","path":"Lonely","kind":"struct"}
{"k":"surface","path":"Marker","kind":"struct"}
{"k":"surface","path":"Probe","kind":"struct"}
{"k":"surface","path":"Runner","kind":"struct"}
{"k":"surface","path":"Wrapped","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":7,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Types that only function bodies name."}
{"k":"struct","path":"Listed","rust":"pub struct Listed","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Listed","text":"Named only inside a free function's body."}
{"k":"struct","path":"Probe","rust":"pub struct Probe","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Probe","text":"Named only inside a method body of another type."}
{"k":"struct","path":"Lonely","rust":"pub struct Lonely","loc":{"file":"src/lib.rs","line":0},"token_hits":2}
{"k":"doc","path":"Lonely","text":"Named only inside its own method body, so it stays unconnected."}
{"k":"struct","path":"Marker","rust":"pub struct Marker","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Marker","text":"Named only in a trait's default method body."}
{"k":"struct","path":"Runner","rust":"pub struct Runner","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Runner","text":"Runs things."}
{"k":"impl","path":"Runner","rust":"impl Runner","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Runner::probe","sig":"pub fn probe(&self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":7}}
{"k":"doc","path":"Runner::probe","text":"Makes a probe."}
{"k":"impl","path":"Lonely","rust":"impl Lonely","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Lonely::again","sig":"pub fn again()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":7}}
{"k":"doc","path":"Lonely::again","text":"Makes another one."}
{"k":"trait","path":"Lister","rust":"pub trait Lister","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Lister","text":"Lists things."}
{"k":"fn","path":"Lister::mark","sig":"fn mark(&self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":7}}
{"k":"doc","path":"Lister::mark","text":"Marks."}
{"k":"struct","path":"Wrapped","rust":"pub struct Wrapped","loc":{"file":"src/lib.rs","line":0},"token_hits":2}
{"k":"doc","path":"Wrapped","text":"Named only inside an impl on a wrapper of itself, so it stays\nunconnected."}
{"k":"impl","path":"Box<Wrapped>","rust":"impl Default for Box<Wrapped>","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Box<Wrapped>::default","sig":"fn default() -> Self","loc":{"file":"src/lib.rs","line":0},"impl":"impl Default for Box<Wrapped>","body":{"statements":1,"tokens":9,"single":"call Box::new"}}
{"k":"fn","path":"list","sig":"pub fn list()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":7}}
{"k":"doc","path":"list","text":"Lists one thing."}
