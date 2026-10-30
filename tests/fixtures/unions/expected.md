*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"unions","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":10,"to":19}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Bits"],"evidence":"no hit of `Bits` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Bits::float"],"evidence":"no hit of `float` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Bits::int"],"evidence":"no hit of `int` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Hidden"],"evidence":"no hit of `Hidden` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Bits","kind":"union"}
{"k":"surface","path":"Hidden","kind":"union"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2}
{"k":"doc","path":"crate","text":"Unions."}
{"k":"union","path":"Bits","rust":"pub union Bits { pub int: u32, pub float: f32, raw: [u8; 4] }","loc":{"file":"src/lib.rs","line":0},"attrs":["repr(C)"]}
{"k":"doc","path":"Bits","text":"Raw bits viewed two ways."}
{"k":"field","path":"Bits::int","rust":"pub int: u32","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Bits::int","text":"As an integer."}
{"k":"field","path":"Bits::float","rust":"pub float: f32","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Bits::float","text":"As a float."}
{"k":"union","path":"Hidden","rust":"pub union Hidden { b: i8, a: u8 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Hidden","text":"A union with only private fields."}
