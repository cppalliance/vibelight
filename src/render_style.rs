//! The page's style sheet.

/// The page's whole style sheet: light and dark colors follow the system.
pub(crate) const STYLE: &str = "\
:root{color-scheme:light dark;--bg:#fff;--fg:#1f2328;--muted:#59636e;--side:#f6f8fa;--rule:#d1d9e0;--key:#8250df;--com:#59636e;--link:#0969da;--cross:#8a5c00;--mark:#953800;--mark-bg:#fff1e5;--target:#fff8c5}
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--muted:#9198a1;--side:#151b23;--rule:#3d444d;--key:#d2a8ff;--com:#9198a1;--link:#4493f8;--cross:#e3b341;--mark:#ffa657;--mark-bg:#3d2611;--target:#3a3000}}
body{margin:0;display:flex;background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif}
nav{position:sticky;top:0;align-self:flex-start;box-sizing:border-box;width:18rem;height:100vh;flex:none;overflow-y:auto;padding:1rem;background:var(--side);border-right:1px solid var(--rule);font-size:13px}
nav ul{list-style:none;margin:0;padding-left:1rem}
nav>ul{padding-left:0}
main{flex:1;min-width:0;padding:1rem 2rem}
a{color:var(--link);text-decoration:none}
a:hover{text-decoration:underline}
pre{background:var(--side);border-radius:6px;padding:.75rem 1rem;overflow-x:auto}
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
            STYLE.find("a.cross").unwrap() > STYLE.find(".c a").unwrap(),
            "a cross-crate link stays yellow inside a comment"
        );
        assert!(!STYLE.contains("url(") && !STYLE.contains("@import"));
    }
}
