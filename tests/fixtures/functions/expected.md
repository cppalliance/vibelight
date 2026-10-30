*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"functions","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":15,"to":35}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["all_qualifiers","apply","callback","merge","parse","peek","render","square","wait"],"evidence":"no hit of `all_qualifiers` in the scanned sources beyond its 1 declaration; no hit of `apply` in the scanned sources beyond its 1 declaration; no hit of `callback` in the scanned sources beyond its 1 declaration; no hit of `merge` in the scanned sources beyond its 1 declaration; no hit of `parse` in the scanned sources beyond its 1 declaration; no hit of `peek` in the scanned sources beyond its 1 declaration; no hit of `render` in the scanned sources ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":9}
{"k":"more","path":"all_qualifiers","id":"no-observed-name-use","field":"evidence","part":2,"text":"beyond its 1 declaration; no hit of `square` in the scanned sources beyond its 1 declaration; no hit of `wait` in the scanned sources beyond its 1 declaration"}
{"k":"surface","path":"all_qualifiers","kind":"fn"}
{"k":"surface","path":"apply","kind":"fn"}
{"k":"surface","path":"callback","kind":"fn"}
{"k":"surface","path":"merge","kind":"fn"}
{"k":"surface","path":"parse","kind":"fn"}
{"k":"surface","path":"peek","kind":"fn"}
{"k":"surface","path":"render","kind":"fn"}
{"k":"surface","path":"square","kind":"fn"}
{"k":"surface","path":"wait","kind":"fn"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","functions":{"public":9,"private":1}}
{"k":"doc","path":"crate","text":"Functions and signatures."}
{"k":"fn","path":"square","sig":"pub const fn square(x: u32) -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5},"ops":["x"]}
{"k":"doc","path":"square","text":"Computes at compile time."}
{"k":"fn","path":"wait","sig":"pub async fn wait(seconds: u64)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"wait","text":"Waits for input."}
{"k":"fn","path":"peek","sig":"pub unsafe fn peek(ptr: *const u8) -> u8","loc":"src/lib.rs:0","body":{"statements":1,"tokens":7},"ops":["ptr"]}
{"k":"doc","path":"peek","text":"Reads raw memory.\n\n# Safety\nThe pointer must be valid."}
{"k":"fn","path":"callback","sig":"pub extern \"C\" fn callback(code: i32) -> i32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3},"ops":["code"]}
{"k":"doc","path":"callback","text":"Exported to C."}
{"k":"fn","path":"all_qualifiers","sig":"pub const unsafe extern \"C\" fn all_qualifiers()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"all_qualifiers","text":"Every qualifier at once."}
{"k":"fn","path":"private_helper","sig":"fn private_helper()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"render","sig":"pub fn render(out: &mut impl std::io::Write, items: &[Box<dyn std::fmt::Display>])","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"render","text":"Takes impl and dyn traits."}
{"k":"fn","path":"apply","sig":"pub fn apply<F>(f: F) -> usize where F: for<'a> Fn(&'a str) -> usize","loc":"src/lib.rs:0","body":{"statements":1,"tokens":6,"single":"call f"}}
{"k":"doc","path":"apply","text":"Uses a higher-ranked bound."}
{"k":"fn","path":"merge","sig":"pub fn merge<A, B, C>(a: A, b: B) -> C where A: Into<C>, B: Into<C>, C: std::ops::Add<Output = C>","loc":"src/lib.rs:0","body":{"statements":1,"tokens":13},"ops":["a.into","b.into"]}
{"k":"doc","path":"merge","text":"Has a where clause on several lines."}
{"k":"fn","path":"parse","sig":"pub fn parse(input: &str, execution: &str) -> (std::result::Result<String, String>, Vec<String>)","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3,"panics":1,"macros":["todo"]}}
{"k":"doc","path":"parse","text":"A rustfmt-style signature over several lines."}
