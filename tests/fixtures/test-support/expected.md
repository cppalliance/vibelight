*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"test_support","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":16,"to":35},{"path":"fakes","from":36,"to":41}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Engine","Engine::model","Live","record","ScriptedChat","ScriptedChat::model","ScriptedChat::recording"],"evidence":"no hit of `Engine` in the scanned sources beyond its 1 declaration; no hit of `model` in the scanned sources beyond its 2 declarations; no hit of `Live` in the scanned sources beyond its 1 declaration; no hit of `record` in the scanned sources beyond its 1 declaration; no hit of `ScriptedChat` in the scanned sources beyond its 1 declaration; no hit of `recording` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":7}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["fakes::clock"],"evidence":"no hit of `clock` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Engine::model"],"evidence":"0 struct literals of `Engine` set it, and no `.model` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Engine","kind":"struct"}
{"k":"surface","path":"fakes","kind":"mod","test_support":true}
{"k":"surface","path":"fakes::clock","kind":"fn","test_support":true}
{"k":"surface","path":"fakes::FakeClock","kind":"struct","test_support":true}
{"k":"surface","path":"Live","kind":"struct"}
{"k":"surface","path":"Model","kind":"struct"}
{"k":"surface","path":"record","kind":"fn","test_support":true}
{"k":"surface","path":"Recording","kind":"struct"}
{"k":"surface","path":"ScriptedChat","kind":"struct","test_support":true}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":5,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Test-support code beside production code."}
{"k":"struct","path":"Engine","rust":"pub struct Engine { pub model: Model }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Engine","text":"A production engine."}
{"k":"field","path":"Engine::model","rust":"pub model: Model","loc":"src/lib.rs:0"}
{"k":"doc","path":"Engine::model","text":"The model it talks to."}
{"k":"struct","path":"Model","rust":"pub struct Model","loc":"src/lib.rs:0","named_by":["Engine","ScriptedChat"],"token_hits":2}
{"k":"doc","path":"Model","text":"A production model."}
{"k":"struct","path":"Recording","rust":"pub struct Recording","loc":"src/lib.rs:0","named_by":["crate","ScriptedChat"],"token_hits":2}
{"k":"doc","path":"Recording","text":"Used only by test support."}
{"k":"struct","path":"ScriptedChat","rust":"pub struct ScriptedChat { pub model: Model, pub recording: Recording }","loc":"src/lib.rs:0","cfgs":["any(test, feature = \"test-support\")"],"test_support":true}
{"k":"doc","path":"ScriptedChat","text":"A chat that plays back a script, for tests and benches."}
{"k":"field","path":"ScriptedChat::model","rust":"pub model: Model","loc":"src/lib.rs:0","test_support":true}
{"k":"doc","path":"ScriptedChat::model","text":"The model the script stands in for."}
{"k":"field","path":"ScriptedChat::recording","rust":"pub recording: Recording","loc":"src/lib.rs:0","test_support":true}
{"k":"doc","path":"ScriptedChat::recording","text":"Where it records."}
{"k":"struct","path":"Live","rust":"pub struct Live","loc":"src/lib.rs:0","cfgs":["not(test)"]}
{"k":"doc","path":"Live","text":"Production only."}
{"k":"fn","path":"record","sig":"pub fn record(into: &mut Recording)","loc":"src/lib.rs:0","cfgs":["any(test, feature = \"test-support\")"],"test_support":true,"body":{"statements":1,"tokens":7},"ops":["into"]}
{"k":"doc","path":"record","text":"Records into the recording."}
{"k":"mod","path":"fakes","loc":"src/lib.rs:0","vis":"public","cfgs":["any(test, feature = \"test-support\")"],"test_support":true,"types":1,"functions":{"public":1}}
{"k":"doc","path":"fakes","text":"Fakes for tests and benches."}
{"k":"struct","path":"fakes::FakeClock","rust":"pub struct FakeClock","loc":"src/lib.rs:0","test_support":true,"named_by":["fakes"],"token_hits":2}
{"k":"doc","path":"fakes::FakeClock","text":"A fake clock."}
{"k":"fn","path":"fakes::clock","sig":"pub fn clock() -> FakeClock","loc":"src/lib.rs:0","test_support":true,"body":{"statements":1,"tokens":3}}
{"k":"doc","path":"fakes::clock","text":"Makes a fake clock."}
