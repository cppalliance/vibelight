*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"facade","target":"lib","dir":"crates/facade"}
{"k":"modules","items":[{"path":"crate","from":16,"to":28}]}
{"k":"lead","id":"one-observed-trait-impl","mode":"resolved","subjects":["::alpha::Reads","Panel"],"evidence":"one impl in the run: `impl alpha::Reads for Panel`","locs":["alpha:src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-impls","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Panel::meter"],"evidence":"no hit of `meter` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["show"],"evidence":"no hit of `show` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Panel"],"evidence":"1 production hits of `Panel` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Panel::meter"],"evidence":"0 struct literals of `Panel` set it, and no `.meter` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Gauge","kind":"external","item":"beta::Gauge"}
{"k":"surface","path":"Meter","kind":"external","item":"alpha::Meter"}
{"k":"surface","path":"Panel","kind":"struct"}
{"k":"surface","path":"Reads","kind":"external","item":"alpha::Reads"}
{"k":"surface","path":"show","kind":"fn"}
{"k":"external","path":"alpha::Meter","uses":1,"named_by":["Panel"],"summary":"A meter that reads a gauge of beta."}
{"k":"external","path":"alpha::Reads","uses":2,"named_by":["crate","Panel"],"summary":"A trait other crates implement."}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Facade: re-exports the workspace's public types."}
{"k":"use","path":"Meter","rust":"pub use alpha::Meter","loc":"src/lib.rs:0"}
{"k":"use","path":"Reads","rust":"pub use alpha::Reads","loc":"src/lib.rs:0"}
{"k":"use","path":"Gauge","rust":"pub use beta::Gauge","loc":"src/lib.rs:0"}
{"k":"struct","path":"Panel","rust":"pub struct Panel { pub meter: alpha::Meter }","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Panel","text":"A panel that reads through alpha's trait."}
{"k":"field","path":"Panel::meter","rust":"pub meter: alpha::Meter","loc":"src/lib.rs:0"}
{"k":"doc","path":"Panel::meter","text":"A meter from alpha."}
{"k":"impl","path":"Panel","rust":"impl alpha::Reads for Panel","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Panel::read","sig":"fn read(&self) -> u32","loc":"src/lib.rs:0","impl":"impl alpha::Reads for Panel","body":{"statements":1,"tokens":3}}
{"k":"fn","path":"show","sig":"pub fn show<T: alpha::Reads>(reader: &T) -> u32","loc":"src/lib.rs:0","body":{"statements":1,"tokens":7,"single":"method read"},"ops":["reader.read"]}
{"k":"doc","path":"show","text":"Shows any reader."}
