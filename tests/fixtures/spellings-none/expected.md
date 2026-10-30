*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"spellings_none","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":16,"to":28}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Line::end"],"evidence":"no hit of `end` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Line::label"],"evidence":"no hit of `label` in the scanned sources beyond its 2 declarations","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Line::start"],"evidence":"no hit of `start` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Point::label"],"evidence":"no hit of `label` in the scanned sources beyond its 2 declarations","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Point::tags"],"evidence":"no hit of `tags` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Line"],"evidence":"2 production hits of `Line` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Line::end"],"evidence":"0 struct literals of `Line` set it, and no `.end` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Line::start"],"evidence":"0 struct literals of `Line` set it, and no `.start` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Point::label"],"evidence":"0 struct literals of `Point` set it, and no `.label` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Point::tags"],"evidence":"0 struct literals of `Point` set it, and no `.tags` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Line","kind":"struct"}
{"k":"surface","path":"Point","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":2}
{"k":"doc","path":"crate","text":"Spellings: each item from outside the crate is written one way, so the\noutline has no Spellings section. A derive name is never counted, so a\nderived Debug beside `fmt::Debug` in an impl is still one way."}
{"k":"struct","path":"Point","rust":"pub struct Point { pub label: Option<String>, pub tags: Vec<String> }","loc":"src/lib.rs:0","derives":["Clone","Debug","PartialEq"],"named_by":["Line"]}
{"k":"doc","path":"Point","text":"A point whose standard names each have one spelling."}
{"k":"impl","path":"Point","rust":"impl fmt::Display for Point","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Point::fmt","sig":"fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result","loc":"src/lib.rs:0","impl":"impl fmt::Display for Point","body":{"statements":2,"tokens":19,"nones":1,"macros":["write"]}}
{"k":"struct","path":"Line","rust":"pub struct Line { pub start: crate::Point, pub end: Point }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Line","text":"A local type written two ways, which the section leaves out."}
{"k":"impl","path":"Line","rust":"impl fmt::Debug for Line","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Line::fmt","sig":"fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result","loc":"src/lib.rs:0","impl":"impl fmt::Debug for Line","body":{"statements":1,"tokens":3,"macros":["write"]}}
{"k":"impl","path":"Line","rust":"impl Line","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Line::label","sig":"pub fn label<T: Clone>(&self, fallback: T) -> Option<T>","loc":"src/lib.rs:0","body":{"statements":2,"tokens":22},"ops":["fallback"]}
{"k":"doc","path":"Line::label","text":"Returns the label; see `std::option::Option`."}
