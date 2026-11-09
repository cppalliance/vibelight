//! The page of a facade: its public API and nothing else. Every path its
//! users can name, grouped by the module they name it in, each with the
//! declaration it names, pulled from the crate that defines it and shown as
//! those users see it, with its `pub` members alone. Each module shows its
//! items twice, at a glance and documented, and each item says where the
//! crate that defines it writes it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::document::Document;
use crate::layout::{Block, Line, Span, key, plain};
use crate::layout_decl::public_block;
use crate::layout_items::{docs_block, item_block};
use crate::model::{Crate, Loc, ModId, Reach, name_order};
use crate::render_html::{Anchors, ApiIds, Page, attr, escape};
use crate::surface::{Entry, Kind};

/// One target of the run, as a facade's page reaches into it.
pub(crate) struct Owner<'a> {
    pub(crate) krate: &'a Crate,
    pub(crate) doc: &'a Document,
    pub(crate) links: Option<&'a dyn Fn(Loc) -> String>,
    /// The file of its own page.
    pub(crate) file: String,
}

/// What a facade's page is made of: the facade's index among `owners`,
/// every target of the run in target order, its public surface, and the
/// pages of the run.
pub(crate) struct Facade<'a> {
    pub(crate) index: usize,
    pub(crate) owners: &'a [Owner<'a>],
    pub(crate) surface: &'a [Entry],
    pub(crate) run: &'a [Anchors],
}

/// One path of the API: its name, the block of what it names, the target
/// that defines it, and where that target writes it.
struct Named {
    name: String,
    block: Block,
    owner: usize,
    /// What the path names in the crate that defines it, when that is an
    /// item of the run.
    item: Option<Reach>,
    /// The item's id on the run's pages when the facade defines it, which
    /// its heading takes the first time it shows.
    anchor: Option<String>,
    /// The id of its heading.
    id: String,
    defined: Option<Defined>,
}

/// The paths of a surface by module: each module's id in the facade when
/// the surface names it, and the module's items.
type Groups = BTreeMap<Vec<String>, (Option<ModId>, Vec<Named>)>;

/// A facade's page as it renders: the facade, and the heading of each item
/// of the run that the page shows, which every link to that item goes to.
struct View<'a> {
    facade: &'a Facade<'a>,
    api: ApiIds,
}

/// Where the target that defines an item writes it.
struct Defined {
    path: String,
    id: Option<String>,
    loc: Loc,
}

/// Leaves the facade's document with ids for only what its page shows: the
/// declarations and values its public surface names. Every other id is
/// emptied, so no link of the run points at an item the page leaves out.
pub(crate) fn keep_public(doc: &mut Document, surface: &[Entry]) {
    let mut decls = BTreeSet::new();
    let mut values = BTreeSet::new();
    for entry in surface.iter().filter(|e| !e.test_support) {
        match entry.kind {
            Kind::Decl(d) => {
                decls.insert(d);
            }
            Kind::Function(at) | Kind::Value(at) => {
                values.insert(at);
            }
            _ => {}
        }
    }
    for (d, id) in doc.decl_ids.iter_mut().enumerate() {
        if !decls.contains(&d) {
            id.clear();
        }
    }
    doc.value_ids.retain(|at, _| values.contains(at));
}

/// Renders the facade's page content, with `stamp` at the top of `<main>`,
/// and returns it with the ids of the headings that no other page of the
/// run has, for the search index.
pub(crate) fn fragment(facade: &Facade<'_>, stamp: &str) -> (String, Vec<String>) {
    let own = &facade.owners[facade.index];
    let title = &own.doc.title;
    let groups = groups(facade);
    let mut api = ApiIds::new();
    for named in groups.values().flat_map(|(_, items)| items) {
        if let Some(reach) = named.item {
            (api.entry((named.owner, reach))).or_insert_with(|| named.id.clone());
        }
    }
    let view = View { facade, api };
    let mut out = String::from("<nav>\n<ul>\n<li><a href=\"#api\">API</a>\n<ul>\n");
    for (path, (_, items)) in &groups {
        let _ = writeln!(
            out,
            "<li><a href=\"#{}\">{}</a> <span class=\"dim\">{}</span></li>",
            attr(&module_id(path)),
            escape(&module_label(title, path)),
            items.len()
        );
    }
    out.push_str("</ul>\n</li>\n</ul>\n</nav>\n<main>\n");
    let _ = writeln!(out, "{}", crate::stamp::html_paragraph(stamp));
    let _ = writeln!(
        out,
        "<h1 id=\"{}\">{} <span class=\"badge facade\">facade</span></h1>",
        attr(&own.doc.id),
        escape(title)
    );
    if let Some(docs) = &own.doc.crate_docs {
        view.page(facade.index).crate_docs(&mut out, docs);
    }
    let items: usize = groups.values().map(|(_, items)| items.len()).sum();
    let mut from: Vec<usize> = (groups.values())
        .flat_map(|(_, items)| items.iter().map(|named| named.owner))
        .filter(|&owner| owner != facade.index)
        .collect();
    from.sort_unstable();
    from.dedup();
    let from = match from.len() {
        0 => String::new(),
        n => format!(", from {}", counted(n, "other crate")),
    };
    let _ = writeln!(
        out,
        "<p class=\"note\" id=\"api\">The public API of {}: {}{from}. Each shows only what a \
         user of {} can name.</p>",
        escape(title),
        counted(items, "item"),
        escape(title)
    );
    for (path, (module, items)) in &groups {
        view.module_section(&mut out, (path, *module), items);
    }
    out.push_str("</main>\n");
    let headings = (groups.into_values())
        .flat_map(|(_, items)| items.into_iter().map(|named| named.id))
        .filter(|id| id.starts_with("api."))
        .collect();
    (out, headings)
}

/// The paths of the surface by module, crate root first, each module's
/// items by name. Each heading takes the item's own id the first time the
/// facade's own item shows, and its path's id otherwise.
fn groups(facade: &Facade<'_>) -> Groups {
    let mut groups = Groups::new();
    for entry in facade.surface.iter().filter(|e| !e.test_support) {
        let Some((name, module)) = entry.path.split_last() else {
            continue;
        };
        if let Kind::Module(m) = entry.kind {
            groups.entry(entry.path.clone()).or_default().0 = Some(m);
            continue;
        }
        if let Some(named) = named(facade, name, &entry.kind) {
            groups.entry(module.to_vec()).or_default().1.push(named);
        }
    }
    groups.retain(|_, (_, items)| !items.is_empty());
    let mut used = BTreeSet::new();
    for (module, (_, items)) in &mut groups {
        items.sort_by(|a, b| name_order(&a.name, &b.name));
        for named in items {
            named.id = match named.anchor.take() {
                Some(anchor) if used.insert(anchor.clone()) => anchor,
                _ => format!("api.{}", dotted(module, &named.name)),
            };
        }
    }
    groups
}

/// Returns a module path and a name joined by dots.
fn dotted(module: &[String], name: &str) -> String {
    let mut out = String::new();
    for part in module {
        out.push_str(part);
        out.push('.');
    }
    out.push_str(name);
    out
}

/// Returns what one path of the surface names, with its block as a user of
/// the facade sees it.
fn named(facade: &Facade<'_>, name: &str, kind: &Kind) -> Option<Named> {
    let own = facade.index;
    let (owner, reach) = match kind {
        Kind::Cross(_, to) => (to.krate, to.reach),
        Kind::Decl(d) => (own, Reach::Decl(*d)),
        Kind::Function(at) | Kind::Value(at) => (own, Reach::Value(*at)),
        Kind::Macro(_) => {
            let line = Line {
                indent: 0,
                spans: vec![key("macro_rules!"), plain(&format!(" {name} {{ ... }}"))],
                trailing: None,
            };
            return Some(plain_named(name, own, line));
        }
        Kind::Unlinked(path) => {
            let line = Line {
                indent: 0,
                spans: vec![key("pub use"), plain(&format!(" {path};"))],
                trailing: None,
            };
            return Some(plain_named(name, own, line));
        }
        Kind::Module(_) => return None,
    };
    let target = facade.owners.get(owner)?;
    let krate = target.krate;
    let (block, mut defined) = match reach {
        Reach::Decl(d) => {
            let decl = krate.decls.get(d)?;
            let path = item_path(krate, decl.module, &decl.name);
            let id = target.doc.decl_ids.get(d).cloned();
            (
                public_block(krate, d, name),
                Defined {
                    path,
                    id,
                    loc: decl.loc,
                },
            )
        }
        Reach::Value(at) => {
            let item = krate.modules.get(at.module)?.items.get(at.item)?;
            let path = item_path(krate, at.module, &item.name);
            let id = target.doc.value_ids.get(&at).cloned();
            let mut block = item_block(krate, at, name);
            block.lines.retain(|line| !is_reach(line));
            (
                block,
                Defined {
                    path,
                    id,
                    loc: item.loc,
                },
            )
        }
    };
    let anchor = if owner == own {
        defined.id.take()
    } else {
        None
    };
    Some(Named {
        name: name.to_owned(),
        block,
        owner,
        item: Some(reach),
        anchor,
        id: String::new(),
        defined: Some(defined),
    })
}

/// Whether a line says how far an item reaches inside its crate, which no
/// user of a facade can see.
fn is_reach(line: &Line) -> bool {
    matches!(line.spans.first(), Some(Span::Dim(text)) if text.starts_with("reaches "))
}

fn plain_named(name: &str, owner: usize, line: Line) -> Named {
    Named {
        name: name.to_owned(),
        block: Block {
            lines: vec![line],
            repeats: Vec::new(),
        },
        owner,
        item: None,
        anchor: None,
        id: String::new(),
        defined: None,
    }
}

/// Returns a count with its noun, `1 item` or `2 items`.
fn counted(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// Returns an item's path in the crate that defines it, from its name.
fn item_path(krate: &Crate, module: ModId, name: &str) -> String {
    let mut parts = vec![krate.name.clone()];
    parts.extend(krate.modules[module].path.iter().cloned());
    parts.push(name.to_owned());
    parts.join("::")
}

fn module_id(path: &[String]) -> String {
    if path.is_empty() {
        "api.mod".to_owned()
    } else {
        format!("api.mod.{}", path.join("."))
    }
}

fn module_label(title: &str, path: &[String]) -> String {
    if path.is_empty() {
        title.to_owned()
    } else {
        path.join("::")
    }
}

impl View<'_> {
    /// Returns the renderer of the blocks of target `owner` on this page.
    fn page(&self, owner: usize) -> Page<'_> {
        let facade = self.facade;
        let target = &facade.owners[owner];
        Page {
            doc: target.doc,
            member_links: target.links,
            run: facade.run,
            foreign: (owner != facade.index).then_some(target.file.as_str()),
            ids: false,
            api: Some((&self.api, owner)),
        }
    }

    /// Writes one module's section: its heading, its docs, its items at a
    /// glance, then each item documented.
    fn module_section(
        &self,
        out: &mut String,
        (path, module): (&[String], Option<ModId>),
        items: &[Named],
    ) {
        let facade = self.facade;
        let own = &facade.owners[facade.index];
        let title = &own.doc.title;
        let crumb = if path.is_empty() {
            String::new()
        } else {
            format!("<span class=\"dim\">{}::</span>", escape(title))
        };
        let _ = writeln!(
            out,
            "<h2 id=\"{}\">{crumb}{}</h2>",
            attr(&module_id(path)),
            escape(&module_label(title, path))
        );
        if let Some(docs) = module.and_then(|m| docs_block(&own.krate.modules[m].docs, &[])) {
            self.page(facade.index).pre(out, &docs);
        }
        let compacts: Vec<(usize, Vec<Line>)> = items
            .iter()
            .map(|named| {
                let refs = &facade.owners[named.owner].krate.refs;
                (
                    named.owner,
                    crate::glance::compact(&named.block.lines, refs),
                )
            })
            .filter(|(_, lines)| !lines.is_empty())
            .collect();
        if !compacts.is_empty() {
            out.push_str("<p class=\"view\">At a glance</p>\n<pre class=\"glance\"><code>");
            for (i, (owner, lines)) in compacts.iter().enumerate() {
                if i > 0 {
                    out.push_str("\n\n");
                }
                self.page(*owner).code_lines(out, lines);
            }
            out.push_str("</code></pre>\n<p class=\"view\">Documented</p>\n");
        }
        for named in items {
            self.item_section(out, path, named);
        }
    }

    /// Writes one item: its heading, the path users write, where its crate
    /// writes it, and its block.
    fn item_section(&self, out: &mut String, module: &[String], named: &Named) {
        let facade = self.facade;
        let title = &facade.owners[facade.index].doc.title;
        let mut crumb = format!("{title}::");
        for part in module {
            crumb.push_str(part);
            crumb.push_str("::");
        }
        let _ = writeln!(
            out,
            "<h3 id=\"{}\"><span class=\"dim\">{}</span>{}</h3>",
            attr(&named.id),
            escape(&crumb),
            escape(&named.name)
        );
        let target = &facade.owners[named.owner];
        if let Some(defined) = &named.defined {
            let loc = target.krate.real_loc(defined.loc);
            let file = format!("{}:{}", target.krate.files[loc.file].rel, loc.line);
            let file = match target.links {
                Some(links) => format!(
                    "<a class=\"src\" href=\"{}\">{}</a>",
                    attr(&links(defined.loc)),
                    escape(&file)
                ),
                None => format!("<span class=\"src\">{}</span>", escape(&file)),
            };
            let written = format!("{crumb}{}", named.name);
            let at = match &defined.id {
                Some(id) => format!(
                    "Defined in <a class=\"cross\" href=\"{}#{}\">{}</a>, ",
                    attr(&target.file),
                    attr(id),
                    escape(&defined.path)
                ),
                None if defined.path != written => {
                    format!("Defined in {}, ", escape(&defined.path))
                }
                None => String::new(),
            };
            let _ = writeln!(out, "<p class=\"defined\">{at}{file}</p>");
        }
        self.page(named.owner).pre(out, &named.block);
    }
}
