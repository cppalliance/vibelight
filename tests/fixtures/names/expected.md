*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"names","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":29,"to":51},{"path":"api","from":52,"to":56},{"path":"api::inner","from":57,"to":61},{"path":"model","from":62,"to":67}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Catalog","Catalog::fifth","Catalog::first","Catalog::fourth","Catalog::second","Catalog::third","Pairs","Twin","Wrapper"],"evidence":"no hit of `Catalog` in the scanned sources beyond its 1 declaration; no hit of `fifth` in the scanned sources beyond its 1 declaration; no hit of `first` in the scanned sources beyond its 1 declaration; no hit of `fourth` in the scanned sources beyond its 1 declaration; no hit of `second` in the scanned sources beyond its 1 declaration; no hit of `third` in the scanned sources beyond its 1 declaration; no hit of `Pairs` in the scanned sources ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":10}
{"k":"more","path":"Catalog","id":"no-observed-name-use","field":"evidence","part":2,"text":"beyond its 1 declaration; no hit of `Twin` in the scanned sources beyond its 2 declarations; no hit of `Wrapper` in the scanned sources beyond its 1 declaration"}
{"k":"more","path":"Catalog","id":"no-observed-name-use","field":"locs","part":3,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["model::Record::id"],"evidence":"no hit of `id` in the scanned sources beyond its 1 declaration","locs":["src/model.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["api::inner::Shared"],"evidence":"1 production hits of `Shared` outside macros, none a resolved or unique qualified path to it","locs":["src/api.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["model::Shared"],"evidence":"1 production hits of `Shared` outside macros, none a resolved or unique qualified path to it","locs":["src/model.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Catalog::fifth","Catalog::first","Catalog::fourth","Catalog::second","Catalog::third","Wrapper::inner"],"evidence":"0 struct literals of `Catalog` set it, and no `.fifth` access or pattern reads it; 0 struct literals of `Catalog` set it, and no `.first` access or pattern reads it; 0 struct literals of `Catalog` set it, and no `.fourth` access or pattern reads it; 0 struct literals of `Catalog` set it, and no `.second` access or pattern reads it; 0 struct literals of `Catalog` set it, and no `.third` access or pattern reads it; 0 struct literals of `Wrapper` set it, and no `.inner` access or pattern reads ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":6}
{"k":"more","path":"Catalog::fifth","id":"no-observed-field-read","field":"evidence","part":2,"text":"it"}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["model::Record::id"],"evidence":"0 struct literals of `Record` set it, and no `.id` access or pattern reads it","locs":["src/model.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"api","kind":"mod"}
{"k":"surface","path":"api::Handle","kind":"struct","item":"api::inner::Handle"}
{"k":"surface","path":"api::inner","kind":"mod"}
{"k":"surface","path":"api::inner::Handle","kind":"struct"}
{"k":"surface","path":"api::inner::Shared","kind":"struct"}
{"k":"surface","path":"api::Shared","kind":"struct","item":"api::inner::Shared"}
{"k":"surface","path":"api::Token","kind":"struct"}
{"k":"surface","path":"Catalog","kind":"struct"}
{"k":"surface","path":"Entry","kind":"struct","item":"model::Record"}
{"k":"surface","path":"model","kind":"mod"}
{"k":"surface","path":"model::Record","kind":"struct"}
{"k":"surface","path":"model::Shared","kind":"struct"}
{"k":"surface","path":"Pairs","kind":"alias"}
{"k":"surface","path":"Twin","kind":"struct"}
{"k":"surface","path":"Twin","kind":"struct"}
{"k":"surface","path":"Wrapper","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":5}
{"k":"doc","path":"crate","text":"Name resolution."}
{"k":"use","path":"Entry","rust":"pub use model::Record as Entry","loc":"src/lib.rs:0"}
{"k":"struct","path":"Catalog","rust":"pub struct Catalog { pub first: crate::model::Record, pub second: Entry, pub third: api::Handle, pub fourth: Token, pub fifth: Shared }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Catalog","text":"Uses names resolved every way rule 39 allows."}
{"k":"field","path":"Catalog::first","rust":"pub first: crate::model::Record","loc":"src/lib.rs:0"}
{"k":"doc","path":"Catalog::first","text":"Through a prefix path."}
{"k":"field","path":"Catalog::second","rust":"pub second: Entry","loc":"src/lib.rs:0"}
{"k":"doc","path":"Catalog::second","text":"Through a pub use rename at the root."}
{"k":"field","path":"Catalog::third","rust":"pub third: api::Handle","loc":"src/lib.rs:0"}
{"k":"doc","path":"Catalog::third","text":"Through a glob import."}
{"k":"field","path":"Catalog::fourth","rust":"pub fourth: Token","loc":"src/lib.rs:0"}
{"k":"doc","path":"Catalog::fourth","text":"A unique name across the target."}
{"k":"field","path":"Catalog::fifth","rust":"pub fifth: Shared","loc":"src/lib.rs:0"}
{"k":"doc","path":"Catalog::fifth","text":"An ambiguous name stays plain."}
{"k":"struct","path":"Wrapper","rust":"pub struct Wrapper<Record> { pub inner: Record }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Wrapper","text":"A generic parameter that shares a type's name stays plain."}
{"k":"alias","path":"Pairs","rust":"pub type Pairs<'a> = &'a [(Token, Option<Vec<Token>>)]","loc":"src/lib.rs:0"}
{"k":"doc","path":"Pairs","text":"Escapes in HTML: <, >, and & in types and comments."}
{"k":"struct","path":"Twin","rust":"pub struct Twin","loc":"src/lib.rs:0","cfgs":["unix"]}
{"k":"doc","path":"Twin","text":"The first of two types named Twin, under different cfgs."}
{"k":"struct","path":"Twin","rust":"pub struct Twin","loc":"src/lib.rs:0","cfgs":["windows"]}
{"k":"doc","path":"Twin","text":"The second Twin."}
{"k":"mod","path":"api","loc":"src/api.rs:0","vis":"public","types":1}
{"k":"doc","path":"api","text":"Re-exports through a glob."}
{"k":"use","path":"api::*","rust":"pub use self::inner::*","loc":"src/api.rs:0"}
{"k":"struct","path":"api::Token","rust":"pub struct Token","loc":"src/api.rs:0","named_by":["Catalog","Pairs"]}
{"k":"doc","path":"api::Token","text":"A unique token type."}
{"k":"mod","path":"api::inner","loc":"src/api.rs:0","vis":"public","types":2}
{"k":"struct","path":"api::inner::Handle","rust":"pub struct Handle","loc":"src/api.rs:0","named_by":["Catalog"]}
{"k":"doc","path":"api::inner::Handle","text":"A handle."}
{"k":"struct","path":"api::inner::Shared","rust":"pub struct Shared","loc":"src/api.rs:0"}
{"k":"doc","path":"api::inner::Shared","text":"The other type named Shared."}
{"k":"mod","path":"model","loc":"src/model.rs:0","vis":"public","types":2}
{"k":"doc","path":"model","text":"The record types."}
{"k":"struct","path":"model::Record","rust":"pub struct Record { pub id: u32 }","loc":"src/model.rs:0","named_by":["Catalog"],"reexported_by":["crate"]}
{"k":"doc","path":"model::Record","text":"A record."}
{"k":"struct","path":"model::Shared","rust":"pub struct Shared","loc":"src/model.rs:0"}
{"k":"doc","path":"model::Shared","text":"One of two types named Shared."}
