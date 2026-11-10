//! The catalog's records: one per module, declaration, member, and module
//! item, and one per field or variant that has comments of its own. Each
//! signature and shape is one line of Rust; docs are `doc` records.
//!
//! The use facts mean the same on every record that has them: `named_by`
//! lists the declarations and modules of the target whose signatures name
//! the item, `reexported_by` the modules that re-export it, `crates` the
//! other targets of the run whose code is observed to use it, and `tests`
//! the kinds of test, bench, example, build, and doc test code observed to
//! use it. The body facts are `body`, `ops`, and `comments`. `eff` spells
//! how far a declaration or a module item reaches, from its own module,
//! only when a re-export makes that differ from its module's reach
//! narrowed by its own visibility. `via` names the macro, such as
//! `events!`, whose invocation wrote a top-level record's item; the record's
//! location is the invocation's.

use std::collections::BTreeMap;

#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;

use crate::md::slop::schema::{Loc, absent};

/// A module, first among its catalog records. The counts take in what the
/// module declares itself, extern items by their kind, and `mix` counts
/// the same items by their written visibility.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ModRecord {
    pub(crate) path: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    /// Its structs, enums, unions, traits, and aliases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) types: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) functions: Option<usize>,
    /// Its constants and statics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) values: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reexports: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mix: Option<BTreeMap<String, usize>>,
    /// How many modules use its items, as their `deps` records show.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fan_in: Option<usize>,
    /// How many modules' items it uses, as its `deps` record shows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fan_out: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
    /// The comments with nothing after them in the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) dangling: Option<String>,
}

/// A struct, an enum, or a union, with its whole shape on one line.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct TypeRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) derives: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) named_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) reexported_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) crates: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) dangling: Option<String>,
}

/// A trait's header; its items are records of their own.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct TraitRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) named_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) reexported_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) crates: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) dangling: Option<String>,
}

/// A type alias, a trait alias, or an associated type.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct AliasRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    /// The header of the trait impl that holds it.
    #[serde(rename = "impl", skip_serializing_if = "Option::is_none")]
    pub(crate) impl_header: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) named_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) reexported_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) crates: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// A field with docs or comments; its declaration's shape holds every field.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct FieldRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// A variant with docs or comments; its enum's shape holds every variant.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct VariantRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    /// The comments inside its discriminant.
    #[serde(skip_serializing_if = "absent")]
    pub(crate) comments: Option<Vec<BodyComment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// The doc and comments above one parameter.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ParamDoc {
    pub(crate) param: String,
    pub(crate) doc: String,
}

/// What a body's code holds, counted; no code text. `tokens` counts its
/// canonical tokens, and `single` says what a body of one statement does:
/// `call <path>`, `method <name>`, or `field <name>`.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct BodyRecord {
    pub(crate) statements: usize,
    pub(crate) tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) loops: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) loop_depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) errs: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) nones: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) awaits: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) panics: Option<usize>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) macros: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) single: Option<String>,
}

/// One ordinary comment inside a body, with the line it starts on.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct BodyComment {
    pub(crate) text: String,
    pub(crate) line: u32,
}

/// A function: free, inherent, a trait impl's, or a trait's own. `ops`
/// lists each direct use of a `self` field or a parameter with the methods
/// called on it, such as `self.items.push`, `n.len [loop]`, or
/// `self.total [write]`.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct FnRecord {
    pub(crate) path: String,
    pub(crate) sig: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    /// The header of the trait impl that holds it.
    #[serde(rename = "impl", skip_serializing_if = "Option::is_none")]
    pub(crate) impl_header: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) param_docs: Option<Vec<ParamDoc>>,
    /// The bodies of the run whose direct calls resolve to it.
    #[serde(skip_serializing_if = "absent")]
    pub(crate) direct_callers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) reexported_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) crates: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<BodyRecord>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) ops: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) comments: Option<Vec<BodyComment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// A constant or a static: module-level, or a trait's or an impl's.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ValueRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    /// The initializer, when it is one short literal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(rename = "impl", skip_serializing_if = "Option::is_none")]
    pub(crate) impl_header: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) reexported_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) crates: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) tests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<BodyRecord>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) comments: Option<Vec<BodyComment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// An item of an extern block, in an `extern` block of its own ABI.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ExternRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) lints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// A re-export, under the name it binds in its module.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct UseRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// The impl blocks of one owner with one header, wherever they are written.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct ImplRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) locs: Vec<Loc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attributes: Option<Vec<String>>,
    /// The macros the impl's body invokes.
    #[serde(skip_serializing_if = "absent")]
    pub(crate) macros: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) dangling: Option<String>,
}

/// A macro definition or invocation, under the module, declaration, or
/// impl owner that holds it.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct MacroRecord {
    pub(crate) path: String,
    pub(crate) rust: String,
    pub(crate) loc: Loc,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) via: Option<String>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) cfgs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "absent")]
    pub(crate) attrs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) test_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
}

/// A construct the outline does not know, or a module file it cannot read,
/// under the module, declaration, or impl owner that holds it.
#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(Deserialize), serde(deny_unknown_fields))]
pub(crate) struct UnsupportedRecord {
    pub(crate) path: String,
    pub(crate) text: String,
    pub(crate) loc: Loc,
}
