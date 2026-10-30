*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"structs","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":20,"to":53}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Buffer","Buffer::data","Empty","Index::get","Marker","Meters","Pair","Point","Point::x","Point::y","Span","Token::new"],"evidence":"no hit of `Buffer` in the scanned sources beyond its 1 declaration; no hit of `data` in the scanned sources beyond its 1 declaration; no hit of `Empty` in the scanned sources beyond its 1 declaration; no hit of `get` in the scanned sources beyond its 1 declaration; no hit of `Marker` in the scanned sources beyond its 1 declaration; no hit of `Meters` in the scanned sources beyond its 1 declaration; no hit of `Pair` in the scanned sources ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":12}
{"k":"more","path":"Buffer","id":"no-observed-name-use","field":"evidence","part":2,"text":"beyond its 1 declaration; no hit of `Point` in the scanned sources beyond its 1 declaration; no hit of `x` in the scanned sources beyond its 1 declaration; no hit of `y` in the scanned sources beyond its 1 declaration; no hit of `Span` in the scanned sources beyond its 1 declaration; no hit of `new` in the scanned sources beyond its 1 declaration"}
{"k":"more","path":"Buffer","id":"no-observed-name-use","field":"locs","part":3,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Index"],"evidence":"1 production hits of `Index` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Buffer::data"],"evidence":"0 struct literals of `Buffer` set it, and no `.data` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Point::x"],"evidence":"0 struct literals of `Point` set it, and no `.x` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Point::y"],"evidence":"0 struct literals of `Point` set it, and no `.y` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Buffer","kind":"struct"}
{"k":"surface","path":"Empty","kind":"struct"}
{"k":"surface","path":"Index","kind":"struct"}
{"k":"surface","path":"Marker","kind":"struct"}
{"k":"surface","path":"Meters","kind":"struct"}
{"k":"surface","path":"Pair","kind":"struct"}
{"k":"surface","path":"Point","kind":"struct"}
{"k":"surface","path":"Span","kind":"struct"}
{"k":"surface","path":"Token","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":9}
{"k":"doc","path":"crate","text":"Struct shapes: named, tuple, unit, and generic."}
{"k":"struct","path":"Point","rust":"pub struct Point { pub x: i32, pub y: i32, label: String }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Point","text":"A named struct with public and private fields."}
{"k":"field","path":"Point::x","rust":"pub x: i32","loc":"src/lib.rs:0"}
{"k":"doc","path":"Point::x","text":"Horizontal position."}
{"k":"field","path":"Point::y","rust":"pub y: i32","loc":"src/lib.rs:0"}
{"k":"doc","path":"Point::y","text":"Vertical position."}
{"k":"struct","path":"Meters","rust":"pub struct Meters(pub f64)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Meters","text":"A tuple struct."}
{"k":"struct","path":"Pair","rust":"pub struct Pair(pub u8, u16)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Pair","text":"A tuple struct with two fields, one private."}
{"k":"struct","path":"Marker","rust":"pub struct Marker","loc":"src/lib.rs:0"}
{"k":"doc","path":"Marker","text":"A unit struct."}
{"k":"struct","path":"Buffer","rust":"pub struct Buffer<'a, T: Clone = u8, const N: usize = 16> { pub data: &'a [T; N], len: usize }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Buffer","text":"Generics with a lifetime, a bound, a default, and a const generic."}
{"k":"struct","path":"Span","rust":"pub struct Span(pub usize, pub usize)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Span","text":"A tuple struct whose fields carry comments."}
{"k":"field","path":"Span::0","rust":"pub usize","loc":"src/lib.rs:0"}
{"k":"doc","path":"Span::0","text":"The first byte."}
{"k":"field","path":"Span::1","rust":"pub usize","loc":"src/lib.rs:0"}
{"k":"doc","path":"Span::1","text":"One past the last byte."}
{"k":"struct","path":"Index","rust":"pub struct Index(usize)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Index","text":"A tuple struct with methods."}
{"k":"impl","path":"Index","rust":"impl Index","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Index::get","sig":"pub fn get(&self) -> usize","loc":"src/lib.rs:0","body":{"statements":1,"tokens":5,"single":"field 0"},"ops":["self.0"]}
{"k":"doc","path":"Index::get","text":"Returns the index."}
{"k":"struct","path":"Token","rust":"pub struct Token","loc":"src/lib.rs:0"}
{"k":"doc","path":"Token","text":"A unit struct with a method."}
{"k":"impl","path":"Token","rust":"impl Token","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Token::new","sig":"pub fn new() -> Token","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3}}
{"k":"doc","path":"Token::new","text":"Makes a token."}
{"k":"struct","path":"Empty","rust":"pub struct Empty {}","loc":"src/lib.rs:0"}
{"k":"doc","path":"Empty","text":"An empty braced struct."}
