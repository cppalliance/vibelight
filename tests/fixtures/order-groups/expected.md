*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"order_groups","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":37,"to":44},{"path":"engine","from":45,"to":50},{"path":"engine::sched","from":51,"to":54},{"path":"engine::sched::deep","from":55,"to":58},{"path":"store","from":59,"to":62},{"path":"util","from":63,"to":66}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["engine::sched::deep"],"evidence":"holds only `Slot`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["store"],"evidence":"holds only `Db`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["util"],"evidence":"holds only `Spare`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["App"],"evidence":"no hit of `App` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["App::a_core"],"evidence":"no hit of `a_core` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["App::b_db"],"evidence":"no hit of `b_db` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["App::c_log"],"evidence":"no hit of `c_log` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["engine::Core::a_plan"],"evidence":"no hit of `a_plan` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["engine::Core::b_queue"],"evidence":"no hit of `b_queue` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["engine::sched::Queue::slot"],"evidence":"no hit of `slot` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Log::level"],"evidence":"no hit of `level` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["util::Spare"],"evidence":"no hit of `Spare` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["App::a_core"],"evidence":"0 struct literals of `App` set it, and no `.a_core` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["App::b_db"],"evidence":"0 struct literals of `App` set it, and no `.b_db` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["App::c_log"],"evidence":"0 struct literals of `App` set it, and no `.c_log` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["engine::Core::a_plan"],"evidence":"0 struct literals of `Core` set it, and no `.a_plan` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["engine::Core::b_queue"],"evidence":"0 struct literals of `Core` set it, and no `.b_queue` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["engine::sched::Queue::slot"],"evidence":"0 struct literals of `Queue` set it, and no `.slot` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-field-read","mode":"lexical","subjects":["Log::level"],"evidence":"0 struct literals of `Log` set it, and no `.level` access or pattern reads it","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"App","kind":"struct"}
{"k":"surface","path":"engine","kind":"mod"}
{"k":"surface","path":"engine::Core","kind":"struct"}
{"k":"surface","path":"engine::Plan","kind":"struct"}
{"k":"surface","path":"engine::sched","kind":"mod"}
{"k":"surface","path":"engine::sched::deep","kind":"mod"}
{"k":"surface","path":"engine::sched::deep::Slot","kind":"struct"}
{"k":"surface","path":"engine::sched::Queue","kind":"struct"}
{"k":"surface","path":"Level","kind":"struct"}
{"k":"surface","path":"Log","kind":"struct"}
{"k":"surface","path":"store","kind":"mod"}
{"k":"surface","path":"store::Db","kind":"struct"}
{"k":"surface","path":"util","kind":"mod"}
{"k":"surface","path":"util::Spare","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":3}
{"k":"doc","path":"crate","text":"Groups of types by module."}
{"k":"struct","path":"App","rust":"pub struct App { pub a_core: engine::Core, pub b_db: store::Db, pub c_log: Log }","loc":"src/lib.rs:0"}
{"k":"doc","path":"App","text":"The only root: it names Core, Db, and Log, in that order."}
{"k":"struct","path":"Log","rust":"pub struct Log { pub level: Level }","loc":"src/lib.rs:0","named_by":["App"],"token_hits":1}
{"k":"doc","path":"Log","text":"Named only by App, in the crate root."}
{"k":"struct","path":"Level","rust":"pub struct Level","loc":"src/lib.rs:0","named_by":["Log"],"token_hits":1}
{"k":"doc","path":"Level","text":"Named only by Log, in the crate root."}
{"k":"mod","path":"engine","loc":"src/lib.rs:0","vis":"public","types":2}
{"k":"doc","path":"engine","text":"The engine."}
{"k":"struct","path":"engine::Core","rust":"pub struct Core { pub a_plan: Plan, pub b_queue: sched::Queue }","loc":"src/lib.rs:0","named_by":["App"],"token_hits":1}
{"k":"doc","path":"engine::Core","text":"Named only by App; names Plan, then Queue."}
{"k":"struct","path":"engine::Plan","rust":"pub struct Plan","loc":"src/lib.rs:0","named_by":["engine::Core"],"token_hits":1}
{"k":"doc","path":"engine::Plan","text":"Named only by Core."}
{"k":"mod","path":"engine::sched","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"engine::sched","text":"The scheduler."}
{"k":"struct","path":"engine::sched::Queue","rust":"pub struct Queue { pub slot: deep::Slot }","loc":"src/lib.rs:0","named_by":["engine::Core"],"token_hits":1}
{"k":"doc","path":"engine::sched::Queue","text":"Named only by Core; names Slot."}
{"k":"mod","path":"engine::sched::deep","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"engine::sched::deep","text":"Three segments deep: its group is named for the first two."}
{"k":"struct","path":"engine::sched::deep::Slot","rust":"pub struct Slot","loc":"src/lib.rs:0","named_by":["engine::sched::Queue"],"token_hits":1}
{"k":"doc","path":"engine::sched::deep::Slot","text":"Named only by Queue."}
{"k":"mod","path":"store","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"store","text":"The store."}
{"k":"struct","path":"store::Db","rust":"pub struct Db","loc":"src/lib.rs:0","named_by":["App"],"token_hits":1}
{"k":"doc","path":"store::Db","text":"Named only by App: a lone type in its module."}
{"k":"mod","path":"util","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"util","text":"Helpers nothing uses."}
{"k":"struct","path":"util::Spare","rust":"pub struct Spare","loc":"src/lib.rs:0"}
{"k":"doc","path":"util::Spare","text":"Names nothing and nothing names it."}
