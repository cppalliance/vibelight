*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"prelude_order","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":26,"to":27},{"path":"alpha","from":28,"to":33},{"path":"beta","from":34,"to":39},{"path":"own","from":40,"to":47},{"path":"plain","from":48,"to":51},{"path":"starred","from":52,"to":55}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["plain"],"evidence":"holds only `Report`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["own::Owned"],"evidence":"no hit of `Owned` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["own::Owned::value"],"evidence":"no hit of `value` in the scanned sources beyond its 2 declarations","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["plain::Report"],"evidence":"no hit of `Report` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["plain::Report::outcome"],"evidence":"no hit of `outcome` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["starred::Starred"],"evidence":"no hit of `Starred` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["starred::Starred::value"],"evidence":"no hit of `value` in the scanned sources beyond its 2 declarations","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["own::Owned::value"],"evidence":"0 struct literals of `Owned` set it, and no `.value` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["plain::Report::outcome"],"evidence":"0 struct literals of `Report` set it, and no `.outcome` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["starred::Starred::value"],"evidence":"0 struct literals of `Starred` set it, and no `.value` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"alpha","kind":"mod"}
{"k":"surface","path":"alpha::AlphaError","kind":"struct"}
{"k":"surface","path":"beta","kind":"mod"}
{"k":"surface","path":"beta::BetaError","kind":"struct"}
{"k":"surface","path":"own","kind":"mod"}
{"k":"surface","path":"own::Owned","kind":"struct"}
{"k":"surface","path":"own::OwnError","kind":"struct"}
{"k":"surface","path":"own::Result","kind":"alias"}
{"k":"surface","path":"plain","kind":"mod"}
{"k":"surface","path":"plain::Report","kind":"struct"}
{"k":"surface","path":"starred","kind":"mod"}
{"k":"surface","path":"starred::Starred","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"crate","text":"Prelude order: a bare name is the prelude's unless the module declares\nit, imports it, or a glob brings it in."}
{"k":"mod","path":"alpha","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":2}
{"k":"doc","path":"alpha","text":"The first module with a private Result alias."}
{"k":"struct","path":"alpha::AlphaError","rust":"pub struct AlphaError","loc":{"file":"src/lib.rs","line":0},"named_by":["alpha::Result"],"token_hits":1}
{"k":"doc","path":"alpha::AlphaError","text":"Alpha's error."}
{"k":"alias","path":"alpha::Result","rust":"type Result<T> = std::result::Result<T, AlphaError>","loc":{"file":"src/lib.rs","line":0},"token_hits":6}
{"k":"doc","path":"alpha::Result","text":"The results of alpha."}
{"k":"mod","path":"beta","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":2}
{"k":"doc","path":"beta","text":"The second module with a private Result alias."}
{"k":"struct","path":"beta::BetaError","rust":"pub struct BetaError","loc":{"file":"src/lib.rs","line":0},"named_by":["beta::Result"],"token_hits":1}
{"k":"doc","path":"beta::BetaError","text":"Beta's error."}
{"k":"alias","path":"beta::Result","rust":"type Result<T> = std::result::Result<T, BetaError>","loc":{"file":"src/lib.rs","line":0},"token_hits":6}
{"k":"doc","path":"beta::Result","text":"The results of beta."}
{"k":"mod","path":"own","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":3}
{"k":"doc","path":"own","text":"A module that declares its own Result and uses it."}
{"k":"struct","path":"own::OwnError","rust":"pub struct OwnError","loc":{"file":"src/lib.rs","line":0},"named_by":["own::Result"],"token_hits":1}
{"k":"doc","path":"own::OwnError","text":"The error of this module."}
{"k":"alias","path":"own::Result","rust":"pub type Result<T> = std::result::Result<T, OwnError>","loc":{"file":"src/lib.rs","line":0},"named_by":["own::Owned","starred::Starred"],"token_hits":6}
{"k":"doc","path":"own::Result","text":"The results of this module."}
{"k":"struct","path":"own::Owned","rust":"pub struct Owned { pub value: Result<u8> }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"own::Owned","text":"A value in this module's Result."}
{"k":"mod","path":"plain","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":1}
{"k":"doc","path":"plain","text":"A module that names the prelude Result and imports nothing."}
{"k":"struct","path":"plain::Report","rust":"pub struct Report { pub outcome: Result<u8, String> }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"plain::Report","text":"A report whose outcome is the standard Result."}
{"k":"mod","path":"starred","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":1}
{"k":"doc","path":"starred","text":"A module whose glob import brings in the Result of own."}
{"k":"struct","path":"starred::Starred","rust":"pub struct Starred { pub value: Result<u16> }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"starred::Starred","text":"A value in the Result that the glob brings in."}
