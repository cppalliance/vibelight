*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"enums","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":10,"to":35}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Event","Event::Finished","Event::Started","Level::Low","Level::max","Shape","Shape::Named","Shape::Tuple","Shape::Unit","Shape::Valued"],"evidence":"no hit of `Event` in the scanned sources beyond its 1 declaration; no hit of `Finished` in the scanned sources beyond its 1 declaration; no hit of `Started` in the scanned sources beyond its 1 declaration; no hit of `Low` in the scanned sources beyond its 1 declaration; no hit of `max` in the scanned sources beyond its 1 declaration; no hit of `Shape` in the scanned sources beyond its 1 declaration; no hit of `Named` in the scanned sources beyond its 1 declaration; no hit of `Tuple` in the scanned sources beyond its 1 declaration; no hit of `Unit` in the scanned sources beyond its 1 declaration; no hit of `Valued` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":10}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Event::Finished","Event::Started","Level::Low","Shape::Named","Shape::Tuple","Shape::Unit","Shape::Valued"],"evidence":"no observed path builds `Event::Finished`; 0 patterns or comparisons match it; no observed path builds `Event::Started`; 0 patterns or comparisons match it; no observed path builds `Level::Low`; 0 patterns or comparisons match it; no observed path builds `Shape::Named`; 0 patterns or comparisons match it; no observed path builds `Shape::Tuple`; 0 patterns or comparisons match it; no observed path builds `Shape::Unit`; 0 patterns or comparisons match it; no observed path builds `Shape::Valued`; 0 patterns or comparisons match it","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":7}
{"k":"lead","id":"no-observed-pattern","mode":"resolved","subjects":["Level::High"],"evidence":"1 observed paths build `Level::High`, and no pattern or comparison matches it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"Event","kind":"enum"}
{"k":"surface","path":"Level","kind":"enum"}
{"k":"surface","path":"Shape","kind":"enum"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":3}
{"k":"doc","path":"crate","text":"Enum shapes."}
{"k":"enum","path":"Shape","rust":"pub enum Shape { Unit, Tuple(u8, String), Named { x: i32, y: i32 }, Valued = 4 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Shape","text":"Every variant shape."}
{"k":"variant","path":"Shape::Unit","rust":"Unit","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Shape::Unit","text":"A unit variant."}
{"k":"variant","path":"Shape::Tuple","rust":"Tuple(u8, String)","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Shape::Tuple","text":"A tuple variant."}
{"k":"variant","path":"Shape::Named","rust":"Named { x: i32, y: i32 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Shape::Named","text":"A struct-like variant."}
{"k":"variant","path":"Shape::Valued","rust":"Valued = 4","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Shape::Valued","text":"A variant with a discriminant."}
{"k":"enum","path":"Event","rust":"pub enum Event { Started { id: u64, name: String }, Finished { identifier: u64, summary: String, duration_in_milliseconds: u128 } }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Event","text":"Struct-like variants that expand."}
{"k":"variant","path":"Event::Started","rust":"Started { id: u64, name: String }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Event::Started","text":"Expands because a field has a comment."}
{"k":"field","path":"Event::Started::id","rust":"id: u64","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Event::Started::id","text":"The run identifier."}
{"k":"variant","path":"Event::Finished","rust":"Finished { identifier: u64, summary: String, duration_in_milliseconds: u128 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Event::Finished","text":"Expands because its one-line form is too wide."}
{"k":"enum","path":"Level","rust":"pub enum Level { Low, High }","loc":{"file":"src/lib.rs","line":0},"token_hits":3}
{"k":"doc","path":"Level","text":"An enum with methods."}
{"k":"impl","path":"Level","rust":"impl Level","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Level::max","sig":"pub fn max() -> Level","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":6}}
{"k":"doc","path":"Level::max","text":"Returns the highest level."}
{"k":"fn","path":"Level::rank","sig":"fn rank(&self) -> u8","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":3}}
