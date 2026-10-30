*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"struct_class","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":15,"to":33}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Bare","Mixed","Mixed::open","Toggle::Off","Toggle::On","Wrapper"],"evidence":"no hit of `Bare` in the scanned sources beyond its 1 declaration; no hit of `Mixed` in the scanned sources beyond its 1 declaration; no hit of `open` in the scanned sources beyond its 1 declaration; no hit of `Off` in the scanned sources beyond its 1 declaration; no hit of `On` in the scanned sources beyond its 1 declaration; no hit of `Wrapper` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":6}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Toggle"],"evidence":"1 production hits of `Toggle` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Toggle::Off"],"evidence":"no observed path builds `Toggle::Off`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Toggle::On"],"evidence":"no observed path builds `Toggle::On`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Mixed::open"],"evidence":"0 struct literals of `Mixed` set it, and no `.open` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"forwarder","mode":"direct-syntax","subjects":["Cursor::into_text"],"evidence":"a private method that only reads the private field `text`","locs":["src/lib.rs:0"]}
{"k":"surface","path":"Bare","kind":"struct"}
{"k":"surface","path":"Cursor","kind":"struct"}
{"k":"surface","path":"Mixed","kind":"struct"}
{"k":"surface","path":"Toggle","kind":"enum"}
{"k":"surface","path":"Wrapper","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":5}
{"k":"doc","path":"crate","text":"Struct versus class."}
{"k":"struct","path":"Mixed","rust":"pub struct Mixed { pub open: u8, closed: u8 }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Mixed","text":"Public and private members."}
{"k":"struct","path":"Cursor","rust":"pub struct Cursor { text: String, pos: usize }","loc":"src/lib.rs:0","token_hits":3}
{"k":"doc","path":"Cursor","text":"Holds the text the lexer is still reading."}
{"k":"impl","path":"Cursor","rust":"impl Cursor","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Cursor::new","sig":"fn new(text: String) -> Cursor","loc":"src/lib.rs:0","body":{"statements":1,"tokens":9},"ops":["text"]}
{"k":"fn","path":"Cursor::peek","sig":"fn peek(&self) -> Option<char>","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3,"nones":1}}
{"k":"fn","path":"Cursor::bump","sig":"fn bump(&mut self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Cursor::into_text","sig":"fn into_text(self) -> String","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field text"},"ops":["self.text"]}
{"k":"struct","path":"Bare","rust":"pub struct Bare {}","loc":"src/lib.rs:0"}
{"k":"doc","path":"Bare","text":"No members at all."}
{"k":"enum","path":"Toggle","rust":"pub enum Toggle { On, Off }","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Toggle","text":"An enum with only private methods."}
{"k":"impl","path":"Toggle","rust":"impl Toggle","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Toggle::flip","sig":"fn flip(&mut self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"struct","path":"Wrapper","rust":"pub struct Wrapper(u8)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Wrapper","text":"A tuple struct with only private fields."}
