*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"cfg_attr","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":15,"to":27}]}
{"k":"lead","id":"doc-echo","mode":"direct-syntax","subjects":["crate"],"evidence":"every content word of \"cfg and cfg_attr.\" is in its name, signature, or types","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Plain"],"evidence":"no hit of `Plain` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Settings::both"],"evidence":"no hit of `both` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Settings::doubly"],"evidence":"no hit of `doubly` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Settings::extra"],"evidence":"no hit of `extra` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Settings::gated"],"evidence":"no hit of `gated` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Settings"],"evidence":"1 production hits of `Settings` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Settings::both"],"evidence":"0 struct literals of `Settings` set it, and no `.both` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Settings::extra"],"evidence":"0 struct literals of `Settings` set it, and no `.extra` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Plain","kind":"struct"}
{"k":"surface","path":"Settings","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2}
{"k":"doc","path":"crate","text":"cfg and cfg_attr."}
{"k":"struct","path":"Settings","rust":"pub struct Settings { #[cfg(feature = \"extra\")] pub extra: u32, #[cfg(feature = \"fast\")] #[cfg(unix)] pub both: u8 }","loc":{"file":"src/lib.rs","line":0},"derives":["Clone","Debug"],"token_hits":1}
{"k":"doc","path":"Settings","text":"Derives through cfg_attr."}
{"k":"field","path":"Settings::extra","rust":"#[cfg(feature = \"extra\")] pub extra: u32","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Settings::extra","text":"Only with a feature."}
{"k":"impl","path":"Settings","rust":"impl Settings","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Settings::gated","sig":"pub fn gated(&self)","loc":{"file":"src/lib.rs","line":0},"cfgs":["feature = \"methods\""],"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Settings::gated","text":"Gated by the impl's cfg."}
{"k":"fn","path":"Settings::doubly","sig":"pub fn doubly(&self)","loc":{"file":"src/lib.rs","line":0},"cfgs":["feature = \"methods\"","windows"],"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Settings::doubly","text":"Also gated, with its own cfg."}
{"k":"struct","path":"Plain","rust":"pub struct Plain","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Plain","text":"A test-only cfg_attr is ignored."}
