*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"receivers","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":9,"to":26}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["free","Node::boxed","Node::consume","Node::consume_mut","Node::counted","Node::exclusive","Node::explicit","Node::explicit_mut","Node::lifetime","Node::none","Node::pinned","Node::shared"],"evidence":"no hit of `free` in the scanned sources beyond its 1 declaration; no hit of `boxed` in the scanned sources beyond its 1 declaration; no hit of `consume` in the scanned sources beyond its 1 declaration; no hit of `consume_mut` in the scanned sources beyond its 1 declaration; no hit of `counted` in the scanned sources beyond its 1 declaration; no hit of `exclusive` in the scanned sources beyond its 1 declaration; no hit of `explicit` in the ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":12}
{"k":"more","path":"free","id":"no-observed-name-use","field":"evidence","part":2,"text":"scanned sources beyond its 1 declaration; no hit of `explicit_mut` in the scanned sources beyond its 1 declaration; no hit of `lifetime` in the scanned sources beyond its 1 declaration; no hit of `none` in the scanned sources beyond its 1 declaration; no hit of `pinned` in the scanned sources beyond its 1 declaration; no hit of `shared` in the scanned sources beyond its 1 declaration"}
{"k":"more","path":"free","id":"no-observed-name-use","field":"locs","part":3,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"surface","path":"free","kind":"fn"}
{"k":"surface","path":"Node","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":1,"functions":{"public":1}}
{"k":"doc","path":"crate","text":"Receivers."}
{"k":"struct","path":"Node","rust":"pub struct Node","loc":"src/lib.rs:0","named_by":["crate"]}
{"k":"doc","path":"Node","text":"Every receiver form."}
{"k":"impl","path":"Node","rust":"impl Node","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Node::shared","sig":"pub fn shared(&self) -> u8","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3}}
{"k":"fn","path":"Node::exclusive","sig":"pub fn exclusive(&mut self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::consume","sig":"pub fn consume(self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::consume_mut","sig":"pub fn consume_mut(mut self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::boxed","sig":"pub fn boxed(self: Box<Self>)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::counted","sig":"pub fn counted(self: Rc<Self>)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::pinned","sig":"pub fn pinned(self: Pin<&mut Self>)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::explicit","sig":"pub fn explicit(self: &Self) -> u8","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3}}
{"k":"fn","path":"Node::explicit_mut","sig":"pub fn explicit_mut(self: &mut Self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"Node::lifetime","sig":"pub fn lifetime<'a>(&'a self) -> &'a str","loc":"src/lib.rs:0","body":{"statements":1,"tokens":3}}
{"k":"fn","path":"Node::none","sig":"pub fn none()","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"fn","path":"free","sig":"pub fn free(node: &Node)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"free","text":"A free function never gets static."}
