//! The page's style sheet.

/// The page's whole style sheet: light and dark colors follow the system.
pub(crate) const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1f2328;--muted:#59636e;--side:#f6f8fa;--rule:#d1d9e0;--key:#8250df;--com:#59636e;--link:#0969da;--cross:#8a5c00;--mark:#953800;--mark-bg:#fff1e5;--target:#fff8c5;--box:transparent}
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--muted:#9198a1;--side:#151b23;--rule:#3d444d;--key:#d2a8ff;--com:#9198a1;--link:#4493f8;--cross:#e3b341;--mark:#ffa657;--mark-bg:#3d2611;--target:#3a3000;--box:transparent}}
body{margin:0;display:flex;background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif}
nav{position:sticky;top:0;align-self:flex-start;box-sizing:border-box;width:18rem;height:100vh;flex:none;overflow-y:auto;padding:1rem;background:var(--side);border-right:1px solid var(--rule);font-size:13px}
nav ul{list-style:none;margin:0;padding-left:1rem}
nav>ul{padding-left:0}
main{flex:1;min-width:0;padding:1rem 2rem}
hr{border:0;border-top:1px solid var(--rule);margin:2rem 0}
a{color:var(--link);text-decoration:none}
a:hover{text-decoration:underline}
pre{background:none;padding:.75rem 0;overflow-x:auto}
pre,code{font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.k{color:var(--key)}
.c{color:var(--com)}
.c a{color:inherit;text-decoration:underline}
a.cross{color:var(--cross)}
b.m{color:var(--mark);background:var(--mark-bg);padding:0 .2em;border-radius:3px}
.source{color:var(--muted)}
a.src{color:inherit}
span.src{cursor:default}
.stamp{margin:0;color:var(--muted);font-size:13px}
:target{background:var(--target)}
.c code{font:inherit;background:none;padding:0}
.doc p,.doc ul,.doc h4{max-width:46rem}
.doc p,.doc li{font:15px/1.55 system-ui,sans-serif;color:var(--fg);margin:.5rem 0}
.doc ul{margin:.5rem 0;padding-left:1.25rem;list-style:disc}
.doc h4{margin:1rem 0 .35rem;font-size:15px}
.doc code{font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;background:var(--side);border-radius:3px;padding:.05em .25em}
.doc pre{max-width:64rem}
.dim{opacity:.5}
.toggle{position:absolute;width:1px;height:1px;opacity:0;pointer-events:none}
.button{display:block;margin:0 0 1rem;padding:.3rem .6rem;border:1px solid var(--rule);border-radius:5px;background:var(--bg);color:var(--fg);font-size:13px;text-align:center;cursor:pointer}
.button:hover{background:var(--side)}
.toggle:focus-visible~nav .button{outline:2px solid var(--link)}
.toggle~nav .button .on{display:none}
.toggle:checked~nav .button .on{display:inline}
.toggle:checked~nav .button .off{display:none}
.toggle:checked~main .mc,.toggle:checked~main .mc-doc{display:none}
.gap{display:block;height:1.45em}
.gap-box{height:0}
.toggle:checked~main .gap{height:.725em}
.mc{display:block;width:fit-content;white-space:pre;background:var(--box);border-radius:4px;padding:.2rem .9rem;margin:.45rem 0;font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;max-width:calc(100% - 1rem)}
";

#[cfg(test)]
mod tests {
    use super::STYLE;

    #[test]
    fn the_style_sheet_colors_cross_crate_links_in_both_themes() {
        let dark = STYLE.find("@media (prefers-color-scheme:dark)").unwrap();
        let (light, dark) = STYLE.split_at(dark);
        assert!(
            light.contains("--cross:#8a5c00"),
            "a darker gold when light"
        );
        let dark_line = dark.lines().next().unwrap();
        assert!(
            dark_line.contains("--cross:#e3b341"),
            "a lighter yellow when dark"
        );
        assert!(STYLE.contains("\na.cross{color:var(--cross)}\n"));
        assert!(STYLE.contains("\nspan.src{cursor:default}\n"));
        assert!(
            STYLE.contains("\nhr{border:0;border-top:1px solid var(--rule);margin:2rem 0}\n"),
            "a rule uses the page's line color"
        );
        assert!(
            STYLE.find("a.cross").unwrap() > STYLE.find(".c a").unwrap(),
            "a cross-crate link stays yellow inside a comment"
        );
        assert!(!STYLE.contains("url(") && !STYLE.contains("@import"));
    }
}
