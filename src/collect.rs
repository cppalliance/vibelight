//! Collects declarations, impls, and module items from the parsed tree.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::Path;

use proc_macro2::Span;
use syn::{Attribute, Generics, Item, ItemMacro, ItemMod};

use crate::attrs;
use crate::comments::{Attached, NoteCx, Pool, attach};
use crate::macros;
use crate::model::{
    BodyName, Crate, Decl, FileId, Holder, Item as ModItem, ItemKind, Loc, MacroKind, MacroUse,
    Meta, ModId, Module, Note, TypeRef, Vis, Warning,
};
use crate::model_body::{AttrPath, Body};
use crate::modtree::{ModSource, Tree};
use crate::source::{Comment, SourceFile};
use crate::typerefs::{Slicer, range_of, strip_raw};

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
    exported_macros: Vec<(String, ModId)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) bodies: Vec<Body>,
    pub(crate) test_ranges: Vec<(FileId, Range<usize>)>,
    pub(crate) attr_paths: Vec<AttrPath>,
    /// The notes, `cfg`s, and visibility of each module's `mod` line.
    mod_lines: BTreeMap<ModId, (Vec<Note>, Vec<String>, Vis)>,
    /// Each child module, keyed by its parent and the `mod` item's index.
    children: BTreeMap<(ModId, usize), ModId>,
    /// Module-level invocations and the identifiers their tokens hold.
    invocations: Vec<(ModId, String, BTreeSet<String>)>,
}

/// Collects every module of a walked tree into a crate model.
pub(crate) fn collect(tree: Tree, crate_dir: &Path, name: &str, title: &str) -> Crate {
    let Tree {
        files,
        modules: sources,
        warnings,
        test_files,
    } = tree;
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
            test_support: false,
            raw_doc: None,
            lints: Vec::new(),
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
        let Attached {
            leading,
            trailing,
            dangling,
        } = attach(
            &mut self.pools[source.file],
            file,
            inner_end..source.scope.end,
            &children,
        );
        for (index, ((item, lead), trail)) in
            source.items.iter().zip(leading).zip(trailing).enumerate()
        {
            self.item(&here, index, item, lead, &trail);
        }
        let dangling = self.notes(&here, &[], dangling);
        self.modules[id].dangling = dangling;
    }

    fn item(
        &mut self,
        here: &Here,
        index: usize,
        item: &Item,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
        match item {
            Item::Const(c) => self.item_const(here, c, lead, trail),
            Item::Enum(e) => self.item_enum(here, e, lead, trail),
            Item::ExternCrate(_) => {}
            Item::Fn(f) => self.item_fn(here, f, lead, trail),
            Item::ForeignMod(f) => self.item_foreign(here, f, lead, trail),
            Item::Impl(i) => self.item_impl(here, i, lead, trail),
            Item::Macro(m) => self.item_macro(here, m, lead, trail),
            Item::Mod(m) => self.item_mod(here, index, m, lead, trail),
            Item::Static(s) => self.item_static(here, s, lead, trail),
            Item::Struct(s) => self.item_struct(here, s, lead, trail),
            Item::Trait(t) => self.item_trait(here, t, lead, trail),
            Item::TraitAlias(t) => self.item_trait_alias(here, t, lead, trail),
            Item::Type(t) => self.item_type(here, t, lead, trail),
            Item::Union(u) => self.item_union(here, u, lead, trail),
            Item::Use(u) => self.item_use(here, u, lead, trail),
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

    fn item_macro(&mut self, here: &Here, m: &ItemMacro, lead: Vec<Comment>, trail: &[Comment]) {
        if self.test_only(here, &m.attrs, m) {
            return;
        }
        self.discard(
            here,
            macros::delimiter_range(&m.mac.delimiter, &self.files[here.file]),
        );
        let notes = self.node_notes(here, &m.attrs, lead, range_of(m, &self.files[here.file]));
        let name = macros::path_text(&m.mac.path);
        let (text, sort, kind) = match &m.ident {
            Some(ident) if m.mac.path.is_ident("macro_rules") => {
                let ident = ident.to_string();
                if macros::is_exported(&m.attrs) {
                    self.exported_macros.push((ident.clone(), here.module));
                }
                (
                    format!("macro_rules! {ident}"),
                    ident,
                    MacroKind::Definition,
                )
            }
            _ => (format!("{name}!"), name, MacroKind::Invocation),
        };
        let loc = loc(here, m.mac.path.segments[0].ident.span());
        if kind == MacroKind::Invocation {
            self.invocations
                .push((here.module, text.clone(), macros::idents(&m.mac.tokens)));
        }
        self.macros.push(MacroUse {
            kind,
            text: text.clone(),
            holder: Holder::Module(here.module),
            loc,
        });
        let (raw_doc, lints) = self.attr_facts(here, &m.attrs);
        let meta = Meta {
            notes,
            trailing: self.trailing(here, trail),
            cfgs: attrs::cfgs(&m.attrs, &self.files[here.file]),
            attrs: attrs::api(&m.attrs, &self.files[here.file]),
            marks: Vec::new(),
            vis: String::new(),
            raw_doc,
            lints,
        };
        self.modules[here.module].items.push(ModItem {
            name: sort,
            vis: Vis::Public,
            kind: ItemKind::Mark(text),
            meta,
            loc,
        });
    }

    fn item_mod(
        &mut self,
        here: &Here,
        index: usize,
        m: &ItemMod,
        lead: Vec<Comment>,
        trail: &[Comment],
    ) {
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
        let mut notes = self.node_notes(here, &outer, lead, range.start..head_end);
        if let Some(text) = self.trailing(here, trail) {
            notes.push(Note::Plain(vec![text]));
        }
        let cfgs = attrs::cfgs(&m.attrs, file);
        self.mod_lines
            .insert(child, (notes, cfgs, attrs::vis(&m.vis)));
    }

    /// Adds a `// UNSUPPORTED` entry to the module's trailing block.
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

    /// Gives each declaration named by a module-level invocation in its
    /// module a mark at the end of its block.
    fn finish_invocations(&mut self) {
        for (module, text, idents) in std::mem::take(&mut self.invocations) {
            for decl in self.decls.iter_mut().filter(|d| d.module == module) {
                if idents.contains(&strip_raw(&decl.name)) && !decl.end_marks.contains(&text) {
                    decl.end_marks.push(text.clone());
                }
            }
        }
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

    pub(crate) fn warn(&mut self, loc: Loc, message: String) {
        self.warnings.push(Warning {
            file: self.files[loc.file].rel.clone(),
            line: loc.line,
            message,
        });
    }

    /// Records attribute macros on a node in the kill list and returns its
    /// mark, `attribute: a, b`, if it has any.
    pub(crate) fn attribute_mark(
        &mut self,
        attrs: &[Attribute],
        helpers: bool,
        holder: Holder,
        loc: Loc,
    ) -> Option<String> {
        let names = macros::attribute_macros(attrs, helpers);
        if names.is_empty() {
            return None;
        }
        let text = format!("attribute: {}", names.join(", "));
        self.macros.push(MacroUse {
            kind: MacroKind::Attribute,
            text: text.clone(),
            holder,
            loc,
        });
        Some(text)
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
pub(crate) fn generic_names(generics: &Generics) -> Vec<String> {
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
