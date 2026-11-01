*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":2,"crate":"macro_expansion","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":10,"to":20},{"path":"events","from":21,"to":38},{"path":"shapes","from":39,"to":42}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["shapes"],"evidence":"holds only `markers`","locs":["src/shapes.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["run_of"],"evidence":"no hit of `run_of` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Alpha","kind":"struct"}
{"k":"surface","path":"Beta","kind":"struct"}
{"k":"surface","path":"Event","kind":"enum","item":"events::Event"}
{"k":"surface","path":"run_of","kind":"fn"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":2,"functions":1,"reexports":1,"mix":{"pub":4}}
{"k":"doc","path":"crate","text":"Macro look-through: the items that the crate's own macros write, from a\nmacro in this file and one in another."}
{"k":"use","path":"Event","rust":"pub use events::Event","loc":"src/lib.rs:0"}
{"k":"macro","path":"crate","rust":"markers!","loc":"src/lib.rs:0"}
{"k":"struct","path":"Alpha","rust":"pub struct Alpha","loc":"src/lib.rs:0","via":"markers!"}
{"k":"doc","path":"Alpha","text":"A marker that the macro writes."}
{"k":"struct","path":"Beta","rust":"pub struct Beta","loc":"src/lib.rs:0","via":"markers!"}
{"k":"doc","path":"Beta","text":"A marker that the macro writes."}
{"k":"macro","path":"crate","rust":"markers!","loc":"src/lib.rs:0"}
{"k":"fn","path":"run_of","sig":"pub fn run_of(event: &Event) -> &str","loc":"src/lib.rs:0","body":{"statements":1,"tokens":7,"single":"method execution"},"ops":["event.execution"]}
{"k":"doc","path":"run_of","text":"Reads the run of an event."}
{"k":"mod","path":"events","loc":"src/events.rs:0","types":1,"mix":{"pub":1}}
{"k":"doc","path":"events","text":"Events whose variants all carry the same stamp field first."}
{"k":"macro","path":"events::events","rust":"macro_rules! events","loc":"src/events.rs:0"}
{"k":"doc","path":"events::events","text":"Declares the event enum and its accessor: every variant carries\n`execution` first, then its own fields."}
{"k":"macro","path":"events","rust":"events!","loc":"src/events.rs:0"}
{"k":"enum","path":"events::Event","rust":"pub enum Event { Started { execution: String }, Stopped { execution: String, code: i32 }, Moved { execution: String, from: u32, to: u32 } }","loc":"src/events.rs:0","eff":"pub","via":"events!","derives":["Clone","Debug"],"named_by":["crate"],"reexported_by":["crate"]}
{"k":"doc","path":"events::Event","text":"One thing that happened during a run."}
{"k":"variant","path":"events::Event::Started","rust":"Started { execution: String }","loc":"src/events.rs:0"}
{"k":"doc","path":"events::Event::Started","text":"The run started."}
{"k":"field","path":"events::Event::Started::execution","rust":"execution: String","loc":"src/events.rs:0"}
{"k":"doc","path":"events::Event::Started::execution","text":"The run that produced the event, as the scheduler names it in every log line."}
{"k":"field","path":"events::Event::Stopped::execution","rust":"execution: String","loc":"src/events.rs:0"}
{"k":"doc","path":"events::Event::Stopped::execution","text":"The run that produced the event, as the scheduler names it in every log line."}
{"k":"field","path":"events::Event::Moved::execution","rust":"execution: String","loc":"src/events.rs:0"}
{"k":"doc","path":"events::Event::Moved::execution","text":"The run that produced the event, as the scheduler names it in every log line."}
{"k":"impl","path":"events::Event","rust":"impl Event","locs":["src/events.rs:0"],"via":"events!"}
{"k":"fn","path":"events::Event::execution","sig":"pub fn execution(&self) -> &str","loc":"src/events.rs:0","body":{"statements":1,"tokens":38}}
{"k":"doc","path":"events::Event::execution","text":"Returns the run that produced the event."}
{"k":"mod","path":"shapes","loc":"src/shapes.rs:0"}
{"k":"doc","path":"shapes","text":"A macro that the root module invokes, with an internal rule it invokes\nitself."}
{"k":"macro","path":"shapes::markers","rust":"macro_rules! markers","loc":"src/shapes.rs:0"}
{"k":"doc","path":"shapes::markers","text":"Writes one marker type per name, one name at a time."}
