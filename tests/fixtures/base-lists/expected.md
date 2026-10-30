*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"base_lists","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":57,"to":132},{"path":"bare","from":133,"to":137},{"path":"custom","from":138,"to":144},{"path":"globbed","from":145,"to":149},{"path":"other","from":150,"to":154}]}
{"k":"lead","id":"single-variant-enum","mode":"direct-syntax","subjects":["Error"],"evidence":"one variant, `Failed`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"one-observed-trait-impl","mode":"resolved","subjects":["Clone","Local"],"evidence":"one impl in the run: `impl Clone for Local`","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-impls","macros"]}
{"k":"lead","id":"one-observed-trait-impl","mode":"resolved","subjects":["custom::Display","custom::Homemade"],"evidence":"one impl in the run: `impl Display for Homemade`","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-impls","macros"]}
{"k":"lead","id":"prelude-name","mode":"direct-syntax","subjects":["Clone"],"evidence":"named like `std::clone::Clone`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["bare"],"evidence":"holds only `Imported`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["globbed"],"evidence":"holds only `Starred`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["other"],"evidence":"holds only `Foreign`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Bits","Bits::float","Bits::int","Error::Failed","Holder::value","Labeled","Marker","Named::name","Plain","Run::id","RunError::reason","Wrapped"],"evidence":"no hit of `Bits` in the scanned sources beyond its 1 declaration; no hit of `float` in the scanned sources beyond its 1 declaration; no hit of `int` in the scanned sources beyond its 1 declaration; no hit of `Failed` in the scanned sources beyond its 1 declaration; no hit of `value` in the scanned sources beyond its 2 declarations; no hit of `Labeled` in the scanned sources beyond its 1 declaration; no hit of `Marker` in the scanned sources ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":12}
{"k":"more","path":"Bits","id":"no-observed-name-use","field":"evidence","part":2,"text":"beyond its 1 declaration; no hit of `name` in the scanned sources beyond its 4 declarations; no hit of `Plain` in the scanned sources beyond its 1 declaration; no hit of `id` in the scanned sources beyond its 1 declaration; no hit of `reason` in the scanned sources beyond its 1 declaration; no hit of `Wrapped` in the scanned sources beyond its 1 declaration"}
{"k":"more","path":"Bits","id":"no-observed-name-use","field":"locs","part":3,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["bare::Imported"],"evidence":"1 production hits of `Imported` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["custom::Homemade"],"evidence":"1 production hits of `Homemade` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Error","Holder","Local","Mixed","Ordered","Renamed","Run","RunError","Secret","Sink","Spelled","Wide"],"evidence":"4 production hits of `Error` outside macros, none a resolved or unique qualified path to it; 2 production hits of `Holder` outside macros, none a resolved or unique qualified path to it; 3 production hits of `Local` outside macros, none a resolved or unique qualified path to it; 3 production hits of `Mixed` outside macros, none a resolved or unique qualified path to it; 2 production hits of `Ordered` outside macros, none a resolved or unique ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","macros"],"module":"crate","count":12}
{"k":"more","path":"Error","id":"no-observed-path-use","field":"evidence","part":2,"text":"qualified path to it; 1 production hits of `Renamed` outside macros, none a resolved or unique qualified path to it; 2 production hits of `Run` outside macros, none a resolved or unique qualified path to it; 3 production hits of `RunError` outside macros, none a resolved or unique qualified path to it; 1 production hits of `Secret` outside macros, none a resolved or unique qualified path to it; 2 production hits of `Sink` outside macros, none a resolved or unique qualified path to it; 1 production hits of `Spelled` outside macros, none a resolved or unique qualified path to it; 8 production hits of `Wide` outside macros, none a resolved or unique qualified path to it"}
{"k":"more","path":"Error","id":"no-observed-path-use","field":"locs","part":3,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["globbed::Starred"],"evidence":"1 production hits of `Starred` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["other::Foreign"],"evidence":"1 production hits of `Foreign` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-construction","mode":"resolved","subjects":["Error::Failed"],"evidence":"no observed path builds `Error::Failed`; 0 patterns or comparisons match it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Holder::value"],"evidence":"0 struct literals of `Holder` set it, and no `.value` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Run::id"],"evidence":"0 struct literals of `Run` set it, and no `.id` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["RunError::reason"],"evidence":"0 struct literals of `RunError` set it, and no `.reason` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"bare","kind":"mod"}
{"k":"surface","path":"bare::Imported","kind":"struct"}
{"k":"surface","path":"Bits","kind":"union"}
{"k":"surface","path":"Clone","kind":"trait"}
{"k":"surface","path":"custom","kind":"mod"}
{"k":"surface","path":"custom::Display","kind":"trait"}
{"k":"surface","path":"custom::Homemade","kind":"struct"}
{"k":"surface","path":"Error","kind":"enum"}
{"k":"surface","path":"globbed","kind":"mod"}
{"k":"surface","path":"globbed::Starred","kind":"struct"}
{"k":"surface","path":"Holder","kind":"struct"}
{"k":"surface","path":"Labeled","kind":"trait"}
{"k":"surface","path":"Local","kind":"struct"}
{"k":"surface","path":"Marker","kind":"struct"}
{"k":"surface","path":"Mixed","kind":"struct"}
{"k":"surface","path":"Named","kind":"trait"}
{"k":"surface","path":"Ordered","kind":"struct"}
{"k":"surface","path":"other","kind":"mod"}
{"k":"surface","path":"other::Foreign","kind":"struct"}
{"k":"surface","path":"Plain","kind":"trait"}
{"k":"surface","path":"Renamed","kind":"struct"}
{"k":"surface","path":"Run","kind":"struct"}
{"k":"surface","path":"RunError","kind":"struct"}
{"k":"surface","path":"Secret","kind":"struct"}
{"k":"surface","path":"Sink","kind":"struct"}
{"k":"surface","path":"Spelled","kind":"struct"}
{"k":"surface","path":"Wide","kind":"struct"}
{"k":"surface","path":"Wrapped","kind":"struct"}
{"k":"external","path":"other_crate::Display","uses":1,"named_by":["other::Foreign"]}
{"k":"external","path":"serde::Serialize","uses":1,"named_by":["Mixed"]}
{"k":"spelling","path":"std::fmt::Debug","preferred":"Debug","spellings":[{"text":"Dbg","count":1,"locs":["src/lib.rs:0"]},{"text":"fmt::Debug","count":4,"locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}]}
{"k":"spelling","path":"std::fmt::Display","spellings":[{"text":"D","count":1,"locs":["src/lib.rs:0"]},{"text":"fmt::Display","count":5,"locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]},{"text":"std::fmt::Display","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":19}
{"k":"doc","path":"crate","text":"Base lists: short names, long paths, wrapping, collisions, and supertraits."}
{"k":"enum","path":"Error","rust":"pub enum Error { Failed }","loc":"src/lib.rs:0","derives":["Debug"],"token_hits":4}
{"k":"doc","path":"Error","text":"The crate's own error, whose name collides with std::error::Error."}
{"k":"impl","path":"Error","rust":"impl std::error::Error for Error","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Error","rust":"impl Named for Error","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Error::name","sig":"fn name(&self) -> String","loc":"src/lib.rs:0","impl":"impl Named for Error","body":{"statements":1,"tokens":8,"single":"call String::new"}}
{"k":"trait","path":"Named","rust":"pub trait Named","loc":"src/lib.rs:0","named_by":["Error","Holder","Labeled","Mixed"],"token_hits":4}
{"k":"doc","path":"Named","text":"A trait of this crate."}
{"k":"fn","path":"Named::name","sig":"fn name(&self) -> String","loc":"src/lib.rs:0"}
{"k":"trait","path":"Labeled","rust":"pub trait Labeled: fmt::Display + Named","loc":"src/lib.rs:0"}
{"k":"doc","path":"Labeled","text":"A standard supertrait and a supertrait of this crate."}
{"k":"trait","path":"Plain","rust":"pub trait Plain: Send + Sync + 'static","loc":"src/lib.rs:0"}
{"k":"doc","path":"Plain","text":"Standard supertraits and a lifetime bound."}
{"k":"struct","path":"Run","rust":"pub struct Run { pub id: u32 }","loc":"src/lib.rs:0","token_hits":2}
{"k":"doc","path":"Run","text":"Standard traits only, so the list stays on the head line."}
{"k":"impl","path":"Run","rust":"impl fmt::Debug for Run","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Run","rust":"impl fmt::Display for Run","locs":["src/lib.rs:0"]}
{"k":"struct","path":"RunError","rust":"pub struct RunError { pub reason: String }","loc":"src/lib.rs:0","token_hits":3}
{"k":"doc","path":"RunError","text":"The crate's Error makes std::error::Error keep its full path."}
{"k":"impl","path":"RunError","rust":"impl fmt::Debug for RunError","locs":["src/lib.rs:0"]}
{"k":"impl","path":"RunError","rust":"impl fmt::Display for RunError","locs":["src/lib.rs:0"]}
{"k":"impl","path":"RunError","rust":"impl std::error::Error for RunError","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Mixed","rust":"pub struct Mixed","loc":"src/lib.rs:0","token_hits":3}
{"k":"doc","path":"Mixed","text":"A trait of this crate and one of another crate take the comma form."}
{"k":"impl","path":"Mixed","rust":"impl fmt::Debug for Mixed","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Mixed","rust":"impl Named for Mixed","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Mixed::name","sig":"fn name(&self) -> String","loc":"src/lib.rs:0","impl":"impl Named for Mixed","body":{"statements":1,"tokens":8,"single":"call String::new"}}
{"k":"impl","path":"Mixed","rust":"impl serde::Serialize for Mixed","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Sink","rust":"pub struct Sink","loc":"src/lib.rs:0","token_hits":2}
{"k":"doc","path":"Sink","text":"Two traits named Write keep their full paths."}
{"k":"impl","path":"Sink","rust":"impl fmt::Write for Sink","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Sink","rust":"impl io::Write for Sink","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Wide","rust":"pub struct Wide(u64)","loc":"src/lib.rs:0","derives":["Copy","Debug","Eq","Hash","Ord"],"token_hits":8}
{"k":"doc","path":"Wide","text":"So many standard traits that the list wraps."}
{"k":"impl","path":"Wide","rust":"impl fmt::Binary for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::Display for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::LowerExp for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::LowerHex for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::Octal for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::Pointer for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::UpperExp for Wide","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Wide","rust":"impl fmt::UpperHex for Wide","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Holder","rust":"pub struct Holder<T> where T: Copy { pub value: T }","loc":"src/lib.rs:0","token_hits":2}
{"k":"doc","path":"Holder","text":"Generics, a where clause, and a trait of this crate."}
{"k":"impl","path":"Holder","rust":"impl<T: Copy> fmt::Debug for Holder<T>","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Holder","rust":"impl<T> Named for Holder<T> where T: Copy","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Holder::name","sig":"fn name(&self) -> String","loc":"src/lib.rs:0","impl":"impl<T> Named for Holder<T> where T: Copy","body":{"statements":1,"tokens":8,"single":"call String::new"}}
{"k":"union","path":"Bits","rust":"pub union Bits { pub int: u32, pub float: f32 }","loc":"src/lib.rs:0","derives":["Copy","Debug"]}
{"k":"doc","path":"Bits","text":"A union with standard derives."}
{"k":"struct","path":"Secret","rust":"pub struct Secret { value: u8 }","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Secret","text":"A class with a standard trait."}
{"k":"impl","path":"Secret","rust":"impl fmt::Display for Secret","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Marker","rust":"pub struct Marker","loc":"src/lib.rs:0","derives":["Debug","Eq","Hash"]}
{"k":"doc","path":"Marker","text":"A unit struct whose list fits on the head line."}
{"k":"trait","path":"Clone","rust":"pub trait Clone","loc":"src/lib.rs:0","named_by":["Local"],"token_hits":2}
{"k":"doc","path":"Clone","text":"A trait of this crate that shares a prelude name."}
{"k":"struct","path":"Local","rust":"pub struct Local","loc":"src/lib.rs:0","token_hits":3}
{"k":"doc","path":"Local","text":"The crate's own Clone, a trait nothing declares, and a renamed standard\ntrait."}
{"k":"impl","path":"Local","rust":"impl Clone for Local","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Local","rust":"impl Unknown for Local","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Local","rust":"impl Dbg for Local","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Wrapped","rust":"pub struct Wrapped(u8)","loc":"src/lib.rs:0","derives":["Debug","From"]}
{"k":"doc","path":"Wrapped","text":"A bare derive of From comes from another crate, so it stays on the mark."}
{"k":"struct","path":"Ordered","rust":"pub struct Ordered(u8)","loc":"src/lib.rs:0","derives":["Clone","Eq","Ord","PartialEq","PartialOrd"],"token_hits":2}
{"k":"doc","path":"Ordered","text":"Standard derives that are not special members, and comparisons with\nanother type written by hand. The crate's own Clone trait never sits on\nthe shared line, so the standard Clone keeps its short name."}
{"k":"impl","path":"Ordered","rust":"impl PartialEq<u8> for Ordered","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Ordered::eq","sig":"fn eq(&self, other: &u8) -> bool","loc":"src/lib.rs:0","impl":"impl PartialEq<u8> for Ordered","body":{"statements":1,"tokens":9},"ops":["other","self.0"]}
{"k":"impl","path":"Ordered","rust":"impl PartialOrd<u8> for Ordered","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Ordered::partial_cmp","sig":"fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering>","loc":"src/lib.rs:0","impl":"impl PartialOrd<u8> for Ordered","body":{"statements":1,"tokens":10,"single":"method partial_cmp"},"ops":["other","self.0.partial_cmp"]}
{"k":"struct","path":"Spelled","rust":"pub struct Spelled","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Spelled","text":"The standard Display written in full, with no use."}
{"k":"impl","path":"Spelled","rust":"impl std::fmt::Display for Spelled","locs":["src/lib.rs:0"]}
{"k":"struct","path":"Renamed","rust":"pub struct Renamed","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Renamed","text":"The standard Display through a renaming use."}
{"k":"impl","path":"Renamed","rust":"impl D for Renamed","locs":["src/lib.rs:0"]}
{"k":"mod","path":"bare","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"bare","text":"The standard Display after a use of the trait itself."}
{"k":"struct","path":"bare::Imported","rust":"pub struct Imported","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"bare::Imported","text":"Display through the use, which is the standard one."}
{"k":"impl","path":"bare::Imported","rust":"impl Display for Imported","locs":["src/lib.rs:0"]}
{"k":"mod","path":"custom","loc":"src/lib.rs:0","vis":"public","types":2}
{"k":"doc","path":"custom","text":"A trait of this crate named Display."}
{"k":"trait","path":"custom::Display","rust":"pub trait Display","loc":"src/lib.rs:0","named_by":["custom::Homemade"],"token_hits":13}
{"k":"doc","path":"custom::Display","text":"The crate's own Display."}
{"k":"struct","path":"custom::Homemade","rust":"pub struct Homemade","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"custom::Homemade","text":"The crate's Display, which prints its path on its own line."}
{"k":"impl","path":"custom::Homemade","rust":"impl Display for Homemade","locs":["src/lib.rs:0"]}
{"k":"mod","path":"globbed","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"globbed","text":"A Display that can only come from a glob import."}
{"k":"struct","path":"globbed::Starred","rust":"pub struct Starred","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"globbed::Starred","text":"A Display from the glob, which prints as written."}
{"k":"impl","path":"globbed::Starred","rust":"impl Display for Starred","locs":["src/lib.rs:0"]}
{"k":"mod","path":"other","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"other","text":"Another crate's trait that is also named Display."}
{"k":"struct","path":"other::Foreign","rust":"pub struct Foreign","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"other::Foreign","text":"The other crate's Display, which stays apart from the standard one."}
{"k":"impl","path":"other::Foreign","rust":"impl Display for Foreign","locs":["src/lib.rs:0"]}
