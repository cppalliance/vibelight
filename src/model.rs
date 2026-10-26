//! The data model every step shares: modules, declarations, impls, and text.

use std::cmp::Ordering;

use crate::source::SourceFile;

/// Index into [`Crate::files`].
pub(crate) type FileId = usize;
/// Index into [`Crate::modules`]; the root module is 0.
pub(crate) type ModId = usize;
/// Index into [`Crate::decls`].
pub(crate) type DeclId = usize;
/// Index into [`Crate::refs`].
pub(crate) type RefId = usize;

/// A file and a 1-based line, the target of a source link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Loc {
    pub(crate) file: FileId,
    pub(crate) line: u32,
}

pub(crate) use crate::model_impl::{Impl, ImplLoc, ImplNotes, Special, SpecialKind};
pub(crate) use crate::text::{CrossRef, Reach, Seg, Text, TypeRef, ValueRef, Where, Written};

/// One comment printed above a node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Note {
    /// A doc comment, flattened, one entry per line.
    Doc(Vec<Text>),
    /// An ordinary comment as written, one entry per line, re-indented
    /// relative to its first line.
    Plain(Vec<String>),
}

/// What every shown node carries besides its own text.
#[derive(Clone, Debug, Default)]
pub(crate) struct Meta {
    /// Comments above the node, in written order.
    pub(crate) notes: Vec<Note>,
    /// An ordinary comment at the end of the node's last line.
    pub(crate) trailing: Option<String>,
    /// One entry per `cfg` that is not test-only, sorted.
    pub(crate) cfgs: Vec<String>,
    /// The attributes that shape an API or its wire format, as written,
    /// sorted; each prints as a `[[...]]` line after the `cfg` lines.
    pub(crate) attrs: Vec<String>,
    /// Macro marks printed directly above the node inside a block.
    pub(crate) marks: Vec<String>,
}

/// How far a node is visible, which picks the section it prints in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Vis {
    /// `pub`.
    Public,
    /// `pub(crate)`, `pub(super)`, or `pub(in path)`.
    Internal,
    /// `pub(self)` or no keyword.
    Private,
}

/// How a function takes its object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Receiver {
    /// No receiver.
    None,
    /// `&self`, dropped with nothing in its place.
    Shared,
    /// `&mut self`, dropped.
    Mutable,
    /// Any other receiver, kept as the first parameter.
    Value,
}

/// A function signature split at the places the long-line rule breaks.
#[derive(Clone, Debug)]
pub(crate) struct Sig {
    /// Qualifiers in print order, such as `constexpr` or `extern "C"`.
    pub(crate) quals: Vec<String>,
    /// The name as written.
    pub(crate) name: String,
    /// The generic parameter list, `<...>`, or empty.
    pub(crate) generics: Text,
    pub(crate) receiver: Receiver,
    /// The parameters, a kept receiver first.
    pub(crate) params: Vec<Param>,
    /// The return type, without the arrow.
    pub(crate) ret: Option<Text>,
    pub(crate) where_clause: Option<Where>,
}

/// One parameter or tuple field and the comments above it.
#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub(crate) text: Text,
    pub(crate) meta: Meta,
}

/// One entry inside a type's block.
#[derive(Clone, Debug)]
pub(crate) struct Member {
    /// The name members are sorted by.
    pub(crate) name: String,
    pub(crate) vis: Vis,
    pub(crate) kind: MemberKind,
    pub(crate) meta: Meta,
    pub(crate) loc: Loc,
}

#[derive(Clone, Debug)]
pub(crate) enum MemberKind {
    /// An associated type: the text after `using`, such as `Item: Clone`.
    Type(Text),
    /// An associated constant.
    Const {
        ty: Text,
        value: Option<String>,
    },
    /// A named field and its type.
    Field(Text),
    Fn(Sig),
    /// A macro invocation inside an impl or trait body, such as `name!`.
    Mark(String),
    /// A construct syn reports but the collector does not know.
    Unsupported(String),
}

impl MemberKind {
    /// Tells whether a member is a static function: one with no receiver. A
    /// function that takes `self` by value is a member function that keeps
    /// `self` as its first parameter.
    pub(crate) fn is_static_fn(&self) -> bool {
        matches!(self, MemberKind::Fn(sig) if sig.receiver == Receiver::None)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclKind {
    Struct,
    Enum,
    Union,
    Trait,
    TraitAlias,
    Alias,
}

/// The body of a declaration.
#[derive(Clone, Debug)]
pub(crate) enum Shape {
    /// A struct or union with named fields, which live in the members.
    Named,
    /// A tuple struct's fields.
    Tuple(Vec<Param>),
    Unit,
    Enum(Vec<Variant>),
    /// A trait: the keyword prefix (`unsafe `, `auto `) and supertraits.
    Trait {
        prefix: String,
        supertraits: Vec<Base>,
    },
    /// A type alias target, or a trait alias's bounds.
    Alias(Text),
}

#[derive(Clone, Debug)]
pub(crate) struct Variant {
    pub(crate) name: String,
    pub(crate) fields: VariantFields,
    /// The discriminant expression after `=`.
    pub(crate) discriminant: Option<String>,
    pub(crate) meta: Meta,
    pub(crate) loc: Loc,
}

#[derive(Clone, Debug)]
pub(crate) enum VariantFields {
    Unit,
    Tuple(Vec<Param>),
    /// Named fields: each `name: Type` with its comments.
    Named(Vec<Param>),
}

/// A trait in a type's base list, or the trait of an impl.
#[derive(Clone, Debug, Default)]
pub(crate) struct Base {
    /// The entry as printed, such as `unsafe Send` or `From<String>`.
    pub(crate) text: Text,
    /// The trait's own name, which base lists sort by.
    pub(crate) name: String,
    /// The reference to the trait's path, which routing classifies.
    pub(crate) path_ref: Option<RefId>,
    /// The type argument of a one-argument trait such as `From<T>`.
    pub(crate) arg: Option<Text>,
    /// Whether the impl has its own generic parameters or a `where`
    /// clause, which keeps it out of the special members.
    pub(crate) generic: bool,
    /// Whether routing put the entry in the long list.
    pub(crate) long: bool,
}

/// A struct, enum, union, trait, or type alias.
#[derive(Clone, Debug)]
pub(crate) struct Decl {
    pub(crate) kind: DeclKind,
    pub(crate) name: String,
    pub(crate) module: ModId,
    /// The definition's line.
    pub(crate) loc: Loc,
    pub(crate) generics: Text,
    pub(crate) where_clause: Option<Where>,
    pub(crate) shape: Shape,
    /// Fields, trait items, and the members of attached inherent impls.
    pub(crate) members: Vec<Member>,
    /// The type's own comments and `cfg` lines.
    pub(crate) meta: Meta,
    /// Traits from every derive attribute, merged and sorted; after routing,
    /// only the ones the `derive:` mark still shows.
    pub(crate) derives: Vec<String>,
    /// A reference for each name in `derives` as collected, for routing.
    pub(crate) derive_refs: Vec<RefId>,
    /// The special members routing builds from standard traits.
    pub(crate) special: Vec<Special>,
    /// Attribute macros on the type, sorted.
    pub(crate) attributes: Vec<String>,
    /// Macro marks printed at the end of the block.
    pub(crate) end_marks: Vec<String>,
    /// Comments with nothing after them in the body.
    pub(crate) dangling: Vec<Note>,
    /// Base-list entries from trait impls.
    pub(crate) bases: Vec<Base>,
    /// Comments above impl lines, one group per impl, in print order.
    pub(crate) impl_notes: Vec<ImplNotes>,
    /// Impl blocks that add members or base-list entries.
    pub(crate) impl_locs: Vec<ImplLoc>,
    /// The declaration's own visibility, which the public surface reads and
    /// a line above the header shows when it is not public.
    pub(crate) vis: Vis,
    /// Whether it is test support (`test_support::mark`), which keeps it
    /// out of the levels.
    pub(crate) test_support: bool,
}

/// One entry in a module's trailing block.
#[derive(Clone, Debug)]
pub(crate) struct Item {
    /// The name the entry sorts by.
    pub(crate) name: String,
    pub(crate) vis: Vis,
    pub(crate) kind: ItemKind,
    pub(crate) meta: Meta,
    pub(crate) loc: Loc,
}

#[derive(Clone, Debug)]
pub(crate) enum ItemKind {
    /// A re-export, the text after `using`: `path::Item`, `Alias =
    /// path::Item`, or `path::*`. Names
    /// from one commented `use` share a group, keyed by where the `use`
    /// starts, and print together.
    Reexport {
        text: Text,
        group: Option<usize>,
    },
    /// A macro definition (`macro_rules! name`) or invocation (`name!`).
    Mark(String),
    Const {
        ty: Text,
        value: Option<String>,
    },
    Static {
        mutable: bool,
        ty: Text,
        value: Option<String>,
    },
    Fn(Sig),
    /// An item of an extern block with the given ABI string, such as `"C"`.
    Foreign {
        abi: String,
        item: Foreign,
    },
    /// A construct syn reports but the collector does not know, or a
    /// module file that is missing or cannot be parsed.
    Unsupported(String),
}

/// What an extern block declares.
#[derive(Clone, Debug)]
pub(crate) enum Foreign {
    Fn(Sig),
    Static {
        mutable: bool,
        ty: Text,
    },
    /// An opaque type.
    Type,
}

/// A name a `use` item binds, for name resolution.
#[derive(Clone, Debug)]
pub(crate) struct Use {
    /// The bound name, or `None` for a glob.
    pub(crate) name: Option<String>,
    /// The path the name comes from; for a glob, the module path.
    pub(crate) path: Vec<String>,
    /// True when the path starts with `::`.
    pub(crate) global: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Module {
    /// The path below the crate root; empty for the root.
    pub(crate) path: Vec<String>,
    /// The file holding the module's items.
    pub(crate) file: FileId,
    /// The `mod` line of an inline module.
    pub(crate) inline: Option<Loc>,
    /// Outer docs and comments on the `mod` line, then inner docs.
    pub(crate) docs: Vec<Note>,
    pub(crate) cfgs: Vec<String>,
    /// The trailing block's entries.
    pub(crate) items: Vec<Item>,
    /// Every name a `use` binds here.
    pub(crate) uses: Vec<Use>,
    /// Comments with nothing after them in the module.
    pub(crate) dangling: Vec<Note>,
    /// The visibility on the `mod` line; the root module is public.
    pub(crate) vis: Vis,
    /// Whether it, or a module around it, is test support.
    pub(crate) test_support: bool,
}

/// The kind of a macro, in kill list order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MacroKind {
    Definition,
    Invocation,
    Derive,
    Attribute,
}

/// Where a macro mark is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Holder {
    Decl(DeclId),
    Module(ModId),
    Impl(usize),
}

/// One kill list entry.
#[derive(Clone, Debug)]
pub(crate) struct MacroUse {
    pub(crate) kind: MacroKind,
    /// The mark text, such as `macro_rules! name`, `name!`, or
    /// `derive: Clone, Debug`.
    pub(crate) text: String,
    pub(crate) holder: Holder,
    pub(crate) loc: Loc,
}

/// A problem reported on stderr.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Warning {
    /// The crate-relative file, with forward slashes.
    pub(crate) file: String,
    pub(crate) line: u32,
    pub(crate) message: String,
}

/// A type name a function body writes. It keeps the type out of
/// Unconnected (rule 26) and names nothing else.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BodyName {
    pub(crate) name: RefId,
    pub(crate) owner: BodyOwner,
}

/// What a function body belongs to. A type that owns a body does not count
/// as named by it.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BodyOwner {
    /// A free function.
    Module,
    /// A method of this impl, which every type its header names owns, such
    /// as `Engine` in `impl Clone for Box<Engine>`.
    Impl(usize),
    /// A default method of this trait.
    Trait(DeclId),
}

/// Everything collected from one target.
#[derive(Debug)]
pub(crate) struct Crate {
    /// The crate name ids use.
    pub(crate) name: String,
    /// The target name the H1 shows.
    pub(crate) title: String,
    pub(crate) files: Vec<SourceFile>,
    pub(crate) modules: Vec<Module>,
    pub(crate) decls: Vec<Decl>,
    pub(crate) impls: Vec<Impl>,
    pub(crate) refs: Vec<TypeRef>,
    pub(crate) body_names: Vec<BodyName>,
    pub(crate) macros: Vec<MacroUse>,
    /// Each `#[macro_export]` macro's name and the module that defines it;
    /// such a macro is named at the crate root.
    pub(crate) exported_macros: Vec<(String, ModId)>,
    pub(crate) warnings: Vec<Warning>,
}

impl Crate {
    /// Returns a declaration's path below the root, such as
    /// `["prompt", "Prompt"]`.
    pub(crate) fn decl_path(&self, id: DeclId) -> Vec<String> {
        let decl = &self.decls[id];
        let mut path = self.modules[decl.module].path.clone();
        path.push(decl.name.clone());
        path
    }
}

/// Compares names as rule 9 sorts members: an `r#` prefix ignored, then
/// case ignored, then exact spelling.
pub(crate) fn name_order(a: &str, b: &str) -> Ordering {
    let strip = |s: &'_ str| -> String { s.strip_prefix("r#").unwrap_or(s).to_owned() };
    let (x, y) = (strip(a), strip(b));
    x.to_lowercase()
        .cmp(&y.to_lowercase())
        .then_with(|| x.cmp(&y))
        .then_with(|| a.cmp(b))
}

/// Compares paths segment by segment with [`name_order`]; a prefix sorts
/// first.
pub(crate) fn path_order(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let order = name_order(x, y);
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_sort_without_raw_prefix_and_case() {
        let mut names = vec!["title", "Entry", "r#type", "entry", "h1_blocks"];
        names.sort_by(|a, b| name_order(a, b));
        assert_eq!(names, ["Entry", "entry", "h1_blocks", "title", "r#type"]);
    }

    #[test]
    fn paths_sort_by_segment_with_prefixes_first() {
        let p = |s: &str| -> Vec<String> {
            s.split("::")
                .filter(|x| !x.is_empty())
                .map(str::to_owned)
                .collect()
        };
        let mut paths = vec![p("b::A"), p("a::b"), p(""), p("a"), p("A::z")];
        paths.sort_by(|a, b| path_order(a, b));
        assert_eq!(paths, [p(""), p("A::z"), p("a"), p("a::b"), p("b::A")]);
    }
}
