*Updated 2026-01-01T00:00:00Z*
{"k":"vibelight-outline","schema":1,"crate":"modules","target":"lib"}
{"k":"modules","items":[{"path":"crate","from":24,"to":25},{"path":"files","from":26,"to":29},{"path":"inline","from":30,"to":33},{"path":"inline::deeper","from":34,"to":36},{"path":"nested","from":37,"to":38},{"path":"nested::child","from":39,"to":42},{"path":"only_modules","from":43,"to":43},{"path":"only_modules::a","from":44,"to":46},{"path":"only_modules::b","from":47,"to":49},{"path":"renamed","from":50,"to":53}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["files"],"evidence":"holds only `InFile`","locs":[{"file":"src/files.rs","line":0}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["inline::deeper"],"evidence":"holds only `Deepest`","locs":[{"file":"src/lib.rs","line":0}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["nested"],"evidence":"holds only `mod child`","locs":[{"file":"src/nested/mod.rs","line":0}]}
{"k":"lead","id":"single-item-module","mode":"direct-syntax","subjects":["renamed"],"evidence":"holds only `LOADED`","locs":[{"file":"src/custom-name.rs","line":0}]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["nested::child"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/nested/child.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["nested::child::child_fn"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/nested/child.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["only_modules::a"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/only_modules/a.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["only_modules::a::A"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/only_modules/a.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["only_modules::b"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/only_modules/b.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["only_modules::b::B"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/only_modules/b.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"pub-off-surface","mode":"resolved","subjects":["renamed::LOADED"],"evidence":"`pub`, but no public path from the crate root reaches it","locs":[{"file":"src/custom-name.rs","line":0}],"limits":["macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["files::InFile"],"evidence":"no hit of `InFile` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/files.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["inline::deeper::Deepest"],"evidence":"no hit of `Deepest` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"lead","id":"no-observed-name-use","mode":"lexical","subjects":["inline::Inside"],"evidence":"no hit of `Inside` in the scanned sources beyond its 1 declaration","locs":[{"file":"src/lib.rs","line":0}],"limits":["external-consumers","generated-code","macros"]}
{"k":"surface","path":"files","kind":"mod"}
{"k":"surface","path":"files::InFile","kind":"struct"}
{"k":"surface","path":"inline","kind":"mod"}
{"k":"surface","path":"inline::deeper","kind":"mod"}
{"k":"surface","path":"inline::deeper::Deepest","kind":"struct"}
{"k":"surface","path":"inline::Inside","kind":"struct"}
{"k":"mod","path":"crate","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"crate","text":"Module layouts."}
{"k":"mod","path":"files","loc":{"file":"src/files.rs","line":0},"vis":"public","types":1}
{"k":"doc","path":"files","text":"A file module."}
{"k":"struct","path":"files::InFile","rust":"pub struct InFile","loc":{"file":"src/files.rs","line":0}}
{"k":"doc","path":"files::InFile","text":"A type in a file module."}
{"k":"mod","path":"inline","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":1}
{"k":"doc","path":"inline","text":"An inline module.\nInner docs of the inline module."}
{"k":"struct","path":"inline::Inside","rust":"pub struct Inside","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"inline::Inside","text":"A type inside the inline module."}
{"k":"mod","path":"inline::deeper","loc":{"file":"src/lib.rs","line":0},"vis":"public","types":1}
{"k":"struct","path":"inline::deeper::Deepest","rust":"pub struct Deepest","loc":{"file":"src/lib.rs","line":0}}
{"k":"doc","path":"inline::deeper::Deepest","text":"The deepest type."}
{"k":"mod","path":"nested","loc":{"file":"src/nested/mod.rs","line":0},"vis":"private"}
{"k":"doc","path":"nested","text":"A module in a mod.rs file."}
{"k":"mod","path":"nested::child","loc":{"file":"src/nested/child.rs","line":0},"vis":"public","functions":{"public":1}}
{"k":"doc","path":"nested::child","text":"A child of a mod.rs module."}
{"k":"fn","path":"nested::child::child_fn","sig":"pub fn child_fn()","loc":{"file":"src/nested/child.rs","line":0},"body":{"statements":0,"tokens":2}}
{"k":"doc","path":"nested::child::child_fn","text":"A function in a nested file."}
{"k":"mod","path":"only_modules","loc":{"file":"src/only_modules.rs","line":0},"vis":"internal"}
{"k":"mod","path":"only_modules::a","loc":{"file":"src/only_modules/a.rs","line":0},"vis":"public","types":1}
{"k":"struct","path":"only_modules::a::A","rust":"pub struct A","loc":{"file":"src/only_modules/a.rs","line":0}}
{"k":"doc","path":"only_modules::a::A","text":"The first leaf type."}
{"k":"mod","path":"only_modules::b","loc":{"file":"src/only_modules/b.rs","line":0},"vis":"public","types":1}
{"k":"struct","path":"only_modules::b::B","rust":"pub struct B","loc":{"file":"src/only_modules/b.rs","line":0}}
{"k":"doc","path":"only_modules::b::B","text":"The second leaf type."}
{"k":"mod","path":"renamed","loc":{"file":"src/custom-name.rs","line":0},"vis":"private"}
{"k":"doc","path":"renamed","text":"Loaded through a path attribute."}
{"k":"const","path":"renamed::LOADED","rust":"pub const LOADED: bool","value":"true","loc":{"file":"src/custom-name.rs","line":0},"body":{"statements":1,"tokens":1}}
{"k":"doc","path":"renamed::LOADED","text":"Shows the file was loaded."}
