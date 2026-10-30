*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"raw_idents","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":16,"to":26},{"path":"r#impl","from":27,"to":30}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["r#impl"],"evidence":"holds only `Inner`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["r#fn"],"evidence":"no hit of `fn` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["r#impl::Inner"],"evidence":"no hit of `Inner` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Keywords::r#as"],"evidence":"no hit of `as` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Keywords::r#async"],"evidence":"no hit of `async` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Keywords::r#match"],"evidence":"no hit of `match` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Keywords"],"evidence":"1 production hits of `Keywords` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Keywords::r#match"],"evidence":"0 struct literals of `Keywords` set it, and no `.match` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"r#fn","kind":"fn"}
{"k":"surface","path":"r#impl","kind":"mod"}
{"k":"surface","path":"r#impl::Inner","kind":"struct"}
{"k":"surface","path":"Keywords","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Raw identifiers."}
{"k":"struct","path":"Keywords","rust":"pub struct Keywords { pub r#type: u8, pub r#match: u8, kind: u8 }","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Keywords","text":"Raw names in a struct."}
{"k":"field","path":"Keywords::r#type","rust":"pub r#type: u8","loc":"src/lib.rs:0"}
{"k":"doc","path":"Keywords::r#type","text":"A raw field."}
{"k":"impl","path":"Keywords","rust":"impl Keywords","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Keywords::r#as","sig":"pub fn r#as(&self) -> u8","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field type"},"ops":["self.type"]}
{"k":"doc","path":"Keywords::r#as","text":"A raw method."}
{"k":"fn","path":"Keywords::r#async","sig":"pub fn r#async()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"r#fn","sig":"pub fn r#fn()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"mod","path":"r#impl","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"r#impl","text":"A module with a raw name."}
{"k":"struct","path":"r#impl::Inner","rust":"pub struct Inner","loc":"src/lib.rs:0"}
{"k":"doc","path":"r#impl::Inner","text":"Lives in a raw module."}
