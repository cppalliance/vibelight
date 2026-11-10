//! The page of a facade: its public API and nothing else. Every path its
//! users can name, grouped by the module they name it in, each with the
//! declaration it names, pulled from the crate that defines it and shown as
//! those users see it, with its `pub` members and no `pub` in a type's
//! header, which every item of the page is. A type's private data shows
//! too, dim, so a reader can see what a value holds. Each item says where
//! the crate that defines it writes it. The sidebar lists the crate root with the
//! modules of the API indented below it, then the symbols of each module,
//! then every symbol by name.
//!
//! The page also reads what the code of those items names. A type that no
//! other item of the API names is where a reader starts, so the sidebar
//! prints it bold. An item of the run that the code names and the API does
//! not hold is listed last, under the crate a user names it through.
//!
//! And the page says who uses the API. When other crates of the run use the
//! facade, each item's line of where it is defined ends with how many of
//! them do not use the item, and the sidebar's list by module prints dim an
//! item that none of them uses. A crate uses an item that its code names,
//! and each item that the code of an item it uses names.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::html::document::Document;
use crate::html::fn_ids;
use crate::html::layout::decl::public_block;
use crate::html::layout::items::{docs_block, item_block};
use crate::html::layout::{Block, Line, Span, key, plain};
use crate::html::render_html::{Anchors, ApiIds, Page, attr, escape};
use crate::model::{Crate, ItemKind, Loc, ModId, Reach, name_order, path_order};
use crate::module_graph::Users;
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
/// every target of the run in target order, its public surface, the pages
/// of the run, the targets that use the facade, and the targets that use
/// each item of the run.
pub(crate) struct Facade<'a> {
    pub(crate) index: usize,
    pub(crate) owners: &'a [Owner<'a>],
    pub(crate) surface: &'a [Entry],
    pub(crate) run: &'a [Anchors],
    /// The other targets whose production code uses items through the
    /// facade: its users in the overview's layers.
    pub(crate) used_by: Vec<usize>,
    pub(crate) users: &'a Users,
}

/// A page as it is written: its content, the ids of the headings that no
/// other page of the run has, for the search index, and how many items of
/// a facade's API none of the facade's users uses.
pub(crate) struct Rendered {
    pub(crate) content: String,
    pub(crate) headings: Vec<String>,
    pub(crate) unused: usize,
}

/// Where a user names an item of the run that a facade's page shows: the
/// facade, the item's heading on its page, and the name it has there.
pub(crate) struct Export {
    facade: usize,
    id: String,
    name: String,
}

/// Where a user names each item of the run that some facade's page shows,
/// by the target that defines it and the item: the first facade, in the
/// order given, that shows it.
pub(crate) type Exports = BTreeMap<(usize, Reach), Export>;

/// Returns where a user names each item that the pages of `facades` show.
pub(crate) fn exports(facades: &[Facade<'_>]) -> Exports {
    let mut out = Exports::new();
    for facade in facades {
        for named in groups(facade).values().flat_map(|(_, items)| items) {
            if let Some(reach) = named.item {
                out.entry((named.owner, reach)).or_insert_with(|| Export {
                    facade: facade.index,
                    id: named.id.clone(),
                    name: format!("{}{}", named.name, named.suffix),
                });
            }
        }
    }
    out
}

/// One path of the API: its name, the block of what it names, the target
/// that defines it, and where that target writes it.
struct Named {
    name: String,
    /// What the symbol list prints after the name: `()` for a function and
    /// `!` for a macro, as the Public surface does.
    suffix: &'static str,
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
    /// How many of the facade's users do not use the item, and how many
    /// users the facade has, when it has one and the item is of the run.
    unused: Option<(usize, usize)>,
}

/// The paths of a surface by module: each module's id in the facade when
/// the surface names it, and the module's items.
type Groups = BTreeMap<Vec<String>, (Option<ModId>, Vec<Named>)>;

/// A facade's page as it renders: the facade, the heading of each item of
/// the run that the page shows, which every link to that item goes to, and
/// where a user names what the other facades show.
struct View<'a> {
    facade: &'a Facade<'a>,
    api: ApiIds,
    exports: &'a Exports,
}

/// For each item of the API, the other items of the API that its code
/// names.
type Links = BTreeMap<(usize, Reach), BTreeSet<(usize, Reach)>>;

/// What the code of the API's items names.
#[derive(Default)]
struct Names {
    /// The items of the API that another item of the API names.
    used: BTreeSet<(usize, Reach)>,
    /// The items of the run that the API names and does not hold.
    outside: BTreeSet<(usize, Reach)>,
    /// Which items of the API each item of the API names.
    links: Links,
}

/// The items the API names and does not hold, by the crate a user names
/// them through: a facade that shows them, first, or else the crate that
/// defines them. Each item has its name and the link to it, if it has one.
type Via = BTreeMap<(bool, usize), Vec<(String, Option<String>)>>;

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
/// run has, for the search index, and with how many items of the API none
/// of the facade's users uses. `exports` says where a user names what the
/// facades of the run show.
pub(crate) fn fragment(facade: &Facade<'_>, exports: &Exports, stamp: &str) -> Rendered {
    let own = &facade.owners[facade.index];
    let title = &own.doc.title;
    let mut groups = groups(facade);
    let mut api = ApiIds::new();
    for named in groups.values().flat_map(|(_, items)| items) {
        if let Some(reach) = named.item {
            (api.entry((named.owner, reach))).or_insert_with(|| named.id.clone());
        }
    }
    let names = names(facade, &groups, &api);
    let users = used_by(&facade.used_by, facade.users, &names.links);
    for named in groups.values_mut().flat_map(|(_, items)| items) {
        named.unused = unused(named, &users, facade.used_by.len());
    }
    let view = View {
        facade,
        api,
        exports,
    };
    let via = view.via(&names.outside);
    let outside: usize = via.values().map(Vec::len).sum();
    let mut out = String::from("<nav>\n<ul>\n");
    let counts: Vec<(&[String], usize)> = (groups.iter())
        .map(|(path, (_, items))| (path.as_slice(), items.len()))
        .collect();
    api_tree(&mut out, title, &counts);
    if outside > 0 {
        let _ = writeln!(
            out,
            "<li><a href=\"#named\">Named, not re-exported</a> \
             <span class=\"dim\">{outside}</span></li>"
        );
    }
    out.push_str("</ul>\n");
    module_symbols(&mut out, title, &groups, &names.used);
    symbols(&mut out, &groups, &names.used);
    out.push_str("</nav>\n<main>\n");
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
    let also = match outside {
        0 => String::new(),
        n => format!(
            " Their code names <a href=\"#named\">{} that {} does not re-export</a>.",
            counted(n, "item"),
            escape(title)
        ),
    };
    let unused = (groups.values())
        .flat_map(|(_, items)| items)
        .filter(|named| named.unused_by_all())
        .count();
    let _ = writeln!(
        out,
        "<p class=\"note\" id=\"api\">The public API of {}: {}{from}. Each shows what a user \
         of {} can name, and a type's private data, dim.{also}{}</p>",
        escape(title),
        counted(items, "item"),
        escape(title),
        escape(&usage(title, facade.used_by.len(), unused, items))
    );
    for (path, (module, items)) in &groups {
        view.module_section(&mut out, (path, *module), items);
    }
    view.named_section(&mut out, via);
    out.push_str("</main>\n");
    let headings = (groups.into_values())
        .flat_map(|(_, items)| items.into_iter().map(|named| named.id))
        .filter(|id| id.starts_with("api."))
        .collect();
    Rendered {
        content: out,
        headings,
        unused,
    }
}

/// Returns the sentences on who uses a facade named `title`: how many
/// crates of the run use it, and how many of its `items` are unused, which
/// is to say that none of those crates uses them. A facade that no crate
/// of the run uses gets no sentence.
fn usage(title: &str, users: usize, unused: usize, items: usize) -> String {
    let crates = match users {
        0 => return String::new(),
        1 => format!("1 crate here uses {title}."),
        n => format!("{n} crates here use {title}."),
    };
    let all = counted(items, "item");
    match unused {
        0 => format!(" {crates} Every item is used."),
        1 => format!(" {crates} 1 of its {all} is unused."),
        n => format!(" {crates} {n} of its {all} are unused."),
    }
}

/// Returns, for each item that a crate of `users` uses, the crates of
/// `users` that use it. A crate uses an item that its code names, as
/// `named_by` says, and each item that the code of an item it uses names,
/// as `links` says. So a crate that builds a value uses the types of the
/// value's public fields, though it never writes their names.
fn used_by(users: &[usize], named_by: &Users, links: &Links) -> Users {
    let mut out = Users::new();
    for &user in users {
        let mut reached: BTreeSet<(usize, Reach)> = (named_by.iter())
            .filter(|(_, crates)| crates.contains(&user))
            .map(|(&item, _)| item)
            .collect();
        let mut pending: Vec<(usize, Reach)> = reached.iter().copied().collect();
        while let Some(item) = pending.pop() {
            for &next in links.get(&item).into_iter().flatten() {
                if reached.insert(next) {
                    pending.push(next);
                }
            }
        }
        for item in reached {
            out.entry(item).or_default().insert(user);
        }
    }
    out
}

/// Returns how many of a facade's `total` users do not use an item, with
/// that total: none when no crate of the run uses the facade, and none for
/// an item that is not of the run. `users` holds the users of each item.
fn unused(named: &Named, users: &Users, total: usize) -> Option<(usize, usize)> {
    let reach = named.item?;
    let using = users.get(&(named.owner, reach)).map_or(0, BTreeSet::len);
    (total > 0).then(|| (total.saturating_sub(using), total))
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

/// Returns what the code of the API's items names: for each item, every
/// item of the run that a name in its block reaches, other than itself. A
/// name in a comment counts for nothing, and neither does a name in a
/// type's private data, which no user of the facade writes.
fn names(facade: &Facade<'_>, groups: &Groups, api: &ApiIds) -> Names {
    let mut names = Names::default();
    for named in groups.values().flat_map(|(_, items)| items) {
        let own = named.item.map(|reach| (named.owner, reach));
        let refs = &facade.owners[named.owner].doc.refs;
        for id in named.block.lines.iter().flat_map(public_refs) {
            let reference = &refs[id];
            let key = match (reference.decl(), reference.value(), reference.foreign) {
                (Some(decl), ..) => (named.owner, Reach::Decl(decl)),
                (None, Some(value), _) => (named.owner, Reach::Value(value)),
                (None, None, Some(to)) => (to.krate, to.reach),
                (None, None, None) => continue,
            };
            if Some(key) == own {
                continue;
            }
            if api.contains_key(&key) {
                names.used.insert(key);
                if let Some(own) = own {
                    names.links.entry(own).or_default().insert(key);
                }
            } else {
                names.outside.insert(key);
            }
        }
    }
    names
}

/// Returns the type names in a line's code that a user of the page can
/// write: each one outside a dim run, which holds a type's private data.
fn public_refs(line: &Line) -> Vec<usize> {
    let mut dim = false;
    let mut found = Vec::new();
    for span in &line.spans {
        match span {
            Span::DimOpen => dim = true,
            Span::DimClose => dim = false,
            Span::Ref(id) if !dim => found.push(*id),
            _ => {}
        }
    }
    found
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
            return Some(plain_named(name, "!", own, line));
        }
        Kind::Unlinked(path) => {
            let line = Line {
                indent: 0,
                spans: vec![key("using"), plain(&format!(" {path}"))],
                trailing: None,
            };
            return Some(plain_named(name, "", own, line));
        }
        Kind::Module(_) => return None,
    };
    let target = facade.owners.get(owner)?;
    let krate = target.krate;
    let mut suffix = "";
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
            if matches!(item.kind, ItemKind::Fn(_)) {
                suffix = "()";
            }
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
        suffix,
        block,
        owner,
        item: Some(reach),
        anchor,
        id: String::new(),
        defined: Some(defined),
        unused: None,
    })
}

/// Whether a line says how far an item reaches inside its crate, which no
/// user of a facade can see.
fn is_reach(line: &Line) -> bool {
    matches!(line.spans.first(), Some(Span::Dim(text)) if text.starts_with("reaches "))
}

fn plain_named(name: &str, suffix: &'static str, owner: usize, line: Line) -> Named {
    Named {
        name: name.to_owned(),
        suffix,
        block: Block {
            lines: vec![line],
            repeats: Vec::new(),
        },
        owner,
        item: None,
        anchor: None,
        id: String::new(),
        defined: None,
        unused: None,
    }
}

impl Named {
    /// Whether this is a type that no other item of the API names in its
    /// code, which makes it a place to start reading. Only a type can be
    /// one: no declaration names a function or a macro, and hardly one a
    /// constant, so the rule would pick every one of them and tell nothing.
    fn is_root(&self, used: &BTreeSet<(usize, Reach)>) -> bool {
        matches!(self.item, Some(reach @ Reach::Decl(_)) if !used.contains(&(self.owner, reach)))
    }

    /// Whether the facade has users and none of them uses the item.
    fn unused_by_all(&self) -> bool {
        matches!(self.unused, Some((unused, users)) if unused == users)
    }

    /// Returns what ends the item's `Defined in` line: how many of the
    /// facade's users do not use the item, when some do not.
    fn unused_note(&self) -> String {
        match self.unused {
            Some((unused, users)) if unused > 0 => {
                format!(" &middot; not used by {unused} of {users}")
            }
            _ => String::new(),
        }
    }
}

/// Writes the sidebar's entries of the API: the link to its summary, and
/// at the same level the crate root, with each module of the API indented
/// below it by its path, so that no module reads as a sibling of the
/// crate. Each has the count of its items, and `counts` gives them in page
/// order, crate root first. A crate root that holds no item of the API has
/// no section, so its name is plain text.
fn api_tree(out: &mut String, title: &str, counts: &[(&[String], usize)]) {
    out.push_str("<li><a href=\"#api\">API</a></li>\n");
    let modules = match counts {
        [] => return,
        [(path, count), modules @ ..] if path.is_empty() => {
            let _ = write!(out, "<li>{}", module_entry(title, path, *count));
            modules
        }
        modules => {
            let _ = write!(out, "<li>{}", escape(title));
            modules
        }
    };
    if modules.is_empty() {
        out.push_str("</li>\n");
    } else {
        out.push_str("\n<ul>\n");
        for (path, count) in modules {
            let _ = writeln!(out, "<li>{}</li>", module_entry(title, path, *count));
        }
        out.push_str("</ul>\n</li>\n");
    }
}

/// Returns the link to a module's section, with the count of its items.
fn module_entry(title: &str, path: &[String], count: usize) -> String {
    format!(
        "<a href=\"#{}\">{}</a> <span class=\"dim\">{count}</span>",
        attr(&module_id(path)),
        escape(&module_label(title, path))
    )
}

/// Writes the sidebar's list of the API's symbols by module, under a rule:
/// the crate root, then each module of the API by its path, each a link to
/// its section with its own symbols by name below it. Here, and only here,
/// the name of an item that none of the facade's users uses is dim. An API
/// that one module holds has no such list, which would repeat the list of
/// every symbol that follows it.
fn module_symbols(out: &mut String, title: &str, groups: &Groups, used: &BTreeSet<(usize, Reach)>) {
    if groups.len() < 2 {
        return;
    }
    out.push_str("<hr>\n<ul class=\"modules\">\n");
    for (path, (_, items)) in groups {
        let _ = writeln!(
            out,
            "<li><a href=\"#{}\">{}</a>\n<ul>",
            attr(&module_id(path)),
            escape(&module_label(title, path))
        );
        for named in items {
            let _ = writeln!(out, "<li>{}</li>", symbol_link(named, used, true));
        }
        out.push_str("</ul>\n</li>\n");
    }
    out.push_str("</ul>\n");
}

/// Returns the link to a symbol's heading: its name, a function's ending
/// in `()` and a macro's in `!`, bold for a type that no other item of the
/// API names in its code, none of `used`. With `dims`, the link is dim for
/// an item that none of the facade's users uses.
fn symbol_link(named: &Named, used: &BTreeSet<(usize, Reach)>, dims: bool) -> String {
    let (open, close) = if named.is_root(used) {
        ("<b>", "</b>")
    } else {
        ("", "")
    };
    let class = if dims && named.unused_by_all() {
        " class=\"dim\""
    } else {
        ""
    };
    format!(
        "<a{class} href=\"#{}\">{open}{}{}{close}</a>",
        attr(&named.id),
        escape(&named.name),
        named.suffix
    )
}

/// Writes the sidebar's list of every symbol of the API by name, under a
/// rule, each linking to its heading. A name that more than one module
/// holds shows its module, dim. No name is dim here: this list is for
/// finding a symbol, and the list by module says which ones are unused.
fn symbols(out: &mut String, groups: &Groups, used: &BTreeSet<(usize, Reach)>) {
    let mut all: Vec<(&[String], &Named)> = (groups.iter())
        .flat_map(|(path, (_, items))| items.iter().map(move |named| (path.as_slice(), named)))
        .collect();
    if all.is_empty() {
        return;
    }
    all.sort_by(|a, b| name_order(&a.1.name, &b.1.name).then_with(|| path_order(a.0, b.0)));
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, named) in &all {
        *seen.entry(named.name.as_str()).or_insert(0) += 1;
    }
    out.push_str("<hr>\n<ul class=\"symbols\">\n");
    for (path, named) in all {
        let module = if seen[named.name.as_str()] > 1 && !path.is_empty() {
            format!(" <span class=\"dim\">{}</span>", escape(&path.join("::")))
        } else {
            String::new()
        };
        let _ = writeln!(out, "<li>{}{module}</li>", symbol_link(named, used, false));
    }
    out.push_str("</ul>\n");
}

/// Returns the name of an item of a crate, a function's with `()`.
fn item_name(krate: &Crate, reach: Reach) -> Option<String> {
    match reach {
        Reach::Decl(decl) => krate.decls.get(decl).map(|decl| decl.name.clone()),
        Reach::Value(at) => {
            let item = krate.modules.get(at.module)?.items.get(at.item)?;
            let suffix = if matches!(item.kind, ItemKind::Fn(_)) {
                "()"
            } else {
                ""
            };
            Some(format!("{}{suffix}", item.name))
        }
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

    /// Writes one module's section: its heading, its docs, then each of
    /// its items.
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
            let _ = writeln!(
                out,
                "<p class=\"defined\">{at}{file}{}</p>",
                named.unused_note()
            );
        }
        let block = Block {
            lines: fn_ids::anchored(&named.block.lines, &fn_ids::fn_prefix(&named.id)),
            repeats: named.block.repeats.clone(),
        };
        self.page(named.owner).pre(out, &block);
    }

    /// Sorts the items the API names and does not hold by the crate a user
    /// names each through: the first facade whose page shows it, where the
    /// link goes to its heading, or else the crate that defines it, where
    /// the link goes to it on that crate's page, if the page has it.
    fn via(&self, outside: &BTreeSet<(usize, Reach)>) -> Via {
        let facade = self.facade;
        let mut via = Via::new();
        for &(owner, reach) in outside {
            if let Some(export) = self.exports.get(&(owner, reach)) {
                let file = &facade.owners[export.facade].file;
                let item = (export.name.clone(), Some(format!("{file}#{}", export.id)));
                via.entry((false, export.facade)).or_default().push(item);
                continue;
            }
            let Some(name) = item_name(facade.owners[owner].krate, reach) else {
                continue;
            };
            let link = facade.run.get(owner).and_then(|page| {
                let anchor = match reach {
                    Reach::Decl(decl) => page.ids.get(decl)?,
                    Reach::Value(value) => page.values.get(&value)?,
                };
                (!anchor.is_empty()).then(|| format!("{}#{anchor}", page.file))
            });
            via.entry((true, owner)).or_default().push((name, link));
        }
        for items in via.values_mut() {
            items.sort_by(|a, b| name_order(&a.0, &b.0).then_with(|| a.1.cmp(&b.1)));
        }
        via
    }

    /// Writes the last section: what the code of the API names and the API
    /// does not hold, under each crate a user names it through.
    fn named_section(&self, out: &mut String, via: Via) {
        if via.is_empty() {
            return;
        }
        let facade = self.facade;
        let title = escape(&facade.owners[facade.index].doc.title);
        let total: usize = via.values().map(Vec::len).sum();
        let _ = writeln!(
            out,
            "<h2 id=\"named\">Named, not re-exported <span class=\"count\">{total}</span></h2>\n\
             <p class=\"hint\">What the code above names that {title} does not re-export, \
             under the crate a user of {title} names it through.</p>"
        );
        for ((unexported, krate), items) in via {
            let target = &facade.owners[krate];
            let (class, note) = if unexported {
                ("crate", ", which no facade re-exports")
            } else {
                ("crate facade", "")
            };
            let _ = writeln!(
                out,
                "<p class=\"via\"><a class=\"{class}\" href=\"{}\">{}</a> \
                 <span class=\"num\">{}{note}</span></p>",
                attr(&target.file),
                escape(&target.doc.title),
                counted(items.len(), "item")
            );
            out.push_str("<p class=\"chips\">");
            for (name, link) in items {
                let _ = match link {
                    Some(link) => write!(
                        out,
                        "<a class=\"chip\" href=\"{}\">{}</a>",
                        attr(&link),
                        escape(&name)
                    ),
                    None => write!(out, "<span class=\"chip\">{}</span>", escape(&name)),
                };
            }
            out.push_str("</p>\n");
        }
    }
}

#[cfg(test)]
#[path = "facade_page-tests.rs"]
mod tests;
