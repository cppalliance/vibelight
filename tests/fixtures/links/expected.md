*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"links","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":15,"to":29},{"path":"parts","from":30,"to":35}]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["parts::Gauge"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/parts.rs","line":12,"url":"../src/parts.rs#L12"}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["parts::Part"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/parts.rs","line":9,"url":"../src/parts.rs#L9"}],"limits":["macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Dial"],"evidence":"no hit of `Dial` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":26,"url":"../src/lib.rs#L26"}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Dial::limit"],"evidence":"no hit of `limit` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":27,"url":"../src/lib.rs#L27"}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::start"],"evidence":"no hit of `start` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":12,"url":"../src/lib.rs#L12"}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine::stop"],"evidence":"no hit of `stop` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/parts.rs","line":5,"url":"../src/parts.rs#L5"}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Dial::limit"],"evidence":"0 struct literals of `Dial` set it, and no `.limit` access or pattern reads it","locs":[{"file":"src/lib.rs","line":27,"url":"../src/lib.rs#L27"}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Engine::speed"],"evidence":"1 struct literals of `Engine` set it, and no `.speed` access or pattern reads it","locs":[{"file":"src/lib.rs","line":7,"url":"../src/lib.rs#L7"}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Dial","kind":"struct"}
{"k":"surface","path":"Engine","kind":"struct"}
{"k":"spelling","path":"std::option::Option","preferred":"Option","spellings":[{"text":"Option","count":1,"locs":[{"file":"src/parts.rs","line":13,"url":"../src/parts.rs#L13"}]},{"text":"std::option::Option","count":2,"locs":[{"file":"src/lib.rs","line":27,"url":"../src/lib.rs#L27"},{"file":"src/parts.rs","line":14,"url":"../src/parts.rs#L14"}]}]}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":1,"url":"../src/lib.rs#L1"},"types":2}
{"k":"doc","path":"crate","text":"Source links."}
{"k":"struct","path":"Engine","rust":"pub struct Engine { pub speed: u32 }","loc":{"file":"src/lib.rs","line":6,"url":"../src/lib.rs#L6"},"token_hits":4}
{"k":"doc","path":"Engine","text":"A type with an impl in another file."}
{"k":"impl","path":"Engine","rust":"impl Engine","locs":[{"file":"src/lib.rs","line":10,"url":"../src/lib.rs#L10"}]}
{"k":"fn","path":"Engine::start","sig":"pub fn start(&self)","loc":{"file":"src/lib.rs","line":12,"url":"../src/lib.rs#L12"},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::start","text":"Starts the engine."}
{"k":"macro","path":"noop","rust":"macro_rules! noop","loc":{"file":"src/lib.rs","line":15,"url":"../src/lib.rs#L15"}}
{"k":"impl","path":"Box<Engine>","rust":"impl Clone for Box<Engine>","locs":[{"file":"src/lib.rs","line":19,"url":"../src/lib.rs#L19"}]}
{"k":"fn","path":"Box<Engine>::clone","sig":"fn clone(&self) -> Self","loc":{"file":"src/lib.rs","line":20,"url":"../src/lib.rs#L20"},"impl":"impl Clone for Box<Engine>","body":{"statements":1,"tokens":14,"single":"call Box::new"}}
{"k":"struct","path":"Dial","rust":"pub struct Dial { pub limit: std::option::Option<u32> }","loc":{"file":"src/lib.rs","line":26,"url":"../src/lib.rs#L26"}}
{"k":"doc","path":"Dial","text":"A dial with an optional limit."}
{"k":"impl","path":"Engine","rust":"impl crate::Engine","locs":[{"file":"src/parts.rs","line":3,"url":"../src/parts.rs#L3"}]}
{"k":"fn","path":"Engine::stop","sig":"pub fn stop(&self)","loc":{"file":"src/parts.rs","line":5,"url":"../src/parts.rs#L5"},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Engine::stop","text":"Stops the engine."}
{"k":"mod","path":"parts","loc":{"file":"src/parts.rs","line":1,"url":"../src/parts.rs#L1"},"vis":"private","types":2}
{"k":"doc","path":"parts","text":"Parts of the engine."}
{"k":"struct","path":"parts::Part","rust":"pub struct Part","loc":{"file":"src/parts.rs","line":9,"url":"../src/parts.rs#L9"}}
{"k":"doc","path":"parts::Part","text":"A part."}
{"k":"struct","path":"parts::Gauge","rust":"pub struct Gauge { pub reading: Option<u32>, pub spare: std::option::Option<u32> }","loc":{"file":"src/parts.rs","line":12,"url":"../src/parts.rs#L12"}}
{"k":"doc","path":"parts::Gauge","text":"A gauge that writes one standard type two ways."}
