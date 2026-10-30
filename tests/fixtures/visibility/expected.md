*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"visibility","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":17,"to":47},{"path":"outer","from":48,"to":67}]}
{"k":"lead","id":"hidden-non-exhaustive","mode":"resolved","subjects":["Mode"],"evidence":"`#[non_exhaustive]` off every surface; publish_allowed is false","locs":[{"file":"src/lib.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Hidden"],"evidence":"no hit of `Hidden` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Inner"],"evidence":"no hit of `Inner` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Mixed::into_name"],"evidence":"no hit of `into_name` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["outer::ext_open"],"evidence":"no hit of `ext_open` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["outer::open"],"evidence":"no hit of `open` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Mixed::new"],"evidence":"1 production hits of `new` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"Hidden","kind":"struct"}
{"k":"surface","path":"Inner","kind":"struct"}
{"k":"surface","path":"Mixed","kind":"struct"}
{"k":"surface","path":"outer","kind":"mod"}
{"k":"surface","path":"outer::ext_open","kind":"fn"}
{"k":"surface","path":"outer::open","kind":"fn"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":5}
{"k":"doc","path":"crate","text":"Visibility sections: public, internal, and private."}
{"k":"struct","path":"Mixed","rust":"pub struct Mixed { pub name: String, pub(crate) size: usize, secret: u8 }","loc":{"file":"src/lib.rs","line":0},"token_hits":3}
{"k":"doc","path":"Mixed","text":"A struct with every section."}
{"k":"field","path":"Mixed::name","rust":"pub name: String","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Mixed::name","text":"A public field."}
{"k":"field","path":"Mixed::size","rust":"pub(crate) size: usize","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Mixed::size","text":"An internal field."}
{"k":"field","path":"Mixed::secret","rust":"secret: u8","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Mixed::secret","text":"A private field."}
{"k":"impl","path":"Mixed","rust":"impl Mixed","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Mixed::new","sig":"pub fn new() -> Mixed","loc":{"file":"src/lib.rs","line":0},"token_hits":1,"body":{"statements":1,"tokens":19}}
{"k":"doc","path":"Mixed::new","text":"Makes one."}
{"k":"fn","path":"Mixed::into_name","sig":"pub fn into_name(self) -> String","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":5,"single":"field name"},"ops":["self.name"]}
{"k":"doc","path":"Mixed::into_name","text":"Takes the name out."}
{"k":"fn","path":"Mixed::grow","sig":"pub(crate) fn grow(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":9},"ops":["self.size [write]"]}
{"k":"doc","path":"Mixed::grow","text":"Grows it."}
{"k":"fn","path":"Mixed::check","sig":"fn check(&self) -> bool","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":8},"ops":["self.secret"]}
{"k":"doc","path":"Mixed::check","text":"Checks it."}
{"k":"struct","path":"Inner","rust":"pub struct Inner { pub(crate) id: u32, state: u8 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Inner","text":"A class with internal and private members."}
{"k":"field","path":"Inner::id","rust":"pub(crate) id: u32","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Inner::id","text":"Seen in the crate."}
{"k":"field","path":"Inner::state","rust":"state: u8","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Inner::state","text":"Seen only here."}
{"k":"struct","path":"Hidden","rust":"pub struct Hidden { value: u8 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Hidden","text":"A class with private members only."}
{"k":"enum","path":"Mode","rust":"pub(crate) enum Mode { Fast, Slow }","loc":{"file":"src/lib.rs","line":0},"attrs":["non_exhaustive"]}
{"k":"doc","path":"Mode","text":"An enum seen only in the crate."}
{"k":"alias","path":"Count","rust":"type Count = u32","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Count","text":"A type alias seen only here."}
{"k":"mod","path":"outer","loc":{"file":"src/lib.rs","line":0},"vis":"public","functions":{"public":1,"internal":2,"private":2}}
{"k":"doc","path":"outer","text":"Module items in every section."}
{"k":"fn","path":"outer::open","sig":"pub fn open()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"outer::open","text":"Visible everywhere."}
{"k":"fn","path":"outer::shared","sig":"pub(crate) fn shared()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"outer::shared","text":"Visible in the crate."}
{"k":"fn","path":"outer::parent","sig":"pub(super) fn parent()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"outer::parent","text":"Visible in the parent module."}
{"k":"fn","path":"outer::hidden","sig":"fn hidden()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"outer::hidden","text":"Visible only here."}
{"k":"fn","path":"outer::own","sig":"pub(self) fn own()","loc":{"file":"src/lib.rs","line":0},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"outer::own","text":"Also visible only here."}
{"k":"const","path":"outer::LIMIT","rust":"pub(crate) const LIMIT: u8","value":"4","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"outer::LIMIT","text":"Visible in the crate, through a path."}
{"k":"static","path":"outer::COUNT","rust":"pub(crate) static COUNT: u8","value":"0","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"outer::COUNT","text":"A shared counter."}
{"k":"extern","path":"outer::ext_open","rust":"extern \"C\" { pub fn ext_open() -> i32 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"outer::ext_open","text":"Opens a stream."}
{"k":"extern","path":"outer::ext_share","rust":"extern \"C\" { pub(crate) fn ext_share() -> i32 }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"outer::ext_share","text":"Shares a stream."}
