*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"spellings","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":40,"to":61},{"path":"bare","from":62,"to":66},{"path":"debug","from":67,"to":76},{"path":"other","from":77,"to":84}]}
{"k":"lead","id":"prelude-name","mode":"direct-syntax","subjects":["Duration"],"evidence":"named like `std::time::Duration`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["bare"],"evidence":"holds only `Label`","locs":["src/lib.rs:0"]}
{"k":"lead","id":"doc-echo","mode":"direct-syntax","subjects":["Timer"],"evidence":"every content word of \"A timer that names its laps.\" is in its name, signature, or types","locs":["src/lib.rs:0"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["debug::Gauge"],"evidence":"no hit of `Gauge` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Timer::next"],"evidence":"no hit of `next` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Timer::wait"],"evidence":"no hit of `wait` in the scanned sources beyond its 1 declaration","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["bare::Label"],"evidence":"1 production hits of `Label` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["debug::Dial"],"evidence":"1 production hits of `Dial` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["debug::Knob"],"evidence":"1 production hits of `Knob` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Duration"],"evidence":"3 production hits of `Duration` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["other::Badge"],"evidence":"1 production hits of `Badge` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["other::Pin"],"evidence":"1 production hits of `Pin` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Timer"],"evidence":"2 production hits of `Timer` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Wrapper"],"evidence":"1 production hits of `Wrapper` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"surface","path":"bare","kind":"mod"}
{"k":"surface","path":"bare::Label","kind":"struct"}
{"k":"surface","path":"Clock","kind":"struct"}
{"k":"surface","path":"debug","kind":"mod"}
{"k":"surface","path":"debug::Dial","kind":"struct"}
{"k":"surface","path":"debug::Gauge","kind":"struct"}
{"k":"surface","path":"debug::Knob","kind":"struct"}
{"k":"surface","path":"Duration","kind":"struct"}
{"k":"surface","path":"other","kind":"mod"}
{"k":"surface","path":"other::Badge","kind":"struct"}
{"k":"surface","path":"other::Pin","kind":"struct"}
{"k":"surface","path":"Timer","kind":"struct"}
{"k":"surface","path":"Wrapper","kind":"struct"}
{"k":"external","path":"other_crate::Display","uses":2,"named_by":["other::Badge","other::Pin"]}
{"k":"spelling","path":"other_crate::Display","spellings":[{"text":"Display","count":1,"locs":["src/lib.rs:0"]},{"text":"other_crate::Display","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"spelling","path":"std::collections::BTreeMap","preferred":"BTreeMap","spellings":[{"text":"Map","count":1,"locs":["src/lib.rs:0"]},{"text":"std::collections::BTreeMap","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"spelling","path":"std::fmt::Debug","preferred":"Debug","spellings":[{"text":"fmt::Debug","count":1,"locs":["src/lib.rs:0"]},{"text":"std::fmt::Debug","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"spelling","path":"std::fmt::Display","preferred":"Display","spellings":[{"text":"Display","count":1,"locs":["src/lib.rs:0"]},{"text":"fmt::Display","count":1,"locs":["src/lib.rs:0"]},{"text":"std::fmt::Display","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"spelling","path":"std::option::Option","preferred":"Option","spellings":[{"text":"Option","count":2,"locs":["src/lib.rs:0","src/lib.rs:0"]},{"text":"std::option::Option","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"spelling","path":"std::time::Duration","spellings":[{"text":"::std::time::Duration","count":1,"locs":["src/lib.rs:0"]},{"text":"core::time::Duration","count":1,"locs":["src/lib.rs:0"]},{"text":"std::time::Duration","count":1,"locs":["src/lib.rs:0"]}]}
{"k":"wire-keys","path":"Clock","keys":["alarm","label","snooze","step"],"loc":"src/lib.rs:0"}
{"k":"wire-keys","path":"Timer","keys":["laps","names","period"],"loc":"src/lib.rs:0"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":4}
{"k":"doc","path":"crate","text":"Spellings: items from outside the crate written in more than one way."}
{"k":"struct","path":"Clock","rust":"pub struct Clock { pub alarm: Option<u8>, pub label: String, pub snooze: Option<u8>, pub step: core::time::Duration }","loc":"src/lib.rs:0","derives":["Serialize"],"named_by":["Timer"]}
{"k":"doc","path":"Clock","text":"A clock with an optional alarm."}
{"k":"field","path":"Clock::alarm","rust":"pub alarm: Option<u8>","loc":"src/lib.rs:0"}
{"k":"doc","path":"Clock::alarm","text":"When the alarm rings; see `core::option::Option`."}
{"k":"impl","path":"Clock","rust":"impl fmt::Display for Clock","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Clock::fmt","sig":"fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result","loc":"src/lib.rs:0","impl":"impl fmt::Display for Clock","body":{"statements":1,"tokens":3,"macros":["write"]}}
{"k":"struct","path":"Timer","rust":"pub struct Timer { pub laps: Map<u32, String>, pub names: std::collections::BTreeMap<u32, String>, pub period: ::std::time::Duration }","loc":"src/lib.rs:0","derives":["serde::Serialize"]}
{"k":"doc","path":"Timer","text":"A timer that names its laps."}
{"k":"impl","path":"Timer","rust":"impl std::fmt::Display for Timer","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Timer::fmt","sig":"fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result","loc":"src/lib.rs:0","impl":"impl std::fmt::Display for Timer","body":{"statements":1,"tokens":3,"macros":["write"]}}
{"k":"impl","path":"Timer","rust":"impl Timer","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Timer::next","sig":"pub fn next(&self, clock: &crate::Clock) -> std::option::Option<Clock>","loc":"src/lib.rs:0","body":{"statements":2,"tokens":34,"nones":2},"ops":["clock"]}
{"k":"fn","path":"Timer::wait","sig":"pub fn wait(&self, extra: std::time::Duration) -> Vec<String>","loc":"src/lib.rs:0","body":{"statements":2,"tokens":13},"ops":["extra"]}
{"k":"struct","path":"Wrapper","rust":"pub struct Wrapper<T> { pub inner: T }","loc":"src/lib.rs:0"}
{"k":"doc","path":"Wrapper","text":"A wrapper whose impl names std::string::String only in its self type,\nwhich does not count."}
{"k":"impl","path":"Wrapper","rust":"impl Wrapper<std::string::String>","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Wrapper::len","sig":"pub fn len(&self) -> usize","loc":"src/lib.rs:0","body":{"statements":1,"tokens":9,"single":"method len"},"ops":["self.inner.len"]}
{"k":"fn","path":"Wrapper::is_empty","sig":"pub fn is_empty(&self) -> bool","loc":"src/lib.rs:0","body":{"statements":1,"tokens":9,"single":"method is_empty"},"ops":["self.inner.is_empty"]}
{"k":"struct","path":"Duration","rust":"pub struct Duration","loc":"src/lib.rs:0"}
{"k":"doc","path":"Duration","text":"The crate's own Duration, so the standard one has no bare spelling to\nprefer."}
{"k":"mod","path":"bare","loc":"src/lib.rs:0","vis":"public","types":1}
{"k":"doc","path":"bare","text":"The standard Display written bare after a use."}
{"k":"struct","path":"bare::Label","rust":"pub struct Label","loc":"src/lib.rs:0"}
{"k":"doc","path":"bare::Label","text":"A label that shows itself through the standard Display."}
{"k":"impl","path":"bare::Label","rust":"impl Display for Label","locs":["src/lib.rs:0"]}
{"k":"mod","path":"debug","loc":"src/lib.rs:0","vis":"public","types":3}
{"k":"doc","path":"debug","text":"Debug by derive and by hand. A derive name is never counted."}
{"k":"struct","path":"debug::Gauge","rust":"pub struct Gauge","loc":"src/lib.rs:0","derives":["Debug"]}
{"k":"doc","path":"debug::Gauge","text":"A gauge that derives Debug, which adds no spelling."}
{"k":"struct","path":"debug::Dial","rust":"pub struct Dial","loc":"src/lib.rs:0"}
{"k":"doc","path":"debug::Dial","text":"A dial that writes Debug through the fmt module."}
{"k":"impl","path":"debug::Dial","rust":"impl fmt::Debug for Dial","locs":["src/lib.rs:0"]}
{"k":"struct","path":"debug::Knob","rust":"pub struct Knob","loc":"src/lib.rs:0"}
{"k":"doc","path":"debug::Knob","text":"A knob that writes Debug with its full path."}
{"k":"impl","path":"debug::Knob","rust":"impl std::fmt::Debug for Knob","locs":["src/lib.rs:0"]}
{"k":"mod","path":"other","loc":"src/lib.rs:0","vis":"public","types":2}
{"k":"doc","path":"other","text":"Another crate's Display, which never merges with the standard one."}
{"k":"struct","path":"other::Badge","rust":"pub struct Badge","loc":"src/lib.rs:0"}
{"k":"doc","path":"other::Badge","text":"A badge that shows itself through the other crate's Display."}
{"k":"impl","path":"other::Badge","rust":"impl Display for Badge","locs":["src/lib.rs:0"]}
{"k":"struct","path":"other::Pin","rust":"pub struct Pin","loc":"src/lib.rs:0"}
{"k":"doc","path":"other::Pin","text":"A pin that names the other crate's Display by its full path."}
{"k":"impl","path":"other::Pin","rust":"impl other_crate::Display for Pin","locs":["src/lib.rs:0"]}
