*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"facts","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":22,"to":58}]}
{"k":"lead","id":"option-shaped-enum","mode":"direct-syntax","subjects":["ChatDispatch"],"evidence":"`Stop` beside `Send(String)`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"option-shaped-enum","mode":"direct-syntax","subjects":["ToolCallDispatch"],"evidence":"`Stop` beside `Send(String)`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"same-shape","mode":"direct-syntax","subjects":["ChatDispatch","ToolCallDispatch"],"evidence":"enums with the same 2 variants","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"same-shape","mode":"direct-syntax","subjects":["Point","Size"],"evidence":"structs with the same 2 fields","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["ChatDispatch::Send","dispatch","Point","Point::x","Point::y","Run::MAX","Run::stop","Size","Size::x","Size::y","ToolCallDispatch::Send"],"evidence":"no hit of `Send` in the scanned sources beyond its 2 declarations; no hit of `dispatch` in the scanned sources beyond its 1 declaration; no hit of `Point` in the scanned sources beyond its 1 declaration; no hit of `x` in the scanned sources beyond its 2 declarations; no hit of `y` in the scanned sources beyond its 2 declarations; no hit of `MAX` in the scanned sources beyond its 1 declaration; no hit of `stop` in the scanned sources beyond its 1 declaration; no hit of `Size` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":11}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["ChatDispatch::Send"],"evidence":"no observed path builds `ChatDispatch::Send`; 0 patterns or comparisons match it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["ToolCallDispatch::Send"],"evidence":"no observed path builds `ToolCallDispatch::Send`; 0 patterns or comparisons match it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["ToolCallDispatch::Stop"],"evidence":"no observed path builds `ToolCallDispatch::Stop`; 0 patterns or comparisons match it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-pattern","mode":"resolved","subjects":["ChatDispatch::Stop"],"evidence":"2 observed paths build `ChatDispatch::Stop`, and no pattern or comparison matches it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Point::x","Point::y","Run::limit","Run::name","Run::started","Size::x","Size::y"],"evidence":"0 struct literals of `Point` set it, and no `.x` access or pattern reads it; 0 struct literals of `Point` set it, and no `.y` access or pattern reads it; 1 struct literals of `Run` set it, and no `.limit` access or pattern reads it; 1 struct literals of `Run` set it, and no `.name` access or pattern reads it; 1 struct literals of `Run` set it, and no `.started` access or pattern reads it; 0 struct literals of `Size` set it, and no `.x` access or pattern reads it; 0 struct literals of `Size` set it, and no `.y` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":7}
{"k":"surface","path":"ChatDispatch","kind":"enum"}
{"k":"surface","path":"dispatch","kind":"fn"}
{"k":"surface","path":"Point","kind":"struct"}
{"k":"surface","path":"Run","kind":"struct"}
{"k":"surface","path":"Size","kind":"struct"}
{"k":"surface","path":"ToolCallDispatch","kind":"enum"}
{"k":"external","path":"chrono::DateTime","uses":1,"named_by":["Run"]}
{"k":"external","path":"chrono::Utc","uses":2,"named_by":["crate","Run"]}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":5,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Facts about each declaration: what it holds, what names it, and which\ntypes share a shape."}
{"k":"enum","path":"ChatDispatch","rust":"pub enum ChatDispatch { Send(String), Stop }","loc":{"file":"src/lib.rs","line":0},"named_by":["Run"],"token_hits":3}
{"k":"doc","path":"ChatDispatch","text":"A chat request."}
{"k":"variant","path":"ChatDispatch::Send","rust":"Send(String)","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"ChatDispatch::Send","text":"Send a message."}
{"k":"variant","path":"ChatDispatch::Stop","rust":"Stop","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"ChatDispatch::Stop","text":"Stop the chat."}
{"k":"enum","path":"ToolCallDispatch","rust":"pub enum ToolCallDispatch { Send(String), Stop }","loc":{"file":"src/lib.rs","line":0},"named_by":["crate"],"token_hits":1}
{"k":"doc","path":"ToolCallDispatch","text":"A tool call request."}
{"k":"variant","path":"ToolCallDispatch::Send","rust":"Send(String)","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"ToolCallDispatch::Send","text":"Run a tool."}
{"k":"variant","path":"ToolCallDispatch::Stop","rust":"Stop","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"ToolCallDispatch::Stop","text":"Halt."}
{"k":"struct","path":"Run","rust":"pub struct Run { pub name: String, pub limit: Option<u32>, pub started: chrono::DateTime<chrono::Utc>, chat: ChatDispatch }","loc":{"file":"src/lib.rs","line":0},"token_hits":5}
{"k":"doc","path":"Run","text":"A run of the engine."}
{"k":"field","path":"Run::name","rust":"pub name: String","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Run::name","text":"The name of the run."}
{"k":"field","path":"Run::limit","rust":"pub limit: Option<u32>","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Run::limit","text":"The limit, when there is one."}
{"k":"field","path":"Run::started","rust":"pub started: chrono::DateTime<chrono::Utc>","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Run::started","text":"When the run started."}
{"k":"impl","path":"Run","rust":"impl Run","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"const","path":"Run::MAX","rust":"pub const MAX: u32","value":"9","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"Run::MAX","text":"The largest run."}
{"k":"fn","path":"Run::start","sig":"pub fn start(name: String) -> Run","loc":{"file":"src/lib.rs","line":0},"direct_callers":["Run::default"],"token_hits":1,"body":{"statements":1,"tokens":26,"nones":1},"ops":["name"]}
{"k":"doc","path":"Run::start","text":"Starts a run."}
{"k":"fn","path":"Run::stop","sig":"pub fn stop(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":11},"ops":["self.chat [write]"]}
{"k":"doc","path":"Run::stop","text":"Stops it."}
{"k":"impl","path":"Run","rust":"impl Default for Run","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Run::default","sig":"fn default() -> Self","loc":{"file":"src/lib.rs","line":0},"impl":"impl Default for Run","body":{"statements":1,"tokens":14,"single":"call Run::start"}}
{"k":"struct","path":"Point","rust":"pub struct Point { pub x: i32, pub y: i32 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Point","text":"A point."}
{"k":"struct","path":"Size","rust":"pub struct Size { pub x: i32, pub y: i32 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Size","text":"A size."}
{"k":"fn","path":"dispatch","sig":"pub fn dispatch(call: ToolCallDispatch, at: chrono::Utc)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":10},"ops":["at","call"]}
{"k":"doc","path":"dispatch","text":"Starts a tool call."}
