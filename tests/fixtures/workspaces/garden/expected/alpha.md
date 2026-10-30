*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"alpha","target":"lib","dir":"crates/alpha"}
{"k":"modules","items":[{"path":"crate","from":19,"to":33}]}
{"k":"lead","id":"one-observed-trait-impl","mode":"resolved","subjects":["Reads","::facade::Panel"],"evidence":"one impl in the run: `impl alpha::Reads for Panel`","locs":[{"file":"src/lib.rs","line":0},{"target":"facade","file":"src/lib.rs","line":0},{"target":"facade","file":"src/lib.rs","line":0}],"limits":["external-impls","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Meter::gauge"],"evidence":"no hit of `gauge` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Meter::stray"],"evidence":"no hit of `stray` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Meter::valve"],"evidence":"no hit of `valve` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Meter::vendored"],"evidence":"no hit of `vendored` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Meter::gauge"],"evidence":"0 struct literals of `Meter` set it, and no `.gauge` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Meter::stray"],"evidence":"0 struct literals of `Meter` set it, and no `.stray` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Meter::valve"],"evidence":"0 struct literals of `Meter` set it, and no `.valve` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Meter::vendored"],"evidence":"0 struct literals of `Meter` set it, and no `.vendored` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Meter","kind":"struct"}
{"k":"surface","path":"Reads","kind":"trait"}
{"k":"external","path":"b::Gauge","uses":1,"named_by":["Meter"],"summary":"A gauge."}
{"k":"external","path":"b::parts::Valve","uses":1,"named_by":["Meter"],"summary":"A valve."}
{"k":"external","path":"delta::Stray","uses":1,"named_by":["Meter"]}
{"k":"external","path":"gamma::Vendored","uses":1,"named_by":["Meter"]}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2}
{"k":"doc","path":"crate","text":"Alpha: a crate that names types of its dependencies."}
{"k":"struct","path":"Meter","rust":"pub struct Meter { pub gauge: Gauge, pub valve: b::parts::Valve, pub stray: delta::Stray, pub vendored: gamma::Vendored }","loc":{"file":"src/lib.rs","line":0},"reexported_by":["::facade"],"crates":["facade"],"token_hits":2}
{"k":"doc","path":"Meter","text":"A meter that reads a gauge of beta."}
{"k":"field","path":"Meter::gauge","rust":"pub gauge: Gauge","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Meter::gauge","text":"Through a use of the re-export at beta's root."}
{"k":"field","path":"Meter::valve","rust":"pub valve: b::parts::Valve","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Meter::valve","text":"Through beta's module path."}
{"k":"field","path":"Meter::stray","rust":"pub stray: delta::Stray","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Meter::stray","text":"A crate outside the run stays plain."}
{"k":"field","path":"Meter::vendored","rust":"pub vendored: gamma::Vendored","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Meter::vendored","text":"A crate that no follow glob allows stays plain."}
{"k":"trait","path":"Reads","rust":"pub trait Reads","loc":{"file":"src/lib.rs","line":0},"reexported_by":["::facade"],"crates":["facade"],"token_hits":3}
{"k":"doc","path":"Reads","text":"A trait other crates implement."}
{"k":"fn","path":"Reads::read","sig":"fn read(&self) -> u32","loc":{"file":"src/lib.rs","line":0},"method_name_hits":1,"token_hits":1}
