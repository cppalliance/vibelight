//! The page's style sheet.

/// The page's whole style sheet: light and dark colors follow the system.
pub(crate) const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1f2328;--muted:#59636e;--side:#f6f8fa;--rule:#d1d9e0;--key:#8250df;--com:#59636e;--link:#0969da;--cross:#8a5c00;--mark:#953800;--mark-bg:#fff1e5;--target:#fff8c5;--bar:#eef1f4;--bar-h:44px;--box:transparent}
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--muted:#9198a1;--side:#151b23;--rule:#3d444d;--key:#d2a8ff;--com:#9198a1;--link:#4493f8;--cross:#e3b341;--mark:#ffa657;--mark-bg:#3d2611;--target:#3a3000;--bar:#010409;--box:transparent}}
body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif}
.vl-bar{position:sticky;top:0;z-index:10;display:flex;align-items:flex-end;gap:1rem;box-sizing:border-box;height:var(--bar-h);padding:0 1rem;background:var(--bar);border-bottom:1px solid var(--rule)}
.vl-home{align-self:center;font-weight:650;font-size:14px;color:var(--fg)}
.vl-tabs{display:flex;gap:3px;flex:1;min-width:0;overflow-x:auto;overflow-y:hidden;scrollbar-width:thin}
.vl-tab{display:flex;align-items:center;gap:.5rem;padding:.4rem .85rem;border:1px solid transparent;border-bottom:none;border-radius:7px 7px 0 0;color:var(--muted);font-size:13px;white-space:nowrap;cursor:pointer;user-select:none}
.vl-tab.on{background:var(--bg);border-color:var(--rule);color:var(--fg);margin-bottom:-1px;padding-bottom:calc(.4rem + 1px)}
.vl-close{color:var(--muted);font-size:14px;line-height:1;border-radius:3px;padding:0 .15rem}
.vl-close:hover{background:var(--side);color:var(--fg)}
.vl-search{position:relative;align-self:center;display:flex;align-items:center;width:20rem}
.vl-search input{box-sizing:border-box;width:100%;padding:.3rem 4rem .3rem .6rem;border:1px solid var(--rule);border-radius:7px;background:var(--bg);color:var(--fg);font:13px system-ui,sans-serif}
.vl-search kbd{position:absolute;right:.5rem;font:11px ui-monospace,monospace;border:1px solid var(--rule);border-bottom-width:2px;border-radius:4px;padding:0 .3rem;color:var(--muted);pointer-events:none}
.vl-results{position:absolute;top:calc(100% + 4px);right:0;width:34rem;max-height:70vh;overflow-y:auto;margin:0;padding:.3rem;list-style:none;background:var(--bg);border:1px solid var(--rule);border-radius:8px;box-shadow:0 8px 24px rgba(0,0,0,.25)}
.vl-results li{display:flex;align-items:baseline;gap:.6rem;padding:.3rem .5rem;border-radius:5px;cursor:pointer;font-size:13px}
.vl-results li.on{background:var(--side)}
.vl-kind{width:3.2rem;flex:none;color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.04em}
.vl-name{font:600 13px ui-monospace,monospace;color:var(--link)}
.vl-path{color:var(--muted);font:12px ui-monospace,monospace;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.vl-view{display:flex}
.vl-toast{position:fixed;bottom:1.2rem;left:50%;transform:translateX(-50%);z-index:20;padding:.45rem .9rem;border:1px solid var(--rule);border-radius:7px;background:var(--side);color:var(--fg);font:13px ui-monospace,monospace;box-shadow:0 6px 18px rgba(0,0,0,.25)}
.vl-missing{margin:2rem;color:var(--mark)}
.vibelight-target{background:var(--target)}
nav{position:sticky;top:var(--bar-h);align-self:flex-start;box-sizing:border-box;width:18rem;height:calc(100vh - var(--bar-h));flex:none;overflow-y:auto;padding:1rem;background:var(--side);border-right:1px solid var(--rule);font-size:13px}
nav ul{list-style:none;margin:0;padding-left:1rem}
nav>ul{padding-left:0}
nav hr{border:0;border-top:1px solid var(--rule);margin:.9rem 0}
main{flex:1;min-width:0;padding:1rem 2rem}
main>h2{border-top:1px solid var(--rule);margin-top:2rem;padding-top:1.25rem}
a{color:var(--link);text-decoration:none}
a:hover{text-decoration:underline}
pre{background:none;padding:.75rem 0;overflow-x:auto}
pre,code{font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.k{color:var(--key)}
.c{color:var(--com)}
.c a{color:inherit;text-decoration:underline}
a.cross{color:var(--cross)}
b.m{color:var(--mark);background:var(--mark-bg);padding:0 .2em;border-radius:3px}
.source,.anatomy,.coupling{color:var(--muted)}
a.src{color:inherit}
a.fn{color:inherit}
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
.gap{display:block;height:1.45em}
.gap-box{height:0}
.mc{display:block;width:fit-content;white-space:normal;background:var(--box);border-radius:4px;padding:.2rem .9rem 0;margin:.45rem 0 0;font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;max-width:min(80ch,calc(100% - 1rem))}
.cfg,.sec{display:block}
.view{margin:1.1rem 0 .3rem;color:var(--muted);font:600 11px system-ui,sans-serif;letter-spacing:.08em;text-transform:uppercase}
pre.glance{background:var(--side);border:1px solid var(--rule);border-radius:8px;padding:.8rem 1rem}
main.overview{max-width:78rem;margin:0 auto;padding:1.5rem 2.2rem 4rem}
.lede{max-width:56rem;color:var(--muted);font-size:15.5px}
.stats{display:flex;flex-wrap:wrap;gap:.5rem}
.stats span{border:1px solid var(--rule);border-radius:7px;padding:.25rem .65rem;background:var(--side);color:var(--muted);font-size:13px}
.stats b{color:var(--fg);margin-right:.3rem}
ul.legend{display:grid;grid-template-columns:repeat(auto-fill,minmax(22rem,1fr));gap:.4rem 1.6rem;margin:.4rem 0 0;padding:.9rem 1.1rem;list-style:none;border:1px solid var(--rule);border-radius:10px;background:var(--side);color:var(--muted);font-size:13px}
ul.legend code{color:var(--fg)}
h2 .count{font-size:12px;font-weight:400;color:var(--muted);border:1px solid var(--rule);border-radius:10px;padding:0 .5rem;vertical-align:2px}
.hint{color:var(--muted);margin:.2rem 0 .8rem;font-size:14px;max-width:60rem}
table.crates{width:100%;border-collapse:collapse;border:1px solid var(--rule);font-size:14px}
table.crates th{text-align:left;padding:.45rem .8rem;background:var(--side);border-bottom:1px solid var(--rule);color:var(--muted);font-size:11px;font-weight:600;letter-spacing:.06em;text-transform:uppercase}
table.crates td{padding:.55rem .8rem;border-bottom:1px solid var(--rule);vertical-align:top}
table.crates tr.group td{padding:.35rem .8rem;background:var(--side);color:var(--muted);font-size:12.5px}
table.crates td.in{padding-left:1.8rem}
a.crate{font:600 14px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;color:var(--fg);white-space:nowrap}
a.facade{color:var(--link)}
a.implementation{color:var(--cross)}
.path{display:block;color:var(--muted);font:12px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.desc{max-width:44rem}
.desc code{font-size:12.5px;background:var(--side);border-radius:3px;padding:0 .25rem}
.num{white-space:nowrap;color:var(--muted);font-size:13px}
.num b{color:var(--fg)}
.chip{display:inline-block;margin:0 .3rem .3rem 0;padding:.05rem .5rem;border:1px solid var(--rule);border-radius:999px;background:var(--side);color:var(--muted);font:12px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;white-space:nowrap}
a.chip{color:var(--cross)}
a.chip.facade{color:var(--link)}
a.chip b{color:var(--muted);font-weight:400}
.via{margin:1.1rem 0 .45rem}
.via .num{margin-left:.35rem}
.chips{margin:0}
.warn{display:block;margin-top:.3rem;color:var(--mark);font-size:12.5px}
.badge{font:600 10.5px system-ui,sans-serif;letter-spacing:.04em;text-transform:uppercase;border:1px solid var(--rule);border-radius:4px;padding:0 .35rem;color:var(--muted)}
.badge.facade{color:var(--link);border-color:var(--link);vertical-align:4px}
.defined{margin:-.2rem 0 .4rem;color:var(--muted);font:12.5px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
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
            STYLE.contains("\nmain>h2{border-top:1px solid var(--rule);"),
            "a section's rule uses the page's line color"
        );
        assert!(
            STYLE.find("a.cross").unwrap() > STYLE.find(".c a").unwrap(),
            "a cross-crate link stays yellow inside a comment"
        );
        assert!(!STYLE.contains("url(") && !STYLE.contains("@import"));
    }
}
