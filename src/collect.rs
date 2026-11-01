//! Collects declarations, impls, and module items from the parsed tree.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::Path;

use proc_macro2::Span;
use syn::{Attribute, Generics, Item, ItemMod};

use crate::comments::{NoteCx, Placed, Pool, place_comments};
use crate::macro_expand::Expansions;
use crate::model::{
    BodyName, Crate, Decl, FileId, Item as ModItem, ItemKind, Loc, MacroUse, Meta, ModId, Module,
    Note, TypeRef, Vis, Warning,
};
use crate::model_body::{AttrPath, Body};
use crate::modtree::{ModSource, Tree};
use crate::source::{Comment, SourceFile, real};
use crate::typerefs::{Slicer, range_of, strip_raw};
use crate::{attrs, macro_expand, macros};

/// Where the node being collected lives.
#[derive(Clone)]
pub(crate) struct Here {
    pub(crate) module: ModId,
    pub(crate) file: FileId,
    /// Generic parameter names in scope.
    pub(crate) generics: Vec<String>,
}

impl Here {
    /// Returns this place with more generic parameters in scope.
    pub(crate) fn with(&self, generics: &Generics) -> Here {
        let mut here = self.clone();
        here.generics.extend(generic_names(generics));
        here
    }
}

/// Collects one target.
pub(crate) struct Collector<'a> {
    pub(crate) files: &'a [SourceFile],
    pub(crate) crate_dir: &'a Path,
    pub(crate) pools: Vec<Pool>,
    pub(crate) refs: Vec<TypeRef>,
    pub(crate) body_names: Vec<BodyName>,
    pub(crate) modules: Vec<Module>,
    pub(crate) decls: Vec<Decl>,
    pub(crate) impls: Vec<crate::model::Impl>,
    pub(crate) macros: Vec<MacroUse>,
    pub(crate) exported_macros: Vec<(String, ModId)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) bodies: Vec<Body>,
    pub(crate) test_ranges: Vec<(FileId, Range<usize>)>,
    pub(crate) attr_paths: Vec<AttrPath>,
    /// The notes, `cfg`s, and visibility of each module's `mod` line.
    mod_lines: BTreeMap<ModId, (Vec<Note>, Vec<String>, Vis)>,
    /// Each child module, keyed by its parent and the `mod` item's index.
    children: BTreeMap<(ModId, usize), ModId>,
    /// Module-level invocations and the identifiers their tokens hold.
    pub(crate) invocations: Vec<(ModId, String, BTreeSet<String>)>,
    /// The items the crate's own `macro_rules!` invocations expand into.
    pub(crate) expansions: &'a Expansions,
}

/// Collects every module of a walked tree into a crate model, with the
/// items each of the crate's own item-position `macro_rules!` invocations
/// expands into collected from a virtual file into the invoking module.
pub(crate) fn collect(tree: Tree, crate_dir: &Path, name: &str, title: &str) -> Crate {
    let Tree {
        mut files,
        modules: sources,
        mut warnings,
        test_files,
    } = tree;
    let expansions = macro_expand::expand(&mut files, &sources, &mut warnings);
    let mut collector = Collector {
        files: &files,
        crate_dir,
        pools: files.iter().map(Pool::new).collect(),
        refs: Vec::new(),
        body_names: Vec::new(),
        modules: Vec::new(),
        decls: Vec::new(),
        impls: Vec::new(),
        macros: Vec::new(),
        exported_macros: Vec::new(),
        warnings,
        bodies: Vec::new(),
        test_ranges: Vec::new(),
        attr_paths: Vec::new(),
        mod_lines: BTreeMap::new(),
        children: BTreeMap::new(),
        invocations: Vec::new(),
        expansions: &expansions,
    };
    for (id, source) in sources.iter().enumerate() {
        collector.modules.push(Module {
            path: source.path.clone(),
            file: source.file,
            inline: source.inline_line.map(|line| Loc {
                file: source.file,
                line,
            }),
            docs: Vec::new(),
            cfgs: Vec::new(),
            items: Vec::new(),
            uses: Vec::new(),
            dangling: Vec::new(),
            vis: Vis::Public,
            access: None,
            parent: source.decl.map(|(parent, _)| parent),
            test_support: false,
            raw_doc: None,
            lints: Vec::new(),
            scope: source.scope.clone(),
        });
        if let Some(decl) = source.decl {
            collector.children.insert(decl, id);
        }
    }
    for (id, source) in sources.iter().enumerate() {
        collector.module(id, source);
    }
    collector.finish_invocations();
    collector.module_facts(&sources);
    let Collector {
        refs,
        body_names,
        modules,
        decls,
        impls,
        macros,
        exported_macros,
        warnings,
        bodies,
        mut test_ranges,
        attr_paths,
        ..
    } = collector;
    test_ranges.sort_by_key(|(file, range)| (*file, range.start, range.end));
    Crate {
        name: name.to_owned(),
        title: title.to_owned(),
        files,
        modules,
        decls,
        impls,
        refs,
        body_names,
        macros,
        exported_macros,
        warnings,
        bodies,
        test_ranges,
        test_files,
        attr_paths,
    }
}

impl Collector<'_> {
    fn module(&mut self, id: ModId, source: &ModSource) {
        let here = Here {
            module: id,
            file: source.file,
            generics: Vec::new(),
        };
        let (mut docs, cfgs, vis) =
            self.mod_lines
                .remove(&id)
                .unwrap_or((Vec::new(), Vec::new(), Vis::Public));
        self.modules[id].cfgs = cfgs;
        self.modules[id].vis = vis;
        if let Some(problem) = &source.problem {
            self.modules[id].docs = docs;
            let loc = Loc {
                file: source.file,
                line: source.inline_line.unwrap_or(1),
            };
            self.unsupported_item(&here, problem, loc, false);
            return;
        }
        let files = self.files;
        let file = &files[source.file];
        let inner_end = source
            .inner
            .iter()
            .map(|a| range_of(a, file).end)
            .fold(source.scope.start, usize::max);
        let comments = self.pools[source.file].take(source.scope.start..inner_end);
        docs.extend(self.notes(&here, &source.inner, comments));
        self.modules[id].docs = docs;

        let children: Vec<Range<usize>> = source.items.iter().map(|i| range_of(i, file)).collect();
        let (placed, dangling) = place_comments(
            &mut self.pools[source.file],
            file,
            inner_end..source.scope.end,
            &children,
        );
        for (index, (item, placed)) in source.items.iter().zip(placed).enumerate() {
            self.item(&here, index, item, placed);
        }
        let dangling = self.notes(&here, &[], dangling);
        self.modules[id].dangling = dangling;
    }

    /// Collects one module item, `index` among its module's items.
    pub(crate) fn item(&mut self, here: &Here, index: usize, item: &Item, placed: Placed) {
        match item {
            Item::Const(c) => self.item_const(here, c, placed),
            Item::Enum(e) => self.item_enum(here, e, placed),
            Item::ExternCrate(_) => {}
            Item::Fn(f) => self.item_fn(here, f, placed),
            Item::ForeignMod(f) => self.item_foreign(here, f, placed),
            Item::Impl(i) => self.item_impl(here, i, placed),
            Item::Macro(m) => self.item_macro(here, m, placed),
            Item::Mod(m) => self.item_mod(here, index, m, placed),
            Item::Static(s) => self.item_static(here, s, placed),
            Item::Struct(s) => self.item_struct(here, s, placed),
            Item::Trait(t) => self.item_trait(here, t, placed),
            Item::TraitAlias(t) => self.item_trait_alias(here, t, placed),
            Item::Type(t) => self.item_type(here, t, placed),
            Item::Union(u) => self.item_union(here, u, placed),
            Item::Use(u) => self.item_use(here, u, placed),
            Item::Verbatim(tokens) => {
                let loc = loc(here, macros::tokens_span(tokens));
                self.unsupported_item(here, "verbatim item", loc, true);
            }
            _ => {
                let loc = Loc {
                    file: here.file,
                    line: self.files[here.file]
                        .line_of(range_of(item, &self.files[here.file]).start),
                };
                self.unsupported_item(here, "unknown item", loc, true);
            }
        }
    }

    fn item_mod(&mut self, here: &Here, index: usize, m: &ItemMod, placed: Placed) {
        if self.test_only(here, &m.attrs, m) {
            return;
        }
        let Some(&child) = self.children.get(&(here.module, index)) else {
            return;
        };
        let files = self.files;
        let file = &files[here.file];
        let range = range_of(m, file);
        let head_end = m
            .content
            .as_ref()
            .map_or(range.end, |(brace, _)| file.start(brace.span.open()));
        let (outer, _) = attrs::split_inner(&m.attrs);
        let mut notes = self.node_notes(here, &outer, placed.leading, range.start..head_end);
        if let Some(text) = self.trailing(here, &placed.trailing) {
            notes.push(Note::Plain(vec![text]));
        }
        let cfgs = attrs::cfgs(&m.attrs, file);
        let vis = self.vis(here, &m.vis);
        self.mod_lines.insert(child, (notes, cfgs, vis));
    }

    /// Returns a visibility written in `here`'s module, with the module a
    /// `pub(in path)` names.
    pub(crate) fn vis(&self, here: &Here, vis: &syn::Visibility) -> Vis {
        attrs::vis(vis, |path| self.module_named(here.module, path))
    }

    /// Returns the module that the path of a `pub(in path)` written in
    /// module `from` names: a path from `crate`, `self`, or `super`.
    fn module_named(&self, from: ModId, path: &syn::Path) -> Option<ModId> {
        let mut target: Vec<String> = Vec::new();
        for (k, segment) in path.segments.iter().enumerate() {
            let word = segment.ident.to_string();
            match word.as_str() {
                "crate" if k == 0 => {}
                "self" if k == 0 => target.clone_from(&self.modules[from].path),
                "super" => {
                    if k == 0 {
                        target.clone_from(&self.modules[from].path);
                    }
                    target.pop()?;
                }
                _ => target.push(word),
            }
        }
        self.modules.iter().position(|m| m.path == target)
    }

    /// Adds a `// UNSUPPORTED` entry to the module's trailing block. It also
    /// warns when `warn` is set; a module whose `problem` the tree walk
    /// already warned about passes `false`.
    pub(crate) fn unsupported_item(&mut self, here: &Here, kind: &str, loc: Loc, warn: bool) {
        if warn {
            self.warn(loc, format!("unsupported {kind}"));
        }
        self.modules[here.module].items.push(ModItem {
            name: String::new(),
            vis: Vis::Public,
            kind: ItemKind::Unsupported(kind.to_owned()),
            meta: Meta::default(),
            loc,
        });
    }

    /// Builds notes from attributes and comments, sorted into written order.
    pub(crate) fn notes(
        &mut self,
        here: &Here,
        attrs: &[Attribute],
        comments: Vec<Comment>,
    ) -> Vec<Note> {
        let files = self.files;
        let mut cx = NoteCx {
            file: &files[here.file],
            refs: &mut self.refs,
            module: here.module,
            generics: &here.generics,
            crate_dir: self.crate_dir,
            warnings: &mut self.warnings,
        };
        cx.notes(attrs, comments)
    }

    /// Builds a node's notes: its leading comments, its doc attributes, and
    /// every comment still unused inside `range`.
    pub(crate) fn node_notes(
        &mut self,
        here: &Here,
        attrs: &[Attribute],
        mut lead: Vec<Comment>,
        range: Range<usize>,
    ) -> Vec<Note> {
        lead.extend(self.pools[here.file].take(range));
        self.notes(here, attrs, lead)
    }

    /// Returns a trailing comment's text.
    pub(crate) fn trailing(&self, here: &Here, comments: &[Comment]) -> Option<String> {
        let file = &self.files[here.file];
        if comments.is_empty() {
            return None;
        }
        let parts: Vec<&str> = comments
            .iter()
            .map(|c| file.text[c.range.clone()].trim_end())
            .collect();
        Some(parts.join(" "))
    }

    /// Drops every comment inside `range`, such as a function body's.
    pub(crate) fn discard(&mut self, here: &Here, range: Range<usize>) {
        self.pools[here.file].take(range);
    }

    pub(crate) fn slicer<'s>(&'s mut self, here: &'s Here) -> Slicer<'s> {
        let files = self.files;
        Slicer {
            file: &files[here.file],
            file_id: here.file,
            refs: &mut self.refs,
            module: here.module,
            generics: &here.generics,
        }
    }

    /// Warns about a location, named where it is written in a real file.
    pub(crate) fn warn(&mut self, loc: Loc, message: String) {
        let loc = real(self.files, loc);
        self.warnings.push(Warning {
            file: self.files[loc.file].rel.clone(),
            line: loc.line,
            message,
        });
    }
}

/// Returns the file and line where a span starts.
pub(crate) fn loc(here: &Here, span: Span) -> Loc {
    Loc {
        file: here.file,
        line: u32::try_from(span.start().line).unwrap_or(0),
    }
}

/// Returns the names of type and const generic parameters.
fn generic_names(generics: &Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(strip_raw(&t.ident.to_string())),
            syn::GenericParam::Const(c) => Some(strip_raw(&c.ident.to_string())),
            syn::GenericParam::Lifetime(_) => None,
        })
        .collect()
}
