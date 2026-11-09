//! The HTML folder is one page that works from `file:`: `index.html` loads
//! `vibelight.js`, which shows each outline's page from its data file and
//! finds declarations through `search.js`. Cursor's browser drops a click on
//! a link to a `file:` URL, so no page is an HTML file of its own.

use std::fs;
use std::path::Path;

use crate::config::{MEMBERS, garden, lines, vibelight_in};
use crate::support::{page_content, read_pages};

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name)).unwrap_or_else(|_| panic!("{name} was not written"))
}

#[test]
fn the_folder_is_one_page_with_its_script_and_a_data_file_per_page() {
    let (_tmp, repo) = garden(MEMBERS);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success(), "{:?}", lines(&result.stderr));
    let html = repo.join(".vibelight").join("html");
    let shell = read(&html, "index.html");
    assert!(shell.contains("</style>\n<script src=\"vibelight.js\" defer></script>\n</head>"));
    assert!(shell.contains("<noscript>"), "{shell}");
    assert!(!shell.contains("<main>"), "the shell holds no page");
    assert_eq!(
        read(&html, "vibelight.js"),
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("browse.js")
        )
        .unwrap(),
        "the script on disk is the one the binary ships"
    );
    for stem in ["alpha", "beta", "facade", "index"] {
        assert!(
            !html.join(format!("{stem}.html")).exists() || stem == "index",
            "{stem} has an HTML file of its own"
        );
        let data = read(&html, &format!("{stem}.page.js"));
        assert!(
            data.starts_with(&format!("vibelightPage(\"{stem}\", \"")),
            "{stem}: {}",
            &data[..data.len().min(80)]
        );
        let content = page_content(&data);
        assert!(content.contains(">\n<p class=\"stamp\">"), "{stem}");
        assert!(content.ends_with("</main>\n"), "{stem}");
    }
}

#[test]
fn the_pages_keep_their_links_to_each_other() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let pages = read_pages(&repo.join(".vibelight").join("html"));
    let index = &pages
        .iter()
        .find(|(name, _)| name == "index.html")
        .expect("the index is a page")
        .1;
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
    page_content(&read(
        &repo.join(".vibelight").join("html"),
        "index.page.js",
    ))
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
    assert!(page.contains("<h2 id=\"legend\">Legend</h2>\n<ul class=\"legend\">"));
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

#[test]
fn the_search_index_lists_every_id_that_a_page_declares() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let html = repo.join(".vibelight").join("html");
    let search = read(&html, "search.js");
    let json = search
        .strip_prefix("vibelightSearch(")
        .and_then(|rest| rest.strip_suffix(");\n"))
        .expect("the search index is one call");
    let index: serde_json::Value = serde_json::from_str(json).expect("its argument is JSON");
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
    for (page, id) in &listed {
        let content = page_content(&read(&html, &format!("{}.page.js", pages[*page])));
        assert!(
            content.contains(&format!("id=\"{id}\"")),
            "{id} is not an id of {}",
            pages[*page]
        );
    }
    let alpha = page_content(&read(&html, "alpha.page.js"));
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
fn a_run_lists_the_folder_files_on_stdout() {
    let (_tmp, repo) = garden(MEMBERS);
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    let out = lines(&result.stdout);
    for name in [
        "index.html",
        "index.page.js",
        "search.js",
        "vibelight.js",
        "alpha.page.js",
        "beta.page.js",
        "facade.page.js",
    ] {
        assert!(
            out.iter().any(|path| path.ends_with(name)),
            "{name} is not listed: {out:?}"
        );
    }
}

#[test]
fn a_changed_stamp_alone_leaves_the_data_files_alone() {
    let (_tmp, repo) = garden(MEMBERS);
    assert!(vibelight_in(&repo, &[]).status.success());
    let html = repo.join(".vibelight").join("html");
    let before = read(&html, "alpha.page.js");
    let shell = read(&html, "index.html");
    let result = crate::support::vibelight()
        .env("SOURCE_DATE_EPOCH", "1798761600")
        .current_dir(&repo)
        .output()
        .expect("the vibelight binary runs");
    assert!(result.status.success());
    assert_eq!(read(&html, "alpha.page.js"), before);
    assert_eq!(read(&html, "index.html"), shell);
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
    let html = repo.join(".vibelight").join("html");
    let old = "<!-- Updated 2026-01-01T00:00:00Z -->\n<!DOCTYPE html>\n<html>\n";
    fs::write(html.join("alpha.html"), old).unwrap();
    let result = vibelight_in(&repo, &[]);
    assert!(result.status.success());
    assert!(!html.join("alpha.html").exists());
    assert!(
        lines(&result.stderr)
            .iter()
            .any(|note| note.contains("alpha.html: no crate of this run writes it")),
        "{:?}",
        lines(&result.stderr)
    );
}
