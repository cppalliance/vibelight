*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"extern_block","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":12,"to":21}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["abs"],"evidence":"no hit of `abs` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["errno"],"evidence":"no hit of `errno` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["exit"],"evidence":"no hit of `exit` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["GetTickCount"],"evidence":"no hit of `GetTickCount` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"abs","kind":"fn"}
{"k":"surface","path":"errno","kind":"value"}
{"k":"surface","path":"exit","kind":"fn"}
{"k":"surface","path":"GetTickCount","kind":"fn"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0"}
{"k":"doc","path":"crate","text":"Extern blocks."}
{"k":"extern","path":"abs","rust":"extern \"C\" { pub fn abs(x: i32) -> i32 }","loc":"src/lib.rs:0"}
{"k":"doc","path":"abs","text":"Absolute value."}
{"k":"extern","path":"errno","rust":"extern \"C\" { pub static errno: i32 }","loc":"src/lib.rs:0"}
{"k":"doc","path":"errno","text":"The last error."}
{"k":"extern","path":"secret","rust":"extern \"C\" { fn secret() -> u8 }","loc":"src/lib.rs:0"}
{"k":"extern","path":"exit","rust":"extern \"C\" { pub fn exit(code: i32) -> ! }","loc":"src/lib.rs:0"}
{"k":"doc","path":"exit","text":"Exits the process."}
{"k":"extern","path":"GetTickCount","rust":"extern \"system\" { pub fn GetTickCount() -> u32 }","loc":"src/lib.rs:0"}
