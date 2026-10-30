*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"impls","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":12,"to":49}]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Describe::describe"],"evidence":"no hit of `describe` in the scanned sources beyond its 2 declarations","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["Render::render"],"evidence":"no hit of `render` in the scanned sources beyond its 5 declarations","locs":["src/lib.rs:0"],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Holder"],"evidence":"1 production hits of `Holder` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"lead","id":"no-observed-path-use","mode":"resolved","subjects":["Widget::new"],"evidence":"4 production hits of `new` outside macros, none a resolved or unique qualified path to it","locs":["src/lib.rs:0"],"limits":["external-consumers","macros"]}
{"k":"surface","path":"Describe","kind":"trait"}
{"k":"surface","path":"Holder","kind":"struct"}
{"k":"surface","path":"Render","kind":"trait"}
{"k":"surface","path":"Widget","kind":"struct"}
{"k":"mod","path":"crate","loc":"src/lib.rs:0","types":4}
{"k":"doc","path":"crate","text":"Impl blocks of every kind."}
{"k":"struct","path":"Widget","rust":"pub struct Widget { id: u32 }","loc":"src/lib.rs:0","token_hits":13}
{"k":"doc","path":"Widget","text":"A type with split impls and trait impls."}
{"k":"impl","path":"Widget","rust":"impl Widget","locs":["src/lib.rs:0","src/lib.rs:0"]}
{"k":"fn","path":"Widget::new","sig":"pub fn new(id: u32) -> Widget","loc":"src/lib.rs:0","token_hits":4,"body":{"statements":1,"tokens":6},"ops":["id"]}
{"k":"doc","path":"Widget::new","text":"Makes a widget."}
{"k":"impl","path":"Widget","rust":"impl fmt::Display for Widget","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Widget::fmt","sig":"fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result","loc":"src/lib.rs:0","impl":"impl fmt::Display for Widget","body":{"statements":1,"tokens":3,"macros":["write"]}}
{"k":"impl","path":"Widget","rust":"unsafe impl Send for Widget","locs":["src/lib.rs:0"]}
{"k":"impl","path":"Widget","rust":"impl !Sync for Widget","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Widget::id","sig":"pub fn id(&self) -> u32","loc":"src/lib.rs:0","token_hits":5,"body":{"statements":1,"tokens":5,"single":"field id"},"ops":["self.id"]}
{"k":"doc","path":"Widget::id","text":"Returns the identifier."}
{"k":"impl","path":"Widget","rust":"impl<'a> From<&'a str> for Widget","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Widget::from","sig":"fn from(_: &'a str) -> Widget","loc":"src/lib.rs:0","impl":"impl<'a> From<&'a str> for Widget","body":{"statements":1,"tokens":8}}
{"k":"struct","path":"Holder","rust":"pub struct Holder<'a, T> { item: &'a T }","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Holder","text":"A generic holder."}
{"k":"impl","path":"Holder","rust":"impl<'a, T> Holder<'a, T>","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Holder::item","sig":"pub fn item(&self) -> &'a T","loc":"src/lib.rs:0","token_hits":1,"body":{"statements":1,"tokens":5,"single":"field item"},"ops":["self.item"]}
{"k":"doc","path":"Holder::item","text":"Returns the held item."}
{"k":"trait","path":"Describe","rust":"pub trait Describe","loc":"src/lib.rs:0","token_hits":1}
{"k":"doc","path":"Describe","text":"Describes itself."}
{"k":"fn","path":"Describe::describe","sig":"fn describe(&self) -> String","loc":"src/lib.rs:0"}
{"k":"doc","path":"Describe::describe","text":"Returns a description."}
{"k":"impl","path":"T","rust":"impl<T: fmt::Debug> Describe for T","locs":["src/lib.rs:0"]}
{"k":"fn","path":"T::describe","sig":"fn describe(&self) -> String","loc":"src/lib.rs:0","impl":"impl<T: fmt::Debug> Describe for T","body":{"statements":1,"tokens":3,"macros":["format"]}}
{"k":"trait","path":"Render","rust":"pub trait Render","loc":"src/lib.rs:0","token_hits":4}
{"k":"doc","path":"Render","text":"Renders itself."}
{"k":"fn","path":"Render::render","sig":"fn render(&self) -> String","loc":"src/lib.rs:0"}
{"k":"doc","path":"Render::render","text":"Renders to a string."}
{"k":"impl","path":"&Widget","rust":"impl Render for &Widget","locs":["src/lib.rs:0"]}
{"k":"fn","path":"&Widget::render","sig":"fn render(&self) -> String","loc":"src/lib.rs:0","impl":"impl Render for &Widget","body":{"statements":1,"tokens":8,"single":"call String::new"}}
{"k":"impl","path":"Box<Widget>","rust":"impl Render for Box<Widget>","locs":["src/lib.rs:0"]}
{"k":"fn","path":"Box<Widget>::render","sig":"fn render(&self) -> String","loc":"src/lib.rs:0","impl":"impl Render for Box<Widget>","body":{"statements":1,"tokens":8,"single":"call String::new"}}
{"k":"impl","path":"(u8, u8)","rust":"impl Render for (u8, u8)","locs":["src/lib.rs:0"]}
{"k":"fn","path":"(u8, u8)::render","sig":"fn render(&self) -> String","loc":"src/lib.rs:0","impl":"impl Render for (u8, u8)","body":{"statements":1,"tokens":8,"single":"call String::new"}}
{"k":"impl","path":"[Widget]","rust":"impl Render for [Widget]","locs":["src/lib.rs:0"]}
{"k":"fn","path":"[Widget]::render","sig":"fn render(&self) -> String","loc":"src/lib.rs:0","impl":"impl Render for [Widget]","body":{"statements":1,"tokens":8,"single":"call String::new"}}
