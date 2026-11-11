//! The HTML is one file that works from `file:`: `index.html`, in the
//! folder a run writes into, holds the app and, packed in its data block,
//! each outline's page and the search index. Cursor's browser drops a click
//! on a link to a `file:` URL, so no page is an HTML file of its own.

use std::fs;
use std::path::Path;

use crate::config::{MEMBERS, garden, lines, vibelight_in};
use crate::support::{data_block, page, read_pages, search_index};

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name)).unwrap_or_else(|_| panic!("{name} was not written"))
}

#[test]
fn the_html_is_one_file_that_holds_the_app_and_every_page() {
    let (_tmp, repo) = garden(MEMBERS);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let html = repo.join(".vibelight");
    let mut names: Vec<String> = fs::read_dir(&html)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [".gitignore", "index.html", "md"],
        "the folder holds the one HTML file, the folder of the markdown, and no `html` folder"
    );
    let file = read(&html, "index.html");
    let app = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("html")
            .join("browse.js"),
    )
    .unwrap();
    assert!(
        file.contains(&format!("<script>\n{app}</script>\n</body>")),
        "the file holds the app the binary ships"
    );
    assert!(file.contains("<noscript>"), "{file}");
    assert!(
        !file.contains("<main>"),
        "the file holds no page in the clear"
    );
    let block = data_block(&file);
    let stems: Vec<&str> = block["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["stem"].as_str().unwrap())
        .collect();
    assert_eq!(stems, ["index", "alpha", "beta", "facade"]);
    for (name, content) in read_pages(&html) {
        assert!(content.contains(">\n<p class=\"stamp\">"), "{name}");
        assert!(content.ends_with("</main>\n"), "{name}");
    }
}

#[test]
fn the_pages_keep_their_links_to_each_other() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let index = page(&repo.join(".vibelight"), "index");
    for stem in ["alpha", "beta", "facade"] {
        assert!(
            index.contains(&format!("href=\"{stem}.html\">{stem}</a>")),
            "the overview lost its link to {stem}"
        );
    }
}

/// Returns the overview's content of a run of the garden with `config`.
fn overview(config: &str) -> String {
    let (_tmp, repo) = garden(config);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    page(&repo.join(".vibelight"), "index")
}

#[test]
fn the_overview_puts_the_facade_first_and_the_crates_behind_it_after() {
    let page = overview(MEMBERS);
    let facades = page.find("<h2 id=\"facades\">").expect("a facades table");
    let behind = page
        .find("<h2 id=\"implementation\">")
        .expect("an implementation table");
    assert!(facades < behind, "the facades come first");
    assert!(page.contains("<span><b>1</b> facade</span><span><b>2</b> implementation</span>"));
    assert!(page.contains("<a class=\"crate facade\" href=\"facade.html\">facade</a>"));
    assert!(page.contains(
        "<tr class=\"group\"><td colspan=\"3\">Behind \
         <a class=\"crate facade\" href=\"facade.html\">facade</a></td></tr>"
    ));
    for stem in ["alpha", "beta"] {
        let row = format!("<a class=\"crate implementation\" href=\"{stem}.html\">{stem}</a>");
        assert!(page[behind..].contains(&row), "{stem} is behind the facade");
    }
    assert!(
        page.contains(
            "<h2 id=\"legend\">Legend</h2>\n<div class=\"legend\">\n\
             <pre><code><span class=\"k\">pub</span> <span class=\"k\">struct</span> Sample"
        ),
        "the legend opens with its sample"
    );
}

#[test]
fn the_config_s_facades_choose_the_facades() {
    let page = overview(&format!("{MEMBERS}facades = [\"crates/beta\"]\n"));
    let start = page.find("<h2 id=\"facades\">").expect("a facades table");
    let end = page[start + 1..]
        .find("<h2 ")
        .map_or(page.len(), |at| start + 1 + at);
    let facades = &page[start..end];
    assert!(facades.contains("<a class=\"crate facade\" href=\"beta.html\">beta</a>"));
    assert!(
        !facades.contains("href=\"facade.html\""),
        "the crate the counts would choose is no facade: {facades}"
    );
}

#[test]
fn a_facades_key_that_is_no_list_of_strings_is_an_error() {
    let (_tmp, repo) = garden(&format!("{MEMBERS}facades = \"crates/beta\"\n"));
    let result = vibelight_in(&repo, &[]);
    assert_eq!(result.status.code(), Some(1));
    assert!(
        lines(&result.stderr)[0].ends_with("`facades` is a list of strings"),
        "{:?}",
        lines(&result.stderr)
    );
}

/// Returns the sidebar's line for one symbol of a facade's page.
fn symbol(page: &str, name: &str) -> String {
    let nav = &page[..page.find("</nav>").expect("a page has a sidebar")];
    let list = &nav[nav.find("<ul class=\"symbols\">").expect("a symbol list")..];
    list.lines()
        .find(|line| line.contains(&format!(">{name}<")))
        .unwrap_or_else(|| panic!("{name} is no symbol of the page"))
        .to_owned()
}

#[test]
fn a_facade_s_symbol_list_prints_bold_each_type_no_other_item_of_its_api_names() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let page = page(&repo.join(".vibelight"), "facade");
    // `Panel` holds a `Meter` and implements `Reads`, `show` takes a
    // `Reads`, and `Meter` holds a `Gauge`. Nothing names `Panel`. Nothing
    // names `show` either, but no declaration can name a function, so the
    // rule tells nothing of one.
    assert_eq!(
        symbol(&page, "Panel"),
        "<li><a href=\"#type.facade.Panel\"><b>Panel</b></a></li>"
    );
    for (name, id) in [
        ("Gauge", "api.Gauge"),
        ("Meter", "api.Meter"),
        ("Reads", "api.Reads"),
        ("show()", "fn.facade.show"),
    ] {
        assert_eq!(
            symbol(&page, name),
            format!("<li><a href=\"#{id}\">{name}</a></li>")
        );
    }
}

#[test]
fn a_facade_lists_what_its_code_names_and_does_not_re_export() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let page = page(&repo.join(".vibelight"), "facade");
    // `Meter` holds a `Valve` of beta, which the facade does not re-export.
    // The stray and vendored types are of crates outside the run.
    assert!(page.contains(
        "<li><a href=\"#named\">Named, not re-exported</a> <span class=\"dim\">1</span></li>"
    ));
    assert!(page.contains(
        " Their code names <a href=\"#named\">1 item that facade does not re-export</a>.</p>"
    ));
    assert!(
        page.ends_with(
            "<h2 id=\"named\">Named, not re-exported <span class=\"count\">1</span></h2>\n\
             <p class=\"hint\">What the code above names that facade does not re-export, under \
             the crate a user of facade names it through.</p>\n\
             <p class=\"via\"><a class=\"crate\" href=\"beta.html\">beta</a> \
             <span class=\"num\">1 item, which no facade re-exports</span></p>\n\
             <p class=\"chips\"><a class=\"chip\" href=\"beta.html#type.beta.parts.Valve\">\
             Valve</a></p>\n</main>\n"
        ),
        "{}",
        &page[page.len().saturating_sub(700)..]
    );
}

#[test]
fn what_a_facade_names_is_listed_under_the_facade_that_re_exports_it() {
    let (_tmp, repo) = garden(&format!(
        "{MEMBERS}facades = [\"crates/facade\", \"crates/front\"]\n"
    ));
    let front = repo.join("crates").join("front");
    fs::create_dir_all(front.join("src")).unwrap();
    fs::write(
        front.join("Cargo.toml"),
        "[package]\nname = \"front\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n\
         [dependencies]\nbeta = { workspace = true }\n",
    )
    .unwrap();
    fs::write(
        front.join("src").join("lib.rs"),
        "//! Front: re-exports a part.\n\npub use beta::parts::Valve;\n",
    )
    .unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let html = repo.join(".vibelight");
    let facade = page(&html, "facade");
    assert!(
        facade.ends_with(
            "<p class=\"via\"><a class=\"crate facade\" href=\"front.html\">front</a> \
             <span class=\"num\">1 item</span></p>\n\
             <p class=\"chips\"><a class=\"chip\" href=\"front.html#api.Valve\">Valve</a></p>\n\
             </main>\n"
        ),
        "{}",
        &facade[facade.len().saturating_sub(500)..]
    );
    let front = page(&html, "front");
    assert!(front.contains("<h3 id=\"api.Valve\">"), "the link lands");
    assert!(
        !front.contains("id=\"named\""),
        "front's one item names nothing"
    );
    assert_eq!(
        symbol(&front, "Valve"),
        "<li><a href=\"#api.Valve\"><b>Valve</b></a></li>"
    );
}

#[test]
fn the_search_index_lists_every_id_that_a_page_declares() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let html = repo.join(".vibelight");
    let index = search_index(&html);
    let pages: Vec<&str> = index["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert_eq!(pages, ["alpha", "beta", "facade"]);
    let listed: Vec<(usize, String)> = index["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let page = usize::try_from(e[0].as_u64().unwrap()).unwrap();
            (page, e[1].as_str().unwrap().to_owned())
        })
        .collect();
    assert!(listed.len() > 3, "{listed:?}");
    for (at, id) in &listed {
        assert!(
            page(&html, pages[*at]).contains(&format!("id=\"{id}\"")),
            "{id} is not an id of {}",
            pages[*at]
        );
    }
    let alpha = page(&html, "alpha");
    for (at, _) in alpha.match_indices(" id=\"type.") {
        let id = alpha[at + 5..].split('"').next().unwrap();
        assert!(
            listed
                .iter()
                .any(|(page, listed)| *page == 0 && listed == id),
            "{id} is not listed"
        );
    }
}

#[test]
fn a_run_lists_the_one_html_file_on_stdout() {
    let (_tmp, repo) = garden(MEMBERS);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    let listed: Vec<String> = lines(&result.stdout)
        .into_iter()
        .filter(|path| (Path::new(path).extension()).is_some_and(|ext| ext == "html"))
        .collect();
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert!(
        Path::new(&listed[0]).ends_with(".vibelight/index.html"),
        "{listed:?}"
    );
}

#[test]
fn a_changed_stamp_alone_leaves_the_file_alone() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let html = repo.join(".vibelight");
    let before = read(&html, "index.html");
    let result = crate::support::vibelight()
        .env("SOURCE_DATE_EPOCH", "1798761600")
        .current_dir(&repo)
        .output()
        .expect("the vibelight binary runs");
    assert!(result.status.success());
    assert_eq!(read(&html, "index.html"), before);
}

#[test]
fn a_run_upgrades_the_gitignore_of_an_earlier_release_and_leaves_an_edited_one() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let ignore = repo.join(".vibelight").join(".gitignore");
    assert_eq!(fs::read_to_string(&ignore).unwrap(), "*.md\n*.html\n*.js\n");

    fs::write(&ignore, "*.md\n*.html\n").unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    let notes = lines(&result.stderr);
    assert!(
        notes.iter().any(|note| note.starts_with("note: updated ")
            && note.ends_with(".gitignore to ignore the pages' script files too")),
        "{notes:?}"
    );
    assert_eq!(fs::read_to_string(&ignore).unwrap(), "*.md\n*.html\n*.js\n");

    fs::write(&ignore, "*.md\n*.html\nmine\n").unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert!(
        !lines(&result.stderr)
            .iter()
            .any(|note| note.contains(".gitignore")),
        "{:?}",
        lines(&result.stderr)
    );
    assert_eq!(
        fs::read_to_string(&ignore).unwrap(),
        "*.md\n*.html\nmine\n",
        "an edited .gitignore stays as it is"
    );
}

#[test]
fn a_page_of_an_earlier_release_is_removed() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    // An earlier release wrote a page for each crate into `html`.
    let html = repo.join(".vibelight").join("html");
    fs::create_dir(&html).unwrap();
    let old = "<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n<html>\n";
    fs::write(html.join("alpha.html"), old).unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert!(!html.join("alpha.html").exists());
    assert!(
        lines(&result.stderr)
            .iter()
            .any(|note| note.starts_with("note: removed 1 file that an earlier release wrote: ")),
        "{:?}",
        lines(&result.stderr)
    );
}

#[test]
fn the_script_files_of_an_earlier_release_are_removed() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let html = repo.join(".vibelight").join("html");
    fs::create_dir(&html).unwrap();
    let old = [
        (
            "alpha.page.js",
            "vibelightPage(\"alpha\", \"alpha\", `\n<main>\n</main>\n`);\n",
        ),
        ("search.js", "vibelightSearch({\"pages\":[],\"ids\":[]});\n"),
        (
            "vibelight.js",
            "// vibelight: the outlines as one page that works from file:.\n",
        ),
    ];
    for (name, text) in old {
        fs::write(html.join(name), text).unwrap();
    }
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    let notes = lines(&result.stderr);
    for (name, _) in old {
        assert!(!html.join(name).exists(), "{name} is still there");
    }
    assert!(
        notes
            .iter()
            .any(|note| note.starts_with("note: removed 3 files that an earlier release wrote: ")),
        "{notes:?}"
    );
    assert!(!html.exists(), "the run emptied the folder, so it went");
}
