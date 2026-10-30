*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"field_index","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":10,"to":37}]}
{"k":"lead","id":"direct-field-clusters","mode":"direct-syntax","subjects":["Scheduler"],"evidence":"issued, pending by apply, issue, withdraw; queue, tasks by admit, count_tasks, spawn; hubs chains","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Scheduler::admit","Scheduler::apply","Scheduler::count_tasks","Scheduler::issue","Scheduler::spawn","Scheduler::withdraw","Small::both"],"evidence":"no hit of `admit` in the scanned sources beyond its 1 declaration; no hit of `apply` in the scanned sources beyond its 1 declaration; no hit of `count_tasks` in the scanned sources beyond its 1 declaration; no hit of `issue` in the scanned sources beyond its 1 declaration; no hit of `spawn` in the scanned sources beyond its 1 declaration; no hit of `withdraw` in the scanned sources beyond its 1 declaration; no hit of `both` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0},{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":7}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Scheduler::new"],"evidence":"5 production hits of `new` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Small"],"evidence":"1 production hits of `Small` outside macros, none a resolved or unique qualified path to it","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","macros"]}
{"k":"surface","path":"Scheduler","kind":"struct"}
{"k":"surface","path":"Small","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0},"types":2}
{"k":"doc","path":"crate","text":"Which functions reach each field."}
{"k":"struct","path":"Scheduler","rust":"pub struct Scheduler { chains: Vec<u32>, tasks: Vec<u32>, queue: Vec<u32>, pending: Vec<u32>, issued: Vec<u32>, spare: u8 }","loc":{"file":"src/lib.rs","line":0},"token_hits":3}
{"k":"doc","path":"Scheduler","text":"A scheduler whose fields its functions reach in groups."}
{"k":"impl","path":"Scheduler","rust":"impl Scheduler","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Scheduler::new","sig":"pub fn new() -> Scheduler","loc":{"file":"src/lib.rs","line":0},"token_hits":5,"body":{"statements":1,"tokens":48}}
{"k":"doc","path":"Scheduler::new","text":"Makes one."}
{"k":"fn","path":"Scheduler::spawn","sig":"pub fn spawn(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":2,"tokens":28},"ops":["self.chains.len","self.queue.push(0)","self.tasks.push"]}
{"k":"doc","path":"Scheduler::spawn","text":"Starts a task."}
{"k":"fn","path":"Scheduler::admit","sig":"pub fn admit(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":2,"tokens":24},"ops":["self.chains.len","self.queue.pop"]}
{"k":"doc","path":"Scheduler::admit","text":"Admits queued tasks."}
{"k":"fn","path":"Scheduler::count_tasks","sig":"pub fn count_tasks(&self) -> usize","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":9,"single":"method len"},"ops":["self.tasks.len"]}
{"k":"doc","path":"Scheduler::count_tasks","text":"Counts the tasks."}
{"k":"fn","path":"Scheduler::issue","sig":"pub fn issue(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":2,"tokens":28},"ops":["self.chains.len","self.issued.push","self.pending.push(1)"]}
{"k":"doc","path":"Scheduler::issue","text":"Issues an effect."}
{"k":"fn","path":"Scheduler::apply","sig":"pub fn apply(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":2,"tokens":24},"ops":["self.chains.first","self.pending.pop"]}
{"k":"doc","path":"Scheduler::apply","text":"Applies an answer."}
{"k":"fn","path":"Scheduler::withdraw","sig":"pub fn withdraw(&mut self)","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":10,"single":"method clear"},"ops":["self.issued.clear"]}
{"k":"doc","path":"Scheduler::withdraw","text":"Forgets the issued effects."}
{"k":"fn","path":"Scheduler::len","sig":"pub fn len(&self) -> usize","loc":{"file":"src/lib.rs","line":0},"method_name_hits":5,"token_hits":5,"body":{"statements":1,"tokens":9,"single":"method len"},"ops":["self.chains.len"]}
{"k":"doc","path":"Scheduler::len","text":"Counts the chains."}
{"k":"struct","path":"Small","rust":"pub struct Small { a: u8, b: u8 }","loc":{"file":"src/lib.rs","line":0},"token_hits":1}
{"k":"doc","path":"Small","text":"A type with too few functions for a fields list."}
{"k":"impl","path":"Small","rust":"impl Small","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"fn","path":"Small::first","sig":"pub fn first(&self) -> u8","loc":{"file":"src/lib.rs","line":0},"method_name_hits":1,"token_hits":1,"body":{"statements":1,"tokens":5,"single":"field a"},"ops":["self.a"]}
{"k":"doc","path":"Small::first","text":"Returns the first."}
{"k":"fn","path":"Small::both","sig":"pub fn both(&self) -> u8","loc":{"file":"src/lib.rs","line":0},"body":{"statements":1,"tokens":9},"ops":["self.a","self.b"]}
{"k":"doc","path":"Small::both","text":"Returns both."}
