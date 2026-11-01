//! Expands the crate's own item-position `macro_rules!` invocations before
//! collection. Each expansion prints as Rust into a virtual source file
//! whose origin names the invocation, and the collector reads that file's
//! items into the invoking module. A name resolves to the crate's one
//! definition of it, else to the nearest definition in a module around the
//! invocation; textual order is ignored, so the result is deterministic.
//! An invocation that names no definition of the crate stays a mark, and
//! one that cannot expand stays a mark with a warning.

use std::collections::BTreeMap;

use syn::{Attribute, Item, ItemMacro, Path};

use crate::attrs;
use crate::macro_rules::{MacroRules, expand_items};
use crate::macros;
use crate::model::{FileId, Loc, ModId, Warning};
use crate::modtree::ModSource;
use crate::source::{Origin, SourceFile};

/// The items one invocation expands into, parsed from the virtual file
/// that holds them.
pub(crate) struct Expansion {
    pub(crate) file: FileId,
    pub(crate) items: Vec<Item>,
}

/// The expansions of a crate, keyed by the invoking file and the byte where
/// the invocation's path starts.
pub(crate) type Expansions = BTreeMap<(FileId, usize), Expansion>;

/// The crate's definitions, each parsed or with why it does not parse, by
/// name and with the module that holds it.
type Definitions = BTreeMap<String, Vec<(ModId, Result<MacroRules, String>)>>;

/// Returns the byte where an invocation's path starts in its file, the key
/// of its expansion.
pub(crate) fn key(file: &SourceFile, mac: &ItemMacro) -> usize {
    file.start(mac.mac.path.segments[0].ident.span())
}

/// Expands every item-position invocation of a walked tree that names a
/// `macro_rules!` definition of the crate, test-only ones aside. Appends a
/// virtual file per expansion to `files` and a warning per invocation that
/// does not expand, and returns the expansions.
pub(crate) fn expand(
    files: &mut Vec<SourceFile>,
    sources: &[ModSource],
    warnings: &mut Vec<Warning>,
) -> Expansions {
    let mut definitions = Definitions::new();
    for (m, source) in sources.iter().enumerate() {
        for item in &source.items {
            let Item::Macro(mac) = item else {
                continue;
            };
            if let (Some(name), false) = (&mac.ident, attrs::is_test_only(&mac.attrs)) {
                if mac.mac.path.is_ident("macro_rules") {
                    let rules = MacroRules::parse(&mac.mac.tokens).map_err(|f| f.to_string());
                    let name = name.to_string();
                    definitions.entry(name).or_default().push((m, rules));
                }
            }
        }
    }
    let parents: Vec<Option<ModId>> = sources.iter().map(|s| s.decl.map(|(p, _)| p)).collect();
    let resolver = Resolver {
        definitions: &definitions,
        parents: &parents,
    };
    let mut out = Expansions::new();
    for (m, source) in sources.iter().enumerate() {
        for item in &source.items {
            let Item::Macro(mac) = item else {
                continue;
            };
            if mac.mac.path.is_ident("macro_rules") || attrs::is_test_only(&mac.attrs) {
                continue;
            }
            let rules = match resolver.resolve(m, &mac.mac.path) {
                Ok(Some(rules)) => rules,
                Ok(None) => continue,
                Err(reason) => {
                    warnings.push(warning(&files[source.file], mac, &reason));
                    continue;
                }
            };
            let lookup = |path: &Path| resolver.resolve(m, path).ok().flatten();
            let printed = expand_items(rules, &mac.mac.tokens, &lookup)
                .map_err(|failure| failure.to_string())
                .and_then(|items| printed(items, &mac.attrs));
            let invoking = &files[source.file];
            let (text, parsed) = match printed {
                Ok(text) => {
                    let parsed = syn::parse_file(&text);
                    (text, parsed)
                }
                Err(reason) => {
                    warnings.push(warning(invoking, mac, &reason));
                    continue;
                }
            };
            let Ok(parsed) = parsed else {
                warnings.push(warning(invoking, mac, "the expansion prints as no items"));
                continue;
            };
            if parsed.items.is_empty() {
                continue;
            }
            let at = mac.mac.path.segments[0].ident.span().start();
            let origin = Origin {
                loc: Loc {
                    file: source.file,
                    line: u32::try_from(at.line).unwrap_or(0),
                },
                module: m,
                via: format!("{}!", macros::path_text(&mac.mac.path)),
            };
            let key = (source.file, key(invoking, mac));
            let mut file = SourceFile::new(invoking.rel.clone(), &text);
            file.origin = Some(origin);
            let expansion = Expansion {
                file: files.len(),
                items: parsed.items,
            };
            files.push(file);
            out.insert(key, expansion);
        }
    }
    out
}

/// Resolves macro names to the crate's definitions.
struct Resolver<'d> {
    definitions: &'d Definitions,
    /// Each module's parent, `None` for the root.
    parents: &'d [Option<ModId>],
}

impl<'d> Resolver<'d> {
    /// Returns the definition that an invocation in module `m` names: `None`
    /// for a name the crate does not define, or a path into another crate,
    /// and why not when the crate defines the name but no definition is
    /// the one, or the one does not parse.
    fn resolve(&self, m: ModId, path: &Path) -> Result<Option<&'d MacroRules>, String> {
        let first = path.segments[0].ident.to_string();
        let inside = path.segments.len() == 1 || ["crate", "self", "super"].contains(&&*first);
        let Some(last) = path.segments.last().filter(|_| inside) else {
            return Ok(None);
        };
        let Some(found) = self.definitions.get(&last.ident.to_string()) else {
            return Ok(None);
        };
        let rules = if let [(_, only)] = found.as_slice() {
            only
        } else {
            let mut at = Some(m);
            loop {
                let Some(module) = at else {
                    let n = found.len();
                    return Err(format!(
                        "the crate defines it {n} times, and no module around the invocation does"
                    ));
                };
                let here: Vec<_> = found.iter().filter(|def| def.0 == module).collect();
                match here.as_slice() {
                    [(_, one)] => break one,
                    [] => at = self.parents[module],
                    _ => return Err(format!("one module defines it {} times", here.len())),
                }
            }
        };
        rules.as_ref().map(Some).map_err(Clone::clone)
    }
}

/// Prints an expansion's items as Rust, each with the invocation's `cfg`
/// attributes first, or says why it cannot: an expansion that writes a
/// `mod` item would give the module tree a module no file walk found.
fn printed(mut items: Vec<Item>, attrs: &[Attribute]) -> Result<String, String> {
    if items.iter().any(|item| matches!(item, Item::Mod(_))) {
        return Err("the expansion writes a `mod` item".to_owned());
    }
    let cfgs: Vec<&Attribute> = attrs.iter().filter(|a| a.path().is_ident("cfg")).collect();
    for item in &mut items {
        if let Some(own) = attrs_mut(item) {
            own.splice(0..0, cfgs.iter().map(|&a| a.clone()));
        }
    }
    let file = syn::File {
        shebang: None,
        frontmatter: None,
        attrs: Vec::new(),
        items,
    };
    Ok(prettyplease::unparse(&file))
}

/// Returns an item's attributes, or `None` for an item syn keeps as tokens.
fn attrs_mut(item: &mut Item) -> Option<&mut Vec<Attribute>> {
    Some(match item {
        Item::Const(i) => &mut i.attrs,
        Item::Enum(i) => &mut i.attrs,
        Item::ExternCrate(i) => &mut i.attrs,
        Item::Fn(i) => &mut i.attrs,
        Item::ForeignMod(i) => &mut i.attrs,
        Item::Impl(i) => &mut i.attrs,
        Item::Macro(i) => &mut i.attrs,
        Item::Mod(i) => &mut i.attrs,
        Item::Static(i) => &mut i.attrs,
        Item::Struct(i) => &mut i.attrs,
        Item::Trait(i) => &mut i.attrs,
        Item::TraitAlias(i) => &mut i.attrs,
        Item::Type(i) => &mut i.attrs,
        Item::Union(i) => &mut i.attrs,
        Item::Use(i) => &mut i.attrs,
        _ => return None,
    })
}

/// Returns the warning for an invocation that does not expand.
fn warning(file: &SourceFile, mac: &ItemMacro, reason: &str) -> Warning {
    let at = mac.mac.path.segments[0].ident.span().start();
    let name = macros::path_text(&mac.mac.path);
    Warning {
        file: file.rel.clone(),
        line: u32::try_from(at.line).unwrap_or(0),
        message: format!("macro `{name}!` not expanded: {reason}"),
    }
}
