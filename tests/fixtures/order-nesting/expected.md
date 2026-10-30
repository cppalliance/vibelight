*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"order_nesting","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":24,"to":62}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Able","Able::leaf","Config::limit","Fork","Fork::a_left","Fork::b_right","Host::run","Left::base","Orphan","Report","Report::summary","Right::base","Run","Run::config","Run::steps","Step::a_trace","Step::b_log","Step::config"],"evidence":"no hit of `Able` in the scanned sources beyond its 1 declaration; no hit of `leaf` in the scanned sources beyond its 1 declaration; no hit of `limit` in the scanned sources beyond its 1 declaration; no hit of `Fork` in the scanned sources beyond its 1 declaration; no hit of `a_left` in the scanned sources beyond its 1 declaration; no hit of `b_right` in the scanned sources beyond its 1 declaration; no hit of `run` in the scanned sources beyond its 1 declaration; no hit of `base` in the scanned sources beyond its 2 declarations; no hit of `Orphan` in the scanned sources beyond its 1 declaration; no hit of `Report` in the scanned sources beyond its 1 declaration; no hit of `summary` in the scanned sources beyond its 1 declaration; no hit of `Run` in the scanned sources beyond its 1 declaration; no hit of `config` in the scanned sources beyond its 2 declarations; no hit of `steps` in the scanned sources beyond its 1 declaration; no hit of `a_trace` in the scanned sources beyond its 1 declaration; no hit of `b_log` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":18}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Host"],"evidence":"1 production hits of `Host` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Able::leaf","Config::limit","Fork::a_left","Fork::b_right","Left::base","Report::summary","Right::base","Run::config","Run::steps","Step::a_trace","Step::b_log","Step::config"],"evidence":"0 struct literals of `Able` set it, and no `.leaf` access or pattern reads it; 0 struct literals of `Config` set it, and no `.limit` access or pattern reads it; 0 struct literals of `Fork` set it, and no `.a_left` access or pattern reads it; 0 struct literals of `Fork` set it, and no `.b_right` access or pattern reads it; 0 struct literals of `Left` set it, and no `.base` access or pattern reads it; 0 struct literals of `Report` set it, and no `.summary` access or pattern reads it; 0 struct literals of `Right` set it, and no `.base` access or pattern reads it; 0 struct literals of `Run` set it, and no `.config` access or pattern reads it; 0 struct literals of `Run` set it, and no `.steps` access or pattern reads it; 0 struct literals of `Step` set it, and no `.a_trace` access or pattern reads it; 0 struct literals of `Step` set it, and no `.b_log` access or pattern reads it; 0 struct literals of `Step` set it, and no `.config` access or pattern reads it","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":12}
{"k":"surface","path":"Able","kind":"struct"}
{"k":"surface","path":"Base","kind":"struct"}
{"k":"surface","path":"Config","kind":"struct"}
{"k":"surface","path":"Fork","kind":"struct"}
{"k":"surface","path":"Host","kind":"struct"}
{"k":"surface","path":"Job","kind":"struct"}
{"k":"surface","path":"Leaf","kind":"struct"}
{"k":"surface","path":"Left","kind":"struct"}
{"k":"surface","path":"Log","kind":"struct"}
{"k":"surface","path":"Orphan","kind":"struct"}
{"k":"surface","path":"Report","kind":"struct"}
{"k":"surface","path":"Right","kind":"struct"}
{"k":"surface","path":"Run","kind":"struct"}
{"k":"surface","path":"Step","kind":"struct"}
{"k":"surface","path":"Store","kind":"struct"}
{"k":"surface","path":"Summary","kind":"struct"}
{"k":"surface","path":"Trace","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":17}
{"k":"doc","path":"crate","text":"Dependency order: each part follows its owner."}
{"k":"struct","path":"Run","rust":"pub struct Run { pub config: Config, pub steps: Vec<Step> }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Run","text":"The main entry: it reaches the most declarations."}
{"k":"struct","path":"Fork","rust":"pub struct Fork { pub a_left: Left, pub b_right: Right }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Fork","text":"A diamond root: it reaches three declarations through two branches."}
{"k":"struct","path":"Host","rust":"pub struct Host { store: Store }","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Host","text":"A root whose public method prints before its private field, so its\nblock names Job first, yet its field type, Store, still comes first."}
{"k":"impl","path":"Host","rust":"impl Host","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Host::run","sig":"pub fn run(&self, job: Job)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":7},"ops":["job"]}
{"k":"doc","path":"Host::run","text":"Runs a job."}
{"k":"struct","path":"Store","rust":"pub struct Store","loc":{"file":"src/lib.rs","line":0},"named_by":["Host"],"token_hits":1}
{"k":"doc","path":"Store","text":"Named only by Host's field."}
{"k":"struct","path":"Job","rust":"pub struct Job","loc":{"file":"src/lib.rs","line":0},"named_by":["Host"],"token_hits":1}
{"k":"doc","path":"Job","text":"Named only by Host's method."}
{"k":"struct","path":"Able","rust":"pub struct Able { pub leaf: Leaf }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Able","text":"A root that ties Report on reach and comes first by path."}
{"k":"struct","path":"Report","rust":"pub struct Report { pub summary: Summary }","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Report","text":"A root that ties Able on reach."}
{"k":"struct","path":"Config","rust":"pub struct Config { pub limit: u32 }","loc":{"file":"src/lib.rs","line":0},"named_by":["Run","Step"],"token_hits":2}
{"k":"doc","path":"Config","text":"Named by Run and by Step, so it follows Step's last part, not Run."}
{"k":"struct","path":"Step","rust":"pub struct Step { pub a_trace: Trace, pub b_log: Log, pub config: Config }","loc":{"file":"src/lib.rs","line":0},"named_by":["Run"],"token_hits":1}
{"k":"doc","path":"Step","text":"Named by Run; first names Trace, then Log, then Config."}
{"k":"struct","path":"Log","rust":"pub struct Log","loc":{"file":"src/lib.rs","line":0},"named_by":["Step"],"token_hits":1}
{"k":"doc","path":"Log","text":"Named only by Step."}
{"k":"struct","path":"Trace","rust":"pub struct Trace","loc":{"file":"src/lib.rs","line":0},"named_by":["Step"],"token_hits":1}
{"k":"doc","path":"Trace","text":"Named only by Step, before Log."}
{"k":"struct","path":"Left","rust":"pub struct Left { pub base: Base }","loc":{"file":"src/lib.rs","line":0},"named_by":["Fork"],"token_hits":1}
{"k":"doc","path":"Left","text":"Named only by Fork."}
{"k":"struct","path":"Right","rust":"pub struct Right { pub base: Base }","loc":{"file":"src/lib.rs","line":0},"named_by":["Fork"],"token_hits":1}
{"k":"doc","path":"Right","text":"Named only by Fork, after Left."}
{"k":"struct","path":"Base","rust":"pub struct Base","loc":{"file":"src/lib.rs","line":0},"named_by":["Left","Right"],"token_hits":2}
{"k":"doc","path":"Base","text":"Named by Left and by Right, so it follows Right, the last of them."}
{"k":"struct","path":"Leaf","rust":"pub struct Leaf","loc":{"file":"src/lib.rs","line":0},"named_by":["Able"],"token_hits":1}
{"k":"doc","path":"Leaf","text":"Named only by Able."}
{"k":"struct","path":"Summary","rust":"pub struct Summary","loc":{"file":"src/lib.rs","line":0},"named_by":["Report"],"token_hits":1}
{"k":"doc","path":"Summary","text":"Named only by Report."}
{"k":"struct","path":"Orphan","rust":"pub struct Orphan","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"Orphan","text":"Names nothing and nothing names it."}
