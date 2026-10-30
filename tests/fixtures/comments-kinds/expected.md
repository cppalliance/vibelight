*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"comments_kinds","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":11,"to":22}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["after_block","BlockComment","DocFirst","Fenced","Linked","PlainFirst"],"evidence":"no hit of `after_block` in the scanned sources beyond its 1 declaration; no hit of `BlockComment` in the scanned sources beyond its 1 declaration; no hit of `DocFirst` in the scanned sources beyond its 1 declaration; no hit of `Fenced` in the scanned sources beyond its 1 declaration; no hit of `Linked` in the scanned sources beyond its 1 declaration; no hit of `PlainFirst` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":6}
{"k":"surface","path":"after_block","kind":"fn"}
{"k":"surface","path":"BlockComment","kind":"struct"}
{"k":"surface","path":"DocFirst","kind":"struct"}
{"k":"surface","path":"Fenced","kind":"struct"}
{"k":"surface","path":"Linked","kind":"struct"}
{"k":"surface","path":"PlainFirst","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":5,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Comment kinds and their order.\n\nCrate docs print as `//!` lines."}
{"k":"struct","path":"DocFirst","rust":"pub struct DocFirst","loc":{"file":"src/lib.rs","line":0},"comment":"// Then an ordinary comment."}
{"k":"doc","path":"DocFirst","text":"A doc comment first."}
{"k":"struct","path":"PlainFirst","rust":"pub struct PlainFirst","loc":{"file":"src/lib.rs","line":0},"comment":"// An ordinary comment first."}
{"k":"doc","path":"PlainFirst","text":"Then a doc comment."}
{"k":"struct","path":"BlockComment","rust":"pub struct BlockComment","loc":{"file":"src/lib.rs","line":0},"comment":"/* A block comment\nkept as written. */"}
{"k":"struct","path":"Fenced","rust":"pub struct Fenced","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Fenced","text":"Docs with their own code fence:\n\n```\nlet x = [1, 2];\n```\n\nAnd a ```` four-backtick run inside."}
{"k":"struct","path":"Linked","rust":"pub struct Linked","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Linked","text":"Links: [`DocFirst`], [the plain one](PlainFirst), and `inline code`."}
{"k":"fn","path":"after_block","sig":"pub fn after_block()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2},"comment":"/*\n```\nnot code\n```\n*/"}
