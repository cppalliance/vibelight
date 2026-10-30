*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"traits","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":11,"to":38}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Convert","Convert::convert","Lender","Lender::lend","Observer","Observer::finish","Observer::flush","Observer::Item","Observer::LIMIT","Observer::make","Observer::record","Zeroable"],"evidence":"no hit of `Convert` in the scanned sources beyond its 1 declaration; no hit of `convert` in the scanned sources beyond its 1 declaration; no hit of `Lender` in the scanned sources beyond its 1 declaration; no hit of `lend` in the scanned sources beyond its 1 declaration; no hit of `Observer` in the scanned sources beyond its 1 declaration; no hit of `finish` in the scanned sources beyond its 1 declaration; no hit of `flush` in the scanned ","locs":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"],"module":"crate","count":12}
{"k":"more","path":"Convert","id":"no-observed-name-use","field":"evidence","part":2,"text":"sources beyond its 1 declaration; no hit of `Item` in the scanned sources beyond its 1 declaration; no hit of `LIMIT` in the scanned sources beyond its 1 declaration; no hit of `make` in the scanned sources beyond its 1 declaration; no hit of `record` in the scanned sources beyond its 1 declaration; no hit of `Zeroable` in the scanned sources beyond its 1 declaration"}
{"k":"more","path":"Convert","id":"no-observed-name-use","field":"locs","part":3,"items":["src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0","src/lib.rs:0"]}
{"k":"surface","path":"Convert","kind":"trait"}
{"k":"surface","path":"Lender","kind":"trait"}
{"k":"surface","path":"Observer","kind":"trait"}
{"k":"surface","path":"Zeroable","kind":"trait"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":4}
{"k":"doc","path":"crate","text":"Traits."}
{"k":"trait","path":"Observer","rust":"pub trait Observer: Send + Sync","loc":"src/lib.rs:0"}
{"k":"doc","path":"Observer","text":"Watches a run."}
{"k":"alias","path":"Observer::Item","rust":"type Item: Clone","loc":"src/lib.rs:0"}
{"k":"doc","path":"Observer::Item","text":"The item type."}
{"k":"const","path":"Observer::LIMIT","rust":"const LIMIT: usize","value":"64","loc":"src/lib.rs:0","body":{"statements":1,"tokens":1}}
{"k":"doc","path":"Observer::LIMIT","text":"The most items kept."}
{"k":"fn","path":"Observer::record","sig":"fn record(&self, event: &str)","loc":"src/lib.rs:0"}
{"k":"doc","path":"Observer::record","text":"Records one event."}
{"k":"fn","path":"Observer::flush","sig":"fn flush(&mut self)","loc":"src/lib.rs:0","body":{"statements":0,"tokens":2}}
{"k":"doc","path":"Observer::flush","text":"Clears the buffer."}
{"k":"fn","path":"Observer::finish","sig":"fn finish(self) -> u32 where Self: Sized","loc":"src/lib.rs:0"}
{"k":"doc","path":"Observer::finish","text":"Ends the run."}
{"k":"fn","path":"Observer::make","sig":"fn make() -> Self where Self: Sized","loc":"src/lib.rs:0"}
{"k":"doc","path":"Observer::make","text":"Makes a default observer."}
{"k":"trait","path":"Zeroable","rust":"pub unsafe trait Zeroable","loc":"src/lib.rs:0"}
{"k":"doc","path":"Zeroable","text":"A marker that is unsafe to implement."}
{"k":"trait","path":"Convert","rust":"pub trait Convert<T> where T: Clone","loc":"src/lib.rs:0"}
{"k":"doc","path":"Convert","text":"A generic trait with a where clause."}
{"k":"fn","path":"Convert::convert","sig":"fn convert(&self, value: T) -> T","loc":"src/lib.rs:0"}
{"k":"doc","path":"Convert::convert","text":"Converts a value."}
{"k":"trait","path":"Lender","rust":"pub trait Lender","loc":"src/lib.rs:0"}
{"k":"doc","path":"Lender","text":"A trait with a generic associated type."}
{"k":"alias","path":"Lender::Loan","rust":"type Loan<'a> where Self: 'a","loc":"src/lib.rs:0"}
{"k":"doc","path":"Lender::Loan","text":"The borrowed item."}
{"k":"fn","path":"Lender::lend","sig":"fn lend(&mut self) -> Self::Loan<'_>","loc":"src/lib.rs:0"}
{"k":"doc","path":"Lender::lend","text":"Lends an item."}
